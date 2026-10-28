# rp1-hal 設計（2026-10-23 版）

根拠の強さ: **HW** = 実機で測定、**STATIC** = 逆アセンブル、**DOC** = 資料・Linux ソース、**HYP** = 仮説。
根拠の出典は `<lab>`（以下 KG）、
`<lab>`、`<lab>`（旧草案）。

## 1. 方針

1. rp-hal と同じ使い勝手（PAC / 所有権トークン / 型状態 GPIO / `Clocks` / embedded-hal 1.0）で RP1 の M3 を書ける。
2. proc0 と proc1 は **1 イメージ**。コア間で共有データを直接受け渡しできる。
   その代わり「どのコアで動く関数か」「どのコアから触るグローバルか」をビルド時に検査する（§5）。
3. **Linux 側はカーネル内部 API に依存しない。** DT の変更とユーザープロセスだけで連携する（§6）。
4. **RTOS は載せない。** 割り込み駆動のドライバとコアごとの main ループ。非同期が欲しければ Embassy executor をコアごとに 1 つ（任意）。
5. 所有権・メモリ・通信路の宣言は **ファーム crate の `Cargo.toml` の `[package.metadata.rp1]`** に書く（別ファイルは置かない）。
   ビルドはこれを読み、型・linker script・`.note.rp1` を生成する。ローダーは `.note.rp1` だけを読む（§4）。
6. BCM2712 側 third stage loader は、Raspberry Pi の通常の起動（SD / TFTP、config.txt、cmdline.txt、overlay、initramfs）に
   1 段挟まるだけの素直な形にする（§7）。

## 2. 構成

```
crates/
  rp1-pac      regdb → SVD → svd2rust。Interrupt / device.x を含む
  rp1-rt       1 イメージ・2 コアの起動。メモリ展開、#[entry] #[proc0] #[proc1] #[isram] #[shared]
  rp1-hal      ドライバ、embedded-hal、Clocks、Resets、multicore、sync（Shared / Spsc / SeqLock / ExacMutex）
  rp1-proto    no_std。.note.rp1 v2、リング、ドアベル規約、wire 形式（M3 / ローダー / Linux 共通、ホストで試験）
  rp1-link     M3 側の Linux 連携（チャネル、bulk、SCMI clock server）
  rp1-boot     no_std aarch64。RP1 の I2C bootstrap、ELF と note の検査、overlay の適用（libfdt）と衝突検査
  rp1-build    build.rs 用。metadata → 型・linker script・note・DT overlay の生成（ボード表を同梱）
boot/rp1boot   BCM2712 ローダー本体（SD / TFTP）
linux/rp1-linux ユーザー空間ライブラリと CLI（UIO）
tools/         rp1-regdb、cargo-rp1（ビルド・ELF 検査・バンドル・run）
boards/rpi-cm5 ヘッダピン名、既定の所有権
examples/      両コア + Linux 連携の最小例
lab/           実験・診断（公開 API ではない）
```

依存の向き: `rp1-pac ← rp1-hal ← rp1-link`、`rp1-proto` は M3・ローダー・Linux の全部から使う。
`rp1-hal` は Linux も AMP も知らない（Linux 無しでも使える）。

## 3. メモリの使い分け

### 3.1 事実

| 領域 | 大きさ | 見える相手 | atomic RMW | 速さ | 根拠 |
|---|---|---|---|---|---|
| 共有 SRAM `0x2000_0000` | 64 KiB | 両コア、Linux（BAR2）、DMA | **壊れる**（STREX が失敗を返しつつ書く） | 遅い | HW（KG-a §1.5、G2） |
| ISRAM `0x1000_0000` | 各コア 8 KiB | 自コアのみ（同アドレス別実体） | 可 | 共有 SRAM 実行の約 3 倍 | HW（PROF v3） |
| DSRAM `0x1000_2000` | 各コア 8 KiB | 自コアのみ（同アドレス別実体） | 可 | 読みは 20 倍以上 | HW |
| Linux DDR（outbound 窓） | 窓は 0x8000_0000..0xbfff_ffff の 1 GiB（DOC）。実績は 64 KiB × 2 | M3 と Linux | — | PCIe 越し。M3 からの読み出し遅延は未測定 | HW（SHM）/ DOC |

