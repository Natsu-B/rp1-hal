# 割り込み

RP1 の Cortex-M3 2 コアでの割り込みの扱い（実装済みの範囲と、その根拠）。
根拠の強さ: HW（実機で観測）、STATIC（コード・生成物・ホスト試験）、DOC（資料）、HYP（推測）。

## 1. 仕組み

各コアは自分の NVIC（外部 64 本）とベクタ表を持つ。RP1 の割り込み源 n は、両コアの NVIC の IRQ n と
PCIe MSI-X の vector n に同じ番号で出る（STATIC、regdb）。どのコアで受けるかは、そのコアの NVIC で
線を有効にするかどうかで決まる。

STATIC（`rp1-rt`）:

- `#[interrupt(proc0)]` / `#[interrupt(proc1)]` で PAC の割り込み名にハンドラを付ける。既定で ISRAM に置き、
  `#[interrupt(proc1, sram)]` で共有 SRAM に置く。1 コアに同じ線を 2 回定義するとリンクエラー。
- ハンドラの無い線は既定ハンドラに入り、その線を NVIC で無効にして戻る。`interrupt::unhandled()` が
  最後の IRQ 番号と回数を返す。
- 優先度は `Priority` で 2〜7（既定 4）。0 と 1 はシステム用（PCIe・SCMI）に空けてある。
- `interrupt::free` は PRIMASK を使うそのコアだけの critical section で、コア間のロックではない。
  システムの割り込みも止めるので 100 µs 未満に保つ。コア間のデータは `rp1_rt::sync` の型を使う。
- `log!` はハンドラの中では待たない。ロックが取れなければその行を捨てて数える。

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

USB には同じ役の `+0x98` があると標準ファームの解析から見ている（STATIC）が、実機では確かめていない。
USB の線は公開していない。Linux 側のドライバが自分の割り込みを受けるのに標準ファームと同じ出力イネーブルが
要るかどうか（rp1-hal は Linux 所有のブロックの `+0x108` / `+0x98` を立てない）は未確認（HYP）。

## 4. 根拠

各線は「源を止める → pending を消す → 源を立てる → pending を確認 → 止めて消す」の対照つきで、
その線の NVIC pending bit だけが立つことを見た。

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
| PWM0（5）、UART1〜4（42〜45） | 番号を実機で確認（ハンドラへの配送は上の試験に含まない） | HW（掃引より前） |
| SPI5（24）、DMA（40）、PIO（50 / 51）、CLOCKS（59）、VBUS（60） | 安全な刺激が無く未確認 | 公開しない |

掃引は lab のイメージだけが行い、ライブラリの起動経路には入っていない（STATIC）。

## 5. まだ無いもの

- I2C / SPI / UART の割り込み駆動の転送（FIFO 処理・エラー取得・中止を公開 API で）。
- PWM / ADC / DMA の割り込み API と、DMA の周辺転送。
- PCIe（53）と SCMI（57）の割り込み処理。今はどちらもポーリング。
- 入れ子（システム割り込み + 利用者のハンドラ）のスタック量の実測。
