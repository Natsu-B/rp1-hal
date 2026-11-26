# Linux RPC and channel example

Use these clients with the matching [dual-core firmware](../dual-core). The
firmware declares an IRQ RPC named `ping` on proc1 and a polling channel named
`channel` on proc0. The firmware starts serving both after Linux requests
shared-memory publication. Open the endpoints by name through the generated
UIO devices.

```sh
cargo build --locked --release --target aarch64-unknown-linux-musl -p linux-ping -p linux-status
```

- `linux-ping` checks 1,000 IRQ RPC echoes and prints `PING_RTT`. Pass a
  positive round count to change the sample size.
- `linux-ping --channel` opens `Firmware::channel::<u32, u32>("channel")`,
  sends one nonce and waits up to one second for the matching reply.
- `linux-status` prints PCIe and SCMI status. Use `--health 2200` to sample
  liveness as well.

**HYP (2026-11-27)** The channel echo and its coexistence with IRQ RPC are
pending verification on a CM5 with the matching boot set.