- ROM は proc0 DSRAM `0x1000_2000..0x1000_30d0` を起動時と watchdog 再起動時に上書きする（STATIC）。
- proc1 は ROM から entry に入る時点で MSP = `0x1000_30d0`（自分の DSRAM）（HW + STATIC）。
- 公式 FW は IRQ53 ハンドラと PCIe EP 状態機械を ISRAM に置く（STATIC、KG-a）。
- 共有 SRAM 末尾 `0x2000_ff00..` は rp1-fw ABI 予約（STATIC）。

### 3.2 配置規則

| 置き場所 | 置くもの | 書き方 |
|---|---|---|
| 共有 SRAM | 全コードと rodata のロード元、`#[shared]` static、Linux から見える領域、DMA バッファ、再起動をまたぐ warm data | `#[shared]` を付けた static（型は §5.3 のものだけ） |
| ISRAM（コアごと） | ISR、ホットループ、実行時の PCIe 状態機械 | `#[isram]`（v0.1 は手で付ける、§3.3） |
| DSRAM（コアごと） | スタック、既定の `.data` / `.bss`、`#[proc0]` / `#[proc1]` static、atomic、executor 状態 | 普通の static（§5 の検査を受ける） |
| 共有 SRAM `0xff00..` | 予約 | linker ASSERT |
| Linux DDR | Linux との共有メモリ（`ring` / `channel` / `block`）。block は「遅くて大きい」外付け RAM としても使う | 宣言 `[[package.metadata.rp1.shm]]`、`Shm::enable` 後のハンドル。api-sketch §4.3 |

既定の `.data` / `.bss` を DSRAM に置く理由: 依存 crate が static の atomic で RMW をしても壊れない。
大きなバッファは DSRAM に入らないので明示的に `#[shared]` にする。

### 3.3 ISRAM に置くものの決め方

ライブラリ全体を ISRAM に入れるのではなく、速さが要る部分だけを入れる。

**v0.1**: 手で付けた `#[isram]` の関数と ISR だけを ISRAM に置く。両コアに同じ ISRAM イメージを展開する（共通、各コア 8 KiB）。
予算超過は linker の ASSERT でエラー。以下の自動配置は v0.2 で検討する。

**v0.2 案（自動配置）**:

- 根（root）: `#[isram]` を付けた関数と、各コアのベクタ表に載る ISR。
- `cargo-rp1` が最終 ELF の呼び出しグラフから根の到達集合を求め、それを ISRAM に配置する（2 段リンク）。
  - 片方のコアだけから到達する関数 → そのコアの ISRAM overlay。
  - 両コアから到達する関数 → ISRAM の「共通部分」。同じアドレスで両コアの ISRAM にコピーする（両コアの予算を同じだけ使う）。
  - それ以外（初期化コード、まれにしか呼ばない関数）は共有 SRAM のまま。両コアから呼べ、ISRAM を消費しない。
- 予算超過、または根から 64 bit 除算・`core::fmt`・panic の書式付き経路に到達したら、警告かエラーにする。

見積もり（現 PoC の `RP1.elf` 56d2e249 を静的解析）: コード全体 40.8 KB（FreeRTOS と OpenAMP を含む）。
proc1 の entry とホットループからの到達集合は 6.9 KB / 38 関数（本体 2.7 KB、64 bit 除算 0.9 KB、プロファイラと初期化を含む）。
ホット部分だけなら 3〜4 KB 程度で 8 KiB に収まる見込み。公式 FW が ISRAM に置いている量は 2.6 KB（STATIC）。
共有 SRAM のアクセスは 1 回 1 µs 未満、コア間の競合でも約 2 倍（HW、§3.4）なので、ISRAM に置かなくても動作はする。

### 3.4 PCIe 初期化中の共有 SRAM

実機確認（2026-10-23、1 cold boot、`<lab>`）:
proc0 が RP1 側の PCIe エンドポイントを立ち上げ、ホストが RC を再初期化して BAR2 を読む区間に、proc1 が ISRAM から共有 SRAM を
3 秒間計測した。**データ不一致 0、フォルト 0、1 アクセス最大 42 サイクル（約 50 MHz 時）**で、PCIe の再初期化の時点に段差は無かった（HW）。
最初の約 1.1 秒だけ 1 回の計測あたり 40→最大 92 サイクルに増えたのは、proc0 が共有 SRAM 上のコードを実行していたためのコア間競合と見られる（推論）。
「不定」に見えたのはホスト側だけで、リンクが落ちている間は BAR2 の読み出しが 0xffffffff になる（HW）。
未確認: Linux 起動時の PCIe probe、実行中の PERST / hot reset、ホスト DMA、200 MHz 時。

