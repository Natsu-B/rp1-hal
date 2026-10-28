# 通信規約 rp1-proto（設計 C、2026-10-23 草案）

M3（proc0 / proc1）と Linux のユーザープロセスが共有メモリでやり取りする形式。実装は `rp1-proto`（no_std）1 つで、
M3 側（`rp1-link`）と Linux 側（`rp1-linux`）の両方がそれを使う。ホストで単体試験できる形にする。

前提:

- v0.1 は `ring` と `block`。`channel` は向きの違う `ring` 2 本と通知で組む（正式 API は v0.2）。
- PCIe の書き込みは posted で速く、読み出しは往復が要って遅い。どちらの側も「相手のメモリには書くだけ、読むのは自分の手元」にする。
- Linux カーネルは 16 KiB ページ（`6.12.75-v8-16k`）。UIO の mmap は 16 KiB 境界・16 KiB 単位。64 KiB ページのカーネルも将来ありうるので、
  DDR 側の領域は 64 KiB 境界にそろえる。

## 1. メモリの配置

### 1.1 2 つの場所

| 場所 | 大きさ | M3 から | Linux から | 置くもの |
|---|---|---|---|---|
| **link ページ**（共有 SRAM の 1 ページ） | 16 KiB（`0x2000_c000..0x2001_0000`、末尾 `0xff00..` の予約を除く） | ローカル（速い） | BAR2 経由。書き込みは posted、読み出しは遅い | Linux → RP1 リングの本体と head、RP1 → Linux リングの tail、通知の待ちフラグ |
| **DDR 領域**（BCM2712 の DDR、reserved-memory） | 宣言の合計（64 KiB 単位） | outbound 窓（0x8000_0000..）。書き込みは posted、読み出しは遅い | ローカル（uncached） | 領域ヘッダー、RP1 → Linux リングの本体と head、Linux → RP1 リングの tail、`block` |

- 配置は宣言で選べる（`placement`、§1.3）。v0.1 が実装するのは `reader-local`（既定）だけ。
- link ページは BAR2 の 16 KiB 1 枚を UIO（名前 `rp1:link`）で見せる。リングごとに別ページにはできない（共有 SRAM は 4 ページしかない）ので、
  link ページは全リングで共用する。ファームのイメージは残り 48 KiB（`0x2000_0000..0x2000_c000`）に収める。
- DDR 領域は `shm` ごとに UIO（名前 `rp1:<name>`）を分ける（権限を分けられる）。

### 1.2 配置の選択（`placement`）

| 値 | 内容 | link ページ | v0.1 |
|---|---|---|---|
| `reader-local`（既定） | 制御語とスロットを「読む側の手元」に置く（§2.2）。どちらも相手側へは書くだけ | 使う | 実装する |
| `linux` | リングを両方向とも Linux の DDR に置く。RP1 は DDR を読む（遅い）が、共有 SRAM を使わない | 使わない | 宣言・note の値だけ予約（選ぶとビルドエラー「未実装」） |

- 宣言: `[package.metadata.rp1] link_page = "16KiB"`（既定）か `"none"`。`"none"` のときは全リングが `placement = "linux"` でなければならない。
- note の SHM_REGION に `placement` を持たせ、critical 扱いにする（知らない配置の note は読み手が拒否する）。
- 将来 `linux` を実装しても、`rp1-linux` / `rp1-link` の公開 API（`RingTx` / `RingRx`）は変えない。違いは配置と内部の手順だけ。

### 1.3 DDR 領域の先頭（領域ヘッダー、64 バイト、DDR 領域全体で 1 つ）

| オフセット | 型 | 内容 | 書く側 |
|---|---|---|---|
| 0 | `[u8; 8]` | `"RP1SHM\0\0"` | RP1 |
| 8 | `u16` | 規約の版 = 1 | RP1 |
| 10 | `u16` | ヘッダー長 = 64 | RP1 |
| 12 | `u32` | 世代番号（§4）。奇数 = 初期化中、偶数 = 使用可 | RP1 |
| 16 | `[u8; 32]` | note の SHA-256（Linux 側が自分の読んだ note と照合） | RP1 |
| 48 | `u32` | 領域数 | RP1 |
| 52 | `u32` | RP1 の起動からの経過（µs、生存確認用。任意） | RP1 |
| 56 | `u64` | 予約 0 | — |

各 `shm` の位置（DDR・link ページ内のオフセット）は note の SHM_REGION にあるので、ヘッダーには目録を置かない。

## 2. ring

1 書き手 1 読み手、固定長スロット、深さは 2 の冪。添字は 32 bit で単調増加（`index & (depth - 1)` がスロット番号）。

### 2.1 スロット

