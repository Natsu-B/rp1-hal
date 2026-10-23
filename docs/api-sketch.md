# API スケッチ（設計 A、2026-10-23 草案）

利用者が書くコードの形を先に決め、ここから crate の公開 API・宣言・検査を逆算する。
コードはすべて**案**で、まだ存在しない。rp-hal（rp2040-hal）と同じ名前・同じ意味にできるところは合わせる。

## 1. 利用者のワークスペース

```
my-app/
  Cargo.toml          [workspace]
  protocol/           no_std。M3 と Linux で共有するメッセージ型
  firmware/           RP1 の M3 で動くファーム（proc0 + proc1、1 イメージ）
  linux-app/          Linux のユーザープロセス
```

`protocol` を両側から使うので、メッセージの型がずれることはない。

## 2. firmware/Cargo.toml（宣言）

```toml
[package]
name = "firmware"

[dependencies]
rp1-hal  = "0.1"
rp1-rt   = "0.1"
rp1-link = "0.1"
rpi-cm5  = "0.1"          # ボード定義（ピン名と既定の所有権）
protocol = { path = "../protocol" }

[package.metadata.rp1]
board = "rpi-cm5"
clock = { authority = "rp1", clk_sys_hz = 200_000_000 }
log = { sink = "uart0", baudrate = 115_200 }   # UART0 をログ専用にする（§4.2）

[package.metadata.rp1.proc0]
peripherals = ["UART2", "TIMER.ALARM0"]
pins = [4, 5]

[package.metadata.rp1.proc1]
peripherals = ["TIMER.ALARM1"]
pins = [22, 27]

# Linux と共有するメモリ（BCM2712 側の DDR）。kind で「リング」か「ブロック」を選ぶ（§4.3）
[[package.metadata.rp1.shm]]            # 要求/応答の通信路 = 向きの違う 2 本のリング
name = "ctl"
kind = "channel"
core = "proc1"
message = "protocol::Cmd"               # Linux → M3
reply = "protocol::Reply"               # M3 → Linux
depth = 16
notify = "irq"                          # Linux → proc1 を割り込みで通知（GPIO INTF）。"poll" も可

[[package.metadata.rp1.shm]]            # 片方向のリング（M3 → Linux の計測値ストリーム）
name = "samples"
kind = "ring"
direction = "rp1-to-linux"
core = "proc1"
message = "protocol::Sample"
depth = 4096

[[package.metadata.rp1.shm]]            # 共有メモリブロック（大きな表やトレース）
name = "trace"
kind = "block"
type = "protocol::Trace"                # 固定長の型。中身の同期は利用者か Seqlock 等で行う
writer = "rp1"                          # "rp1" | "linux" | "both"
linux_map = "uncached"                  # "uncached"（既定）| "cached"（試験的、§4.3）
```

全項目と検査は [declaration-and-note.md](declaration-and-note.md)（設計 B）。

## 3. protocol/src/lib.rs（共有メッセージ）

```rust
#![no_std]
use rp1_proto::Message;

#[derive(Message, Clone, Copy)]          // 固定長・パディング無し・任意のビット列が有効、を derive が検査する
#[repr(C)]
pub enum Cmd { Start { period_us: u32 }, Stop }

#[derive(Message, Clone, Copy)]
#[repr(C)]
pub struct Reply { pub seq: u32, pub value: i32 }
```

## 4. firmware/src/main.rs（proc0 と proc1）

