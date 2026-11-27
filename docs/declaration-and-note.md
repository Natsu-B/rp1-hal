# 宣言と `.note.rp1`（設計 B、2026-10-23 草案）

ファームが「何を持ち、Linux と何を共有するか」を、ビルド前に 1 か所で宣言する。
ビルドはそこから Rust の型・linker script・`.note.rp1` を作り、ローダーと Linux 側は note だけを読む。

```
firmware/Cargo.toml [package.metadata.rp1]
        │  build.rs（1 行: rp1_build::configure()）
        ├─▶ OUT_DIR/rp1.rs        … コアごとの Resources 型、shm ハンドル、cfg、note のバイト列
        ├─▶ OUT_DIR/rp1-layout.x  … 共有 SRAM 区画・ISRAM/DSRAM・予約の linker script
        ├─▶ OUT_DIR/rp1-app.dtbo  … Linux 用の DT overlay（§3）。ELF の .rp1.dtbo セクション（非ロード）に入る
        └─▶ cargo:rustc-cfg=...   … rp1_log など
cargo rp1 build（リンク後）: ELF 検査（設計 E）、overlay をホストで試し当て。成果物は起動一式（§4）
VPU ファーム: いつもどおり config.txt / overlay を処理し、DTB をローダーに渡す（rp1 用の設定は書かない）
ローダー:   ELF の .note.rp1 を読む → ELF の .rp1.dtbo を DTB に最後に当てる → 残りの衝突を検査 → RP1 起動 → Linux へ
Linux:      /proc/device-tree の note のコピーと UIO ノードを読む
```

## 1. 宣言の書き方

### 1.1 置き場所

- ファーム crate の `Cargo.toml` の `[package.metadata.rp1]`。別ファイルは置かない。
- ファーム crate に 1 行の `build.rs` を置く（cortex-m-rt の `memory.x` と同じ理由: linker の引数は build.rs からしか渡せない）。

```rust
// firmware/build.rs
fn main() { rp1_build::configure().unwrap(); }
```

```rust
// firmware/src/main.rs の先頭
rp1_rt::bind!();          // include!(concat!(env!("OUT_DIR"), "/rp1.rs")) → mod rp1 { ... }
```

### 1.2 全項目

```toml
[package.metadata.rp1]
schema = 2                                # 宣言の版（必須。2 のみ）
board = "rpi-cm5"                         # ボード表（Linux に残すピン・ブロック、DT ラベル）。rp1-build に同梱
clock = { authority = "rp1", clk_sys_hz = 200_000_000 }  # authority: "rp1" | "linux"
log = { sink = "uart0", baudrate = 115_200 }             # = システム所有の uart0 { tx = 14, rx = 15 }。省略すると log! はコンパイルエラー
stack = { proc1 = "6KiB" }                # ユーザー分のスタック（DSRAM）。省略時は各 2 KiB。proc0 には宣言の有無によらずプラットフォーム分 768 B が足される
link_page = "16KiB"                       # 共有 SRAM の Linux 用ページ。"none" なら全リングが placement = "linux"（v0.1 未実装）

[package.metadata.rp1.proc0]
gpio = [22]                               # GPIO として使うピン
timer = ["ALARM0"]                        # タイマーのアラーム
i2c1 = { sda = 2, scl = 3 }               # インスタンス = { 信号 = ピン }。ピンは機能表で検査（§1.4 E2）
uart2 = { tx = 4, rx = 5 }                # cts/rts は未対応（E4）

[package.metadata.rp1.proc1]
gpio = [8, 27]                            # 8 = SPI0 のチップセレクト（GPIO で駆動）
gpio_irq = [27]                           # 割り込みを受けるピン（gpio の部分集合）
timer = ["ALARM1"]
adc = {}                                  # ピンの無いブロック
dma = { channels = [0, 2], pool = { slots = 2, bytes = 1024 } } # pool は任意（docs/dma.md）
spi0 = { sck = 11, mosi = 10, miso = 9 }  # ハードの csN は未対応（エラー）
host_irq = true                           # RP1 → Linux 割り込み（1 コアだけ。rp1_link::notify::HostIrq、design.md §6.2）
doorbell = 26                             # Linux → このコアのドアベル（gpio / gpio_irq と重複不可）
heartbeat = { period_ms = 100 }           # 任意: このコアが rp1_rt::status::beat() を N ms ごとに呼ぶ。Linux の health 判定に使う（protocol.md §9）

[[package.metadata.rp1.shm]]
name = "down"                             # [a-z][a-z0-9_]{0,23}。UIO 名は rp1:<name>
kind = "ring"                             # "ring" | "block" | "channel" | "rpc"
direction = "linux-to-rp1"
core = "proc1"                            # M3 側でハンドルを受け取るコア
message = "protocol::Cmd"                 # リングの要素
depth = 16                                # 要素数（2 の冪）
notify = "irq"                            # "poll" が既定。irq は core の doorbell が要る

[[package.metadata.rp1.shm]]
name = "samples"
kind = "ring"
direction = "rp1-to-linux"                # ring のみ: "rp1-to-linux" | "linux-to-rp1"
overflow = "overwrite"                    # ring のみ: "reject"（既定）| "overwrite"
placement = "reader-local"                # ring のみ: "reader-local"（既定）| "linux"（v0.1 未実装）
core = "proc1"
message = "protocol::Sample"
depth = 4096
notify = "irq"                            # irq は core の host_irq = true が要る

[[package.metadata.rp1.shm]]
name = "trace"
kind = "block"
type = "protocol::Trace"                  # block のみ
writer = "rp1"                            # "rp1" | "linux" | "both"
core = "proc0"
linux_map = "uncached"                    # "uncached" | "cached"

[package.metadata.rp1.linux]
uio_group = "rp1"                         # UIO を開けるグループ（udev ルールを生成）
keep = { uart1 = { tx = 0, rx = 1 } }     # Linux に残すインスタンスとピン。省略時はボード既定（rpi-cm5 は uart1 0/1）。書くと既定を置き換える
gpiomem = "disable"                       # "disable"（既定）| "keep"。/dev/gpiomem0 を無効にするか（§3.3）
on_conflict = "disable"                   # "disable"（既定）| "refuse"。起動時の衝突の扱い（§3.2）
```

