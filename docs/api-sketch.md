# API スケッチ（設計 A、2026-10-23 草案）

利用者が書くコードの形を先に決め、ここから crate の公開 API・宣言・検査を逆算する。
STATIC と付けた箇所以外のコードは設計案。rp-hal（rp2040-hal）と同じ名前・同じ意味にできるところは合わせる。

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

    // 宣言 uart2 = { tx = 4, rx = 5 } の Parts（Token + Pin<4> + Pin<5>）。表に無い組は型が無い
    let mut uart = p.uart2.init(&clocks, 115_200).unwrap();

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
    let mut ctl = /* 将来の channel API から取得する "ctl" */;
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

// 割り込みはコアを明示する。そのコアのベクタ表にだけ入り、既定で ISRAM に置かれる。
// 線（p.irq.TIMER_1）を宣言で所有したコアにしか置けない（cargo rp1 check、docs/interrupts.md）。
#[interrupt(proc1)]
fn TIMER_1() { /* alarm.clear_interrupt() 相当で源を落としてから戻る */ }
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
- `log!` は両コア・割り込みから使える。両コアの出力は load / store だけのコア間ロック（Peterson）で 1 行単位に直列化する。
  main は相手コアの行が終わるのを待つ。ハンドラは待たず、ロックが取れなければその行を捨てて数える（`logging::dropped`）。
  同じコアで書いている最中（ハンドラが割り込んだ場合など）の行も捨てて数える。`log::init` 前の出力はエラーを返す。
- `log!` は割り込みを止めない。ロックを取れたハンドラはその中で 1 文字ずつ送るので、長い行はハンドラを数 ms 止める
  （115200 baud は約 11.5 byte/ms）。ハンドラからは短い行にする（STATIC、警告する検査は無い）。
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

M3 側（STATIC）。proc0 は Linux の公開要求を受けて `rp1::link::try_enable(&mut system.pcie)` を呼ぶ。
`samples` を所有する proc1 は、生成されたトークンから端点を取得する。

```rust
let mut init = p.link;
let mut endpoints = loop {
    match init.try_open() {
        Ok(endpoints) => break endpoints,
        Err(rp1_link::OpenError::NotReady(pending)) => init = pending,
        Err(rp1_link::OpenError::Reset) => panic!("SHM window lost"),
        Err(rp1_link::OpenError::Attach(error)) => panic!("SHM attach: {error:?}"),
    }
};
endpoints.samples.push(&protocol::Sample { t: 0, v: 1 }).ok();
```

Linux 側:

```rust
let fw = rp1_linux::Firmware::open()?;                       // note と UIO ノードを名前で照合
fw.wait_ready(Duration::from_secs(10))?;                     // RP1 が shm を公開するまで待つ（STATIC）
let mut samples = fw.ring_rx::<protocol::Sample>("samples")?;
let trace = fw.block::<protocol::Trace>("trace")?;
while let Some(s) = samples.try_recv()? { /* ... */ }        // 待つなら recv(Some(timeout))（STATIC）
let count = trace.read()?.count;                             // 安全なコピー経由で読む（下記）
```

#### 置き場所の規則（「書く側から遠く、読む側に近く」）

PCIe の書き込みは posted で速く、読み出しは往復が要って遅い。そこで各語を「読む側のローカル」に置く:

| リングの向き | payload と head（書き手が更新） | tail（読み手が更新） |
|---|---|---|
| RP1 → Linux | Linux の DDR（RP1 は posted 書き込み、Linux はローカル読み） | 共有 SRAM（Linux は BAR2 へ posted 書き込み、RP1 はローカル読み） |
| Linux → RP1 | 共有 SRAM（Linux は BAR2 へ posted 書き込み、RP1 はローカル読み）。SRAM は小さいので深さは小さく | Linux の DDR |

`block` は大きいので DDR に置く。RP1 からの読み出しは遅い前提（「遅くて大きいメモリ」）。

#### Linux 側の確保と mmap（DT とユーザープロセスだけ）

