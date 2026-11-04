# 宣言と `.note.rp1` v2（設計 B、2026-10-23 草案）

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
schema = 2                                # 宣言の版（1 は読み続けるが非推奨。§1.2.1）
board = "rpi-cm5"                         # ボード表（Linux に残すピン・ブロック、DT ラベル）。rp1-build に同梱
clock = { authority = "rp1", clk_sys_hz = 200_000_000 }  # authority: "rp1" | "linux"
log = { sink = "uart0", baudrate = 115_200 }             # = システム所有の uart0 { tx = 14, rx = 15 }。省略すると log! はコンパイルエラー
stack = { proc0 = "2KiB", proc1 = "2KiB" }               # 省略時は各 2 KiB（DSRAM）
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
spi0 = { sck = 11, mosi = 10, miso = 9 }  # ハードの csN は未対応（エラー）
host_irq = true                           # 不安定: RP1 → Linux 割り込み（rp1-hal の unstable-host-irq、design.md §6.2）

[[package.metadata.rp1.shm]]
name = "ctl"                              # [a-z][a-z0-9_]{0,23}。UIO 名は rp1:<name>
kind = "channel"                          # "ring" | "channel" | "block"
core = "proc1"                            # M3 側でハンドルを受け取るコア
message = "protocol::Cmd"                 # Linux → M3（channel）/ リングの要素（ring）
reply = "protocol::Reply"                 # M3 → Linux（channel のみ）
depth = 16                                # 要素数（2 の冪）
notify = "irq"                            # "irq" | "poll"。Linux → M3 の通知（channel と linux-to-rp1 の ring）

[[package.metadata.rp1.shm]]
name = "samples"
kind = "ring"
direction = "rp1-to-linux"                # ring のみ: "rp1-to-linux" | "linux-to-rp1"
overflow = "overwrite"                    # ring のみ: "reject"（既定）| "overwrite"
placement = "reader-local"                # ring のみ: "reader-local"（既定）| "linux"（v0.1 未実装）
core = "proc1"
message = "protocol::Sample"
depth = 4096

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
- インスタンスのキー: `uart0`..`uart4`、`i2c0`..`i2c3`、`spi0`/`spi1`/`spi3`/`spi5`、`adc`。信号名: UART `tx rx`、
  I2C `sda scl`、SPI `sck mosi miso`（すべて必須）。UART の `cts`/`rts` は RP1 側のフロー制御が未実装なので、`proc0`/`proc1` では
  E4（`UART flow control is not supported yet`）。そのピンを GPIO として使うなら `gpio` に書く。`linux.keep` では Linux のフロー制御
  ピンとして書ける（機能表で検査し、保護する）。
- ピンは「インスタンスの信号への割り当て」から導き、bank 0 の機能表（`rp1-pinmux`、datasheet・pinctrl-rp1・HAL の表と照合）で
  検査する。生成される resources はインスタンスを型付きの `Parts`（例 `i2c::Parts<1, 2, 3>`）で渡し、表に無い組はコンパイルできない。
- `linux.keep` はピンの保護だけ。Linux 側ノードの有効化は config.txt の `dtoverlay=`（例 `uart1-pi5`）に任せる。
  明示した `keep` はボード既定を置き換える（既定への追加ではない）。既定の UART1 を残したまま追加するなら、同じ table に `uart1` も書く
  （例 `keep = { uart1 = { tx = 0, rx = 1 }, i2c1 = { sda = 2, scl = 3 } }`）。`keep` から `uart1` を外すと GPIO 0/1 を RP1 に渡せる
  （ビルド時に警告）。

HW（2026-10-30、lab ファーム = UART0（log）+ I2C1 + SPI0 + ADC を RP1、UART1 を Linux、cold boot ×3）: Linux の
`gpio-reserved-ranges` が所有ピンちょうど `<2 2 8 4 14 2 22 1 27 1>`（I2C1 2/3、SPI0 9..11 と CS 8、log 14/15、GPIO 22、27）になり、
既存の回帰（shm・doorbell・I2C・SPI・ADC・SCMI）が全て合格。同じ起動で Linux の UART1（ttyAMA1、GPIO 0/1）がホストとの
送受信の両方向で合格し、その間も RP1 の UART0 ログは出続けた。

#### 1.2.1 schema 1 からの移行

- `schema = 1`（`pins = [..]`、`peripherals = ["I2C1", "TIMER.ALARM0", ..]`）は読み続け、ビルド時に schema 2 へ変換する
  （`cargo:warning=schema 1 is deprecated`）。UART/I2C/SPI の割り当ては、そのコアの `pins` から機能表で一意に決まるものを探し、
  残りのピンは `gpio` になる。決まらなければエラー（`schema 1: cannot infer I2C1 pins from [2, 3, 10, 11]; write i2c1 = { .. } (schema 2)`）。
