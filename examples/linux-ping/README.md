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
