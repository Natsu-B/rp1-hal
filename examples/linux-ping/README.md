# Linux RPC and channel example

Use these clients with the matching [dual-core firmware](../dual-core). The
firmware serves the `ping` RPC and the `channel` echo on proc1. Both endpoints
share the GPIO27 doorbell and the `rp1:irq` UIO interrupt. The firmware starts
serving them after Linux requests shared-memory publication. Open the endpoints
by name through the generated UIO devices.

```sh
cargo build --locked --release --target aarch64-unknown-linux-musl -p linux-ping -p linux-status
```

- `linux-ping` checks 1,000 IRQ RPC echoes and prints `PING_RTT`. Pass a
  positive round count to change the sample size.
- `linux-ping --channel` opens `Firmware::channel::<u32, u32>("channel")`,
  sends one nonce and waits up to one second for the matching reply.
- `linux-status` prints PCIe and SCMI status. Use `--health 2200` to sample
  liveness as well.

Run `linux-ping 1000` and `linux-ping --channel` against the same boot set to
check both endpoints. Each command prints an `EXAMPLE_END` marker after
verifying its replies.

## Real-time RPC clients on CM5

DOC. On a dedicated CM5, reserve CPU 2 for the request thread. Add these
settings to the Linux kernel command line and reboot.

```text
isolcpus=domain,managed_irq,2 nohz_full=2 irqaffinity=0-1,3 cpuidle.off=1
```

`lab/ping/rtt-realtime.sh` checks that Linux applied the isolation. It stops
`irqbalance` if active, moves writable IRQs and unbound workqueues off CPU 2,
and selects the `performance` governor if CPU frequency control is exposed.
Verify effective IRQ affinity in `/proc/irq` for the IRQs on your system.

Launch the latency-sensitive client with `taskset -c 2 chrt -f 50 ./my-client`.
This requires permission to set a real-time priority. In the client, lock
memory before `Firmware::open()` and leave enough locked-memory allowance for
the client and its future mappings. With the `libc` crate, use:

```rust
fn lock_memory() -> std::io::Result<()> {
    // SAFETY: mlockall takes only flags and applies to the current process.
    if unsafe { libc::mlockall(libc::MCL_CURRENT | libc::MCL_FUTURE) } != 0 {
        return Err(std::io::Error::last_os_error());
    }
    Ok(())
}
```

The lab `ping-test` build opts into the same memory lock with
`RP1_PING_MLOCK=1`. `lab/ping/rtt-observe.sh` records the scheduler, affinity,
memory lock, and events observed around RTTs above 25 µs. Pass `wake rtt` for
ABI v3 and `irq reference` for ABI v2. Set `RP1_RTT_RT=1` to run either probe
under the real-time configuration after applying the boot and IRQ settings.
