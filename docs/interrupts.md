# 割り込み

RP1 の Cortex-M3 2 コアでの割り込みの扱い（実装済みの範囲と、その根拠）。
根拠の強さ: HW（実機で観測）、STATIC（コード・生成物・ホスト試験）、DOC（資料）、HYP（推測）。

## 1. 仕組み

各コアは自分の NVIC（外部 64 本）とベクタ表を持つ。RP1 の割り込み源 n は、NVIC の IRQ n と
PCIe MSI-X の vector n に同じ番号で出る（STATIC、regdb）。両コアの NVIC に同じ線が届くことは
TIMER_1・IO_BANK0・SPI0 を proc1 で見た（HW）だけで、ほかの線の proc1 側は推測（HYP、§4）。
どのコアで受けるかは、そのコアの NVIC で線を有効にするかどうかで決まる。

STATIC（`rp1-rt`）:

- `#[interrupt(proc0)]` / `#[interrupt(proc1)]` で PAC の割り込み名にハンドラを付ける。既定で ISRAM に置き、
  `#[interrupt(proc1, sram)]` で共有 SRAM に置く。1 コアに同じ線を 2 回定義するとリンクエラー。
- ハンドラの無い線は既定ハンドラに入り、その線を NVIC で無効にして戻る。`interrupt::unhandled()` が
  最後の IRQ 番号と回数を返す。
- 優先度は `Priority` で 2〜7（既定 4）。0 と 1 はシステム用（PCIe・SCMI）に空けてある。
- 既定の `interrupt::free` は BASEPRI でそのコアの利用者割り込み（優先度 2〜7）をマスクし、
  抜けるときに元の値を復元する。システム割り込みや fault のハンドラは利用者の `Mutex` に触れない。
  マスク区間は 100 µs 未満に保ち、コア間のデータは `rp1_rt::sync` の型を使う。
- `critical-section-primask` を選ぶと、システム割り込みも遅れ、区間中の構成可能 fault は HardFault になる。
  その場合、プラットフォームの遅延目標と fault 後のサービス継続は保証されない。panic ハンドラが停止前に
  PRIMASK を解除すれば panic の封じ込めは可能。bare-metal ARM の復元状態は独自実装を含めて `u8`。
- `log!` はハンドラの中ではロックを待たない。取れなければその行を捨てて数える。取れたときは
  UART が空くのを 1 バイトずつポーリングして同期送信するので、長い行はハンドラを長引かせる。

## 2. 所有

STATIC（`rp1-build`、`cargo-rp1`）: 線は宣言で所有したコアにだけ渡る。生成される `ProcNResources.irq` に
`rp1_rt::interrupt::Line<N>` が入り、`Line` で NVIC の有効・無効・pending を操作する。周辺側の
割り込みマスクはドライバ（または PAC）で別に開ける。`Line` も周辺のトークンも相手コアには無い。

| 宣言 | `ProcNIrq` のフィールド |
|---|---|
| `gpio_irq = [..]` | `IO_BANK0: Line<0>` と各ピンの `gpioN: PinIrq<N, CORE>` |
| `pwm0` / `pwm1` | `PWM0: Line<5>` / `PWM1: Line<41>` |
| `i2c0`〜`i2c3` | `I2C0: Line<7>`、`I2C1: Line<8>`、`I2C2: Line<9>`、`I2C3: Line<10>` |
| `spi0` / `spi1` / `spi3` | `SPI0: Line<19>`、`SPI1: Line<20>`、`SPI3: Line<22>` |
| `timer = ["ALARMn"]` | `TIMER_n: Line<{26+n}>` |
| `uart1`〜`uart4` | `UARTn: Line<{41+n}>` |
| `adc` | `ADC_FIFO: Line<52>` |

- 表は `rp1_build::board::IRQ_VERIFIED`。生成と ELF 検査の両方がこれを使う。
- `cargo rp1 check` / `build` は、既定ハンドラ以外のベクタを note の OWNERSHIP と照合する。所有しない線や
  表に無い線にハンドラを置くとビルドエラー。