このため規則は「共有 SRAM が壊れる」前提ではなく、次の 2 点を前提にする。
(a) リンクが落ちている間、ホストから見た BAR2 は無効、(b) 実行時の PCIe 状態機械（v1.0）は未確認なので ISRAM に置く。

規則:

1. cold boot のエンドポイント立ち上げ中も、M3 は共有 SRAM を通常どおり使ってよい（HW、1 boot）。コアを止める・PCIe 用の特別な区間を設ける必要はない。
2. Linux から見える領域（BAR2）は世代番号を持ち、リンク確立後に新しい世代を書いてから有効化する。Linux 側は 0xffffffff や世代の変化を見たら読み直す。
3. 実行時の PCIe イベント（Linux の PCIe probe、PERST、hot reset、L2）は未確認。v1.0 の PCIe 状態機械は ISRAM に置き、同じ計測（proc1 サンプラー）で確認してから規則を緩める。
4. コア間の競合で共有 SRAM アクセスは約 2 倍まで遅くなる（HW）。遅延が問題になる処理は ISRAM / DSRAM に置く（§3.2、§3.3）。

## 4. 所有権の宣言と `.note.rp1`

### 4.1 宣言（ビルド前に必要）

ファーム crate の `Cargo.toml` に書く。省略した項目は `boards/rpi-cm5` の既定値。

```toml
[package.metadata.rp1]
board = "rpi-cm5"
clock = { authority = "rp1", clk_sys_hz = 200_000_000 }   # authority = "linux" なら RP1 は clock を書かない

[package.metadata.rp1.proc0]
peripherals = ["UART0", "TIMER.ALARM0"]
pins = [14, 15]

[package.metadata.rp1.proc1]
peripherals = ["TIMER.ALARM1"]
pins = [22]

[package.metadata.rp1.link]                                 # Linux 連携
channels = [{ name = "ctl", core = "proc1", doorbell = "gpio-intf:27", ring = 4096 }]
bulk = [{ name = "obs", size = 65536 }]
```

`rp1-build`（build.rs）がこれを読んで次を生成する:

- コアごとの `Resources` 型（宣言していない周辺は存在しない → 使うとコンパイルエラー）
- `memory.x`（共有 SRAM の区画、ISRAM / DSRAM overlay、予約）
- `.note.rp1` の中身

`cargo rp1 check --dtb <firmware の DTB>` で、宣言と Linux DT の衝突（Linux が使っているピン・周辺・IRQ）をデプロイ前に検出する。

### 4.2 `.note.rp1` v2

詳細な形式は [declaration-and-note.md](declaration-and-note.md) §2。

旧 `Rp1BootInfoV1`（固定長、所有物は 64 bit のビット列）を TLV に広げる。ローダーと Linux 側ツールはこれだけを読む。
中身: ABI 版、entry / スタック、メモリ配置、コアごとの所有物（ピン・ブロック・IRQ）、clock 権限、通信路（領域・ドアベル・UIO 範囲）、
ファーム版、宣言のハッシュ。形式の正本は `rp1-proto`。

## 5. コアの区別とグローバル変数（誤用を「書けなくする」）

方針（2026-10-23 変更）: リンク後に誤用を見つけてエラーにするのではなく、型と配置で誤用を書けない形にする。
リンク後の ELF 検査は、確実に分かる事実（共有 SRAM 上の LDREX/STREX、ISRAM・DSRAM の予算）だけに使う。

### 5.1 規則

| 対象 | 置き場所 | 共有されるか | 守る仕組み |
|---|---|---|---|
| 普通の `static`（自分のものも外部 crate のものも） | コアごとの DSRAM に別コピー | されない（thread_local と同じ意味） | 配置そのもの。両コアから触っても競合は起きない |
| `#[shared] static` | 共有 SRAM | される | 型が `SharedSafe`（RMW atomic を持たない `Shared<T>`、`Spsc`、`SeqLock`、`ExacMutex`）でなければ rustc エラー |
| `#[proc0] static` / `#[proc1] static` | DSRAM | されない | `Local<T, Proc0>` に包まれ、`Proc0` トークンが無いと読めない（rustc） |
| コア専用の処理 | — | — | 資源（`rp1::Proc0Resources` の中身）やトークンを引数に取る。相手コアは持っていないので呼べない（rustc） |
| コア間で渡すもの | — | — | 値渡し（`Copy`）か `#[shared]` 領域へのハンドルだけを受け付ける API。DSRAM へのポインタは渡せない（rustc） |
| 割り込み | 各コアのベクタ表 | — | `#[interrupt(proc1)]` でコアを明示 |