- 所有は**ブロックのインスタンス単位**で 1 つ（`proc0` / `proc1` / Linux）。宣言に書かれていないインスタンスとピンは Linux のもの。
- インスタンスのキー: `uart0`..`uart4`、`i2c0`..`i2c3`、`spi0`/`spi1`/`spi3`/`spi5`、`adc`、`pwm0`/`pwm1`、`dma`。信号名: UART `tx rx`、
  I2C `sda scl`、SPI `sck mosi miso`（すべて必須）。UART の `cts`/`rts` は RP1 側のフロー制御が未実装なので、`proc0`/`proc1` では
  E4（`UART flow control is not supported yet`）。そのピンを GPIO として使うなら `gpio` に書く。`linux.keep` では Linux のフロー制御
  ピンとして書ける（機能表で検査し、保護する）。
- ピンは「インスタンスの信号への割り当て」から導き、bank 0 の機能表（`rp1-pinmux`、datasheet・pinctrl-rp1・HAL の表と照合）で
  検査する。生成される resources はインスタンスを型付きの `Parts`（例 `i2c::Parts<1, (Pin<2>, Pin<3>)>`）で渡し、表に無い組はコンパイルできない。
  手動組み立ても同じ builder を使い、同じ検査を受ける（`i2c::Pins::<1>::builder().sda(sda).scl(scl)` を `Parts::new` に渡す）。
  ピン、IRQ、ドライバーは所有コア内に留める。ISR と共有するドライバーには `interrupt::Slot` を使う。
  Slot の優先度上限は既定で 4。より緊急度の高い IRQ は割り込み可能なので、Slot にアクセスする IRQ は上限と同じか
  それより緊急度の低い優先度に設定する。IRQ を関数に渡すときは生成される型名（例 `rp1::proc0_irq::TIMER_0`）を使える。
- `linux.keep` はピンの保護だけ。Linux 側ノードの有効化は config.txt の `dtoverlay=`（例 `uart1-pi5`）に任せる。
  明示した `keep` はボード既定を置き換える（既定への追加ではない）。既定の UART1 を残したまま追加するなら、同じ table に `uart1` も書く
  （例 `keep = { uart1 = { tx = 0, rx = 1 }, i2c1 = { sda = 2, scl = 3 } }`）。`keep` から `uart1` を外すと GPIO 0/1 を RP1 に渡せる
  （ビルド時に警告）。

- SHM を宣言するファームは `rp1-link` に依存し、`build.rs` の `configure_with_types` で共有型を登録する（STATIC）。
  裸の型名はクレートルート、`protocol::Sample` は外部クレート、`crate::types::Sample` はクレート内の型を指す。
  生成コードは対象型のサイズと `Message::TYPE_ID` を登録値と照合する。各コアの `p.link.try_open()` が返すフィールド名は宣言名
  （`type` などは `r#type`）。ブロックの書き込みには `writer` で指定した所有権が必要。

HW（2026-10-30、lab ファーム = UART0（log）+ I2C1 + SPI0 + ADC を RP1、UART1 を Linux、cold boot ×3）: Linux の
`gpio-reserved-ranges` が所有ピンちょうど `<2 2 8 4 14 2 22 1 27 1>`（I2C1 2/3、SPI0 9..11 と CS 8、log 14/15、GPIO 22、27）になり、
既存の回帰（shm・doorbell・I2C・SPI・ADC・SCMI）が全て合格。同じ起動で Linux の UART1（ttyAMA1、GPIO 0/1）がホストとの
送受信の両方向で合格し、その間も RP1 の UART0 ログは出続けた。

### 名前付き channel と RPC（STATIC）

```toml
[package.metadata.rp1.proc1]
doorbell = 27
host_irq = true

[[package.metadata.rp1.shm]]
name = "ping"
kind = "rpc"
core = "proc1"
message = "Ping"
reply = "Pong"
depth = 8
notify = "irq"
```

`message` は Linux → RP1、`reply` は RP1 → Linux。両方の型を `configure_with_types` に登録する。
名前は `[a-z][a-z0-9_]{0,20}`。`channel` は `depth = { dn = 16, up = 64 }` も選べる。
`notify` は既定で両方向 `poll`。`notify = { dn = "irq", up = "poll" }` で個別に指定できる。
`irq` の dn には `doorbell`、up には同じコアの `host_irq` が必要。ドアベルのピンは
`doorbell` が所有する。`direction`・`overflow`・`writer`・`type`・`placement` は省く。

1 項目につき 2 つの物理領域を数える（上限は合計 64）。UIO は `rp1:ping-dn`、`rp1:ping-up`。
生成される `rp1::link::Ping` は `Proc1Link::ping` の型。Linux は `fw.client("ping")` で開く。
`channel` は `fw.channel(name)` で開く。

### 1.3 語彙（所有の単位）

| 種類 | 名前の例 | 単位 | 付いてくるもの |
|---|---|---|---|
| インスタンス | `uart0`..`uart4`、`i2c0`..`i2c3`、`spi0`/`spi1`/`spi3`/`spi5`、`adc` | インスタンス全体。信号ごとのピン | IRQ 線、リセットビット、クロック、ラッパー `+0x108`、割り当てたピン |
| ブロックの一部 | `timer = ["ALARM0".."ALARM3"]` | アラーム単位。タイマーの計数値は全員が読める | 該当 IRQ（26..29） |
| ピン | `gpio = [..]`、`gpio_irq = [..]` | ピン単位（funcsel、pad、SYS_RIO、イベント選択） | — |
| RP1 → Linux 割り込み | `host_irq = true`（1 コアだけ） | MSI-X vector 29 の `MSIX_CFG.TEST`（`TIMER.ALARM3` も所有） | リソース `host_irq: rp1_link::notify::HostIrq`、そのコアの `notify = "irq"` rp1-to-linux 端点、overlay の `rp1:irq` UIO |
| Linux → RP1 ドアベル | `doorbell = N`（コアごとに 1 本、N は 0..=27） | そのコアの `PROCn_INTF` の bit N とピン N（パッドは使わない） | リソース `doorbell: rp1::DoorbellK`（`rp1_link::notify::Doorbell<N, K>`）、そのコアの `IO_BANK0` 線、`notify = "irq"` linux-to-rp1 端点、note LINK のピン。`Pin` / `PinIrq` は作らない |
| バンク | 自動 | ピンを 1 本でも持つと、そのバンクの電圧設定は読むだけ | — |
| システム | `log`、`clock.authority = "rp1"` | RP1 全体（コアに属さない） | UART0 と GPIO 14/15 / クロック生成器 |
| Linux | `linux.keep`（省略時はボード既定。書くと既定を置き換える） | インスタンスとピン | 宣言できない（E3/E4） |