- 渡さない線: `log` の UART0、`host_irq = true` のときの `TIMER_3`、システム用の PCIE_OUT（53）・
  PROC_MISC（57）・CLOCKS_DEFAULT（59）。53 / 57 のハンドラは proc0 のシステムサービスだけが置ける。

例: `examples/dual-core` は proc0 で `TIMER_0`（1 秒ごとの心拍）、proc1 で `IO_BANK0`（Linux からの GPIO27
ドアベル）をハンドラで受ける。

## 3. 周辺側で開けるもの

| 線 | 周辺側の操作 | 備考 |
|---|---|---|
| GPIO（IO_BANK0） | `PinIrq::enable(Event)`、`force()` | ピンの設定（`into_input` など）は GPIOn_CTRL を書き直してイベントを消すので、`enable` の前に済ませる |
| TIMER | `Alarm::enable_interrupt`、`clear_interrupt` | |
| UART | `enable_rx_interrupt` / `enable_tx_interrupt`、`interrupt_status` | |
| I2C / SPI | `enable_interrupt(mask)`、`interrupt_status`、`clear_interrupt` | 下の `+0x108` を開ける |
| PWM / ADC | ドライバに API は無い（初期化で INTE = 0） | 線は所有できる。源を開けるのは PAC の生書き込み（ドライバ化は後回し） |

I2C と SPI の割り込みは、コントローラのマスクの先にある RP1 の出力イネーブル（ブロックの `+0x108`、
`IRQ_EN` = bit 0）を通る。閉じていると、マスクを外した源が立っていても NVIC に届かない（HW）。
`enable_interrupt` はマスクを外した後にこれを開け、`free()` / `abort()` と初期化は全源をマスクしてこれを閉じる
（STATIC、ホスト模型）。ブロッキングの転送は実行中だけ全源をマスクし、終わると元のマスクに戻す。
転送そのものは今もブロッキングで、割り込み駆動の転送 API は無い。同じコントローラで割り込み駆動と
ブロッキングを混ぜる使い方は対応しない。

Linux に残したブロックには、標準 RP1 ファーム（ブートローダー内蔵）が初期化の最後に全ブロックへ書く
イネーブルの表と同じ値を `linux_blocks::bring_up` が立てる（RMW。表の番地と値は STATIC）。対象は Linux 用に
リセットを解除したインスタンスだけで、宣言したブロックのものはそのドライバが持つ。このうち割り込みの出力
イネーブルと分かっているのは ETH_CFG INTE と I2C / SPI の `+0x108` で、無いと Linux のドライバが割り込みを
受けない（HW: macb の ARP が終わらない、i2c_designware / spi-dw が timeout）。USB cfg の `+0x98` と
I2S の `+0x408` = 0xffff は、何をゲートするか分からないまま標準ファームの値を写している（USB は `+0x98` = 0 でも
xhci の割り込みが届く: HW。I2S は未試験）。SDIO（標準ファームは `+0x28` = 3）は rp1-hal がリセットを解除しないので書かない。
USB の線は公開していない。

## 4. 根拠

掃引の行（I2C0/2/3・SPI1/3・ADC_FIFO・PWM1・IO_BANK1/2）は「源を止める → pending を消す → 源を立てる →
pending を確認 → 止めて消す」の対照つきで、その線の NVIC pending bit だけが立つことを見た。掃引は proc0 の
NVIC だけを見ている。proc1 で受けたのは TIMER_1・IO_BANK0・SPI0 だけで、掃引した線を proc1 で受けられる
ことと、掃引した線でハンドラに入ることは、同じ経路の線からの推測（HYP）。