### 5.2 リンク後の ELF 検査（cargo-rp1、v0.1）

- 共有 SRAM を対象にする LDREX/STREX があればエラー。
- ISRAM・DSRAM・共有 SRAM の予算超過はエラー（linker の ASSERT と同じ内容を分かりやすく表示）。
- 呼び出しグラフの到達解析は v0.1 では行わない（ISRAM の自動配置とあわせて v0.2 で検討）。

### 5.3 この方針の副作用（受け入れる）

- DSRAM は各コア 8 KiB からスタックを引いた残り。外部 crate の大きな static は DSRAM に入らずリンクエラーになる。その場合は
  大きなバッファを `#[shared]` にするか、外部 crate のセクションを共有 SRAM に移す指定（宣言で `shared_sections = [...]`）を使う。
- 外部 crate の「一度だけ取れる」仕組みはコアごとに別のフラグになり、両コアで取れる。rp1-hal の資源は宣言から型で分けるので影響しない。
  抜け道の `pac::Peripherals::take()` は `unsafe fn` にする（両コアで取れること・宣言を迂回することを利用者が引き受ける）。`steal()` も `unsafe`。
  `cortex_m::Peripherals::take()`（NVIC、SysTick など）はコアごとに別の実体なので、コアごとに取れるのが正しい。
- 両コアに同じ static があることを「1 つのグローバル」と思って使うと、値が共有されない。`#[shared]` を使うよう文書と例で示す。

### 5.4 共有に使える型

`Shared<T: Copy>`（volatile load/store のみ）、`Spsc<T, N, Writer, Reader>`（1 書き手 1 読み手、コアを型引数で固定）、`SeqLock<T>`、`ExacMutex<T>`（EXAC によるコア間排他、HW H8）。
RMW atomic のメソッドは持たない。

## 6. Linux 連携（DT とユーザープロセスだけ）

| 機能 | 方法 | 依存するもの |
|---|---|---|
| RP1 が使う周辺を Linux から外す | ローダーが note に従い DT ノードを `disabled`（`__symbols__` のラベルで探す） | DT のラベル |
| ドアベル・リング・bulk | DT の `generic-uio` ノード（共有 SRAM、doorbell、reserved-memory、IRQ）→ `/dev/uioN` で mmap と IRQ 待ち | UIO の公開 ABI |
| clock（RP1 が権限を持つ場合） | clk-rp1 を SCMI（`arm,scmi` + rp1 mailbox + shmem）に差し替え | SCMI の DT binding |
| RPMsg | 使わない（`lab/` に残す） | — |

### 6.1 rp1-proto と OpenAMP/RPMsg の比較

どちらも「共有メモリ上のリング + 通知」で、物理的な経路は同じ。違いは Linux 側の層。

| | OpenAMP/RPMsg（旧 PoC） | rp1-proto |
|---|---|---|
| 実測 | RTT p50 5.7 ms / p99 8.1 ms、179 往復/s（496 B エコー、M3 は FreeRTOS タスク、`reports/openamp-rpmsg.md`） | ドアベル RTT p50 4.1 us / max 8.0 us（proc1 ISRAM ISR、Linux はビジーポーリング、`<lab-log>`） |
| Linux の経路 | mailbox IRQ → remoteproc / virtio / rpmsg bus → rpmsg_char → read/write（コピー） | UIO で mmap（ゼロコピー）。待ち方は IRQ（`read(/dev/uio)`）かポーリングを選べる |
| 1 メッセージ | 最大 496 B | 制限なし（リングの設計次第） |
| カーネル依存 | `stm32-rproc` の attach 挙動、`COMPILE_TEST` + `STM32_RPROC` が必要 | UIO の公開 ABI のみ |

上の 2 つの実測は条件が違う（RTOS の tick、通知方式）ので直接は比べられない。仕組みの上では rp1-proto が遅くなる理由は無い。
注意点:

- **UIO の IRQ 待ちの遅延は未測定。** カーネルの割り込み → ユーザープロセス起床で数十 us 級と見込む（HYP）。低遅延が要る通路はポーリングを使う。
- **キャッシュ**: UIO の物理メモリ mmap は非キャッシュになるので、Linux DDR の bulk を CPU で大量に読むと遅い。
  必要なら cacheable に mmap し、ユーザー空間から DC CIVAC で保守する（arm64 Linux は EL0 からの DC 命令を許可、DOC）。測定して決める。