PCIe、mailbox ch0（SCMI 用）、共有 SRAM の予約域は、ボード表が「RP1 システム」として固定で持ち、宣言はできない。
rpi-cm5 のボード表は GPIO 28..53（ETH_RST_N、VBUS など）と Ethernet・USB・PCIe・PIO を Linux に残す（DMA は 2026-11-05 から宣言できる。宣言しなければ Linux のまま。`dmas` で `rp1_dma` を使う有効なノードは衝突として扱う。[dma.md](dma.md)）。この「Linux に要る
ブロック」の表は、ビルドの宣言検査とローダーの保護ノード（§3.2）が共有する。
宣言しなかったブロックは、`rp1_hal::linux_blocks` が標準ファームと同じようにリセットを解除し、クロックと
割り込み出力イネーブル（ETH_CFG INTE、I2C / SPI の `+0x108` など: [interrupts.md](interrupts.md) §3）を設定してから Linux に渡す。

### 1.3.1 コアごとの割り込み所有

STATIC（2026-11-27）: `ProcNResources.irq` は、宣言でそのコアに割り当てた `Binding<Source, IRQ, CORE>` だけを持つ。既存の NVIC 操作は `Line<IRQ>` への参照で使える。

| 宣言 | `ProcNIrq` のフィールド |
|---|---|
| `timer = ["ALARMn"]` | `TIMER_n: Binding<AlarmIrq<n>, {26+n}, CORE>` |
| `gpio_irq = [..]`（空でない） | `IO_BANK0: Binding<Bank0Irq, 0, CORE>` と、各ピンの `gpioN: rp1_hal::gpio::PinIrq<N, CORE>` |
| `uart1` と同じコアの `wait_timer` | `rp1::Uart1Parts` が線を所有し、UART1 ハンドラを生成 |
| raw `uart1`、`uart2`..`uart4` | `UARTn: Binding<BlockIrq<ID>, {41+n}, CORE>` |
| `i2c0`..`i2c3` | `I2Cn: Binding<BlockIrq<ID>, {7+n}, CORE>` |
| `spi0` と同じコアの `wait_timer` | `rp1::Spi0Parts` が線を所有し、SPI0 ハンドラを生成 |
| raw `spi0` / `spi1` / `spi3` | `SPIn: Binding<BlockIrq<ID>, {19+n}, CORE>` |
| `pwm0` / `pwm1` | `PWM0: Binding<BlockIrq<48>, 5, CORE>` / `PWM1: Binding<BlockIrq<49>, 41, CORE>` |
| `adc` と同じコアの `wait_timer` | `rp1::AdcParts` が線を所有し、ADC_FIFO ハンドラを生成 |
| raw `adc` | `ADC_FIFO: Binding<BlockIrq<ID>, 52, CORE>` |

HW: 上の線の番号は実機で NVIC まで届くと確認された対応に限る（GPIO は両コア。PWM0・TIMER_3 は試作ファームでの確認）。根拠と、I2C / SPI の
出力イネーブル `+0x108` などの条件は `docs/interrupts.md`。
STATIC: `rp1-build::board::IRQ_VERIFIED` が生成と ELF 検査の対応表である。
SPI5 と DMA は IRQ 番号が実機未確認なので `Line` を生成しない。
インスタンスの所有やポーリング用トークンには影響しない。

`gpio_irq` のピンは通常の `pins.gpioN` と別に割り込み権限を持つ。両コアがそれぞれ `gpio_irq` を宣言すると、
両方にコア番号付きの `IO_BANK0` が入る。IO_BANK0 の割り込み状態・イネーブルはコアごとに分かれている（DOC）。
STATIC: `PinIrq` はそのコアから見たピンの割り込みを操作する。`enable(Event)` / `disable(Event)`（`Event` はレベル 2 種とエッジ 2 種）、
`is_pending()`、`clear()`（ラッチしたエッジを消す）、`force()` / `unforce()` / `is_forced()` を持つ。
コンストラクターは unsafe で、生成コードだけが呼ぶ。ピンの設定（`into_input` など）は GPIOn_CTRL を書き直してイベントのマスクを消すので、
`enable` より前に済ませる。

`log` 用 UART0 の線は渡さない。`host_irq = true` でも `TIMER_3` は渡さず、ALARM3 は両コアの `timer` 宣言から予約する。
IRQ53（PCIE_OUT）と IRQ57（PROC_MISC）は proc0 のシステムサービス用で、利用者の resources には入らない。
IRQ59（CLOCKS_DEFAULT）など、それ以外のシステム線も渡さない。

STATIC: `cargo rp1 check` と `cargo rp1 build` は、両コアの外部ベクタの Thumb ビット・アドレス範囲を検査し、
既定ハンドラ以外の線を note の OWNERSHIP と照合する。宣言で所有しない線や未確認線にハンドラを置くとビルドエラーになる。
IRQ53/57 はプラットフォーム専用。ハンドラを定義せず線を所有することは許される。
`#[interrupt(procN)]` は同じコア・線に 1 個。IRQ で受信する link のドアベルは、既定で生成ハンドラが ack して起床を通知する。
GPIO との共有や独自処理が必要な場合は `custom_doorbell_handler = true` を宣言し、
そのコアの `IO_BANK0` で `doorbell.ack_and_signal()` と各 GPIO source を処理する。

