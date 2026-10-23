# 宣言と `.note.rp1` v2（設計 B、2026-10-23 草案）

ファームが「何を持ち、Linux と何を共有するか」を、ビルド前に 1 か所で宣言する。
ビルドはそこから Rust の型・linker script・`.note.rp1` を作り、ローダーと Linux 側は note だけを読む。

```
firmware/Cargo.toml [package.metadata.rp1]
        │  build.rs（1 行: rp1_build::configure()）
        ├─▶ OUT_DIR/rp1.rs        … コアごとの Resources 型、shm ハンドル、cfg、note のバイト列
        ├─▶ OUT_DIR/rp1-layout.x  … 共有 SRAM 区画・ISRAM/DSRAM・予約の linker script
        └─▶ cargo:rustc-cfg=...   … rp1_log など
cargo rp1 build（リンク後）: 到達解析・ISRAM 配置・ELF 検査（設計 E）
ローダー: ELF の .note.rp1 を読む → RP1 起動、DT patch。note のコピーを DT に載せる
Linux:   /proc/device-tree の note のコピーと UIO ノードを読む
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
schema = 1                                # 宣言の版。build.rs が知らない版ならエラー
board = "rpi-cm5"                         # ボード表（既定の所有権・ピン・DT ラベル）。rp1-build に同梱
clock = { authority = "rp1", clk_sys_hz = 200_000_000 }  # authority: "rp1" | "linux"
log = { sink = "uart0", baudrate = 115_200 }             # 省略すると log! はコンパイルエラー
stack = { proc0 = "2KiB", proc1 = "2KiB" }               # 省略時は各 2 KiB（DSRAM）

[package.metadata.rp1.proc0]
peripherals = ["UART2", "TIMER.ALARM0"]   # 語彙は §1.3
pins = [4, 5]                             # GPIO 番号
gpio_irq = []                             # 割り込みを受けるピン（pins の部分集合）

[package.metadata.rp1.proc1]
peripherals = ["TIMER.ALARM1"]
pins = [22, 27]
gpio_irq = [27]

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
```

### 1.3 語彙（所有の単位）

| 種類 | 名前の例 | 単位 | 付いてくるもの |
|---|---|---|---|
| ブロック | `UART0`..`UART5`、`SPI0`..`SPI8`、`I2C0`..`I2C6`、`PWM0`、`PWM1`、`I2S0`..`I2S2`、`ADC`、`DMA`、`PIO` | ブロック全体（設計 §6.1。PWM はチャネル分割しない） | IRQ 線、リセットビット、クロック、ラッパー `+0x108` |
| ブロックの一部 | `TIMER.ALARM0`..`TIMER.ALARM3` | アラーム単位。タイマーの計数値は全員が読める | 該当 IRQ（26..29） |
| ピン | `pins = [..]` | ピン単位（funcsel、pad、SYS_RIO、イベント選択） | — |
| バンク | 自動 | ピンを 1 本でも持つと、そのバンクの電圧設定は読むだけ。書くには `bank_voltage = [0]` を明示 | — |
| システム | `log`、`clock.authority = "rp1"` | RP1 全体（コアに属さない） | UART0 / クロック生成器 |

PCIe、mailbox ch0（SCMI 用）、共有 SRAM の予約域は、ボード表が「RP1 システム」として固定で持ち、宣言はできない。

### 1.4 ビルド時のエラー（build.rs が止める）

| 条件 | メッセージの例 |
|---|---|
| `schema` が未知、知らないキー | `unknown key 'peripheral' in [package.metadata.rp1.proc0] (did you mean 'peripherals'?)` |
| 語彙に無い名前 | `unknown peripheral 'UART9'` |
| 同じピン・ブロックを 2 か所で宣言（proc0 と proc1、log と core など） | `UART0 is claimed by both log and proc0` |
| ボード表で Linux 必須のもの（Ethernet の GEM、USB、SD など）を宣言 | `GEM is required by Linux on rpi-cm5` |
| `log` があるのに UART0 が RP1 の所有になれない（ボード表で Linux 必須） | `log sink uart0 requires UART0 to be owned by RP1` |
| `gpio_irq` が `pins` に無い | `gpio_irq 27 is not in proc1.pins` |
| `clock.authority = "linux"` なのに `clk_sys_hz` がある | `clk_sys_hz requires clock.authority = "rp1"` |
| shm の名前が重複・形式違反、depth が 2 の冪でない、kind と項目が合わない | `shm 'ctl': 'direction' is only valid for kind = "ring"` |
| Linux → RP1 のリングが共有 SRAM の予算を超える | `shm 'cmds': linux-to-rp1 ring needs 8.0 KiB of shared SRAM, 2.5 KiB left` |
| 型が存在しない、`Message` を実装していない | rustc のエラー（生成コードがその型を参照するため） |

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
| 0x0020 | 1 | SHM_AREA | DDR 側の合計サイズ u32、境界 u32、共有 SRAM のハンドオフ語のアドレス u32、SRAM 側の合計 u16 |
| 0x0021 | 1 | SHM_REGION | 1 件 1 レコード: kind u8、direction u8、core u8、notify u8、linux_map u8、writer u8、DDR オフセット/長さ u32×2、SRAM オフセット/長さ u16×2、要素長 u32、depth u32、型 ID u64（要求/応答で 2 つ）、名前（u8 長さ + UTF-8）、型名（同） |

- 文字列は `u8 長さ + UTF-8`（最大 255）。
- id（ピン番号、ブロック番号）の割り当ては `rp1-proto` の enum が正本。番号は一度決めたら変えない。
- 型 ID: `#[derive(Message)]` が「型名・サイズ・各フィールドのオフセットと型」から計算する 64 bit のハッシュ。
  Linux 側の `fw.channel::<Cmd, Reply>("ctl")` はこれを note と照合し、違えば開けない。
- note に含めないもの: イメージのハッシュ（ローダーが自分で計算する）、Linux DT のノード名（ローダーのボード表が持つ）。

### 2.3 読み手の動き

| 読み手 | 使うレコード | 失敗時 |
|---|---|---|
| ローダー | ENTRY、OWNERSHIP、CLOCK、LOG、SHM_* | 知らない critical レコード・CRC 不一致・ボード表と矛盾 → RP1 を起動せず停止（ログに理由） |
| ローダーの DT patch | OWNERSHIP（周辺・ピンを `disabled`、pinctrl から外す）、CLOCK（SCMI へ差し替え）、SHM_*（reserved-memory と UIO ノード） | 同上 |
| Linux 側 `rp1-linux` | DT に載ったコピー（`/rp1-firmware` ノードの `rp1,note` プロパティ）の SHM_*、FIRMWARE | 型 ID 不一致・名前が無い → `Err` |
| `cargo rp1 check --dtb` | OWNERSHIP、LOG | Linux DT との衝突を一覧にして失敗 |

## 3. 決めたいこと（設計 B の論点）

1. 宣言は `Cargo.toml` + 1 行の `build.rs`（案）。build.rs を無くしたい場合は、linker script を固定にして proc-macro だけで型と note を作る方法もある（その場合、共有 SRAM の区画は宣言から変えられない）。
2. ボード表（既定の所有権、Linux 必須のもの、DT ラベル）は `rp1-build` に同梱（案）。別 crate にすると build.rs から読みにくい。
3. Linux 側は note を DT 経由で受け取る（案）。ELF を Linux 側に置いて読む方法より、ローダーが実際に使った値と必ず一致する。
4. 番号（ピン・ブロック・型 ID）の安定性: `rp1-proto` の enum を正本にし、v0.x の間も番号は変えない（案）。
