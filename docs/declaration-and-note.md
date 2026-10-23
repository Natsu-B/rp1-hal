# 宣言と `.note.rp1` v2（設計 B、2026-10-23 草案）

ファームが「何を持ち、Linux と何を共有するか」を、ビルド前に 1 か所で宣言する。
ビルドはそこから Rust の型・linker script・`.note.rp1` を作り、ローダーと Linux 側は note だけを読む。

```
firmware/Cargo.toml [package.metadata.rp1]
        │  build.rs（1 行: rp1_build::configure()）
        ├─▶ OUT_DIR/rp1.rs        … コアごとの Resources 型、shm ハンドル、cfg、note のバイト列
        ├─▶ OUT_DIR/rp1-layout.x  … 共有 SRAM 区画・ISRAM/DSRAM・予約の linker script
        ├─▶ OUT_DIR/rp1-app.dtbo  … Linux 用の DT overlay（§3）
        └─▶ cargo:rustc-cfg=...   … rp1_log など
cargo rp1 build（リンク後）: 到達解析・ISRAM 配置・ELF 検査（設計 E）。rp1.elf と rp1-app.dtbo を出力
VPU ファーム: config.txt の dtoverlay=rp1-app で overlay を適用し、適用済み DTB をローダーに渡す
ローダー:   ELF の .note.rp1 を読む → DTB の印と照合 → RP1 起動 → DTB はそのまま Linux へ（書き換えない）
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
| クロックを共有する相手を Linux が持ち、`clock.authority = "linux"`（例: UART0 を RP1、UART1 を Linux） | `UART0 shares clk_uart with UART1 (linux); set clock.authority = "rp1"` |
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
- note に含めないもの: イメージのハッシュ（ローダーが自分で計算する）、Linux DT のノード名（ビルド時にボード表から overlay に焼き込む）。

### 2.3 読み手の動き

| 読み手 | 使うレコード | 失敗時 |
|---|---|---|
| ローダー | ENTRY、SHM_AREA（ハンドオフ）、note 全体の SHA-256、OWNERSHIP（最終 DTB の検査、§3.2） | 知らない critical レコード・CRC 不一致・DTB の印（§3.2）と不一致 → RP1 を起動せず停止（ログに理由） |
| Linux 側 `rp1-linux` | overlay が載せたコピー（`/rp1-firmware` ノードの `rp1,note` プロパティ）の SHM_*、FIRMWARE | 型 ID 不一致・名前が無い → `Err` |
| `cargo rp1 check --dtb` | OWNERSHIP、LOG | Linux DT との衝突を一覧にして失敗 |

## 3. Linux 用 DT overlay（`rp1-app.dtbo`）

Linux 側の調整はすべてビルド時に overlay として作る。ローダーは DT を書き換えない。

### 3.1 中身（宣言とボード表から生成）

| 断片 | 内容 | 元になる宣言 |
|---|---|---|
| 周辺・ピン | RP1 が持つ周辺のノードを `status = "disabled"`。RP1 のピンを GPIO コントローラーの `gpio-reserved-ranges` に入れ、libgpiod から要求できなくする | OWNERSHIP |
| クロック | clk-rp1 を無効化し、`arm,scmi`（rp1 mailbox + shmem）を足し、各利用者ノードの `clocks` を SCMI 側に付け替える | CLOCK（authority = rp1 のときだけ） |
| ログ | UART0 のノードを `disabled` | LOG |
| 共有メモリ | `reserved-memory`（no-map、連続）1 つと、shm ごとの `generic-uio` ノード（名前 `rp1:<name>`、領域、IRQ） | SHM_* |
| 照合用の印 | `/rp1-firmware { rp1,note = [note のバイト列]; rp1,note-sha256 = [..]; }` | note 全体 |

- 外部参照はボード表が持つラベル（Raspberry Pi の DTB が持つ `__symbols__`）で書く。overlay は `-@` 付きでコンパイルする。
- 共有メモリの物理アドレスはビルド時に決める（ボード表の固定領域。現行の `rp1-ddr-scratch@21000000` と同じやり方）。
  これでローダーが確保・計算する必要がなくなる。
- ホストで検証できる: `fdtoverlay` で標準 DTB に当て、`dtc` で読めること・参照が解決することを `cargo rp1 build` で確認する。

### 3.2 適用と照合

1. `cargo rp1 deploy` が `rp1.elf` を起動元のルートに、`rp1-app.dtbo` を `overlays/` に置く。config.txt には一度だけ
   `kernel=rp1boot.img` と `dtoverlay=rp1-app` を書く（ファイル名は固定、中身だけ差し替える）。
2. VPU ファームが起動時に overlay を適用する。SD でもネットブート（TFTP）でも同じ（ファームの通常機能）。
3. ローダーは ELF の note の SHA-256 を計算し、渡された DTB の `/rp1-firmware/rp1,note-sha256` と比べる。
   - 一致: RP1 を起動し、DTB はそのまま Linux に渡す。
   - 不一致・印が無い: overlay と ELF の組が違う。RP1 を起動せず、理由をログに出して停止する（古い overlay で Linux が起動して RP1 と食い違うのを防ぐ）。
4. 共有メモリの物理アドレスは overlay と note の両方に入っている。ローダーはそれを共有 SRAM のハンドオフ語に書くだけ。
5. **最終 DTB の所有権検査（読み取りのみ）**: config.txt の他の設定（`enable_uart=1`、`dtparam=uart0=on`、後から当てる別の overlay）が
   RP1 の周辺を有効に戻していないかを確かめる。次のどれかに当たれば RP1 を起動せず停止し、どの設定が衝突したかをログに出す。
   - OWNERSHIP の周辺に対応するノードが `status = "okay"`（ノードのパスは overlay の `/rp1-firmware/rp1,owned-nodes` に列挙しておく）
   - 有効なノードの `pinctrl-*` が RP1 所有のピンを含むグループを参照している
   - `gpio-reserved-ranges` から RP1 のピンが外れている

### 3.3 RP1 専用の IO（UART0 など）を Linux から守る範囲

| 経路 | 防げるか | 方法 |
|---|---|---|
| カーネルのドライバが probe する | 防ぐ | overlay の `disabled` + ローダーの最終 DTB 検査 |
| 別のノードが pinctrl で同じピンを取る | 防ぐ | ローダーの最終 DTB 検査 |
| libgpiod などがピンを要求する | 防ぐ（RP1 の GPIO ドライバで効くかは未確認） | `gpio-reserved-ranges` |
| 共有クロック（clk_uart は UART0〜5 共通）を Linux が変える | 条件付き | `clock.authority = "rp1"` なら RP1 が SCMI で変更を拒否できる。`authority = "linux"` で UART0 を RP1、他の UART を Linux が持つ宣言はビルドエラー |
| root が `/dev/mem`・`/dev/gpiomem0`・`pinctrl` ツールで直接書く | 防がない | DT では不可能（標準カーネルは `STRICT_DEVMEM` 無効）。型や宣言による保護はセキュリティ境界ではない（設計 §1.2） |

同じ考え方は UART0 以外の RP1 専用ブロック（SPI、I2C、PWM など）にも当てはまる。PWM のようにブロック単位で共有されるもの・
クロックを共有するものは、ボード表に「共有している相手」を持たせ、ビルド時に同じ検査をする。

### 3.4 注意



- overlay はノードの削除ができない（無効化と上書きだけ）。必要な調整はこの範囲に収める。
- `dtoverlay=rp1-app` を残したままローダーを外す（`kernel=` を標準カーネルに戻す）と、RP1 ファームが無いのに Linux は SCMI を待つ。
  2 行は必ずセットで扱う（`cargo rp1 deploy` が両方を管理する）。
- 標準ファームの overlay 適用でできないこと（サイズ上限、適用順序など）は未確認。TODO に入れる。

## 4. 決めたいこと（設計 B の論点）

1. 宣言は `Cargo.toml` + 1 行の `build.rs`（案）。build.rs を無くしたい場合は、linker script を固定にして proc-macro だけで型と note を作る方法もある（その場合、共有 SRAM の区画は宣言から変えられない）。
2. ボード表（既定の所有権、Linux 必須のもの、DT ラベル、共有メモリの固定領域）は `rp1-build` に同梱（案）。別 crate にすると build.rs から読みにくい。
3. Linux 側の調整はビルド時に生成する overlay（§3）で行い、VPU ファームの `dtoverlay=` で当てる。ローダーは照合だけ（ユーザー提案を発展、2026-10-23）。
4. 番号（ピン・ブロック・型 ID）の安定性: `rp1-proto` の enum を正本にし、v0.x の間も番号は変えない（案）。