- 多重化、流量制御、名前解決、標準ツール（rpmsg_char、tty）は自前で用意する必要がある。

### 6.2 検証の状況（2026-10-28、`<lab-log>`）

- **済（HW）**: `uio_pdrv_genirq` に `of_id=generic-uio` で bind できる。BAR2 末尾 16 KiB（0x1f0040c000）と GPIO INTF ページ
  （0x1f000d0000、pinctrl と同じページ）を RW で mmap でき、読んだ値は /dev/mem と一致。書き込みは未試験。
- **否定（HW）**: HOST_EVENTS → hwirq58（Linux の IRQ159）は `rp1-mailbox` が専有していて、同じ hwirq の UIO ノードは
  `Flags mismatch` / -EBUSY で probe に失敗する。両ドライバとも IRQ 共有を許さない（STATIC）ので、イベント bit を分けても共有できない。
- **mailbox を外して UIO が hwirq58 を取る案（HW 2026-10-28、`<lab-log>`）**:
  overlay で `rp1-mailbox`・`/firmware/scmi`・OpenAMP `remoteproc` を `status = "disabled"` にし、generic-uio ノード
  （SYSCFG ページ + BAR2 末尾 16 KiB、`interrupts = <58 4>`）を足すと、Linux は起動し UIO が IRQ159（hwirq58）を取れる。
  - 往復（Linux が SCMI shmem に要求を書き PROC_EVENTS を立てる → M3 proc0 の SCMI サーバーが応答し HOST_EVENTS を立てる）:
    ビジーポーリング p50 66.2 / p99 66.9 µs、UIO の `poll`+`read` p50 71.1 / p99 88.8 / max 101.5 µs（各 2000 回）。
    約 60 µs は M3 側の SCMI 処理で、IRQ 待ちの上乗せはポーリング比 p50 で約 +5 µs、p99 で約 +22 µs（分布の差、HW）。
  - **注意（HW）**: 1 回の通知につき UIO の起床が 2 回起きる。HOST_EVENTS を消して `write(1)` で再度有効にした直後に、
    もう 1 回届く（原因は HYP: マスク中に保持された割り込みがアンマスクで再送される）。受け手は起床を「合図」として扱い、
    リングの状態で判断する（空振りを許す）。
  - **衝突（HW+STATIC）**: SCMI（`mboxes = <&rp1_mbox 0>`）が使えなくなり、SCMI クロックを使う serial@34000 は deferred のまま。
    `clock.authority = "rp1"`（既定）とは両立しない。
- **v0.1 の決定（2026-10-28、feat/v0.1-core の §9）**: rp1-mailbox と SCMI は残し、proc1 → Linux はポーリングで待つ。
  IRQ が必要になったら、(a) RP1 内の未使用周辺の割り込み線を M3 から立てて UIO で受ける（HYP）、(b) `clock.authority = "linux"`
  の構成でだけ上の mailbox を外す案を使う（HW で成立）、のどちらかを検討する。
- **配置（STATIC）**: 今の SCMI shmem（`scmi@d7c0`、0x100 byte）は BAR2 末尾 16 KiB の中にある。link ページ全体を使う前に
  SCMI shmem を移すか、link の配置でその範囲を予約する。
- **済（HW）**: UIO mmap（BAR2 SRAM）への書き込みは M3 から見える（SCMI 要求 2000/2000 が解釈され応答）。
- 未測定: hwirq58 以外の IRQ 経路、M3 から見た Linux DDR の遅延・帯域。

## 7. BCM2712 third stage loader（`boot/rp1boot`）

```
VPU firmware（config.txt / cmdline.txt / overlay / initramfs を通常どおり処理。rp1 用の設定は書かない）
  → kernel=rp1boot.img
      1. 自分が起動した経路（/chosen/bootloader の boot-mode）で SD(FAT) か TFTP を選ぶ
      2. 同じ経路・同じ配置から rp1.elf と本来のカーネルを読む
      3. .note.rp1 を検査し、ELF の .rp1.dtbo を DTB に最後に当てる（libfdt）。当てた後の DTB で、他のノードが RP1 のピンを
         pinctrl で取っていないかを検査する（当たれば停止）
      4. I2C で RP1 を bootrom に戻して書き込み、起動する。共有メモリの物理アドレスをハンドオフ語に書く
      5. overlay を当てた DTB で、本来のカーネルへ EL2 でジャンプ（initramfs と cmdline は firmware が用意したもの）
```

