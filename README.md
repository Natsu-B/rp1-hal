# rp1-hal

Raspberry Pi CM5 の RP1 にある 2 つの Cortex-M3（proc0 / proc1）向けの、rp-hal 風の Rust HAL と、
BCM2712 上の Linux との AMP 連携、および RP1 ファームを載せる BCM2712 側の third stage loader をまとめたリポジトリ。

状態: v0.1 の静的部分を実装中（ブランチ `feat/v0.1-core`）。実機では未確認。設計は [docs/design.md](docs/design.md)（§9 が決定の記録）。
次の作業: 未検証項目の実機確認（`<local>`、引き継ぎ `<lab>`）。

旧資産（参照のみ、ここへは段階的に移植する）:

- `<local>` — 旧スタンドアロン系列（regdb / PAC 生成器 / rt）
- `<local>` — PoC HAL（OpenAMP 系）と診断ブランチ群
- `<local>` — PoC chainboot loader

## v0.1 の実装状況

「ビルド済み」はホストテスト・clippy・`thumbv7m-none-eabi` ビルドが通ったという意味で、実機では何も確認していない。

| crate | 内容 | 状態 |
|---|---|---|
| `rp1-regdb` | レジスタ DB と PAC 生成器・検証器（rp1-hal-old から移植） | ビルド済み・テスト済み |
| `rp1-pac` | 生成 PAC（RESETS, CLOCKS, PLL, IO/PADS/SYS_RIO, TIMER, WATCHDOG 他）、`Peripherals::take()` は `unsafe` | ビルド済み・テスト済み（rp1-hal からはまだ使っていない） |
| `rp1-proto` | `.note.rp1` v2（TLV）、link ページ、SPSC ring、block、世代番号 | ビルド済み・ホストテスト済み |
| `rp1-macros` / `rp1-rt` | 2 コア 1 イメージ起動、`#[entry(proc0/1)]`、proc1 起動、`#[shared]`+`SharedSafe`、`#[isram]`、`#[proc0]`/`#[proc1]`、`Spsc` | ビルド済み・実機未確認 |
| `rp1-hal` | GPIO / UART / timer(alarm) + embedded-hal 1.0、clocks（読み取りで照合のみ）、`log!`（UART0）、SCMI clock のディスパッチ骨組み | ビルド済み・実機未確認 |
| `rp1-build` | 宣言 → linker script・note・DT overlay（dtbo）・資源型の生成 | ビルド済み。overlay は scmi_linux.dtb のコピーに `fdtoverlay` で適用できることを確認（STATIC） |
| `cargo-rp1` | ELF 検査（共有 SRAM 上の LDREX/STREX、予算、note、PT_LOAD の重なり）、`--dtb` で overlay 適用確認 | ビルド済み。release の例で検査が通る |
| `examples/dual-core` | proc0/proc1 の `log!`、`#[shared] Spsc`、GPIO27 ドアベルのエコー | ビルド済み・実機未確認 |
| `examples/linux-doorbell` | UIO で link / doorbell を mmap してドアベル往復（返事はポーリング、UIO IRQ は使わない） | aarch64-musl ビルド済み・実機未確認 |
| ローダー（`rp1boot`） | — | 未着手（範囲外） |
