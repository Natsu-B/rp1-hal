# rp1-hal

Raspberry Pi CM5 の RP1 にある 2 つの Cortex-M3（proc0 / proc1）向けの、rp-hal 風の Rust HAL と、
BCM2712 上の Linux との AMP 連携、および RP1 ファームを載せる BCM2712 側の third stage loader をまとめたリポジトリ。

状態: **v0.1 完成**（main）。実機（CM5 Lite 1 台、cold boot）で v0.1 一式が通った: ローダー `rp1boot` → RP1 2 コア →
Linux 6.12 の SCMI クロック・UIO ドアベル・共有メモリ（ring / block）。ライブラリのホットパスは ISRAM に置く。
設計は [docs/design.md](docs/design.md)（§9 が決定の記録）、ローダーは [docs/loader.md](docs/loader.md)、共有メモリは [docs/protocol.md](docs/protocol.md)。

根拠の区分: **HW** = 実機で観測、**STATIC** = ホスト試験・ビルド・ELF 検査、**未検証** = どちらでも確かめていない。
実機の記録（RESULT.md）: `<lab>,3,4,5}/`、
`<lab>`。HW の主張は「その記録にある固定の起動一式（boot set）と、そこで動かした経路」に限る。

旧資産（参照のみ、ここへは段階的に移植する）:

- `<local>` — 旧スタンドアロン系列（regdb / PAC 生成器 / rt）
- `<local>` — PoC HAL（OpenAMP 系）と診断ブランチ群
- `<local>` — PoC chainboot loader

## v0.1 の実装状況

| crate | 内容 | 状態 |
|---|---|---|
| `rp1-regdb` / `rp1-pac` | レジスタ DB・PAC 生成器・検証器 / 生成 PAC（`Peripherals::take()` は `unsafe`） | STATIC |
| `rp1-macros` / `rp1-rt` | 2 コア 1 イメージ起動、`#[entry(proc0/1)]`、proc1 起動、`#[shared]`+`SharedSafe`、`#[isram]`、`#[proc0]`/`#[proc1]`、`Spsc` | HW（両コア起動、.bss 初期化、`Spsc` 受け渡し、hwrun4/5）。割り込みハンドラ登録（`#[interrupt]`）は未実装（v0.2） |
| `rp1-hal` | GPIO / UART / timer（alarm はポーリング専用）+ embedded-hal 1.0、`cold`（冷起動と PCIe エンドポイント再設定）、clocks（`init` で clk_sys を公式の 200 MHz へ、HW）、`log!`（UART0）、SCMI クロックサーバー（Base+Clock 2.0、ポーリング）、shm の RP1 側 | HW: UART0 ログ、ドアベル、SCMI probe・clk_disable_unused 後も UART0 継続（hwrun2）、shm（hwrun3〜5）。STATIC のみ: SCMI RATE_SET 拒否、PCIe 再設定の故障経路、UART1 の外部送受信は未検証 |
| `rp1-proto` | `.note.rp1` v2、link ページ、SPSC ring（reject / overwrite、バッチ）、block、世代番号、起動束（bundle） | HW: ring 双方向 10^6 件欠落・破損 0、overwrite の Lost、block 1 MiB 一致（ポーリング）。STATIC のみ: 待機（通知）ハンドシェイク、overwrite の 2^32 周回（レビュー 2 #4） |
| `rp1-linux` | UIO 経由の共有メモリクライアント（uncached） | HW（lab/shm-linux） |
| `rp1-dt` | 純 Rust の FDT と overlay 適用（libfdt `fdtoverlay` と同じ結果を目標）、最終 DTB の衝突検査 | HW: lab の base DTB + app overlay（`fdtoverlay` と一致、STATIC でも比較）。一般の overlay は fixture とレビュー 2 #1/#3/#7 の反例を `fdtoverlay` と比較した範囲（STATIC） |
| `rp1-build` | 宣言 → linker script・note・DT overlay・資源型の生成、source commit の記録 | STATIC（生成物を HW の起動一式で使用） |
| `cargo-rp1` | ELF 検査（共有 SRAM 上の LDREX/STREX、予算、note）、`--dtb` で overlay 適用と衝突確認、`bundle`、`build`（ISRAM 自動配置） | STATIC（生成した束で HW 起動） |
| `boot/rp1boot` | BCM2712 の third stage loader（stable Rust）: 束の SHA-256 検証、overlay 適用、RP1 の I2C 起動、EL2 で Linux へ | HW: cold boot 2 回で `~ #`（電源から約 31 s）、1 bit 破損した束を拒否（hwrun5）。STATIC のみ: 異常な Image ヘッダーの配置拒否（レビュー 2 #2）、I2C の short read（#10） |
| `examples/dual-core`, `examples/linux-doorbell` | 2 コアの `log!`・`Spsc`・ドアベル / UIO でドアベル往復 | HW（p50 約 3〜6 µs） |
| `lab/shm-firmware`, `lab/shm-linux` | 共有メモリの実機証明（WINDOW / RING / BLOCK / OVERWRITE） | HW（ISRAM 版で RP1→Linux 4.58 MB/s、Linux→RP1 5.30 MB/s、バッチ） |

v0.1 の範囲外（設計 §8.1）: 割り込み駆動（RP1 → Linux 通知、alarm IRQ）、Linux 再起動への追従、DMA / PIO / SPI / I2C 等のドライバ。