```rust
#![no_std]
#![no_main]

use rp1_hal::{self as hal, prelude::*};
use rp1_hal::core::{Proc0, Proc1};
use rp1_hal::sync::Spsc;
use rp1_rt::{entry, interrupt, isram, shared};

rp1_rt::bind!();          // 宣言から生成した mod rp1（Proc0Resources など、declaration-and-note.md §1.5）

// 両コアで共有するデータは #[shared] だけ。共有 SRAM に置かれ、型は RMW atomic を持たないものに限られる。
// 書き手と読み手のコアを型で決める（proc1 が書き、proc0 が読む）。
#[shared]
static SAMPLES: Spsc<u32, 64, Proc1, Proc0> = Spsc::new();

#[entry(proc0)]
fn main(core: Proc0, p: rp1::Proc0Resources) -> ! {
    // clock.authority = "rp1" なので、Resources<Proc0> にだけ clock_authority が入っている。
    let clocks = hal::clocks::Clocks::authority(p.clock_authority)
        .clk_sys(200.MHz())
        .freeze()
        .unwrap();

    rp1_rt::log::init(p.log, &clocks);                  // ここから log! が UART0 に出る（それまでは捨てて数える）
    log!("proc0 up, clk_sys={}", clocks.sys());

    let pins = hal::gpio::Pins::new(p.pins);           // 宣言した 4, 5 だけを持つ
    let mut uart = hal::uart::Uart::new(
        p.UART2,
        (pins.gpio4.into_function(), pins.gpio5.into_function()),
        hal::uart::Config::default().baudrate(115_200.Hz()),
        &clocks,
    )
    .unwrap();

    let samples = SAMPLES.consumer(&core);             // Proc0 トークンでしか取れない
    p.proc1.start();                                    // proc1 の #[entry(proc1)] を起動する

    loop {
        while let Some(v) = samples.dequeue() {
            writeln!(uart, "sample {v}").ok();
        }
        hal::arch::wfe();
    }
}

#[entry(proc1)]
fn proc1_main(core: Proc1, p: rp1::Proc1Resources) -> ! {
    let pins = hal::gpio::Pins::new(p.pins);            // 宣言した 22, 27 だけ
    let mut led = pins.gpio22.into_push_pull_output();
    let mut alarm = hal::timer::Alarm::new(p.TIMER_ALARM1);
    let shm = rp1_link::Shm::enable(p.shm).unwrap();    // §4.3
    let mut ctl = shm.ctl;                              // Linux との通信路 "ctl"
    let samples = SAMPLES.producer(&core);

    let mut period = None;
    loop {
        if let Some(cmd) = ctl.try_recv() {
            period = match cmd {
                protocol::Cmd::Start { period_us } => Some(period_us.micros()),
                protocol::Cmd::Stop => None,
            };
        }
        if let Some(t) = period {
            alarm.wait(t);                              // embedded-hal DelayNs 相当
            led.toggle().ok();
            let v = sample();
            samples.enqueue(v).ok();
            ctl.send(&protocol::Reply { seq: 0, value: v as i32 }).ok();   // channel "ctl"（§4.3）
        } else {
            ctl.wait();                                 // ドアベルの割り込みで起きる
        }
    }
}

// ホットな処理は #[isram]。v0.1 は手で付けた関数だけが ISRAM に入る（自動配置は v0.2、設計 §3.3）。
#[isram]
fn sample() -> u32 { /* ... */ 0 }

// 割り込みはコアを明示する。そのコアの NVIC とベクタ表にだけ登録される。
#[interrupt(proc1)]
#[isram]
fn TIMER_IRQ_1() { /* ... */ }
```

### 4.1 コア専用・コアごとの static

```rust
#[rp1_rt::proc1]                 // proc1 専用。Local<AtomicU32, Proc1> になり、Proc1 トークンが無いと読めない
static TICKS: AtomicU32 = AtomicU32::new(0);   // DSRAM なので atomic RMW を使ってよい
// 使い方: TICKS.get(&core).fetch_add(1, Relaxed);

static SCRATCH: AtomicU32 = AtomicU32::new(0); // 印の無い static はコアごとに別コピー（thread_local と同じ意味）
```

### 4.2 log!

- 宣言 `log = { sink = "uart0" }` で UART0 はログ専用になり、`Resources` には `UART0` ではなく `log`（初期化用トークン）が入る。
- `log!` は両コア・割り込みから使える。両コアの出力は EXAC によるコア間ロックで 1 行単位に直列化する。
  割り込み中にロックが取れなければその行を捨てて数える（待たない）。`log::init` 前の出力も捨てて数える。
- 115200 baud は約 11.5 byte/ms なので、`log!` は遅い。ISRAM の根（ISR・`#[isram]`）から `log!` に届くと cargo-rp1 が警告する。
- 将来 `sink = "link"`（Linux への通信路）を足せるようにしておく。

**エラーにする条件**:

| 状況 | 結果 | 検出 |
|---|---|---|
| `log!` を使ったのに `log` を宣言していない | `compile_error!("log! needs [package.metadata.rp1] log = { sink = \"uart0\" }")` | rustc（build.rs が `cfg(rp1_log)` を出し、`log!` の展開先で判定） |
| `log = { sink = "uart0" }` だが UART0 が RP1 の所有でない（ボード既定や宣言で Linux 所有） | ビルドエラー | build.rs（宣言の検査） |
| `log` を宣言し、さらに proc0/proc1 の `peripherals` にも `UART0` を書いた | ビルドエラー（二重所有） | build.rs |
| Linux DT が UART0 を使っている | デプロイ前にエラー | `cargo rp1 check --dtb`。ローダーも note を見て DT の UART0 を無効化する |

### 4.3 Linux と共有するメモリ（リング / ブロック）

BCM2712 側の DDR に連続領域を 1 つ確保し、宣言した `shm` ごとに区切って使う。種類は 3 つ:

| kind | 用途 | M3 側の型 | Linux 側の型 |
|---|---|---|---|
| `ring` | 片方向のメッセージ列（ストリーム） | `RingTx<T>` / `RingRx<T>` | `RingRx<T>` / `RingTx<T>` |
| `channel` | 要求と応答（`ring` 2 本 + 通知） | `Channel<Req, Rep>` | `Channel<Req, Rep>` |
| `block` | 共有メモリブロック（大きな表、トレース、外付け RAM として） | `Block<T>` | `BlockMap<T>` |

M3 側:

```rust
#[entry(proc1)]
fn proc1_main(core: Proc1, p: rp1::Proc1Resources) -> ! {
    // PCIe のリンクと outbound ATU が有効になるまでは shm に触れない。enable がそれを確かめてハンドルを返す。
    let shm = rp1_link::Shm::enable(p.shm).unwrap();
    let mut samples = shm.samples;               // RingTx<protocol::Sample>
    let trace = shm.trace;                       // Block<protocol::Trace>
    samples.push(&protocol::Sample { t: 0, v: 1 }).ok();
    trace.write(|t| t.count += 1);               // 書き込みは posted で速い。読み出しは PCIe の往復で遅い
    // ...
}
```

Linux 側:

```rust
let fw = rp1_linux::Firmware::open()?;                       // note と UIO ノードを名前で照合
let mut samples = fw.ring_rx::<protocol::Sample>("samples")?;
let trace = fw.block::<protocol::Trace>("trace")?;
while let Some(s) = samples.pop() { /* ... */ }
let count = trace.read(|t| t.count);                         // 安全なコピー経由で読む（下記）
```

#### 置き場所の規則（「書く側から遠く、読む側に近く」）

PCIe の書き込みは posted で速く、読み出しは往復が要って遅い。そこで各語を「読む側のローカル」に置く:

| リングの向き | payload と head（書き手が更新） | tail（読み手が更新） |
|---|---|---|
| RP1 → Linux | Linux の DDR（RP1 は posted 書き込み、Linux はローカル読み） | 共有 SRAM（Linux は BAR2 へ posted 書き込み、RP1 はローカル読み） |
| Linux → RP1 | 共有 SRAM（Linux は BAR2 へ posted 書き込み、RP1 はローカル読み）。SRAM は小さいので深さは小さく | Linux の DDR |

`block` は大きいので DDR に置く。RP1 からの読み出しは遅い前提（「遅くて大きいメモリ」）。

#### Linux 側の確保と mmap（DT とユーザープロセスだけ）

- ビルド時に生成し、ローダーが当てる overlay（declaration-and-note.md §3）が `reserved-memory`（`no-map`、連続、2 MiB 境界）を 1 つ足す。物理アドレスはボード表の固定領域。ゼロ化はファームが `Shm::enable` で行う。
  既存の `rp1-ddr-scratch@21000000`（64 KiB、`no-map`）と同じやり方を一般化したもの（HW 実績あり）。
- overlay が `shm` ごとに `generic-uio` ノードを足す（名前 `rp1:<name>`、領域 + 必要なら IRQ）。`/sys/class/uio/uioN/name` で探す。
  標準カーネル（6.12.75-rpt-rpi-2712）は `CONFIG_UIO=m`、`CONFIG_UIO_PDRV_GENIRQ=m`（設定ファイルで確認、実機の動作は未確認）。
  アクセス権は udev ルールでグループ（例 `rp1`）に与え、root を不要にする。
- **mmap の属性**:
  - `uncached`（既定）: UIO の物理メモリ mmap は arm64 では Device 属性になる（DOC: `uio_mmap_physical` が `pgprot_noncached`）。
    Device 属性ではアラインされていないアクセスと `DC ZVA` がフォルトするため、libc の `memcpy` / `memset` を直接使うと SIGBUS になりうる。
    `rp1-linux` は 8 バイト整列の volatile コピーだけを使い、利用者に生の `&[u8]` を渡さない。
  - `cached`（試験的）: `no-map` を外した領域を `/dev/mem`（`O_SYNC` なし）で cacheable に mmap し、読む前・渡す前に `DC CIVAC` を
    ユーザー空間から発行する（標準カーネルは `CONFIG_STRICT_DEVMEM` 無効なので RAM の mmap が可能。root が必要）。
    RP1 からの PCIe 書き込みが CPU キャッシュとコヒーレントでない前提での保守。速さと正しさを測ってから採用を決める。
- 順序: RP1 は payload → head を posted 書き込みし、同じ窓への読み出し 1 回で到達を確定してから通知する（既存の SHM 契約）。
- 世代: 領域の先頭に magic と世代番号を置き、`Shm::enable` 後に RP1 が書く。Linux 側は世代が一致するまで読まない。