- Linux 用の DT の調整はビルド時に overlay として生成して ELF に入れ、ローダーが最後に当てる（declaration-and-note.md §3）。
- SD と TFTP は「ファイルを読む部分」だけが違う。ファイル名と配置は同じ。
- 旧 PoC の実験用 feature（約 50 個）と hash 固定値は持ち込まない（`lab/`）。
- 未検証: firmware が置いた initramfs を上書きせずに引き継げるか、`/chosen` をそのまま使えるか。

## 7.1 Linux 共有メモリの確保（`shm`）

詳細は api-sketch §4.3。物理アドレスはビルド時にボード表の固定領域から決め、overlay の `reserved-memory`（no-map、連続）と
shm ごとの `generic-uio` ノードに焼き込む。ローダーはそのアドレスを共有 SRAM のハンドオフ語に書く。
ゼロ化はファームが `Shm::enable` で行う（ローダーは DDR を触らない）。

STATIC（v0.1 実装）: rpi-cm5 は `0x2200_0000`、予約全体は 1..16 MiB。既存の scratch/openamp 予約を拡張しない。
Linux の PCIe probe/PERSTn 完了後、UIO の link ページで初期化を要求し、proc0 が `rp1_hal::shm::enable` で ATU と領域を初期化する。
具体的な API・ローダーの開発用パッチ・検証方法は [lab/README.md](../lab/README.md)。
HW（hwrun3、同じ組で cold boot 2 回）: 2 MiB 予約の全語を M3 窓から書いて読み戻し一致、ring は各向き 10^6 件で欠落・破損 0、
block 1 MiB は両向き全語一致。M3 から DDR: 読み遅延 約 1.32 µs/語、走査読み 約 3.0 MB/s、書き 約 5.9 MB/s。
ring は RP1→Linux 0.76 MB/s、Linux→RP1 0.35 MB/s（16 byte）で、Linux を止めても M3 側の push 約 20 µs / pop 約 43 µs が律速。
Linux 側 uncached の block 読み 約 335 MB/s、書き 約 247 MB/s。M3 側が遅い原因は未特定（HYP: 共有 SRAM からの命令取得と proc0 のポーリングとの競合）。
ファームは `Shm::enable` で outbound ATU を設定してからハンドルを返す。PCIe リセットで ATU は消える（HW、2026-09-29）ので v0.1 は cold boot のみ保証。

測ること（最初の縦 1 本で）: M3 からの DDR 読み出し遅延・帯域・書き込み帯域、Linux 側 uncached（UIO）と cached（/dev/mem + DC CIVAC）の読み書き速度、UIO の IRQ 待ち遅延。

## 8.1 v0.1 の範囲

| 入れる | 後回し（v0.2 以降） |
|---|---|
| 2 コアのランタイム（1 イメージ、`#[entry(proc0/1)]`、proc1 起動、DSRAM の既定配置、`#[shared]`/`#[proc0]`/`#[proc1]`） | ISRAM の自動配置（2 段リンク）と到達解析 |
| 手で付ける `#[isram]`（共通 ISRAM イメージ） | コアごとの ISRAM overlay |
| 宣言 → 型・linker script・note・overlay の生成（rp1-build） | `channel` の正式 API（v0.1 は `ring` 2 本で組めることを例で示す） |
| 起動一式（config.txt、cmdline.txt、rp1boot.img、rp1.elf）の生成と照合 | async / Embassy |
| ローダー（SD / TFTP、overlay 適用、config.txt 照合、RP1 起動） | Linux 側の cached マップ |
| Linux 共有メモリ `ring` と `block`（UIO、uncached） | 実行時の PCIe 状態機械、Linux 再起動への追従 |
| SCMI クロックサーバー（clock.authority = rp1 が既定のため必須） | DMA / PIO / ADC / I2S / SPI / I2C のドライバ |
| `log!`（UART0） | `log!` の Linux 通信路への出力 |
| GPIO / UART / timer（alarm）ドライバと embedded-hal | |
| ELF 検査（共有 SRAM 上の LDREX/STREX、予算） | |

## 8. 進め方（最初の縦 1 本）

1. 静的: `rp1-proto`（note v2）、`rp1-build`、`rp1-pac`、`rp1-rt`、`rp1-hal`（GPIO / UART / timer）、ELF 検査（§5.2）、`rp1boot`（SD / TFTP）。
2. 実機: proc0 と proc1 が UART に出力し、`#[shared]` の `Spsc` で値を受け渡す。Linux は overlay を当てた DTB で起動し、UIO 経由でドアベルを往復させる。
   §6.2・§7 の未検証項目と、§3.4 の Linux 起動時の PCIe probe をここで確かめる。