- ビルド時に生成し、ローダーが当てる overlay（declaration-and-note.md §3）が `reserved-memory`（`no-map`、連続、2 MiB 境界）を 1 つ足す。物理アドレスはボード表の固定領域。proc0 は `rp1::link::try_enable` で予約領域をゼロ化する（STATIC）。
  既存の `rp1-ddr-scratch@21000000`（64 KiB、`no-map`）と同じやり方を一般化したもの（HW 実績あり）。
- overlay が `shm` ごとに `generic-uio` ノードを足す（名前 `rp1:<name>`、領域 + 必要なら IRQ）。`/sys/class/uio/uioN/name` で探す。
  標準カーネル（6.12.75-rpt-rpi-2712）は `CONFIG_UIO=m`、`CONFIG_UIO_PDRV_GENIRQ=m`（設定ファイルで確認、実機の動作は未確認）。
  アクセス権は udev ルールでグループ（例 `rp1`）に与え、root を不要にする。rp1-build が `70-rp1.rules`
  （`SUBSYSTEM=="uio", ATTR{name}=="rp1:*", GROUP="<uio_group>", MODE="0660"`）を書き出す。HW（2026-11-05、cold boot 1 回）:
  そのルールと同じ所有（root:1000、0660）にした `rp1:*` UIO を uid/gid 1000（補助グループ無し）が開き、mmap・IRQ の
  read/write(1) まで使えた（`linux-doorbell` IRQ 往復 2000 回 成功）。gid 1001 は EACCES。udev 自体の照合は STATIC
  （`udevadm verify` 通過）で、initramfs に udev が無いため実機の udev では未試験。
- **mmap の属性**:
  - `uncached`（既定）: UIO の物理メモリ mmap は arm64 では Device 属性になる（DOC: `uio_mmap_physical` が `pgprot_noncached`）。
    Device 属性ではアラインされていないアクセスと `DC ZVA` がフォルトするため、libc の `memcpy` / `memset` を直接使うと SIGBUS になりうる。
    `rp1-linux` は整列した volatile u32/u64 アクセスだけを使い、利用者に生の `&[u8]` を渡さない（STATIC）。
    実測（HW 2026-10-28、UIO で `rp1-ddr-scratch` 0x21000000 の 64 KiB）:
    64 bit 読み出し 893 MB/s、64 bit 書き込み 531 MB/s、32 bit 読み 1 回 約 129 ns。libc `memcpy` は整列済みなら成功
    （読み出し 1.9 GB/s）、非整列は読み書きとも SIGBUS、`memset` は SIGBUS。
  - `cached`（試験的）: `no-map` を外した領域を `/dev/mem`（`O_SYNC` なし）で cacheable に mmap し、読む前・渡す前に `DC CIVAC` を
    ユーザー空間から発行する（標準カーネルは `CONFIG_STRICT_DEVMEM` 無効なので RAM の mmap が可能。root が必要）。
    HW（2026-11-05）: 今の overlay の `rp1-shm`（`no-map`）を `/dev/mem`（`O_SYNC` なし）で mmap すると Device 属性になる
    （非整列 `ldr` が SIGBUS、64 bit 読み 705 MB/s ≒ UIO）。arm64 の `phys_mem_access_prot` は線形マップに無い pfn を
    noncached にするため（DOC）。比較の通常 RAM は cacheable（7.1 GB/s、非整列可）。EL0 の `DC CIVAC` はどちらでも実行できる。
    よって cached を使うには `no-map` を外した予約が要り、RP1 の書き込みと `DC CIVAC` の正しさは未試験。
    RP1 からの PCIe 書き込みが CPU キャッシュとコヒーレントでない前提での保守。速さと正しさを測ってから採用を決める。
- 順序: RP1 は payload → head を posted 書き込みし、同じ窓への読み出し 1 回で到達を確定してから通知する（既存の SHM 契約）。
- 世代: 領域の先頭に magic と世代番号を置き、`rp1::link::try_enable` が公開する（STATIC）。Linux 側は世代が一致するまで読まない。

#### 制約

- v0.1 は cold boot のみ。PCIe リセットで outbound ATU が消える（HW、2026-09-29）。
- kexec など、ローダーを通らない再起動は非対応（RP1 が古い物理アドレスへ書き続ける恐れがある）。
  通常の再起動では EEPROM のブートローダーが RP1 をリセットするので、書き込みは止まる。
