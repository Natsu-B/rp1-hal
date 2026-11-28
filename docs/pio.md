# Programmable I/O

**STATIC.** Declare the SMs and bank-0 pins on one core with RP1 clock authority.
The declaration gives that core the only PIO owner. Use the platform's clocks or
`Clocks::wait_from_proc0` on proc1.

```toml
[package.metadata.rp1.proc1]
pio = { sms = [0], pins = [] }
```

```rust
use rp1_hal::pio::{PIOBuilder, PIOExt, pio_asm};

let (pio, sms) = parts.split(&clocks)?;
let echo = pio_asm!("pull block", "mov isr, osr", "push block");
let program = pio.install(&echo.program)?;
let (sm, mut rx, mut tx) = PIOBuilder::from_installed_program(&program)
    .clock_divisor_fixed_point(200, 0)
    .build(sms.sm0)?;
let running = sm.start()?;
let accepted = tx.write(21);
let word = rx.read();
```

`split` returns the owner on failure. Retry it on `NotReady` while the reset
transition completes. FIFO operations are immediate. `write` returns false
when the TX FIFO cannot accept a word. `read` returns `None` until a word
arrives. Use `Buffers::OnlyTx` or `Buffers::OnlyRx` to join the FIFOs.

Use `Pins` to move in the declared GPIO owners and choose their pad settings.
The builder's IN, OUT, SET, JMP and side-set selections must fit those GPIOs.
Input-only pins keep the pad output disabled. Output pins get initial latch and
direction settings before start. Stop isolates the outputs and keeps the SM
state. Restart restores the entry address, initial pin settings and divider
phase while stopped. Use `SyncStart` on a tuple of two to four stopped SMs to
enable them and restart their dividers together.

Multiple SMs may borrow one installed program. Recover a stopped SM and its
pin owners with `sm.uninit(rx, tx)`, then uninstall the program. Leaving an SM
or program unreclaimed reserves its allocation for the rest of that PIO owner's
lifetime. Programs use the validated PIO v0 subset. Raw EXEC, dynamic PC,
unbounded pin reads and cross-SM IRQ flag access are rejected.