#### 制約

- v0.1 は cold boot のみ。PCIe リセットで outbound ATU が消える（HW、2026-09-29）。
- kexec など、ローダーを通らない再起動は非対応（RP1 が古い物理アドレスへ書き続ける恐れがある）。
  通常の再起動では EEPROM のブートローダーが RP1 をリセットするので、書き込みは止まる。
- M3 から DDR を読む遅延・帯域、Linux 側の uncached / cached の速度は未測定。最初の縦 1 本で測る。

## 5. 何がエラーになるか

方針: 誤用はリンク後に見つけるのではなく、型と配置で書けなくする（design.md §5）。

| 書いたコード | 結果 | 検出する場所 |
|---|---|---|
| proc0 から `p.UART1` を使う（宣言していない） | フィールドが存在しない | rustc |
| proc1 で `SAMPLES.consumer(&core)` | `Proc1` は consumer を取れない | rustc |
| `Proc1` トークンを proc0 に渡す | `!Send` | rustc |
| `#[shared] static X: AtomicU32` | `AtomicU32` は `SharedSafe` を実装しない | rustc |
| proc0 から `#[rp1_rt::proc1] static` を読む | `Proc1` トークンが無いので読めない | rustc |
| 普通の `static` に両コアから触る | エラーにしない。コアごとに別コピーなので競合しない（値も共有されない） | — |
| DSRAM に置いた値への参照を相手コアに渡す | コア間 API が値渡しか `#[shared]` ハンドルしか受け付けない | rustc |
| 共有 SRAM 上で LDREX/STREX（`unsafe` や外部 crate 経由） | ビルドエラー | cargo-rp1（ELF 検査） |
| ISRAM・DSRAM・共有 SRAM の予算超過 | ビルドエラー | linker / cargo-rp1 |
| `log!` を使ったのに `log` の宣言が無い / UART0 を RP1 が持てない | コンパイルエラー / ビルドエラー | rustc / build.rs |
| Linux DT が使っているピンや周辺を宣言した | デプロイ前にエラー | `cargo rp1 check`、ローダー |

## 6. Linux 側（linux-app/src/main.rs）

```rust
use rp1_linux::Firmware;
use std::time::Duration;

fn main() -> anyhow::Result<()> {
    // ローダーが note から作った UIO ノードを、通信路の名前で探す（/sys/class/uio/*/name）。
    let fw = Firmware::open()?;
    let mut ctl = fw.channel::<protocol::Cmd, protocol::Reply>("ctl")?;   // 型と深さを note と照合

    ctl.send(&protocol::Cmd::Start { period_us: 1000 })?;                 // ドアベルも鳴らす
    for _ in 0..10 {
        let r = ctl.recv_timeout(Duration::from_millis(10))?;              // UIO の割り込み待ち
        println!("{} {}", r.seq, r.value);
    }
    ctl.set_wait(rp1_linux::Wait::Poll);                                   // 低遅延が要るならポーリング
    Ok(())
}
```

## 7. 使い方の流れ

```
cargo rp1 build              # firmware をビルド → 到達解析・ELF 検査 → .note.rp1 と .rp1.dtbo 入りの rp1.elf
cargo rp1 check --dtb X.dtb  # 宣言と Linux DT の衝突を検査
cargo rp1 run                # rp1.elf を TFTP（または SD）へ置き、CM5 を再起動し、UART を表示
```

## 8. この案で決めたいこと（設計 A の論点）

1. **Resources の受け取り方**: 各 `#[entry]` が自分の `Resources<ProcN>` を引数で受け取る（案）。rp-hal 流の
   `pac::Peripherals::take()` は `unsafe fn` の抜け道として残す（2026-10-23）。
2. **proc1 の起動**: proc0 が `p.proc1.start()` で明示的に起動する（案）。自動起動にはしない（クロック設定の後に起動したいため）。
3. **共有データの型**: `Spsc<T, N, Writer, Reader>` のように、書き手と読み手のコアを型引数で持たせる（案）。
4. **Linux 通信のメッセージ**: 固定長の `#[derive(Message)]` 型に限る（案）。可変長は bulk 領域を別に使う。
5. **async**: v0.1 では入れない。ブロッキング API と `try_*` API だけ（案）。Embassy は後から executor を載せられる形にしておく。
6. **ログ**: v0.1 に入れる。出力先は UART0（§4.2）。

2026-10-23: 1〜6 はこの案で了承（ユーザー）。Linux 共有メモリをリング / ブロックから選ぶ形（§4.3）を追加（ユーザー提案）。