STATIC（2026-11-27）: 同じコアで `uart1` と `wait_timer` を宣言すると、
生成された UART1 ハンドラと `uart.blocking(&mut p.wait_timer)` が同期 I/O を受け持つ。
`custom_uart1_handler = true` を指定すると `p.irq.UART1` を受け取り、
そのコアの `#[interrupt(procN)] fn UART1()` の定義がリンク時に必須になる。
SPI0 も同様に `spi.blocking(&mut p.wait_timer)` を使う。
`custom_spi0_handler = true` で `p.irq.SPI0` と独自 `SPI0()` を使える。
STATIC（2026-11-28）: `i2c1` と `wait_timer` が同じコアにあれば
`rp1::I2c1Parts` から初期化し、`bus.blocking(&mut p.wait_timer)` を使う。
`custom_i2c1_handler = true` で `p.irq.I2C1` と独自 `I2C1()` を使える。

### 1.4 ビルド時のエラー（build.rs が止める）

| # | 条件 | メッセージの例 |
|---|---|---|
| E0 | `schema` が未指定または未知、知らないキー・インスタンス、形式違反 | `unknown key 'uart9' in proc1 (instances: uart0, .., adc)` |
| E1 | 同じピンを 2 回（コア間、機能と `gpio`、log と機能、`linux.keep` と RP1） | `GPIO 10 is claimed by both proc1.spi0.mosi and proc0.gpio` |
| E2 | ピンがその信号を出せない・bank 0 にピンが無い | `proc0.uart2.tx = 6: GPIO6 is UART2_CTS (alt2), not UART2_TX; UART2_TX is on GPIO4` / `SPI6 has no bank-0 pins` |
| E3 | 同じインスタンスを 2 回（proc0 と proc1、log と core、RP1 と `linux.keep`）、`host_irq` を両コアで | `I2C1 is declared by both proc0 and proc1` |
| E4 | ボードが Linux に要るピン・ブロック、未対応のインスタンス | `GPIO 32 is board line ETH_RST_N, owned by Linux on rpi-cm5` / `GPIO 0 is kept for Linux (uart1.tx) by board rpi-cm5` / `ETH is required by Linux on rpi-cm5` / `SPI4 is slave-only` / `proc0.uart2.cts: UART flow control is not supported yet` |
| E5 | 必須信号の欠落、`gpio_irq` が `gpio` に無い | `proc1.spi0 needs sck, mosi, miso` / `gpio_irq 27 is not in proc1.gpio` |
| E6 | 環境変数 `RP1_BOARD_DTB` があり、その DTB（config.txt の overlay を当てたもの）で起動時に無効化・拒否が起きる | `RP1_BOARD_DTB <file>: rp1boot would disable 1 Linux node(s) ...`（gpiomem 方針だけなら警告） |
| — | `log` があるのに UART0 が RP1 の所有になれない | `log sink uart0 requires UART0 to be owned by RP1` |
| — | v0.1 の rpi-cm5 で `clock.authority = "linux"` | `clock.authority = linux is unsupported on the v0.1 rpi-cm5 board; use rp1` |
| — | ring の `notify = "irq"` で、宣言した core が `host_irq = true` を持たない（RP1 → Linux の割り込みは生産側のコアが鳴らす） | `notify=irq on ring events requires proc1.host_irq = true (the producing core raises rp1:irq)` |
| — | linux-to-rp1 の ring の `notify = "irq"` で、宣言した core が `doorbell` を持たない（Linux は読み手のコアのドアベルを鳴らす） | `notify=irq on ring down requires proc1.doorbell (Linux raises the reading core's doorbell)` |
| E1 | `doorbell` のピンを `gpio`・`gpio_irq`・もう一方のコアの `doorbell` にも書く | `GPIO 27 is claimed by both proc1.gpio and proc1.doorbell` |
| — | ring の `notify` が `poll`・`irq` 以外 | `invalid notify (poll or irq)` |
| — | shm の名前が重複・形式違反、depth が 2 の冪でない、kind と項目が合わない | `shm 'ctl': 'direction' is only valid for kind = "ring"` |
| — | Linux → RP1 のリングが共有 SRAM の予算を超える | `shm 'cmds': linux-to-rp1 ring needs 8.0 KiB of shared SRAM, 2.5 KiB left` |
| — | 型が存在しない、`Message` を実装していない | rustc のエラー（生成コードがその型を参照するため） |

E1〜E5 は `rp1-build` の単体試験で 1 件ずつ固定している（STATIC）。config.txt の `dtoverlay=` はビルドから見えないので、
Linux 側の衝突は `cargo rp1 check --dtb`（E6 と同じ予測、[build.md](build.md)）か、起動時の rp1boot（§3.2）で扱う。

### 1.5 生成されるもの（`OUT_DIR/rp1.rs`）

例えば、次の宣言から生成する（STATIC、コンストラクターと note は省略）。

```toml
[package.metadata.rp1]
schema = 2
[package.metadata.rp1.proc1]
gpio = [27]
gpio_irq = [27]
timer = ["ALARM1"]
spi0 = { sck = 11, mosi = 10, miso = 9 }
```

```rust
pub mod rp1 {
    pub struct Proc1Pins {
        pub gpio27: ::rp1_hal::gpio::Pin<27>,
    }
    #[allow(non_snake_case)]
    pub struct Proc1Irq {
        pub IO_BANK0: Binding<Bank0Irq, 0, 1>,
        pub SPI0: Binding<BlockIrq<16>, 19, 1>,
        pub TIMER_1: Binding<AlarmIrq<1>, 27, 1>,
        pub gpio27: ::rp1_hal::gpio::PinIrq<27, 1>,
    }
    #[allow(non_snake_case)]
    pub struct Proc1Resources {
        pub pins: Proc1Pins,
        pub irq: Proc1Irq,
        pub TIMER_ALARM1: ::rp1_hal::timer::Alarm<1>,
        pub spi0: ::rp1_hal::spi::Parts<0, (Pin<11>, Pin<10>, Pin<9>)>,
    }
}
```

