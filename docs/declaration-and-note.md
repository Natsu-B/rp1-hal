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
schema = 1                                # 宣言の版。build.rs が知らない版ならエラー
board = "rpi-cm5"                         # ボード表（既定の所有権・ピン・DT ラベル）。rp1-build に同梱
clock = { authority = "rp1", clk_sys_hz = 200_000_000 }  # authority: "rp1" | "linux"
log = { sink = "uart0", baudrate = 115_200 }             # 省略すると log! はコンパイルエラー
stack = { proc0 = "2KiB", proc1 = "2KiB" }               # 省略時は各 2 KiB（DSRAM）
link_page = "16KiB"                       # 共有 SRAM の Linux 用ページ。"none" なら全リングが placement = "linux"（v0.1 未実装）

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
| v0.1 の rpi-cm5 で `clock.authority = "linux"`（ボード DTB が RP1 のクロックを SCMI 経由にしており、overlay で Linux に戻せない。2026-10-29 review #7） | `clock.authority = linux is unsupported on the v0.1 rpi-cm5 board; use rp1` |
| ring の `notify` が `poll` 以外（hwirq58 は SCMI の mailbox が持つ。review #9） | `v0.1 supports only notify=poll (SCMI owns hwirq58)` |
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
| 0x0013 | 1 | BOOT | config.txt の SHA-256、cmdline.txt の SHA-256、本来のカーネルのファイル名（文字列） |
| 0x0020 | 1 | SHM_AREA | 24 bytes: DDR 合計サイズ u32（1 MiB 単位、1..16 MiB）、領域境界 u32（65536）、SRAM ハンドオフアドレス u32、SRAM 使用末尾 u16、予約 0 u16、DDR CPU 物理アドレス u64（STATIC: rpi-cm5 は 0x22000000）。旧 14-byte 形式は拒否 |
| 0x0021 | 1 | SHM_REGION | 1 件 1 レコード: kind u8、direction u8、core u8、notify u8、linux_map u8、writer u8、overflow u8（1 reject / 2 overwrite）、placement u8（1 reader-local / 2 linux）、DDR オフセット/長さ u32×2、SRAM オフセット/長さ u16×2、要素長 u32、depth u32、型 ID u64（要求/応答で 2 つ）、名前（u8 長さ + UTF-8）、型名（同） |

- 文字列は `u8 長さ + UTF-8`（最大 255）。
- id（ピン番号、ブロック番号）の割り当ては `rp1-proto` の enum が正本。番号は一度決めたら変えない。
- 型 ID: `#[derive(Message)]` が「型名・サイズ・各フィールドのオフセットと型」から計算する 64 bit のハッシュ。
  Linux 側の `fw.channel::<Cmd, Reply>("ctl")` はこれを note と照合し、違えば開けない。
- note に含めないもの: イメージのハッシュ（ローダーが自分で計算する）、Linux DT のノード名（ビルド時にボード表から overlay に焼き込む）。

### 2.3 読み手の動き

| 読み手 | 使うレコード | 失敗時 |
|---|---|---|
| ローダー | ENTRY、SHM_AREA（ハンドオフ）、OWNERSHIP（当てた後の DTB の検査、§3.2）。overlay 本体は `.rp1.dtbo` | 知らない critical レコード・CRC 不一致・DTB の印（§3.2）と不一致 → RP1 を起動せず停止（ログに理由） |
| Linux 側 `rp1-linux` | overlay が載せたコピー（`/rp1-firmware` ノードの `rp1,note` プロパティ）の SHM_*、FIRMWARE | 型 ID 不一致・名前が無い → `Err` |
| `cargo rp1 check --dtb` | OWNERSHIP、LOG、SHM_AREA/SHM_REGION、note 全体 | Linux DT との衝突を一覧にして失敗（`rp1_build::overlay::final_conflicts`、STATIC）: 有効なノードの `pinctrl-N` が `rp1_gpio` 配下のグループで RP1 所有ピンを設定、所有ブロックのノード（`rp1_uartN` 等）が有効・ラベルが無い、authority=rp1 なのに `rp1_clocks` が有効、`rp1-shm` 予約が SHM_AREA と違う・他の予約や `/memreserve/` と重なる、`rp1:<name>` の UIO が領域と違う、`/rp1-firmware rp1,note` が ELF の note と違う、有効な gpio-hog・`gpio(s)`/`*-gpio(s)` の利用者が `gpio-reserved-ranges` の線を使う |

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
2. `.rp1.dtbo` を、渡された DTB に当てる。実装は libfdt の `fdt_overlay_apply` を使う（U-Boot と同じ。ビルド時の `fdtoverlay` 検証と挙動がそろう）。
   DTB はローダーの作業領域に広げてから当てる。参照先のラベルが無い（ボード表とカーネル系列が合わない）場合は停止する。
3. 当てた後の DTB を読み取りで検査し、次に当たれば RP1 を起動せず停止して理由をログに出す。
   - 有効なノードの `pinctrl-*` が RP1 所有のピンを含むグループを参照している（他の overlay が同じピンを使っている）
   - OWNERSHIP の周辺に対応するノード（`/rp1-firmware/rp1,owned-nodes` に列挙）が有効
4. RP1 を起動し、共有メモリの物理アドレス（overlay と note の両方にある）を共有 SRAM のハンドオフ語に書く。
5. DTB を Linux に渡す。

### 3.3 RP1 専用の IO（UART0 など）を Linux から守る範囲

| 経路 | 防げるか | 方法 |
|---|---|---|
| カーネルのドライバが probe する | 防ぐ | ローダーが最後に当てる overlay の `disabled` |
| 別のノードが pinctrl で同じピンを取る | 防ぐ | ローダーの最終 DTB 検査 |
| libgpiod などがピンを要求する | 防ぐ（RP1 の GPIO ドライバで効くかは未確認） | `gpio-reserved-ranges` |
| 共有クロック（clk_uart は UART0〜5 共通）を Linux が変える | 条件付き | `clock.authority = "rp1"` なら RP1 が SCMI で変更を拒否できる。`authority = "linux"` で UART0 を RP1、他の UART を Linux が持つ宣言はビルドエラー |
| root が `/dev/mem`・`/dev/gpiomem0`・`pinctrl` ツールで直接書く | 防がない | DT では不可能（標準カーネルは `STRICT_DEVMEM` 無効）。型や宣言による保護はセキュリティ境界ではない（設計 §1.2） |

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