- 変換しても note は同一（旧来の宣言ハッシュを保つ）。schema 2 に書き換えると宣言の SHA-256（とそれを覆うヘッダーの CRC）は必ず
  変わる。ほかは、schema 1 の `pins`・`peripherals` の並びが schema 2 の並び（ピンは昇順、ブロックはタイマーを書いた順の後にインスタンスを名前順）と同じときだけ
  同一。違う並び（例 `pins = [7, 6, 5, 4, 22]`、`peripherals = ["UART2", "TIMER.ALARM1", "ADC"]`）では、所有の集合は同じでも
  OWNERSHIP の並びが正規化されてバイト列が変わる。lab の `shm-firmware` と `examples/dual-core` は schema 2 に移した（STATIC: この 2 つは
  凍結した schema 1 の note と、ハッシュと CRC 以外が一致することを `rp1-build` の試験で確認）。
- 変わる挙動（BREAKING）: `gpio-reserved-ranges` は以前の「GPIO 2..27 全部」から「RP1 所有ピンちょうど」に縮む。宣言外の
  2..27 は Linux が使える。

### 1.3 語彙（所有の単位）

| 種類 | 名前の例 | 単位 | 付いてくるもの |
|---|---|---|---|
| インスタンス | `uart0`..`uart4`、`i2c0`..`i2c3`、`spi0`/`spi1`/`spi3`/`spi5`、`adc` | インスタンス全体。信号ごとのピン | IRQ 線、リセットビット、クロック、ラッパー `+0x108`、割り当てたピン |
| ブロックの一部 | `timer = ["ALARM0".."ALARM3"]` | アラーム単位。タイマーの計数値は全員が読める | 該当 IRQ（26..29） |
| ピン | `gpio = [..]`、`gpio_irq = [..]` | ピン単位（funcsel、pad、SYS_RIO、イベント選択） | — |
| RP1 → Linux 割り込み（不安定） | `host_irq = true`（1 コアだけ） | MSI-X vector 29 の `MSIX_CFG.TEST`（`TIMER.ALARM3` も所有） | `HostIrq`（rp1-hal の feature `unstable-host-irq`）、overlay の `rp1:irq` UIO |
| バンク | 自動 | ピンを 1 本でも持つと、そのバンクの電圧設定は読むだけ | — |
| システム | `log`、`clock.authority = "rp1"` | RP1 全体（コアに属さない） | UART0 と GPIO 14/15 / クロック生成器 |
| Linux | `linux.keep`（省略時はボード既定。書くと既定を置き換える） | インスタンスとピン | 宣言できない（E3/E4） |

PCIe、mailbox ch0（SCMI 用）、共有 SRAM の予約域は、ボード表が「RP1 システム」として固定で持ち、宣言はできない。
rpi-cm5 のボード表は GPIO 28..53（ETH_RST_N、VBUS など）と Ethernet・USB・DMA・PCIe・PIO を Linux に残す。この「Linux に要る
ブロック」の表は、ビルドの宣言検査とローダーの保護ノード（§3.2）が共有する。

### 1.4 ビルド時のエラー（build.rs が止める）

| # | 条件 | メッセージの例 |
|---|---|---|
| E0 | `schema` が未知、知らないキー・インスタンス、形式違反 | `unknown key 'uart9' in proc1 (instances: uart0, .., adc)` |
| E1 | 同じピンを 2 回（コア間、機能と `gpio`、log と機能、`linux.keep` と RP1） | `GPIO 10 is claimed by both proc1.spi0.mosi and proc0.gpio` |
| E2 | ピンがその信号を出せない・bank 0 にピンが無い | `proc0.uart2.tx = 6: GPIO6 is UART2_CTS (alt2), not UART2_TX; UART2_TX is on GPIO4` / `SPI6 has no bank-0 pins` |
| E3 | 同じインスタンスを 2 回（proc0 と proc1、log と core、RP1 と `linux.keep`）、`host_irq` を両コアで | `I2C1 is declared by both proc0 and proc1` |
| E4 | ボードが Linux に要るピン・ブロック、未対応のインスタンス | `GPIO 32 is board line ETH_RST_N, owned by Linux on rpi-cm5` / `GPIO 0 is kept for Linux (uart1.tx) by board rpi-cm5` / `ETH is required by Linux on rpi-cm5` / `SPI4 is slave-only` / `proc0.uart2.cts: UART flow control is not supported yet` |
| E5 | 必須信号の欠落、`gpio_irq` が `gpio` に無い | `proc1.spi0 needs sck, mosi, miso` / `gpio_irq 27 is not in proc1.gpio` |
| E6 | 環境変数 `RP1_BOARD_DTB` があり、その DTB（config.txt の overlay を当てたもの）で起動時に無効化・拒否が起きる | `RP1_BOARD_DTB <file>: rp1boot would disable 1 Linux node(s) ...`（gpiomem 方針だけなら警告） |
| — | `log` があるのに UART0 が RP1 の所有になれない | `log sink uart0 requires UART0 to be owned by RP1` |
| — | v0.1 の rpi-cm5 で `clock.authority = "linux"` | `clock.authority = linux is unsupported on the v0.1 rpi-cm5 board; use rp1` |
| — | ring の `notify` が `poll` 以外（hwirq58 は SCMI の mailbox が持つ） | `v0.1 supports only notify=poll (SCMI owns hwirq58)` |
| — | shm の名前が重複・形式違反、depth が 2 の冪でない、kind と項目が合わない | `shm 'ctl': 'direction' is only valid for kind = "ring"` |
| — | Linux → RP1 のリングが共有 SRAM の予算を超える | `shm 'cmds': linux-to-rp1 ring needs 8.0 KiB of shared SRAM, 2.5 KiB left` |
| — | 型が存在しない、`Message` を実装していない | rustc のエラー（生成コードがその型を参照するため） |