`#[entry(proc0)]` / `#[entry(proc1)]` は、対応する `rp1::ProcNResources` を受け取る。
`Proc0Resources` だけが `proc1: rp1_rt::Proc1Launcher` と、`clock.authority = "rp1"` のときの
`platform: rp1_hal::platform::Token` を持つ。宣言が空でも `ProcNIrq` と `irq` フィールドは生成する。
`host_irq = true` のコアには `host_irq: rp1_link::notify::HostIrq` が、`doorbell = N` のコアには
`doorbell: rp1::DoorbellK`（`rp1_link::notify::Doorbell<N, K>` の別名）と `p.irq.IO_BANK0` が入る。
`p.link.try_open()` が作る端点の型は `notify` で決まる（protocol.md §3）。
`platform` は生成される `rp1::platform::init(&mut core, p.platform)` に渡す。これがクロック・Linux 側ブロック・
宣言した log・PCIe エンドポイント・SCMI サーバーを順に立ち上げ（`rp1_hal::platform`）、`System`（`clocks`、`pcie`、`report`）を返す。

## 2. `.note.rp1` の形式

ELF の note（`SHT_NOTE`、非ロード）。namesz = 4、name = `"RP1\0"`、type は v2 が
`0x52503102`、実験的 v3 が `0x52503103`。type と descriptor version を照合する。
すべてリトルエンディアン。desc は 4 バイト境界。

### 2.1 ヘッダー（32 バイト）

| オフセット | 型 | 内容 |
|---|---|---|
| 0 | `[u8; 8]` | `"RP1NOTE\0"` |
| 8 | `u16` | descriptor version（通常 2、実験的 link wake は 3） |
| 10 | `u16` | ヘッダー長 = 32 |
| 12 | `u32` | desc 全体の長さ |
| 16 | `u32` | flags（予約、0） |
| 20 | `u32` | CRC32（ヘッダーの後ろからの全レコード） |
| 24 | `u32` | 固定値 = 1（読み手は 1 以外を拒否。manifest の schema とは無関係） |
| 28 | `u32` | 予約 0 |

### 2.2 レコード（TLV）

```
u16 type | u16 flags | u32 len | value[len] | 0 埋めで 4 バイト境界
flags bit0 = critical: 読み手が知らない type なら拒否する。0 なら読み飛ばす。
```

| type | critical | 名前 | 中身 |
|---|---|---|---|
| 0x0001 | 1 | FIRMWARE | 名前（文字列）、版（文字列）、ソースの git ハッシュ 20 B、宣言の SHA-256 32 B |
| 0x0002 | 1 | ENTRY | proc0 の entry / 初期 MSP / ベクタ表、イメージの最小・最大アドレス |
| 0x0010 | 1 | OWNERSHIP | 件数 u32 + `{ kind u8, owner u8, id u16 }` の列。kind: 1 ピン / 2 ブロック / 3 アラーム / 4 バンク電圧 / 5 システム / 6 GPIO 割り込みバンク（id = 0、`gpio_irq` が空でないコアにだけ付く）。owner: 1 proc0 / 2 proc1 / 3 RP1 システム |
| 0x0011 | 1 | CLOCK | authority u8（1 rp1 / 2 linux）、clk_sys_hz u32 |
| 0x0012 | 0 | LOG | sink u8（1 uart0）、baudrate u32 |
| 0x0013 | 1 | BOOT | config.txt の SHA-256、cmdline.txt の SHA-256、本来のカーネルのファイル名（文字列） |
| 0x0020 | 1 | SHM_AREA | 24 bytes: DDR 合計サイズ u32（1 MiB 単位、1..16 MiB）、領域境界 u32（65536）、SRAM ハンドオフアドレス u32、SRAM 使用末尾 u16、予約 0 u16、DDR CPU 物理アドレス u64（STATIC: rpi-cm5 は 0x22000000）。旧 14-byte 形式は拒否 |
| 0x0021 | 1 | SHM_REGION | 1 件 1 レコード: kind u8、direction u8、core u8、notify u8、linux_map u8、writer u8、overflow u8（1 reject / 2 overwrite）、placement u8（1 reader-local / 2 linux）、DDR オフセット/長さ u32×2、SRAM オフセット/長さ u16×2、要素長 u32、depth u32、型 ID u64、名前（u8 長さ + UTF-8）、型名（同） |
| 0x0022 | 0 | LINK | 12 bytes: 版 u8 = 1、予約 u8 = 0、状態語のオフセット u16 = 0x40、コアごとに（proc0、proc1）ドアベルのピン u8（0..=27、宣言なしは 0xff）、host_irq の MSI-X vector u8（0 = 無し）、心拍の周期 ms u16（0 = 宣言なし） |
| 0x0023 | 0 | RPC | 論理名（u8 長さ + UTF-8）。RPC ごとに 1 件。対応する `-dn` と `-up` は SHM_REGION |

- 文字列は `u8 長さ + UTF-8`（最大 255）。
- id（ピン番号、ブロック番号）の割り当ては `rp1-proto` の enum が正本。番号は一度決めたら変えない。
- 型 ID: `#[derive(Message)]` が「型名・サイズ・各フィールドのオフセットと型」から計算する 64 bit のハッシュ。
  Linux 側の `fw.channel::<Cmd, Reply>("ctl")` はこれを note と照合し、違えば開けない。
- note に含めないもの: イメージのハッシュ（ローダーが自分で計算する）、Linux DT のノード名（ビルド時にボード表から overlay に焼き込む）。
- 宣言の SHA-256 は、正規化した宣言を TOML にシリアライズした結果から計算する。
- OWNERSHIP はコアごとにピン番号の昇順、タイマーの宣言順、インスタンス名の昇順で記録する。
- `gpio_irq` が空でないコアには OWNERSHIP の kind 6 の項目が加わる。旧版の note パーサーは kind 6 を拒否するため、ツールとファームを同じ版でビルドする。

### 2.3 読み手の動き

