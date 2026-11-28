# Programmable I/O

**STATIC.** Declare the SMs and any PIO pins on one core with RP1 clock
authority. Use the platform clocks or `Clocks::wait_from_proc0` on proc1.
Enable the HAL's `pio-asm` feature to use `pio_asm!` or `pio_file!`.

```toml
[package.metadata.rp1.proc1]
pio = { sms = [0], pins = [] }
wait_timer = "ALARM0"
```

```rust,no_run
# use rp1_hal::{clocks::Clocks, pio::{Irq, PioParts, UninitStateMachine}, timer::WaitTimer};
# #[cfg(feature = "pio-asm")]
use rp1_hal::pio::pio_asm;
# #[cfg(not(feature = "pio-asm"))]
# use pio::pio_asm;
use rp1_hal::{pio::{PIOBuilder, PIOExt}, timer::{Deadline, Duration}};

# struct Sms { sm0: UninitStateMachine<0, Irq<1, 3>> }
# fn echo_once(parts: PioParts<1, Sms>, clocks: &Clocks, wait_timer: &mut WaitTimer) {
let (pio, sms) = parts.split_wait(clocks).unwrap();
let echo = pio_asm!("pull block", "mov isr, osr", "push block", "park:", "jmp park");
let program = pio.install(&echo.program).unwrap();
let (sm, mut rx, mut tx) = PIOBuilder::from_installed_program(&program)
    .build(sms.sm0).unwrap();
let end = Deadline::after(Duration::from_ticks(1_000)).unwrap();
tx.blocking(wait_timer).write_until(21, end).unwrap();
let running = sm.start().unwrap();
let word = rx.blocking(wait_timer).read_until(end).unwrap();
assert_eq!(word, 21);
running.stop().uninit(rx, tx);
program.uninstall().unwrap();
# }
```

`split_wait` polls through reset with a 1 ms deadline per transition.
`split` gives the parts back on `NotReady` for applications that schedule
their own retries. Use `Buffers::OnlyTx` or `Buffers::OnlyRx` to join the FIFOs.

**STATIC.** `try_read` and `try_write` are immediate. Select `polling()` for
an explicit busy wait. With a proc1 PIO declaration and its own `wait_timer`,
`blocking(&mut wait_timer)` uses PIO_0 and WFE. Call `on_irq1()` on the
blocking adapter to select PIO_1 for that operation.
An expired deadline at entry leaves the FIFO untouched. Once a wait starts,
data available at its final probe wins over timeout. Waits require thread mode
with interrupts unmasked.

**HW (2026-11-28).** Gate C verified both lines for each SM's RX and TX on
proc1 over two cold boots, idle and under Linux, SCMI and network load.

`Pins` accepts only `PioPin<N>` tokens generated for `pio.pins`. Input pads
stay isolated. Output pads use an `InitialOutput` setting for their latch and
direction. For example, `HiZHigh` preloads a high latch with the driver off
for side-set PINDIRS. The builder checks the program's pin accesses against
the moved tokens. Recover them with `sm.uninit(rx, tx)` and `Pins::take`.

## rp-hal API differences

- `split`, `build` and `start` return `Result` and return ownership on error.
- `in_pin_base` and `in_count` describe the relative input range. On RP1,
  `in_count` bounds validation of PIO v0 programs in software.
- `program.uninstall()` uses the originating PIO borrowed by the allocation.
  Each SM borrows that handle. A dropped or forgotten SM keeps the
  allocation reserved for the lifetime of the PIO owner.
- `(a, b).sync_start()` combines rp-hal’s `with().sync().start()` steps in
  one operation and one enable/divider write. The tuple gives the caller
  every SM back if validation fails.
- `restart()` runs while stopped and restores the entry address, initial pin
  latches and directions, and divider phase. This replays the initialization
  preamble while the pads are isolated. FIFO contents are retained. rp-hal
  restarts a running SM at its wrap target.
  [rp2040-hal source](https://docs.rs/rp2040-hal/0.12.0/rp2040_hal/pio/struct.StateMachine.html#method.restart)

Parity follow-ups include scoped cross-SM IRQ flags, `MOV x, PINS`
(which reads the whole PIO v0 input window), and sharing IN, JMP and WAIT
inputs between SMs. Shared inputs need a pad owner that keeps the mux
configured until the last reader releases it and preserves exclusive drive
ownership.