E1〜E5 は `rp1-build` の単体試験で 1 件ずつ固定している（STATIC）。config.txt の `dtoverlay=` はビルドから見えないので、
Linux 側の衝突は `cargo rp1 check --dtb`（E6 と同じ予測、[build.md](build.md)）か、起動時の rp1boot（§3.2）で扱う。

### 1.5 生成されるもの（`OUT_DIR/rp1.rs`）

```rust
pub mod rp1 {
    pub struct Proc0Resources {
        pub pins: hal::gpio::Pins0,                 // 4, 5 だけを持つ型
        pub UART2: hal::pac::UART2,
        pub TIMER_ALARM0: hal::timer::AlarmToken<0>,
        pub proc1: rp1_rt::Proc1Launcher,           // proc1 を起動する権利（proc0 だけが持つ）
        pub clock_authority: hal::clocks::Authority,// clock.authority = "rp1" のときだけ
        pub log: rp1_rt::log::Uninit,               // log があるときだけ（proc0 が初期化）
        pub shm: Proc0Shm,                          // core = "proc0" の shm だけ
    }
    pub struct Proc1Resources { /* 同様 */ }
    pub struct Proc0Shm { pub trace: rp1_link::BlockPending<protocol::Trace> }
    pub struct Proc1Shm {
        pub ctl: rp1_link::ChannelPending<protocol::Cmd, protocol::Reply>,
        pub samples: rp1_link::RingTxPending<protocol::Sample>,
    }
    #[used] #[unsafe(link_section = ".note.rp1")]
    static NOTE: [u8; 412] = [/* §2 の形式 */];
}
```

`#[entry(proc0)]` の関数は `rp1::Proc0Resources` を受け取る（api-sketch の `Resources<Proc0>` はこの名前に置き換える）。
`Shm::enable` は `*Pending` を受け取り、PCIe のリンクと ATU が有効なら使えるハンドルに変える。

## 2. `.note.rp1` v2 の形式

ELF の note（`SHT_NOTE`、非ロード）。name = `"RP1\0"`、type = `0x5250_3102`（v1 は `0x5250_3101`）。
すべてリトルエンディアン。desc は 4 バイト境界。

### 2.1 ヘッダー（32 バイト）

| オフセット | 型 | 内容 |
|---|---|---|
| 0 | `[u8; 8]` | `"RP1NOTE\0"` |
| 8 | `u16` | ABI 版 = 2 |
| 10 | `u16` | ヘッダー長 = 32 |
| 12 | `u32` | desc 全体の長さ |
| 16 | `u32` | flags（予約、0） |
| 20 | `u32` | CRC32（ヘッダーの後ろからの全レコード） |
| 24 | `u32` | 宣言の schema 版 |
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
| 0x0010 | 1 | OWNERSHIP | 件数 u32 + `{ kind u8, owner u8, id u16 }` の列。kind: 1 ピン / 2 ブロック / 3 アラーム / 4 バンク電圧 / 5 システム。owner: 1 proc0 / 2 proc1 / 3 RP1 システム |
| 0x0011 | 1 | CLOCK | authority u8（1 rp1 / 2 linux）、clk_sys_hz u32 |
| 0x0012 | 0 | LOG | sink u8（1 uart0）、baudrate u32 |
| 0x0013 | 1 | BOOT | config.txt の SHA-256、cmdline.txt の SHA-256、本来のカーネルのファイル名（文字列） |
| 0x0020 | 1 | SHM_AREA | 24 bytes: DDR 合計サイズ u32（1 MiB 単位、1..16 MiB）、領域境界 u32（65536）、SRAM ハンドオフアドレス u32、SRAM 使用末尾 u16、予約 0 u16、DDR CPU 物理アドレス u64（STATIC: rpi-cm5 は 0x22000000）。旧 14-byte 形式は拒否 |
| 0x0021 | 1 | SHM_REGION | 1 件 1 レコード: kind u8、direction u8、core u8、notify u8、linux_map u8、writer u8、overflow u8（1 reject / 2 overwrite）、placement u8（1 reader-local / 2 linux）、DDR オフセット/長さ u32×2、SRAM オフセット/長さ u16×2、要素長 u32、depth u32、型 ID u64（要求/応答で 2 つ）、名前（u8 長さ + UTF-8）、型名（同） |