| 読み手 | 使うレコード | 失敗時 |
|---|---|---|
| ローダー | ENTRY、SHM_AREA（ハンドオフ）、OWNERSHIP（当てた後の DTB の検査、§3.2）。overlay 本体は `.rp1.dtbo` | 知らない critical レコード・CRC 不一致・DTB の印（§3.2）と不一致 → RP1 を起動せず停止（ログに理由） |
| Linux 側 `rp1-linux` | overlay が載せたコピー（`/rp1-firmware` ノードの `rp1,note` プロパティ）の SHM_*、LINK、RPC、FIRMWARE | 型 ID 不一致・名前が無い → `Err` |
| `cargo rp1 check --dtb` | OWNERSHIP、LOG、SHM_AREA/SHM_REGION、note 全体 | ローダーと同じコード（`rp1_dt::check::enforce`）で起動時の結果を予測して表示する（STATIC）。ローダーが停止する場合（保護ノード、`on_conflict = "refuse"`、DT の不備）は失敗。衝突によるノードの無効化は既定で失敗、`--allow-boot-disable` で警告。gpiomem 方針だけの無効化は表示だけ。予測が起動時と一致するのは、渡した DTB がファームの渡す DTB と同じときだけ（[build.md](build.md) §3）。衝突として数えるもの（§3.2 手順 3）: 有効なノードの `pinctrl-N` が `rp1_gpio` 配下のグループで RP1 所有ピンを設定、所有ブロックのノード（`rp1_uartN` 等）が有効・ラベルが無い、authority=rp1 なのに `rp1_clocks` が有効、`rp1-shm` 予約が SHM_AREA と違う・他の予約や `/memreserve/` と重なる、`rp1:<name>` の UIO が領域と違う、`/rp1-firmware rp1,note` が ELF の note と違う、有効な gpio-hog・`gpio(s)`/`*-gpio(s)` の利用者が `gpio-reserved-ranges` の線を使う |

### 2.4 実験的 link wake（NOTE v3）

**STATIC · 2026-11-27.** `[package.metadata.rp1]` で `schema = 2` と
`link_wake = "experimental"` を指定する。ファームの `rp1-rt` に Cargo feature
`experimental-link-wake` を有効にする。これで初期化が mailbox をゼロ化する。
v3 は `LINK` と `SHM_AREA` に加えて critical `LINK_WAKE`（type `0x0024`、flags 1）を要求する。
DT の `/rp1-firmware` に `rp1,abi = <3>` を置く。ローダーは有効な `rp1:irq` が
RP1 MSI-X vector 29 の edge であることも照合する。

| Byte offset | Type | `LINK_WAKE` value (16 B) |
|---|---|---|
| 0 | `u16` | wake ABI = 1 |
| 2 | `u16` | mailbox start = `0xc0` |
| 4 | `u16` | mailbox length = 64 |
| 6 | `u8` | vector owner（0 = proc0、1 = proc1） |
| 7 | `u8` | proc0 startup doorbell pin（0..27、無しは `0xff`） |
| 8 | `u32` | required caps |
| 12 | `u32` | offered caps |