| 線 | 結果 | 根拠 |
|---|---|---|
| TIMER_0 / TIMER_1（26 / 27） | proc0 / proc1 のハンドラに 1000 回、1 s で 1000 回、IPSR 一致 | HW（2026-11、cold boot 2 回） |
| TIMER_2（28） | ハンドラ無し: 既定ハンドラが 1 回入り、線を無効にして記録 | HW（同上） |
| IO_BANK0（0） | proc0 は GPIO22 の force で 100 回、proc1 は Linux からの GPIO27 ドアベルで 1000 回 | HW（同上） |
| I2C1 / SPI0（8 / 19） | `+0x108` 閉で pending 0、開でハンドラ 1 回 | HW（同上） |
| critical section | 割り込みと競合させて torn 0、最長マスク 4 µs 以下 | HW（同上） |
| I2C0 / I2C2 / I2C3（7 / 9 / 10）、SPI1 / SPI3（20 / 22） | TX FIFO 空を源に、`+0x108` 閉で 0/3、開で 3/3 | HW（2026-11、2 種のイメージで各 2 起動、同じ結果） |
| ADC_FIFO（52） | force と内部温度の実変換の両方で 3/3 | HW（同上） |
| PWM1（41） | force で 3/3 | HW（同上） |
| IO_BANK1 / IO_BANK2（1 / 2） | proc0 の INTF force で 3/3 | HW（同上）。ピンが Linux のものなので公開しない |
| PWM0（5）、UART1〜4（42〜45） | 番号を実機で確認（ハンドラへの配送は上の試験に含まない） | HW（rp1-hal より前の試作ファーム。rp1-hal では再確認していない） |
| TIMER_3（29） | rp1-hal では `TIMER.INTF` bit 3 で Linux の MSI-X vector 29 が起きるのを確認（`host_irq`、design.md §6.2）。NVIC 29 に届いたのは試作ファームでだけ | MSI-X は HW（2026-11）、NVIC は HW（試作ファーム）。rp1-hal では NVIC を再確認していない |
| Linux に残したブロック（上の表を立てた rp1-hal ファーム） | Linux 側で割り込みが届くのを見たのは I2C2 / I2C6 / SPI1 / ETH / USB。I2C0/3/4/5、SPI2..8、I2S は値が標準ファームと同じことまで | HW（2026-11、cold boot と再起動で各 1 回） |
| SPI5（24）、DMA（40）、PIO（50 / 51）、CLOCKS（59）、VBUS（60） | 安全な刺激が無く未確認 | 公開しない |

PWM0 と TIMER_3 は試作ファームでの観測を根拠に `IRQ_VERIFIED` に入れて公開している。線の配線は RP1 の
ものなのでファームが変わっても同じはず（HYP）だが、rp1-hal で再確認するまで regdb ではこの 2 本を `static` にしてある。

掃引は lab のイメージだけが行い、ライブラリの起動経路には入っていない（STATIC）。

SCMI（57）は `platform::init` が proc0 に設定する（STATIC）。

PCIe（53）は `platform::init` の最後に proc0 で PERSTn の検出器として武装する（優先度 0、STATIC。
HW 未確認）。`PCIE_INTE` を bit 1（PERSTn の辺）だけにし、入場で `PCIE_INTS` に何か立っていれば
窓を失ったとして `SHM_STATUS = SHM_WINDOW_LOST`、`PLAT_PCIE = Lost` を書いて線を止める。回復はしない
（RP1 の再ロード）。`PCIE_INTS` が 0 の入場が 16 回続くと検出器を止める（`DetectorOff`）。状態は
`platform::pcie_state()`（両コア）と `Pcie::lost_detail()`（proc0）で読める。
`rp1::link::try_enable(&mut pcie)` は検出器が `Up` のときだけ公開し、検査と書き込みを PRIMASK の区間で行うので、
公開と検出が入れ違わない（区間 B の検査の後に始まったリセットは、区間を抜けた直後に IRQ53 が書き換える。STATIC）。

## 5. まだ無いもの

- I2C / SPI / UART の割り込み駆動の転送（FIFO 処理・エラー取得・中止を公開 API で）。
- PWM / ADC / DMA の割り込み API と、DMA の周辺転送。
- 入れ子（システム割り込み + 利用者のハンドラ）のスタック量の実測。