```
u32 seq     書き手が付ける通し番号（= 書き込み時の head）。読み手は期待値と比べ、ずれたら Error::Corrupt
u16 len     payload の長さ（T のサイズ以下）
u16 flags   予約 0
T   payload 8 バイト境界に 0 埋め
```

スロット長 = `8 + round_up(size_of::<T>(), 8)`。

### 2.2 制御語（それぞれ 1 語。偽共有を避けるため 64 バイトずつ離す）

| 語 | 書く側 | 置き場所 |
|---|---|---|
| `head` | 書き手（次に書くスロットの添字） | 読み手の手元 |
| `tail` | 読み手（次に読むスロットの添字） | 書き手の手元 |
| `waiting` | 読み手（眠る前に 1、起きたら 0） | 書き手の手元 |
| `dropped` | 書き手（満杯で捨てた数） | 読み手の手元（観測用） |

| 向き | スロットと `head`・`dropped` | `tail`・`waiting` |
|---|---|---|
| RP1 → Linux | DDR 領域 | link ページ |
| Linux → RP1 | link ページ（小さい。宣言の深さ × スロット長が link ページの予算に入ること） | DDR 領域 |

### 2.3 手順

満杯時の動きはリングごとに宣言で選ぶ（`overflow`）:

| 値 | 満杯のとき | 使いどころ |
|---|---|---|
| `reject`（既定） | 書き手は `Err(Full)` を返し、`dropped += 1`。データは失われない（書き手が捨てる判断をする） | コマンド、取りこぼせないデータ |
| `overwrite` | 書き手は最古のスロットを上書きして進む（FreeRTOS の上書き送信と同じ考え方）。読み手は追い越されたことを検出して `Lost(n)` を返し、最新側へ追従する | 計測値のストリームなど「最新が大事」なもの |

書き手（`push`、`reject`）:

1. `tail`（手元）を読み、`head - tail == depth` なら満杯 → `Err(Full)`（`dropped += 1`）。
2. スロット `head` に `seq`・`len`・payload を書く。
3. 書き込みの完了を確定する（M3: `dsb`。Linux: Device 属性の書き込みは発行順に出る）。
4. `head += 1` を書く。
5. `waiting`（手元）が 1 なら通知する（§3）。

読み手（`pop`）:

1. `head`（手元）を読む。`tail == head` なら空 → `None`。
2. スロット `tail` を読み、`seq == tail` を確かめる。
3. `tail += 1` を相手側へ書く（posted）。

`overwrite` のときの違い:

- 書き手は `tail` を見ない。スロットに書く前に `seq` を「書き込み中」（`u32::MAX`）にし、payload を書き、最後に `seq = head` を書く。
- 読み手はスロットの `seq` を payload のコピーの前後で読む。
  - 前後とも `tail` に一致 → 正常。
  - `seq > tail`（追い越された）または途中で変わった → 失った数を数え、`tail = head - depth + 1` に進めて読み直す。呼び出し側には `Lost(n)` を返す。
  - `u32::MAX` → 書き込み中なので読み直す。
- `tail` と `waiting` は通知の間引きのためにだけ使う。

眠る（`wait`）:

1. `waiting = 1` を相手側へ書く。
2. **もう一度 `head` を読み**、空でなければ `waiting = 0` にして戻る（通知の取りこぼし防止）。
3. 通知を待つ（Linux: UIO の `read`。M3: WFE / 割り込み）。起きたら `waiting = 0`。

### 2.4 順序の根拠と未確認点

- RP1 → Linux: M3 の posted 書き込み（スロット → head）は同じ outbound 窓を順に通る。PCIe は posted 書き込みの順序を保つ（DOC）。
  ただし通知（MSI-X など別経路）が書き込みより先に着く恐れがあるので、通知の前に同じ窓へ 1 回読み出しをして到達を確定する
  （既存の SHM 契約: payload → barrier → 同じ窓の読み出し → barrier → 通知、HW 実績あり）。
- Linux → RP1: Linux の BAR2 への書き込みは Device 属性で発行順。M3 はローカル SRAM を読む。
- 未確認（TODO）: M3 の outbound 窓が Normal 属性の領域（0x8000_0000..）にあることで、M3 側の書き込みが並べ替えられないか。

## 3. 通知

| 向き | 方法 | 実測 |
|---|---|---|
| Linux → proc1 | GPIO INTF の SET 別名（`IO_BANK0 + 0x2114`、BAR1 経由）に書く。proc1 の ISRAM 上の ISR が受ける | 往復 p50 4.1 µs（HW、ポーリング側） |
| Linux → proc0 | mailbox（SYSCFG PROC_EVENTS）。ch0 は SCMI が使うので ch1 以降 | 配送は HW。遅延は未測定 |
| RP1 → Linux | HOST_EVENTS → hwirq58 → UIO の IRQ（`rp1:link` に付ける）。hwirq58 は rp1-mailbox が専有する。v0.1 は mailbox/SCMI を残し、Linux はポーリング（design.md §6.2） | 共有は不可（HW、-EBUSY）。mailbox/SCMI を外せば UIO で受けられ、往復 p50 71 µs（M3 の SCMI 処理込み）、通知ごとに空振り起床 1 回（HW 2026-10-28） |
| どちらでも | 通知なし（`notify = "poll"`）。受け手がポーリング | — |