capability の bit 0 / 1 / 2 / 3 は `LATCHED_WAKE` / `STARTUP_WAKE` /
`CREDIT_WAKE` / `RPC_REPLY_WAKE`。実験 profile は bit 0 と bit 3 を提供し、
bit 0 を必須とする。v3 で `rp1-to-linux` の `notify = "irq"` を使う
ring は vector owner が担当する。別コアの通知は owner への引渡しが要る。
SRAM の宣言領域は `0x100` から。mailbox、DDR header v2、ticket の手順は
[通信規約 §10](protocol.md#10-実験的-link-wakenote-v3) を参照。

## 3. Linux 用 DT overlay（`rp1-app.dtbo`）

Linux 側の調整はすべてビルド時に overlay として作る。ローダーは DT を書き換えない。

### 3.1 中身（宣言とボード表から生成）

| 断片 | 内容 | 元になる宣言 |
|---|---|---|
| 周辺・ピン | RP1 が持つ周辺のノードを `status = "disabled"`。RP1 のピンを GPIO コントローラーの `gpio-reserved-ranges` に入れ、libgpiod から要求できなくする | OWNERSHIP |
| クロック | clk-rp1 を無効化し、`/firmware/scmi`（`arm,scmi`、`mboxes = <&rp1_mbox 0>`、shmem は `&sram` 配下の `scmi@d7c0`）を足し、`rp1_uart0..5` の `clocks` を SCMI の uart(1)/uart_apb(0) に付け替える。同じ mailbox ch0 を使う標準の RP1 ファームのクライアント（`rp1_firmware`、`rp1_fw_shmem`）と、clk-rp1 か標準ファームでしか動かない有効ノード（`rp1_adc`/`rp1_eth`/`rp1_dma`/`rp1_pio`/`rp1_usb0`/`rp1_usb1`）を無効化。`pcie2` に `brcm,tperst-clk-ms = <20>`（PoC DT と同じ） | CLOCK（authority = rp1 のときだけ） |
| 予約ピンの利用者（ボード表） | 予約に入る線を使う標準ノード（`led_pwr` GPIO44、`cam0_reg` GPIO34）と RP1 の gpio-hog（`micclk1_hog`/`micdat1_hog`、GPIO46/48）を無効化 | ボード表（rpi-cm5） |
| OpenAMP | `/openamp-address-space` を `disabled`（標準 DTB には無いので、compatible の無い空の disabled ノードが残るだけ） | 常に |
| ログ | UART0 のノードを `disabled` | LOG |
| 共有メモリ | `reserved-memory`（no-map、連続）1 つと、shm ごとの `generic-uio` ノード（名前 `rp1:<name>`、領域、IRQ） | SHM_* |
| Linux 向けの note のコピー | `/rp1-firmware { rp1,note = [note のバイト列]; rp1,owned-nodes = ".."; }` | note 全体、OWNERSHIP |

- 外部参照はボード表が持つラベル（Raspberry Pi の DTB が持つ `__symbols__`）で書く。overlay は `-@` 付きでコンパイルする。
- 共有メモリの物理アドレスはビルド時に決める（ボード表の固定領域。現行の `rp1-ddr-scratch@21000000` と同じやり方）。
  これでローダーが確保・計算する必要がなくなる。
- **基準は標準 DTB**（2026-10-29、HW）: ファームが読む `bcm2712-rpi-cm5l-cm5io.dtb`（+ `bcm2712d0.dtbo`、config.txt の
  `dtoverlay=uart1-pi5`）にそのまま当てる。`device_tree=` は使わない。
- ホストで検証できる: `fdtoverlay` で標準 DTB に当て、`dtc` で読めること・参照が解決することを `cargo rp1 build` で確認する。

### 3.2 適用（ローダー）と検査

overlay は ELF の `.rp1.dtbo` セクション（非ロード）に入れ、ローダーが当てる（2026-10-23 変更。前案は VPU ファームの `dtoverlay=` で当て、
ローダーは照合だけだった）。理由:

- ELF と overlay が 1 つのファイルなので、組の食い違いが起きない（ハッシュ照合が不要）。
- ローダーは VPU ファームの後に動くので、最後に当てる側になる。`enable_uart=1` や `dtparam=uart0=on` で有効に戻されても上書きできる。
- config.txt に rp1 用の `dtoverlay=` を書かない。`kernel=` を標準カーネルに戻せば素の Raspberry Pi OS として起動する。

手順:

1. `.note.rp1` を検査する（CRC、知らない critical レコード）。
2. `.rp1.dtbo` を、渡された DTB（ファームの DTB に config.txt の `dtoverlay=` を当てたもの）に最後に当てる。実装は純 Rust の
   `rp1-dt`（`fdtoverlay` と同じ木になることをホストで試験）。参照先のラベルが無い（ボード表とカーネル系列が合わない）場合は停止する。
3. 当てた後の DTB で、RP1 所有のピン・ブロックを使う **Linux 側の有効なノード** を集める（`rp1_dt::check::resolve`）。理由になるもの:
   - `pinctrl-N` が RP1 所有のピンを含む rp1_gpio のグループを参照している（BCM2712 側の pinctrl は対象外）。ピンは `brcm,pins`、
     `pins`、`groups`（pinctrl-rp1 のグループは 1 ピンずつで名前は `gpio0`..`gpio53`）のすべてから読む。`gpioN` でないグループ名は
     確かめられないので拒否する
   - 同じグループの `function`（`uart3`、`i2c1`、`alt0` などの名前）が、RP1 所有ではないピンを RP1 所有インスタンスの信号に
     mux する（関数名とピンから `rp1_pinmux::linux_signal` で信号を引く）。同じ入力を複数のピンに選ぶと周辺は論理 OR を見るので
     （RP1 Peripherals §3.1、DOC）、Linux の別ノードが RP1 の I2C・UART・SPI の入力を乱せる。旧形式の数値 `brcm,function`
     （単一値・ピンごとの配列。Raspberry Pi OS の `spi1-1cs`・`pwm` など）は pinctrl-rp1 の `legacy_fsel_map` と同じ表
     （`rp1_pinmux::linux_legacy_function`）で関数名に直し、`function` と同じに扱う。表に無い値（そのピンに無い ALT、8 以上）は
     確かめられないものとして拒否する。以前は数値を翻訳せず拒否していたが、2026-11-17 の利用者の判断でこの扱いに替えた
   - `*-gpio(s)` が `gpio-reserved-ranges` の線を指す、予約線の `gpio-hog`
   - 割り込みの親が rp1_gpio の `interrupts`（親は Linux の `of_irq_find_parent` と同じ規則: 自分か祖先の `interrupt-parent` の
     先、または `#interrupt-cells` を持つ最も近い祖先）、`interrupts-extended` の rp1_gpio の項目が予約線を使う（pinctrl-rp1 は IRQ の
     割り当て時にそのピンのイベント設定を書き換える）
   - RP1 所有ブロックのノードが enabled のまま（後の overlay が有効に戻した）
   - `/dev/gpiomem0` のノード（`compatible = "raspberrypi,gpiomem"`、reg が RP1 の `0xc0_400d0000`）。`linux.gpiomem = "disable"`
     （既定）で、RP1 がピンを 1 本でも持つとき。衝突ではなく方針なので、`check --dtb` は表示だけ
   rp1_gpio を指す指定子は厳格に読み、壊れていればエラーで停止する。他のコントローラ宛ての読めない指定子は読み飛ばす（Linux も
   無視する）。ただし読めない部分に rp1_gpio の phandle 値が現れるときは拒否する。
4. 方針（`/rp1-firmware` の `rp1,on-conflict`、宣言の `linux.on_conflict`）で決める:
   - `"disable"`（既定）: 理由のあるノードを `status = "disabled"` にし、`rp1,loader-disabled`（空）を付け、
     `/rp1-firmware/rp1,loader-disabled` にパスを並べる。UART10 に 1 件ずつ
     `RP1BOOT_LINUX_DISABLED path=<path> label=<label|-> why=<理由>` と、最後に `RP1BOOT_LINUX_DISABLED_COUNT=<n>`（0 件でも）。
   - `"refuse"`: 理由が 1 件でもあれば停止する（gpiomem 方針だけの無効化は refuse でも行う）。
   - どちらでも、次のときは無効化せずに停止する: 保護ノード（ボード表の Linux 必須ブロック、rp1_gpio、rp1 バス、`pcie*`、
     `/reserved-memory` 配下、overlay 自身の rp1-* ノード、`/chosen/stdout-path` の先とそれぞれの祖先）に理由がある、
     コンソール（`stdout-path`）が RP1 所有のブロックを指している、gpiomem 方針以外の無効化候補があるのに `stdout-path` を解決できない。
   - 停止のときは理由ごとに `RP1BOOT_LINUX_REFUSED why=<理由>` を出し、`FAIL: ... RP1 not started` で止まる（RP1 も Linux も
     起動しない）。DT の不備・不明な方針値・再検査の失敗（エラー）も同じ形で止まる。
   - 無効化の後、元の DTB と比べた検査（予約が減っていない、所有ブロックが disabled、shm の予約）をもう一度行い、残れば停止する。
5. RP1 を起動し、共有メモリの物理アドレス（overlay と note の両方にある）を共有 SRAM のハンドオフ語に書く。
6. DTB を Linux に渡す。Linux からは `/proc/device-tree/rp1-firmware/rp1,loader-disabled` で無効化されたノードが分かる。

起動時は「止める」より「Linux のデバイスを 1 つ減らして印を出す」を既定にしている（2026-10-30 の決定）。config.txt の 1 行で装置が
起動しなくなるのを避けるためで、無効化は RP1 にも Linux の他の部分にも害が無い単調な操作。ビルドする人が見える場面
（`cargo rp1 check --dtb`、`RP1_BOARD_DTB`）では同じ予測で失敗にする（[build.md](build.md)）。

### 3.3 RP1 専用の IO（UART0 など）を Linux から守る範囲

| 経路 | 防げるか | 方法 |
|---|---|---|
| カーネルのドライバが probe する | 防ぐ | ローダーが最後に当てる overlay の `disabled` |
| 別のノードが pinctrl・`*-gpios`・hog・rp1_gpio の割り込みで同じピンを使う | 防ぐ（既定は起動時にそのノードを無効化して印、`refuse` なら停止。HW 2026-10-30） | ローダーの起動時の解決（§3.2） |
| libgpiod などがピンを要求する | 防ぐ（HW 2026-10-30: RP1 所有の GPIO 9/22 の sysfs export は拒否、Linux の GPIO4 は成功） | `gpio-reserved-ranges`（RP1 所有ピンちょうど） |
| 非 root（gpio グループ）が `/dev/gpiomem0` で IO_BANK0/PADS を直接書く | 防ぐ（既定。HW 2026-10-30: `/dev/gpiomem0` が無い） | `linux.gpiomem = "disable"` でローダーが gpiomem@d0000 を無効化。ページ単位の mmap なのでピン単位では守れない。`"keep"` で残せる |
| 共有クロック（clk_uart は UART0〜5 共通）を Linux が変える | 条件付き | `clock.authority = "rp1"` なら RP1 が SCMI で変更を拒否できる。`authority = "linux"` で UART0 を RP1、他の UART を Linux が持つ宣言はビルドエラー |
| 別のノードの pinctrl が、Linux のピンを RP1 所有インスタンスの信号に mux する（`function` 名。STATIC） | 防ぐ（同上） | ローダーの起動時の解決（§3.2） |
| 同上を旧形式の数値 `brcm,function` で行う | 防ぐ（同上。数値は pinctrl-rp1 の `legacy_fsel_map` で関数名に直す。STATIC） | ローダーの起動時の解決（§3.2）。RP1 所有ではないピンだけを使う数値指定は受け入れる。表に無い値はビルド時 `cargo rp1 check` で拒否し、起動時は `on_conflict` に関わらず停止（`RP1BOOT` が Refused） |
| root が `/dev/mem`・`pinctrl` ツールで直接書く、`uio_group` が UIO の `rp1:doorbell`（IO_BANK0 の 16 KiB）を書く | 防がない | DT では不可能（標準カーネルは `STRICT_DEVMEM` 無効）。型や宣言による保護はセキュリティ境界ではない（設計 §1.2） |

同じ考え方は UART0 以外の RP1 専用ブロック（SPI、I2C、PWM など）にも当てはまる。PWM のようにブロック単位で共有されるもの・
クロックを共有するものは、ボード表に「共有している相手」を持たせ、ビルド時に同じ検査をする。

### 3.4 注意



- overlay はノードの削除ができない（無効化と上書きだけ）。必要な調整はこの範囲に収める。
- overlay が参照する RP1 のノード名・ラベルはカーネルの系列で変わりうる（上流カーネルの RP1 ドライバは RP1 のノードを実行時に
  overlay として入れる方式）。ボード表は対応するカーネル系列ごとに持ち、合わなければ手順 2 で必ず停止する。

## 4. 起動一式（成果物）

`cargo rp1 build` は起動に必要なファイルを 1 つのディレクトリにまとめて出す。SD でも TFTP でも、この一式をそのまま置く。

```
target/rp1/boot/
  config.txt       生成。kernel=rp1boot.img、arm_64bit=1 など固定の行 + 宣言の config_extra
  cmdline.txt      生成。console、uio_pdrv_genirq.of_id=generic-uio など必須の引数 + 宣言の cmdline_extra
  rp1boot.img      ローダー
  rp1.elf          ファーム（.note.rp1、.rp1.dtbo 入り）
  （本来のカーネルと initramfs は利用者が用意。ファイル名は宣言の boot.kernel / boot.initramfs）
```

```toml
[package.metadata.rp1.boot]
kernel = "kernel_2712.img"              # 本来の Linux カーネル（config.txt の kernel= はローダーが使う）
initramfs = "initramfs_2712"            # 省略可
config_extra = ["dtparam=audio=off"]    # config.txt に足す行（rp1 の所有物と衝突する行はビルドエラー）
cmdline_extra = "quiet"                 # cmdline.txt に足す引数
```

**必ずこの一式を使う**: config.txt と cmdline.txt の SHA-256 を note の BOOT レコードに入れる。ローダーは自分が起動した経路
（SD / TFTP）から config.txt と cmdline.txt を読み直してハッシュを照合し、違えば RP1 を起動せず停止する
（「手で config.txt を書き換えた」「別の一式と混ざった」を止める）。
これで利用者の dtparam や別の overlay との衝突が原則起きなくなるので、ローダーの当てた後の検査（§3.2 手順 3）は保険として残す。

## 5. 決めたいこと（設計 B の論点）

1. 宣言は `Cargo.toml` + 1 行の `build.rs`（案）。build.rs を無くしたい場合は、linker script を固定にして proc-macro だけで型と note を作る方法もある（その場合、共有 SRAM の区画は宣言から変えられない）。
2. ボード表（既定の所有権、Linux 必須のもの、DT ラベル、共有メモリの固定領域）は `rp1-build` に同梱（案）。別 crate にすると build.rs から読みにくい。
3. Linux 側の調整はビルド時に生成する overlay（§3）で行い、ELF に入れてローダーが最後に当てる（2026-10-23 ユーザー指摘で変更）。
4. 番号（ピン・ブロック・型 ID）の安定性: `rp1-proto` の enum を正本にし、v0.x の間も番号は変えない（案）。

## PWM implementation (STATIC / HW 未検証)

Schema 2 accepts `pwm0 = { channel0 = 12, channel1 = 13, channel2 = 18, channel3 = 19 }`
on either core. All output pins are optional; `pwm1 = {}` selects an internal-only PWM1 block.
PWM1 has no bank-0 pin route, and GPIO28..53 remain Linux board lines. The resource is one
`pwm::Token<N>`; declared output pins remain in `ProcNPins` for explicit channel attachment.
The entire block, reset and clock are reserved, even for a single output. Another core or
`linux.keep` cannot claim another channel of the same block. See [PWM](pwm.md) for the API,
STATIC register evidence and proposed hardware checks.