## 9. 決定の記録

| 日付 | 決定 | 理由 |
|---|---|---|
| 2026-10-23 | 新リポジトリで作る。旧スタンドアロン系列は `<local>` に改名 | PoC の診断コードと feature を持ち込まない |
| 2026-10-23 | RTOS を載せない | RPMsg を外すと必要性がない |
| 2026-10-23 | Linux はカーネル内部 API に依存しない（DT + ユーザープロセス） | カーネル版ごとの変化に追従しない |
| 2026-10-23 | proc0 / proc1 は 1 イメージ | コア間の共有データ受け渡しのため |
| 2026-10-23 | 宣言は `[package.metadata.rp1]`、ローダーは `.note.rp1` を読む | 別ファイル不要、ビルド前に宣言が存在する |
| 2026-10-23 | コア別の関数・static とグローバル変数の両コア到達はビルドエラー | 1 イメージの安全性 |
| 2026-10-23 | ローダーは SD / TFTP を通常の起動の形のまま扱う | 素直な構成 |
| 2026-10-23 | api-sketch §8 の 1〜6 を了承。`log!` を v0.1 に入れ、UART0 専用。RP1 が UART0 を所有しない構成で `log!` を使えばエラー | ユーザー判断 |
| 2026-10-23 | Linux の DDR を共有メモリにし、`ring` / `channel` / `block` から選ぶ。block は外付け RAM（遅く大きいメモリ）を兼ねる | ユーザー要望・提案 |
| 2026-10-23 | 「PCIe 初期化中に SRAM が不定」は記憶違いと確認。実機でも問題なし（§3.4） | ユーザー確認 + HW |
| 2026-10-23 | Linux 用の DT 調整は rp1-build が overlay として生成し、ELF に入れてローダーが最後に当てる（libfdt）。ローダーは当てた後の DTB の衝突も検査 | ユーザー提案・指摘 |
| 2026-10-23 | クロックは RP1 が SCMI で管理する（`clock.authority = "rp1"` が既定）。Linux に任せると、Linux から見て利用者のいない RP1 専用ブロックのクロックを `clk_disable_unused` が止めるため | ユーザー判断 |
| 2026-10-23 | 誤用はリンク後に検出するのではなく、型と配置で書けなくする（§5）。v0.1 では到達解析をしない | ユーザー指摘 |
| 2026-10-23 | config.txt と cmdline.txt を成果物（起動一式）にし、必ずそれを使う。ローダーがハッシュを照合する | ユーザー提案 |
| 2026-10-23 | v0.1 の範囲を絞る（§8.1） | ユーザー許可 |
| 2026-10-23 | ローダー: 大きなファイルは VPU ファームに束として読ませる。config.txt の使用は束と cmdline のハッシュで確かめる。失敗時は公式ブートローダーの BOOT_ORDER の規則に従う（loader.md） | ユーザー判断 |
| 2026-10-28 | v0.1 の `clocks::Authority` は PLL / clk_sys を書かず、EEPROM 段の RP1 ファームが残した状態（PLL_SYS 1 GHz・PRIM 5×1、CLK_UART = XOSC/1）を読んで照合する（`adopt`）。違えばエラー | 冷起動から 200 MHz にする手順は PoC でも実機未確認（PoC は同じ組を読み取りで受け入れていただけ、HW）。未確認の書き込みを既定経路にしない |
| 2026-10-28 | v0.1: rp1-mailbox/SCMI kept; proc1→Linux events by polling; alternate host IRQ path is an open TODO (hwirq58 is owned by rp1-mailbox, verified HW 2026-10-28)。overlay は mailbox と SCMI に触れず、UIO ノード（link・GPIO INTF）は interrupts を持たない。無効にするのは OpenAMP remoteproc だけ。`gpio-reserved-ranges` は基の DT の予約と宣言ピンの和にする（Linux に渡るピンを増やさない、`cargo rp1 check --dtb` で照合） | SCMI クロック権限は必須（上の 2026-10-23 の決定）なので transport を奪えない。UIO は hwirq58 を共有できず -EBUSY（HW、Codex A）。ポーリングのドアベル RTT は約 4 µs（HW、§6.1） |
| 2026-10-28 | 上の `adopt` だけの決定を訂正する。PoC ローダーは RP1 を I2C でリセットしてから起動するので、v0.1 のファームは冷状態（PLL_SYS リセット中、CLK_UART 停止、UART0 リセット中、PCIe エンドポイント未設定）から始まる。`rp1_hal::cold` で PoC ファームと同じ手順（PLL/PRIM/CLK_UART、エンドポイントクロック、PCIe state 1..5、SYSCFG）を実行してから `adopt` で照合する。Linux の brcm-pcie probe の PERSTn でエンドポイントが既定値に戻るので、proc0 が `EndpointService` をポーリングして DBI 設定を再適用する | HW（hwrun1 2026-10-28）: adopt だけでは UART0 無出力・LinkTimeout。移植後はリンク 12.4 ms、PERSTn 後の再設定 105 µs、Linux の rp1 が probe し、ドアベル RTT は 1000 回で p50 5.87 / p99 5.95 / max 32.7 µs |
| 2026-10-28 | `.bss` は PT_LOAD に含めない（`:NONE`）。各コアが起動時に自分の DSRAM で 0 にする | ローダーは p_paddr から p_memsz を展開し重なりを拒否するため、DSRAM に再配置した `.bss` の memsz が共有 SRAM の LMA に被る（STATIC、PoC ローダーのコード） |
| 2026-10-28 | SCMI クロックサーバー（`rp1_hal::scmi`）: Base + Clock 2.0、mailbox ch0 + shmem `0x2000_d7c0`、proc0 のループでポーリング。`Authority::adopt` が `scmi::Service` を返す。公開クロックは uart_apb(0)・uart(1) で、どちらも RP1 所有（固定レート・常時オン）。RATE_SET の別レートは DENIED、CONFIG_SET は Linux の投票だけ記録しレジスタを書かない | HW（hwrun2 2026-10-28、cold boot 2 回）: arm-scmi が probe、serial@34000（ttyAMA1）が probe。`clk_ignore_unused` を外した cmdline で `clk_disable_unused` が uart/uart_apb に CONFIG_SET disable を送っても UART0（clk_uart）は出力を続けた。RATE_SET の拒否は STATIC（単体テスト）のみ、Linux の pl011 は rate を変えに来ない（HW） |
| 2026-10-29 | PERSTn 後のエンドポイント再設定は PoC の transaction 契約ごと移す（`rp1_hal::cold::reconfig`）。各アクセスの前に READY・selector・PERSTn-low からの 5 ms 期限を確認し、書き込み後に失敗したら以後の PERSTn では書かない（Poisoned、冷起動でのみ回復）。PoC の「proc1 はリセット中」の確認は、proc1 が動く前提のため `EndpointService::new` の排他条件（selector/DBI/outbound window に他のコードが触れない）に置き換える。あわせて ELF 検査は pc 相対のリテラルを非書き込みセクションからだけ畳み込み、`cargo rp1 check --dtb` は最終 DTB を note と照合する。`clock.authority = "linux"` はビルドエラー | review（codexReview #1〜#4/#6/#7/#9/#10）。HW（hwrun4）: 再設定 597 µs（旧 104〜110 µs、期限 5 ms 内）で Linux の rp1 が probe、SCMI・ttyAMA1・ドアベル・shm 一式が成立。故障注入はホスト試験（STATIC） |
| 2026-10-28 | 束案は成立（HW: ≥32 MiB、約 2.9 MiB/s、配置は /chosen から読む、bootargs はトークン照合）。DTB に残りの BOOT_ORDER は無いので、失敗時の次の方法は set_reboot_order か EEPROM の BOOT_ORDER で決める（loader.md §3） | 実機検証 codexA |
| 2026-10-28 | PoC ローダーの遅さの主因は TFTP ではなくソフトウェアの gzip 展開（約 18 s）と SHA-256（約 15 s）。I2C 約 3.4 s。旧記録の「initramfs に 36 秒」は誤帰属（loader.md §0） | 実機ログ + コード |
| 2026-10-28 | hwirq58 は rp1-mailbox が専有し UIO と共有できない（HW）。mailbox・SCMI・remoteproc を disabled にすれば UIO が取れ、IRQ 往復 p50 71 µs（うち M3 の SCMI 処理約 60 µs）、通知ごとに空振り 1 回（HW、codexA2）。SCMI クロック（既定）と両立しないので v0.1 は mailbox/SCMI を残してポーリング（feat/v0.1-core の決定と同じ） | 実機検証 codexA / codexA2 |