- 通知は「相手の `waiting` が 1 のときだけ」送る（毎回は送らない）。
- Linux 側のドアベル書き込みには BAR1 の該当 16 KiB ページを UIO で見せる必要があり、そのページには他の GPIO レジスタも入る
  （pinctrl と同じページ）。`rp1-linux` はドアベルの別名レジスタにしか書かない。重ねて mmap できるかは未確認（TODO）。
- UIO の IRQ（`uio_pdrv_genirq`）は割り込みのたびに無効化されるので、`read` の後に `write(1)` で再度有効にする。`poll` / `epoll` も使える。

## 4. 世代番号（RP1 の起動・再起動）

1. `Shm::enable` で RP1 は世代番号を奇数にし、DDR 領域と link ページの自分の担当をゼロ化し、ヘッダーを書き、偶数（前の値 + 2）にする。
2. Linux 側の `open` は、世代番号が偶数であること、note の SHA-256 が一致することを確かめてから使う。
3. Linux 側は各操作の前後で世代番号を見て、変わっていたら `Err(Reset)` を返す。利用者は開き直す。
4. Linux のプロセスが再起動しても、`tail` は共有メモリに残っているので続きから読める。最初から読み直す `reset` は v0.1 では持たない。
5. v0.1 は cold boot のみ（PCIe リセットで outbound ATU が消える）。RP1 の再ロードは Linux の再起動を伴う。

## 5. block

```
offset 0   u32 seq       SeqLock（writer = rp1 か linux のとき）。奇数 = 書き込み中
offset 4   u32 flags     予約 0
offset 8.. T             8 バイト境界
```

| writer | 同期 | 書く手順 | 読む手順 |
|---|---|---|---|
| `rp1` / `linux` | SeqLock | seq を奇数に → T を書く → seq を偶数に（すべて posted、同じ経路なので順序が保たれる） | seq を読む → T をコピー → seq を読む。奇数か変化していたらやり直し |
| `both` | 利用者に任せる | `raw()` で生の読み書き API だけを渡す | 同左 |

- RP1 が `block` を読むのは PCIe の往復になり遅い（「遅くて大きいメモリ」）。大きなデータは RP1 が書き、Linux が読む向きを推奨する。
- Linux 側は uncached（Device 属性）で mmap するので、`rp1-linux` は 8 バイト整列の volatile コピーだけを使い、生の `&[u8]` を渡さない。

## 6. 型

- payload の型は `#[derive(rp1_proto::Message)]`: `#[repr(C)]`、パディング無し、どのビット列も有効（`bool` や参照、`enum` の不正値を持たない）。
  enum は `#[repr(u32)]` の C 風だけを許し、受信時に検査して不正値は `Error::Invalid`。
- 型 ID（64 bit）は型名・サイズ・各フィールドのオフセットと型から derive が計算し、note に入る。Linux 側の `open::<T>` は型 ID が一致しなければ失敗する。
- エンディアンは両側リトルエンディアン（固定）。

## 7. エラー

| エラー | 意味 |
|---|---|
| `Full` | 書き手: リングが満杯（`overflow = "reject"`。捨てた数は `dropped`） |
| `Lost(n)` | 読み手: `overflow = "overwrite"` で n 件追い越された（エラーではなく通知。続けて読める） |
| `Corrupt` | 読み手: スロットの `seq` が期待と違う |
| `Invalid` | 読み手: payload の検査に失敗 |
| `Reset` | Linux 側: 世代番号が変わった（RP1 が再初期化された） |
| `NotReady` | Linux 側: 世代番号が奇数 / ヘッダーが無い |
| `Mismatch` | Linux 側: note の SHA-256・型 ID・名前が合わない |
| `Timeout` | `recv_timeout` など |

## 8. 決定（2026-10-23）

1. link ページ: 共有 SRAM の最後の 16 KiB を使い、ファームのイメージは 48 KiB に収める。将来リングを Linux の DDR だけに置く実装（`placement = "linux"`）を選べるよう、宣言と note に項目を予約する（§1.2）。
2. 満杯時: 既定は `reject`（`Full`）。上書きする `overwrite` も選べる（§2.3）。
3. 通知の間引き: 相手の `waiting` が 1 のときだけ通知する。
4. Linux プロセスの再起動: `tail` を共有メモリに残して続きから読む。
