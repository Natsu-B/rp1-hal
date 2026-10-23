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
  rp1-boot     no_std aarch64。RP1 の I2C bootstrap、ELF と note の検査、DT patch
  rp1-build    build.rs 用。metadata → 型・memory.x・note の生成
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
| Linux DDR（outbound 窓） | 64 KiB × 2（拡張可） | M3 と Linux | — | PCIe 越し | HW（SHM） |

- ROM は proc0 DSRAM `0x1000_2000..0x1000_30d0` を起動時と watchdog 再起動時に上書きする（STATIC）。
- proc1 は ROM から entry に入る時点で MSP = `0x1000_30d0`（自分の DSRAM）（HW + STATIC）。
- 公式 FW は IRQ53 ハンドラと PCIe EP 状態機械を ISRAM に置く（STATIC、KG-a）。
- 共有 SRAM 末尾 `0x2000_ff00..` は rp1-fw ABI 予約（STATIC）。

### 3.2 配置規則

| 置き場所 | 置くもの | 書き方 |
|---|---|---|
| 共有 SRAM | 全コードと rodata のロード元、`#[shared]` static、Linux から見える領域、DMA バッファ、再起動をまたぐ warm data | `#[shared]` を付けた static（型は §5.3 のものだけ） |
| ISRAM（コアごと） | ISR、ホットループ、実行時の PCIe 状態機械 | `#[isram]` を根に自動配置（§3.3） |
| DSRAM（コアごと） | スタック、既定の `.data` / `.bss`、`#[proc0]` / `#[proc1]` static、atomic、executor 状態 | 普通の static（§5 の検査を受ける） |
| 共有 SRAM `0xff00..` | 予約 | linker ASSERT |
| Linux DDR | 大きな bulk | `rp1-proto` の Region |

既定の `.data` / `.bss` を DSRAM に置く理由: 依存 crate が static の atomic で RMW をしても壊れない。
大きなバッファは DSRAM に入らないので明示的に `#[shared]` にする。

### 3.3 ISRAM に置くものの決め方（自動）

ライブラリ全体を ISRAM に入れるのではなく、速さが要る部分だけを入れる。

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

旧 `Rp1BootInfoV1`（固定長、所有物は 64 bit のビット列）を TLV に広げる。ローダーと Linux 側ツールはこれだけを読む。
中身: ABI 版、entry / スタック、メモリ配置、コアごとの所有物（ピン・ブロック・IRQ）、clock 権限、通信路（領域・ドアベル・UIO 範囲）、
ファーム版、宣言のハッシュ。形式の正本は `rp1-proto`。

## 5. コアの区別とグローバル変数の検査

### 5.1 書き方

- `#[rp1::proc0] fn` / `#[rp1::proc1] fn`: そのコアだけで動く関数。セクション `.text.proc0.*` / `.text.proc1.*`。
- 無印の fn: どちらのコアからも呼べる。
- `#[rp1::proc0] static` / `#[rp1::proc1] static`: そのコア専用（DSRAM）。
- `#[rp1::shared] static`: 両コアで共有（共有 SRAM、§5.3 の型のみ）。
- `#[rp1::per_core] static`: 両コアから使うが、コアごとに別の値でよいもの（DSRAM、thread_local と同じ意味）。

### 5.2 検査（2 段）

1. **コンパイル時（rustc）**: コアトークン `Proc0` / `Proc1`（大きさ 0、`!Send` `!Sync`）を各 entry だけが受け取る。
   コア専用 static と `Resources` はトークンを要求するので、別コアのコードからは触れない。
2. **リンク後（cargo-rp1 の ELF 検査、エラーでビルド失敗）**: proc0 entry・proc1 entry・各コアのベクタ表から呼び出しグラフを辿り、
   - proc1 から到達できる関数が `.text.proc0` を呼んでいたらエラー（逆も）
   - 書き込み可能な static のうち、両コアから到達できて `#[shared]` / `#[per_core]` のどちらも付いていないものはエラー
   - 共有 SRAM を指す LDREX/STREX があればエラー

   関数ポインタと vtable は保守的に扱う（アドレスを取ったコアから到達可能とみなす）。依存 crate の static は metadata の許可リストで `per_core` 扱いにできる。

コアごとに区別できるので、ISRAM はコアごとの overlay（各 8 KiB）にできる（配置は §3.3）。

### 5.3 共有に使える型

`Shared<T: Copy>`（volatile load/store のみ）、`Spsc<T, N>`（1 書き手 1 読み手）、`SeqLock<T>`、`ExacMutex<T>`（EXAC によるコア間排他、HW H8）。
RMW atomic のメソッドは持たない。DSRAM を指す参照は他コアへ渡せない（相手は自分の DSRAM を読んでしまう）。
コア間 API は値渡しか、`#[shared]` 領域を指すハンドルだけを受け付ける。

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

### 6.2 未検証（最初の縦 1 本で確認する）

- Raspberry Pi OS の標準カーネルに `uio_pdrv_genirq` があり、`of_id=generic-uio` を cmdline か modprobe 設定で渡せるか。
- proc1 → Linux の割り込み（HOST_EVENTS → hwirq58）を UIO ノードで受けられるか、rp1-mailbox と衝突しないか。
- GPIO INTF のドアベルは pinctrl と同じページ。UIO で重ねてマップできるか。
- UIO の IRQ 待ちの往復遅延。

## 7. BCM2712 third stage loader（`boot/rp1boot`）

```
VPU firmware（config.txt / cmdline.txt / overlay / initramfs を通常どおり処理）
  → kernel=rp1boot.img
      1. 自分が起動した経路（/chosen/bootloader の boot-mode）で SD(FAT) か TFTP を選ぶ
      2. 同じ経路・同じ配置から rp1.elf と本来のカーネルを読む
      3. .note.rp1 を検査し、I2C で RP1 を bootrom に戻して書き込み、起動する
      4. firmware が渡した DTB を note に従って patch（周辺の無効化、UIO、reserved-memory、SCMI）
      5. 本来のカーネルへ EL2 でジャンプ（initramfs と cmdline は firmware が用意したものをそのまま渡す）
```

- SD と TFTP は「ファイルを読む部分」だけが違う。ファイル名と配置は同じ。
- 旧 PoC の実験用 feature（約 50 個）と hash 固定値は持ち込まない（`lab/`）。
- 未検証: firmware が置いた initramfs を上書きせずに引き継げるか、`/chosen` をそのまま使えるか。

## 8. 進め方（最初の縦 1 本）

1. 静的: `rp1-proto`（note v2）、`rp1-build`、`rp1-pac`、`rp1-rt`、`rp1-hal`（GPIO / UART / timer）、ELF 検査（§5.2）、`rp1boot`（SD / TFTP）。
2. 実機: proc0 と proc1 が UART に出力し、`#[shared]` の `Spsc` で値を受け渡す。Linux は patch した DTB で起動し、UIO 経由でドアベルを往復させる。
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
