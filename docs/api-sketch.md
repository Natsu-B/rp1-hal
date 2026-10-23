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
extmem = { size = "16MiB" }                     # Linux の DDR を外付け RAM として借りる（§4.3）

[package.metadata.rp1.proc0]
peripherals = ["UART2", "TIMER.ALARM0"]
pins = [4, 5]

[package.metadata.rp1.proc1]
peripherals = ["TIMER.ALARM1"]
pins = [22, 27]

[[package.metadata.rp1.link.channel]]   # Linux との通信路
name = "ctl"
core = "proc1"
message = "protocol::Cmd"               # Linux → M3
reply = "protocol::Reply"               # M3 → Linux
depth = 16
doorbell = "irq"                        # Linux → proc1 を割り込みで通知（GPIO INTF）
```

形式の詳細は設計 B で決める。ここでは「何をどこに書くか」だけ。

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

// 両コアで共有するデータは #[shared] だけ。共有 SRAM に置かれ、型は RMW atomic を持たないものに限られる。
// 書き手と読み手のコアを型で決める（proc1 が書き、proc0 が読む）。
#[shared]
static SAMPLES: Spsc<u32, 64, Proc1, Proc0> = Spsc::new();

#[entry(proc0)]
fn main(core: Proc0, p: rp1_rt::Resources<Proc0>) -> ! {
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
fn proc1_main(core: Proc1, p: rp1_rt::Resources<Proc1>) -> ! {
    let pins = hal::gpio::Pins::new(p.pins);            // 宣言した 22, 27 だけ
    let mut led = pins.gpio22.into_push_pull_output();
    let mut alarm = hal::timer::Alarm::new(p.TIMER_ALARM1);
    let mut ctl = p.link.ctl;                           // Linux との通信路 "ctl"
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
            ctl.send(&protocol::Reply { seq: 0, value: v as i32 }).ok();
        } else {
            ctl.wait();                                 // ドアベルの割り込みで起きる
        }
    }
}

// ホットな処理は #[isram]。ここから呼ばれる関数も自動で ISRAM に入る（設計 §3.3）。
#[isram]
fn sample() -> u32 { /* ... */ 0 }

// 割り込みはコアを明示する。そのコアの NVIC とベクタ表にだけ登録される。
#[interrupt(proc1)]
#[isram]
fn TIMER_IRQ_1() { /* ... */ }
```

### 4.1 コア専用・コアごとの static

```rust
#[rp1_rt::proc1]                 // proc1 専用（proc1 の DSRAM）。proc0 から届くとビルドエラー
static TICKS: AtomicU32 = AtomicU32::new(0);   // DSRAM なので atomic RMW を使ってよい

#[rp1_rt::per_core]              // 両コアが使うが値はコアごと（thread_local と同じ意味）
static SCRATCH: Cell<u32> = Cell::new(0);
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

### 4.3 外付け RAM（Linux の DDR を借りる）

```rust
use rp1_rt::ext;

#[ext]                                   // Linux の DDR 上（M3 から 0x8000_0000.. の窓で見える）
static TRACE: ExtCell<[u32; 1 << 20]> = ExtCell::zeroed();   // 4 MiB。ゼロ化はローダーが行う

#[entry(proc0)]
fn main(core: Proc0, p: rp1_rt::Resources<Proc0>) -> ! {
    // PCIe のリンクと outbound ATU が有効になるまでは触れない。トークンでそれを表す。
    let ext = rp1_link::ExtMem::enable(p.extmem).unwrap();
    let trace = TRACE.borrow_mut(&ext);          // &mut [u32; 1<<20]（1 コアだけが取れる）
    trace[0] = 1;                                // 書き込みは posted で速い。読み出しは PCIe 往復で遅い
    // ...
}
```

Linux 側も同じ領域を名前で開ける（`fw.extmem("TRACE")`）。

立ち位置は「遅くて大きいメモリ」（Optane 的な位置づけ）:

| | 共有 SRAM | Linux DDR（外付け RAM） |
|---|---|---|
| 容量 | 64 KiB（実質 40 KiB 前後） | 宣言した分（窓は最大 1 GiB、DOC） |
| M3 からの読み出し | 1 µs 未満（HW） | PCIe の往復。**未測定** |
| M3 からの書き込み | 同上 | posted なので速い見込み（HYP） |
| キャッシュ | なし | M3 側なし（毎回 PCIe）。Linux 側は非キャッシュで mmap するか、キャッシュ保守が必要 |
| 使えない時間 | なし | リンク確立と ATU 設定の前、PCIe リセット中（Linux 再起動は v0.1 対象外） |
| 向く用途 | コア間の通信、小さな状態 | 大きなバッファ、トレース、ログの退避、表。ISR やホットループには使わない |

規則: `#[ext]` の static には `ExtMem` トークン無しでは触れない（rustc）。ISRAM の根から `#[ext]` に届いたら警告（cargo-rp1）。
コードを置く（`#[ext] fn`）ことも ARMv7-M 上は可能（0x8000_0000.. は実行可能領域）だが、v0.1 では扱わない。

## 5. 何がエラーになるか

| 書いたコード | 結果 | 検出する場所 |
|---|---|---|
| proc0 から `p.UART1` を使う（宣言していない） | フィールドが存在しない | rustc |
| proc1 で `SAMPLES.consumer(&core)` | `Proc1` は consumer を取れない | rustc |
| `Proc1` トークンを proc0 に渡す | `!Send` | rustc |
| `#[shared] static X: AtomicU32` | `AtomicU32` は `SharedSafe` を実装しない | rustc |
| proc0 から `#[rp1_rt::proc1]` の関数や static に届く | ビルドエラー | cargo-rp1（リンク後の到達解析） |
| 印の無い書き込み可能な static に両コアから届く | ビルドエラー（`#[shared]` か `#[per_core]` を付けるよう促す） | cargo-rp1 |
| ISRAM の予算超過、ISR から 64 bit 除算や `core::fmt` に届く | 警告 / エラー | cargo-rp1 |
| Linux DT が使っているピンや周辺を宣言した | デプロイ前にエラー | `cargo rp1 check --dtb` |

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
cargo rp1 build              # firmware をビルド → 到達解析・ISRAM 配置・ELF 検査 → .note.rp1 付き rp1.elf
cargo rp1 check --dtb X.dtb  # 宣言と Linux DT の衝突を検査
cargo rp1 run                # rp1.elf を TFTP（または SD）へ置き、CM5 を再起動し、UART を表示
```

## 8. この案で決めたいこと（設計 A の論点）

1. **Resources の受け取り方**: 各 `#[entry]` が自分の `Resources<ProcN>` を引数で受け取る（案）。rp-hal 流の
   `pac::Peripherals::take()` は抜け道として残す。
2. **proc1 の起動**: proc0 が `p.proc1.start()` で明示的に起動する（案）。自動起動にはしない（クロック設定の後に起動したいため）。
3. **共有データの型**: `Spsc<T, N, Writer, Reader>` のように、書き手と読み手のコアを型引数で持たせる（案）。
4. **Linux 通信のメッセージ**: 固定長の `#[derive(Message)]` 型に限る（案）。可変長は bulk 領域を別に使う。
5. **async**: v0.1 では入れない。ブロッキング API と `try_*` API だけ（案）。Embassy は後から executor を載せられる形にしておく。
6. **ログ**: v0.1 に入れる。出力先は UART0（§4.2）。

2026-10-23: 1〜6 はこの案で了承（ユーザー）。外付け RAM（§4.3）を追加。