- Linux 側 uncached の速度は上記（HW）。M3 から DDR を読む遅延・帯域と、Linux 側 cached の速度は未測定。

### 4.4 ドライバの解放・中止・エラー

この節は実装済みの契約（STATIC: 偽レジスタを使うホスト試験。HW 未検証）。名前は rp-hal に合わせ、丁寧な解放は `free`、
強制的な停止は `abort` とする。

| 操作 | 形 | 意味 |
|---|---|---|
| 丁寧な解放 | `fn free(self) -> Result<資源, FreeError<Self, Error>>` | ハードが静止するまで（上限つき、経過時間で）待ち、割り込みをマスクし、ブロックを止めて資源を返す。失敗時は何も止めずにドライバ自身（`driver`）をエラー（`error`）と一緒に返すので、呼び出し側はやり直せる。`FreeError` の `Debug` はエラーだけを出すので `.unwrap()` / `.expect()` が使える。`?` で捨てると動作中のハードを持つドライバが失われるので、`From` は実装しない |
| 中止 | `fn abort(self) -> 資源` | 待たない・失敗しない。何が捨てられ、何が保証されるかを各ドライバの doc に書く |
| 失敗しない解放 | `fn free(self) -> 資源` | 待つものが無いドライバ（`Adc`、`Dma`、`spi::Device`）。`abort` は無い |
| 初期化の失敗 | `Err(rp1_hal::InitError { error, resources })` | 全ドライバ共通の 1 つの型（`FreeError` と同じく crate 直下）。`init` が消費したものを `resources` に入れて返す: uart / i2c / spi は解放と同じ `Parts`（token とピン）、pwm / adc / dma は token。どれも `e.resources.init(..)` でやり直せる。`?` は `InitError` をそのドライバの `Error` に変換する |

返る資源は `init` が消費したもの: `uart::Parts` / `i2c::Parts` / `spi::Parts`（そのまま `init` し直せる）、`pwm::Token`、
PWM 出力は `(Channel, Pin)`、ADC と DMA は `Token`。

| ドライバ | `free` が待つもの（失敗） | `abort` で捨てられるもの | `abort` 後の状態 |
|---|---|---|---|
| `Uart` | 送信完了 BUSY=0（`Timeout`）。その後 `abort` と同じくこの UART のリセットを掛け、受信途中の文字と未読の受信分を捨てる | TX FIFO の未送信分、未読の受信分、送受信途中の文字（途中で切れるので相手には framing error に見えうる） | この UART だけをリセット保持（RESETS CTRL1 の自分のビット。他の UART と clk_uart には触れない）。割り込みマスク・無効・FIFO 空はリセット値による（DOC）。`init` がリセットを解除する。PL011 は文字の送受信中に LCR_H を変えることを禁じるので、FEN を落として FIFO を空ける方法は使わない |
| `I2c` | 通常は無し。disable が timeout した（毒された）ハンドルだけ再度 100 ms 待つ（`Timeout`） | 無し（転送の失敗時に `transaction` が ABORT を要求済み。`abort` は disable 要求を繰り返すだけ） | 割り込みマスク、disable 要求済み。スレーブが SCL を離すまでコントローラは動作中のままで、その間の `init` は `Timeout` |
| `Spi` | バスの静止 TFE かつ BUSY=0、100 ms（`Timeout`） | 送出中のフレーム（途中で切れる）と両 FIFO | 割り込みマスク、コントローラ無効、SCK 停止 |
| `pwm::Channels` / `Output` | clk_pwm が 50 MHz で動作中であること（`Precondition`） | — | 停止を書くが、クロックが止まっていると反映されないことがある（HW 未検証）。`Token::init` がクロックを起動し直して全チャネルを止める |
| DMA の `Transfer` | — | `cancel()` がブロックの残りを捨てる | 他と違い上限が無い: DMAC がバッファに触れなくなるまで戻らない（[dma.md](dma.md)）。動作中のブロックのバッファを返す `free` は無い |

一般の契約（静止を待つ → 割り込みマスク → 停止）から外れるドライバ（STATIC: コードから読んだもの。HW 未検証）:

- `Uart`: `free` が待つのは送信側（BUSY）だけで、受信途中の文字は待たない。止め方も「無効化」ではなく
  この UART のリセット保持で、割り込みマスクはリセット値による（上の表）。
