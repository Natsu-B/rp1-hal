# rp1-hal

[日本語](#日本語) | [English](#english)

## 日本語

Raspberry Pi CM5 の RP1 にある 2 つの Cortex-M3（proc0 / proc1）向けの rp-hal 風 Rust HAL です。
BCM2712 上の Linux との AMP 連携と、RP1 ファームを載せる BCM2712 側の third stage loader（`rp1boot`）も含みます。

- 各版の実装状況と実機での確認範囲: [Releases](https://github.com/Natsu-B/rp1-hal/releases)
- 設計: [docs/design.md](docs/design.md)（§9 が決定の記録）
- ローダー: [docs/loader.md](docs/loader.md)
- 共有メモリ: [docs/protocol.md](docs/protocol.md)
- ビルドと宣言: [docs/build.md](docs/build.md)、[docs/declaration-and-note.md](docs/declaration-and-note.md)
- ドライバ: [I2C](docs/i2c.md)、[ADC](docs/adc.md)、[SPI](docs/spi.md)

根拠の区分: **HW** = 実機で観測、**STATIC** = ホスト試験・ビルド・ELF 検査、**未検証** = どちらでも確かめていない。
実機の生ログはリポジトリの外に置き、文書には日付と条件（例: HW、2026-10-29、cold boot ×2）だけを書きます。
HW の主張は、その時の固定の起動一式（boot set）と、そこで動かした経路に限ります。

## English

An rp-hal style Rust HAL for the two Cortex-M3 cores (proc0 / proc1) in the RP1 chip of the Raspberry Pi CM5.
It also covers AMP with Linux on the BCM2712, and a BCM2712 third-stage loader (`rp1boot`) that loads the RP1 firmware.

- Status of each release and what was verified on hardware: [Releases](https://github.com/Natsu-B/rp1-hal/releases)
- Design: [docs/design.md](docs/design.md) (§9 records the decisions)
- Loader: [docs/loader.md](docs/loader.md)
- Shared memory: [docs/protocol.md](docs/protocol.md)
- Build and declarations: [docs/build.md](docs/build.md), [docs/declaration-and-note.md](docs/declaration-and-note.md)
- Drivers: [I2C](docs/i2c.md), [ADC](docs/adc.md), [SPI](docs/spi.md)

Evidence: **HW** = observed on hardware, **STATIC** = host tests, builds and ELF checks, **unverified** = neither.
Raw hardware logs are kept outside the repository; documents give only the date and conditions (e.g. HW, 2026-10-29, cold boot ×2).
A HW claim covers only the fixed boot set and the paths exercised at that time.

The documents under `docs/` are written in Japanese.