- 文字列は `u8 長さ + UTF-8`（最大 255）。
- id（ピン番号、ブロック番号）の割り当ては `rp1-proto` の enum が正本。番号は一度決めたら変えない。
- 型 ID: `#[derive(Message)]` が「型名・サイズ・各フィールドのオフセットと型」から計算する 64 bit のハッシュ。
  Linux 側の `fw.channel::<Cmd, Reply>("ctl")` はこれを note と照合し、違えば開けない。
- note に含めないもの: イメージのハッシュ（ローダーが自分で計算する）、Linux DT のノード名（ビルド時にボード表から overlay に焼き込む）。
- 宣言の SHA-256 は宣言の書き方で決まる。`schema = 1` のまま読む（ビルド時に schema 2 へ自動変換）と旧来のハッシュを保つので note は変わらない。schema 2 に書き換えると、note のこのハッシュ（と、それを覆うヘッダーの CRC）は必ず変わる。ほかが同一なのは schema 1 の並びが schema 2 の正規の並びと同じときだけで、そうでなければ OWNERSHIP の並びも変わる（§1.2.1。lab の 2 宣言はハッシュと CRC 以外が同一: STATIC、`rp1-build` の試験）。

### 2.3 読み手の動き

| 読み手 | 使うレコード | 失敗時 |
|---|---|---|
| ローダー | ENTRY、SHM_AREA（ハンドオフ）、OWNERSHIP（当てた後の DTB の検査、§3.2）。overlay 本体は `.rp1.dtbo` | 知らない critical レコード・CRC 不一致・DTB の印（§3.2）と不一致 → RP1 を起動せず停止（ログに理由） |
| Linux 側 `rp1-linux` | overlay が載せたコピー（`/rp1-firmware` ノードの `rp1,note` プロパティ）の SHM_*、FIRMWARE | 型 ID 不一致・名前が無い → `Err` |
| `cargo rp1 check --dtb` | OWNERSHIP、LOG、SHM_AREA/SHM_REGION、note 全体 | ローダーと同じコード（`rp1_dt::check::enforce`）で起動時の結果を予測して表示する（STATIC）。ローダーが停止する場合（保護ノード、`on_conflict = "refuse"`、DT の不備）は失敗。衝突によるノードの無効化は既定で失敗、`--allow-boot-disable` で警告。gpiomem 方針だけの無効化は表示だけ。予測が起動時と一致するのは、渡した DTB がファームの渡す DTB と同じときだけ（[build.md](build.md) §3）。衝突として数えるもの（§3.2 手順 3）: 有効なノードの `pinctrl-N` が `rp1_gpio` 配下のグループで RP1 所有ピンを設定、所有ブロックのノード（`rp1_uartN` 等）が有効・ラベルが無い、authority=rp1 なのに `rp1_clocks` が有効、`rp1-shm` 予約が SHM_AREA と違う・他の予約や `/memreserve/` と重なる、`rp1:<name>` の UIO が領域と違う、`/rp1-firmware rp1,note` が ELF の note と違う、有効な gpio-hog・`gpio(s)`/`*-gpio(s)` の利用者が `gpio-reserved-ranges` の線を使う |

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
     （単一値・ピンごとの配列）は信号を引かず、RP1 所有ではないピンに使われていれば確かめられないものとして拒否する
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
| 同上を旧形式の数値 `brcm,function` で行う | 防ぐ（保守的。数値は翻訳せず、RP1 所有ではないピンへの使用を確かめられないものとして拒否。STATIC） | ビルド時 `cargo rp1 check` の拒否。起動時は `on_conflict` に関わらず停止（`RP1BOOT` が Refused。Linux 側だけを無効化する方式は未実装）。RP1 所有ピンへの数値指定は通常の衝突として扱い、既定では無効化 |
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