- `I2c`: `free` は先に disable を要求してから、その完了（ENABLE_STATUS=0）を待ち、そのあとで割り込みをマスクする。
  `Timeout` のとき disable は要求済みのまま戻る（「失敗時は何も止めない」の例外）。コントローラは転送の合間には
  disable されているので、通常は待たずに終わる。
- `pwm::Channels` / `Output`: `free` が確かめるのは clk_pwm が動いていることだけで、停止（GLOBAL_CTRL のクリアと
  UPDATE）を書いたら戻る。出力が実際に止まったこと（周期の終わりで反映されたこと）は観測しない。
- `Dma`: `free` はチャネルがすべて戻っていれば失敗しないが、コントローラ（DMAC_CFG.DMAC_EN）・クロック・リセットは
  有効のまま残す。次の `Token::init` はコントローラをソフトリセットしてから設定し直す。
- `Adc`: `free` は CS = 0（電源断）を書くだけで待たない。`read` が `Timeout` を返した後は変換が続いている可能性があり、
  それを止める処理は無いので、`free` 後の ADC が静止しているとは言えない（CS 書き込みの効果は HW 未検証）。

GPIO の `Pin` と `timer::Alarm` はそれ自体が資源で、解放するドライバを持たない。

割り込みを使うドライバ（`Uart` / `I2c` / `Spi`）の `free` / `abort` は周辺側のマスクを閉じるが、NVIC の線
（生成資源の `rp1_rt::interrupt::Line`）はドライバが持たないので触らない。ARMv7-M では一度 pending になった割り込みは
要因が消えても pending のまま残る（DOC）ので、ハンドラが解放後にもう 1 回走りうる（RP1 では HW 未検証）。
呼び出し側が先に `Line::disable` と `Line::unpend` を行うか、ドライバが無いことをハンドラが許容する。

```rust
// 送信が終わらなければ uart が戻るので、待ち直すか、あきらめて abort する。
let parts = match uart.free() {
    Ok(parts) => parts,
    Err(e) => e.driver.abort(),
};
let mut uart = parts.init(&clocks, 9_600).unwrap();    // 返った Parts でそのまま初期化し直せる
```

#### UART のエラーと timeout

- `uart::Error` は受信エラーを `Overrun` / `Break` / `Parity` / `Framing` に分け、ほかに `Timeout` / `Invalid`（baud）/
  `Precondition`（クロック）を持つ。`embedded_io::ErrorKind` は `Break`・`Parity`・`Framing` → `InvalidData`、`Timeout` → `TimedOut`、
  `Invalid` → `InvalidInput`、`Overrun`・`Precondition` → `Other`（overrun は、それまでに読めた文字は正しいが、エラーを付けて届いた文字より前の 1 文字以上が FIFO 満杯で失われたという意味で、対応する kind が無い。エラーを付けて届いた文字自体も捨てる）。
  エラー付きで受信した文字は捨て、その位置でエラーを返す。
- 待ちはすべて 1 MHz タイマーの経過時間で測る（ポーリング回数ではない）。受信待ち（`embedded_io::Read`）の上限は
  `Uart::set_read_timeout_us`（既定 100 ms、0 でポーリング）。送信側の待ち（`write` の FIFO 空き待ち、`flush`、`free`）は
  利用者が変えられず、`init` の baud から決まる: FIFO 満杯の送信時間（330 ビット時間）の 2 倍 + 1 ms。フロー制御が無いので
  送信は相手によらず baud の速さで進む。読み取り用に短くした timeout で `free` が `Timeout` になり、`abort` に落ちて
  出力が切れることはない。
- `Uart::write(&[u8]) -> Result<usize, Error>` は embedded-io の `write` と同じ: 最初の 1 byte 分だけ FIFO の空きを待ち、待たずに入る分を
  入れて、入れた数を返す。`Timeout` は 1 byte も入らなかったときだけなので、同じデータを書き直しても重複しない。
  「FIFO に入った」と「線に出終わった」は別で、後者は `flush`（BUSY=0 を待つ）。`embedded_io::Read::read` は 1 回に 1 byte。

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
    fw.wait_ready(Duration::from_secs(10))?;
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
