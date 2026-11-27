# 通信規約 rp1-proto（設計 C、2026-10-23 草案）

M3（proc0 / proc1）と Linux のユーザープロセスが共有メモリでやり取りする形式。実装は `rp1-proto`（no_std）1 つで、
M3 側（`rp1-link`）と Linux 側（`rp1-linux`）の両方がそれを使う。ホストで単体試験できる形にする。

前提:

- STATIC: `ring` と `block`、双方向の `channel` と要求・応答の `rpc`。双方向の端点は 2 本の ring を使う。
- PCIe の書き込みは posted で速く、読み出しは往復が要って遅い。どちらの側も「相手のメモリには書くだけ、読むのは自分の手元」にする。
- Linux カーネルは 16 KiB ページ（`6.12.75-v8-16k`）。UIO の mmap は 16 KiB 境界・16 KiB 単位。64 KiB ページのカーネルも将来ありうるので、
  DDR 側の領域は 64 KiB 境界にそろえる。

## 1. メモリの配置

### 1.1 2 つの場所

| 場所 | 大きさ | M3 から | Linux から | 置くもの |
|---|---|---|---|---|
| **link ページ**（共有 SRAM の 1 ページ） | 16 KiB（`0x2000_c000..0x2001_0000`、末尾 `0xff00..` の予約を除く） | ローカル（速い） | BAR2 経由。書き込みは posted、読み出しは遅い | Linux → RP1 リングの本体と head、RP1 → Linux リングの tail、通知の待ちフラグ |
| **DDR 領域**（BCM2712 の DDR、reserved-memory） | 領域は 64 KiB 単位、予約全体は 1 MiB 単位（1..16 MiB） | outbound 窓（0x8100_0000..）。書き込みは posted、読み出しは遅い | ローカル（uncached） | 領域ヘッダー、RP1 → Linux リングの本体と head、Linux → RP1 リングの tail、`block` |

- 配置は宣言で選べる（`placement`、§1.3）。v0.1 が実装するのは `reader-local`（既定）だけ。
- link ページは BAR2 の 16 KiB 1 枚を UIO（名前 `rp1:link`）で見せる。リングごとに別ページにはできない（共有 SRAM は 4 ページしかない）ので、
  link ページは全リングで共用する。ファームのイメージは残り 48 KiB（`0x2000_0000..0x2000_c000`）に収める。
- DDR 領域は `shm` ごとに UIO（名前 `rp1:<name>`）を分ける（権限を分けられる）。

### 1.1.1 link ページの先頭（`rp1_proto::link`、`rp1_proto::status`）

| オフセット | 内容 | 書く側 |
|---|---|---|
| 0x00〜0x23 | 固定語: ハンドオフ（0x00〜0x0c）、予約（0x10 / 0x14）、`SHM_GENERATION` / `SHM_STATUS` / `SHM_REQUEST`（0x18 / 0x1c / 0x20） | ローダー / Linux / proc0 |
| 0x24〜0x3f | lab と診断用（lab のファームとスクリプトが 0x24 / 0x28 / 0x2c を使う）。ライブラリは使わない | lab |
| 0x40〜0xbf | ファームの状態語（§9） | rp1-rt |
| 0xc0〜0xff | NOTE v3 の link wake mailbox（§10）。v2 では宣言領域に割り当て可 | §10 |
| 0x100〜 | 宣言した領域の SRAM 部分（v2 は 0xc0 から、v3 は 0x100 から） | 端点 |
| 0x17c0〜0x18bf | SCMI shmem | Linux / proc0 |

0x00〜0x3f は凍結する。新しい語は足さない。

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
u16 flags   bit0 = overwrite 書き込み中、他は予約 0
T   payload 8 バイト境界に 0 埋め
```

スロット長 = `8 + round_up(size_of::<T>(), 8)`。

### 2.2 制御語（それぞれ 1 語。偽共有を避けるため 64 バイトずつ離す）

| 語 | 書く側 | 置き場所 |
|---|---|---|
| `head` | 書き手（次に書くスロットの添字） | 読み手の手元 |
| `tail` | 読み手（次に読むスロットの添字） | 書き手の手元 |
| `waiting` | 読み手（眠りカウンタ。眠る前と起きた後に 1 ずつ足す。奇数 = 眠っている） | 書き手の手元 |
| `dropped` | 書き手（満杯で捨てた数） | 読み手の手元（観測用） |
| `head_hi`（`head` の +4） | 書き手（`overwrite` のみ。`head` が `u32::MAX` から 0 へ回った回数） | 読み手の手元 |
| `tail_hi`（`tail` の +4） | 読み手（`overwrite` のみ。`tail` が回った回数） | 書き手の手元 |

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

1. `tail`（手元）を読み、`head - tail == depth` なら満杯 → `Err(Full)`（`dropped += 1` は手元だけで数え、次に成功した push で相手側へ書く。満杯で回し続ける書き手が相手のメモリへ書き込みを浴びせないため）。
   実装は前回読んだ `tail` を手元に持ち、それで空きが足りないときだけ読み直す（`tail` は増えるだけなので、古い値は空きを少なく見積もるだけ）。
2. スロット `head` に `seq`・payload を書く。`reject` の `len` は書き手の `attach` が全スロットに一度だけ書く（値は変わらないので、未読スロットに同じ値を書き直しても読み手に影響しない）。M3 → DDR の posted 書き込みは 1 語 約 0.54 µs（HW, 2026-10-29）なので 1 件あたり 1 語減らす。
3. 書き込みの完了を確定する（M3: `dsb`。Linux: Device 属性の書き込みは発行順に出る）。
4. `head += 1` を書く。
5. 通知を使うときだけ（`notify = "irq"`）: `head` を同じ窓から読み戻して到達を確定し（`RingTx::wake_needed`）、その後 `waiting`（手元）が奇数で、まだ通知していない値なら通知し、その値を覚える（§3.1）。1 回の眠りに通知は 1 回まで。追いついている読み手（偶数）には通知しない。バッチ（`push_slice`）なら 1 回に 1 度呼べば足りる。Linux → RP1 では記憶を持たず、奇数なら毎回鳴らす（`RingTx::reader_asleep`、§3.2）。

読み手（`pop`）:

1. `head`（手元）を読む。`tail == head` なら空 → `None`。実装は前回読んだ `head` を持ち、それで空に見えるときだけ読み直す（読み直した後に barrier）。
2. barrier 後にスロット `tail` を読み、`seq == tail` と長さを確かめ、payload を普通のメモリへコピーする。
3. barrier 後に `tail += 1` を相手側へ書く（posted）。

まとめて送る・受け取る（`push_slice` / `pop_into`、`reject`）: 手順 2 を複数スロットについて行ってから、barrier と `head`（または `tail`）の公開を 1 回だけ行う。
相手からは一度に n 件が見える。`push_slice` は書けた件数を返し、足りない分は `dropped` に数えない（呼び手がまだ持っている）。
`pop_into` は途中のスロットの検査に失敗したら、それまでの件数を返し、次の呼び出しで誤りを返す。`overwrite` のリングでは `pop_into` は 1 件ずつ（`Lost(n)` を失わないため）。

`overwrite` のときの違い:

- STATIC: 書き手は `tail` を見ず、flags bit0 を立てて barrier → payload と `seq=head` → barrier → flags を 0 → barrier → head を公開する。
  `u32::MAX` も有効な連番なので、書き込み中の印には使わない。
- 読み手は `head-tail > depth` なら `Lost(head-tail-depth)` とともに `tail=head-depth` へ進む。
  `head`・`tail` は上位語（`head_hi`・`tail_hi`）と合わせた 64 bit で比べる。書き手は止まった読み手に制限されないので、
  32 bit だけでは読み手の停止中に `2^32` 件進むと `head == tail` になり、残っているスロットも `Lost` も見えなくなる（レビュー 2 #4）。
  書き手は `head` が 0 に回ったとき、`head` の後に barrier を挟んで `head_hi` を書く。読み手は `head_hi` → barrier → `head` の順に読むので、
  新しい `head_hi` を見たなら `head` も新しい。ずれ得るのは「`head` だけ新しい」場合で、そのとき差は負になり、空（古い `head` と同じ扱い）とみなす。
  `Lost` の件数が `u32::MAX` を超える場合は分けて返す（合計は正確）。`reject` は書き手が `tail` で止まるので上位語を使わない（常に 0）。
  スロットの flags と seq を barrier を挟んでコピーの前後に読み、flags=0・seq=tail が両方一致した場合だけ返す。
  競合が続いたら `NotReady` を返し、呼び手が再試行する。
- `tail` と `waiting` は通知の間引きのためにだけ使う。

眠る（`wait`、`notify = "irq"` の RP1 → Linux のリング）:

1. 空でなければ眠らない。
2. `waiting` を次の奇数にして相手側へ書き、**同じ窓から読み戻して**到達を確定する（posted 書き込みは `dsb` だけでは相手のメモリに届いた保証がない）。
3. **もう一度 `head` を読み**、空でなければ `waiting` を偶数に戻して戻る（通知の取りこぼし防止）。
   書き手側の 5. と対になり、両方が「自分の書き込みの到達を確定してから相手の値を読む」ので、両者が古い値を見て眠り続ける組み合わせは起きない（STATIC/DOC: PCIe の非 posted 読み出しは同じ経路の先行 posted 書き込みを追い越さない）。
   - 書き手が `waiting` を古い（偶数の）値で読んだなら、その読み出しは書き手の `head` の到達より後なので、読み手の 3. の再読は新しい `head` を見る。
   - 読み手の 3. が古い `head` を見たなら、読み手の 2. の到達はその前なので、書き手の 5. は奇数を見て通知する。
4. 通知を上限付きで待つ（Linux: `rp1:irq` UIO の `poll`、1 回 10 ms まで）。戻ったら `waiting` を偶数にし、1. へ（起床は合図。§3）。
   書き手が 1 件ずつ速く書く場合は、arm の前に短くポーリングする（`RingRx::set_spin`）と、取り消される arm とその通知が減る。

- 値を数で持つ理由: 0/1 の旗では、書き手は「通知済みの眠り」と「新しい眠り」を区別できず、眠っている間ずっと push のたびに通知するか、
  通知を 1 回に絞ると、読み手が起きて（0）すぐまた眠った（1）ときに書き手が 0 を見逃して次の眠りを通知しない。数なら続く眠りの値は違う。
- 一巡: 書き手が覚えるのは最後に通知した値だけ。通知のないまま 2^31 回眠ると、その値が再び現れ、その眠りは通知されない
  （10 ms ごとの再確認だけなら約 248 日）。データは読み手の上限付き再確認で拾う（遅れは最大 10 ms）。通知が合図にすぎない理由の一つ。
- 再起動: 読み手の attach は `waiting` を読み、奇数（前の読み手が眠ったまま終わった）なら偶数へ進める。書き手の attach は「通知済み」を
  持たない状態から始めるので、眠っている読み手には次の呼び出しで通知する。
- 再確認で取り消した眠り（3. で空でなかった）の奇数値を書き手がその前に読んでいれば、通知は出る。その起床は後で空振りになる（許容）。
- 代償: 通知を使う push（またはバッチ）ごとに PCIe の読み出し往復 1 回（`head` の読み戻し）。この読み戻しで posted 書き込みも完了する。
- HW（2026-11-05、cold boot 3 回）: 10^6 件で欠落・破損・見直しでの拾い 0。数値は design.md §6.2。

### 2.4 順序の根拠と未確認点

- RP1 → Linux: M3 の posted 書き込み（スロット → head）は同じ outbound 窓を順に通る。PCIe は posted 書き込みの順序を保つ（DOC）。
  ただし通知（MSI-X など別経路）が書き込みより先に着く恐れがあるので、通知の前に同じ窓へ 1 回読み出しをして到達を確定する
  （既存の SHM 契約: payload → barrier → 同じ窓の読み出し → barrier → 通知、HW 実績あり）。
- Linux → RP1: Linux の BAR2 への書き込みは Device 属性で発行順。M3 はローカル SRAM を読む。
- RP1 → Linux を DMA で書く場合（`RingTx::push_slice_staged`、`rp1_link::ring_dma::RingDma`、不安定 feature `rp1-link/unstable-ring-dma`、dma.md「DMA ring producer」）:
  スロットの像（seq・長さ・payload・0 埋め）を共有 SRAM に並べ、DMA が 1 ブロック（折り返すなら 2 ブロック）で読み手のスロットへ写す。
  `SlotCopy` の成功は全書き込みが後続 CPU 窓ストアと共通の順序点へ受理済みで、後続ストアが追い越さない契約。
  各ブロック成功後に barrier → `head` を書く。Channel は bit30（NonPosted_LastWrite_En）と AW_CACHE=0 を明示する（STATIC）。
  最終 AXI 応答から共通 PCIe 順序点までの接続は HYP（dma.md）。値比較 guard は全語の順序を保証しないため廃止。
  copy 失敗なら `head` は不変、その RingTx は fenced となり全 push を拒否する。エンジン停止は受理済み遠隔書き込みの drain ではないため、
  外部 reset/drain 前に再 attach してはならない。wire 形式は CPU 版と同じで、成功時は CPU 版と混在できる（STATIC）。
  旧実験は prefix 後の stale を見逃せたため順序の証拠にしない。新実験は最新スロットを先に読み、`pop_into_report` で prefix 後の
  異常も報告し、同じスロットを重複計数しない。early-head 陽性対照と bit30=0 比較を伴う（dma.md）。
- 未確認（TODO）: M3 の outbound 窓が Normal 属性の領域（0x8000_0000..）にあることで、M3 側の書き込みが並べ替えられないか。

### 名前付き channel と RPC（STATIC）

`<name>-dn` は Linux → RP1、`<name>-up` は RP1 → Linux。どちらも `reader-local` と
`reject` を使う。Linux は up、dn の順に UIO をロックする。channel は両端点を `split` できる。

IRQ の名前付き channel では、Linux が応答の `tail` を公開してから既存のドアベルを鳴らす。
成功した読み取り 1 回（バッチなら 1 バッチ）につき BAR1 への posted 書き込み 1 回で、空読みと RPC では鳴らさない。
再 open 時にも 1 回通知し、`tail` の公開直後に前の読み手が終了した場合を扱う。
`Channel::respond` は返答の空きを処理予算にして、満杯なら要求を消費せず `Idle` を返す。
両リングが IRQ の場合は、この通知で処理を再開できる。旧クライアントにはこの通知がないため、
`examples/dual-core` は既存の 1 秒の心拍でもイベントを送る。応答待ち専用のアラームは使わない。

RPC は同じ深さの 2 本を使う。ファームの `Server::serve` は要求順に 1 件ずつ応答する。
応答の publish に失敗したら drain を止め、以後 `Reset` を返す。Linux の `Client::call` は
送信時の `dn.head` と受信時の `up.tail` を比較する。未回収数 `dn.head.wrapping_sub(up.tail)`
を深さ以下に抑えるので、処理中の要求にも応答の空きがある。再 open では共有の添字を読む。

正の timeout は送信と受信に共通の締切。ゼロなら空きがあるとき 1 件送って `Timeout` を返す。
後続の call は期限切れの応答を回収して捨てる。未回収数が深さに達したら `Busy`。期限切れの要求も
処理されることがあるため、自動で再送しない。停止・panic・fault を観測すると `PeerDown` を返す。

## 3. 通知

| 向き | 方法 | 実測 |
|---|---|---|
| Linux → RP1 | BAR1 の `PROCn_INTF` force → 読み手の `IO_BANK0`（`Doorbell<PIN, CORE>`、§3.2） | HW（2026-11-26、cold boot 1 回） |
| Linux → proc0 | mailbox（SYSCFG PROC_EVENTS）。ch0 は SCMI が使うので ch1 以降。端点の通知には使わない | 配送は HW。遅延は未測定 |
| RP1 → Linux | `host_irq = true` のコア（1 つだけ）が MSI-X vector 29（TIMER アラーム 3）の TEST をパルス → `rp1:irq` UIO の IRQ（`rp1_link::notify::HostIrq`、design.md §6.2）。hwirq58 は rp1-mailbox/SCMI が専有したまま | design.md §6.2（往復 p50 約 8.6 µs、HW 2026-11-05） |
| どちらでも | 通知なし（`notify = "poll"`）。受け手がポーリング | — |

宣言は ring ごとに `notify = "irq"`。rp1-to-linux なら宣言した core が `host_irq = true` を、linux-to-rp1 なら `doorbell` を持つこと
（rp1-build が検査）。RP1 の端点は通知手段を大きさ 0 の型引数（`Poll` / `HostIrq` / `Doorbell<PIN, CORE>`）で持つ。
Linux の端点は note に従い `rp1:irq` または `rp1:doorbell` を共有で開く。

### 3.1 RP1 → Linux

- RP1 の `rp1_link::RingTx::send` / `send_slice` / `flush`（と `send_*_dma`、dma.md）は公開の後に §2.3 の書き手 5. を行い、
  未通知の眠りがあればパルスする。`push` / `push_slice` / `push_with` は鳴らさない（まとめた後に `flush`）。
- Linux の `rp1_linux::RingRx::recv(timeout)` / `recv_into(out, timeout)` / `wait(timeout)` は §2.3 の「眠る」を行う。
  1 つの `RingRx` の待ちは自分の ring だけを arm する。複数を 1 スレッドで待つときは
  `Firmware::wait_any` が全 ring を arm して再確認し、空なら 1 回眠る。
  `rp1:irq` は共有で開くので、別々のプロセスが別々のリングを同時に待てる（HW 2026-11-26、`events0` と `tick0` の 2 プロセス）。
- 割り込みは合図であってメッセージではない。
  - 公開してから通知する。端点の `send*` / `flush` は `head` を同じ窓から読み戻してからパルスする。端点を通らないデータは
    `HostIrq::pulse_after_sram_write`（barrier → パルス）で知らせる。barrier だけで足りるのは RP1 の SRAM（link ページ、共有 SRAM
    上の領域）にあるデータだけで、Linux DDR にあるなら先に `rp1_link::flush_window`（同じ窓の読み戻し）で posted 書き込みを
    完了させる。barrier は PCIe の posted 書き込みの到着を待たないので、DDR のデータより先に Linux が起きて古い値を読みうる（STATIC）。
  - 失う・まとまる: Linux が vector を有効にする前（最初の `write(1)` の前）のパルスは消える。起床から次の `write(1)` までの
    パルスは 1 回にまとまる（lazy disable）。vector 29 の本来の割り込み源（TIMER アラーム 3、`host_irq` が所有し誰も
    立てない）が立っている間のパルスはエッジにならず届かない（design.md §6.2）。
  - よって受け手は、起床のたびと、起床が無くても上限付きの待ち時間（10 ms）ごとに、待っているデータを見直す。
    1 回のエッジを無期限に待たない。起床してもデータが無いこと（空振り）も許す。この向きは通知が失われても遅れるだけで止まらない。
- UIO の IRQ（`uio_pdrv_genirq`）は割り込みのたびに無効化されるので、`read` の後に `write(1)` で再度有効にする（rp1-linux が行う）。
- Raspberry Pi OS（2026-04-21 版）の bcm2712 系 DTB 8 個と overlay 363 個を調べた範囲では、
  RP1 の MSI-X vector 26〜29 に利用者は無い（STATIC、2026-11-26）。overlay 6 個にあった 26・27 は
  `interrupt-parent = <&gpio>` の GPIO 番号。

### 3.2 Linux → RP1

Linux は note の LINK からピンとコアを取り、`rp1:doorbell`（BAR1 の IO_BANK0、16 KiB）の
`PROCn_INTF` の SET 別名（`0x2108 + 0xc·n`）へ `1 << pin` を書く。端点ごとの UIO は
`flock` で排他にする（2 つ目は `Err(Busy)`）。

```
Linux writer (send / send_slice; at open from W3)    RP1 reader (IO_BANK0 handler)
  W1 store slots         (BAR2)                        R0 store INTF CLR                  doorbell.ack(), once
  W2 barrier; store head (BAR2)                        R1 store waiting = even            (outbound window)
  W3 barrier; load head  (BAR2); barrier               R2 pop: load head (SRAM); barrier; load slot;
  W4 load waiting        (DDR, local)                         barrier; store tail (window)   ... up to max
  W5 if odd: store INTF SET (BAR1)                     R3 store waiting = next odd        (outbound window)
                                                       R4 barrier; load waiting (window); barrier
                                                       R5 load head (SRAM); if not empty disarm and goto R2
```

- 書き手（`rp1_linux::RingTx`）: `send` は push の後に、`send_slice` は 1 件以上書けたときに W3〜W5 を 1 回行う
  （`rp1_proto::shm::RingTx::reader_asleep`）。RP1 → Linux と違い、通知済みの記憶を持たず、奇数なら毎回鳴らす。force はレベルなので、
  立っている間の再通知は同じ force にまとまる。通知済みの値を覚えないので、カウンタが一巡しても通知できる。
  費用は 1 件（またはバッチ）ごとに BAR2 の読み戻し 1 回と、読み手が眠っていれば BAR1 への posted 書き込み 1 回。
- `Firmware::ring_tx` は IRQ 端点を開くと W3〜W5 を 1 回行う。前の書き手が公開と通知の間で終了しても、
  次の書き手が送信せずに読み手を起こせる。
- RP1 の `link.open(&mut irq.IO_BANK0)` は Ready まで WFE で待ち、生成ハンドラでドアベルを ack して
  スレッドを起こす。全端点が `Idle` のときに WFE へ戻る。`try_open` は R3〜R5 を行う。
  空なら arm したまま返し、データがあれば arm を解除してドアベルを立てる。
  端点をハンドラで処理する場合は、次のように明示的に制御する。

  ```rust
  // IO_BANK0 handler body; `doorbell`, `down_a` and `down_b` come from the handler's statics.
  doorbell.ack(); // R0: once, before every ring that shares this doorbell
  let more = [down_a.drain(16, handle_a), down_b.drain(16, handle_b)];
  if more.contains(&Ok(Progress::More)) {
      doorbell.raise(); // a budget ran out and that ring is not armed: run the handler again
  }
  ```

  `ack` を ring ごとに行うと、先に arm した ring への通知を次の `ack` が消しうる。
  `drain` は最大 `max` 件（0 は 1 件）を処理し、空で arm できれば `Idle`、予算を使い切れば arm せず `More` を返す。
  `More` なら書き手の通知を待たずに `raise()` するか、もう一度 drain する。
- 端点をハンドラへ渡してから `doorbell.raise()` を 1 回呼ぶ（またはその後で `IO_BANK0` の線を有効にする）。
  渡す前にハンドラが入ると、端点を処理せず `ack` する。開いた時点でデータがあった ring は arm していないため、そのまま止まる。
- RP1 の端点は窓へ触る前に手元の `SHM_STATUS` を読み、Ready でなければ `Err(Reset)` を返す（STATIC）。
  proc0 では判定の後に PERSTn が入る余地がある（`flush_window` と同じ）。

前提。**以下の議論はこの 4 つが成り立つ場合に限る。**

| # | 前提 | 分かっていること |
|---|---|---|
| P1 | 窓を通る読み出しは、同じ側が同じ窓へ先に出した書き込みが相手のメモリに届いてから戻る。(a) RP1 の outbound 窓 → DDR、(b) Linux の BAR2 → SRAM | PCIe のリンク上の順序規則は DOC。RP1 内部（AXI ↔ PCIe ブリッジ）と RC 側の経路は HYP。HW（2026-11-26）: (b) と (a) の両方を使う `DOWNIRQ` 10^6 件で停止 0。読み戻しを外した対照（lab の `controls`、記録のみ）も停止 0 で、反例が出ていないだけ |
| P2 | Linux の UIO mapping は Device 属性で、store は書いた順に 1 つずつ出る | STATIC: `uio_mmap_physical` が `pgprot_noncached`、arm64 では `MT_DEVICE_nGnRnE`（6.12 のソース）。Raspberry Pi のカーネルで同じかは未確認 |
| P3 | M3 は barrier をはさんだ store → load を書いた順に実行し、outbound 窓（M3 から見て Normal 属性の領域）でも入れ替わらない | HYP（§2.4 の未確認点。RP1 → Linux も同じ前提） |
| P4 | `PROCn_INTF` の force は CLR 別名に書くまで残る（レベル） | HW（GPIO27 / GPIO26 のドアベル、2026-11-26 の `DOWNIRQ` / `DOWN0IRQ` と自己試験 `IRQ_P0_DBELL` / `IRQ_P1_DBELL`） |

ホストの模型（`rp1-proto` の `shm::tests::notify`、STATIC）は P1 を仮定して、書き手 2 つ・ドアベル 1 本・読み手 1 つの全スケジュールを
プリエンプション数の上限つきで調べる。`ack` を ring ごとにする・`More` を捨てる・開いた直後の通知を外す・どちらかの読み戻し
（W3 または R4）を外す、のどれでも停止するスケジュールが見つかり、手順どおりなら見つからない。模型が示すのは「P1 の下で
手順に抜けが無い」ことだけ。

W3 と R4 の読み戻しにより、W4 が arm を見逃したときは R5 が公開済みの `head` を見る。
逆に R5 が新しい `head` を見逃したときは W4 が arm を見て鳴らす。R0 で消した通知の `head` はすでに
SRAM にあり、その後の drain が読む。R0 より後の force は次の `ack` まで残る。

RP1 の読み手に時間での再確認は無い。ドアベルを取りこぼすと ring が止まりうるため、lab はメッセージが
残ったまま `tail` が 1 s 動かなければ `<NAME>_STALL` で不合格にする。

| 確認項目 | 強さ |
|---|---|
| `down1` 10^6 件、予算 1・バースト 128 での `More`、proc0 / GPIO26 の `down0` 10^4 件、書き手 50 回の強制終了後の送受信 | HW（2026-11-26、cold boot 1 回、欠落・破損・停止 0） |
| 1 本のドアベルで 2 ring を drain、送信せずに開く場合の通知 | STATIC（ホスト試験と模型）。`DOWNIRQ_REOPEN` は実機未確認 |

## 4. 世代番号（RP1 の起動・再起動）

1. 生成された `rp1::link::try_enable` で RP1 は世代番号を奇数にし、DDR 領域と link ページの割当範囲をゼロ化し、ヘッダーを書き、偶数（前の値 + 2）にする（STATIC）。
2. Linux の `Firmware::open` はローダーの引き継ぎを note と照合する。`wait_ready` が SHM を要求し、
   IRQ リングがあればそのドアベルでも要求を通知する。proc0 の `link::enable` はそのイベントで起きる。
   旧クライアント向けに起動中だけ 10 ms のアラームを併用する。Linux はヘッダーの偶数の世代番号と
   note の SHA-256 を検査してから端点を開けるようにする。
3. Linux 側は読み取りの前後、書き込みの前に世代番号を見て、変わっていたら `Err(Reset)` を返す。書き込み済みのデータを取り消すことはできない。cold boot 以外の同時リセットからの回復は保証しない。
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
- Linux 側は uncached（Device 属性）で mmap するので、`rp1-linux` は整列した volatile u32/u64 アクセスだけを使い、生の `&[u8]` を渡さない。

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
| `Reset` | Linux 側: 世代番号が変わった（RP1 が再初期化された）。RP1 側（rp1-link の端点）: shm の窓が失われた（PERSTn、§3.2） |
| `NotReady` | Linux 側: 世代番号が奇数 / ヘッダーが無い |
| `Mismatch` | Linux 側: note の SHA-256・型 ID・名前が合わない |

rp1-linux は `rp1_linux::Error` を返す: 上の誤りは `Proto(..)`（`Lost(n)` だけは `Lost(n)`）、ほかに `Io`、`Timeout`（`recv(timeout)`・
`wait_ready` などの期限切れ）、`Busy`（その端点を別の所有者が開いている）、`Unknown`（note にその端点が無い）、
`PeerDown { core, state, reason }`（端点を受け持つコアが停止・panic・fault。眠るたびに状態語を見る）、`LinkDown`（BAR2 が
`0xffff_ffff` を読む）、`Abi`（状態語の版が違う）。

## 8. 決定（2026-10-23）

1. link ページ: 共有 SRAM の最後の 16 KiB を使い、ファームのイメージは 48 KiB に収める。将来リングを Linux の DDR だけに置く実装（`placement = "linux"`）を選べるよう、宣言と note に項目を予約する（§1.2）。
2. 満杯時: 既定は `reject`（`Full`）。上書きする `overwrite` も選べる（§2.3）。
3. 通知の間引き: 相手の `waiting` が奇数で、まだ通知していない値のときだけ通知する（2026-11-05、旗から眠りカウンタへ変更）。
4. Linux プロセスの再起動: `tail` を共有メモリに残して続きから読む。

## 9. ファームの状態語（link ページ 0x40〜0xbf）

すべて `u32` LE。書き手は 1 つ: `EPOCH` / `ABI` は proc0、コアのブロックはそのコア。RMW は無い。

| オフセット | 語 | 内容 |
|---|---|---|
| 0x40 | `EPOCH` | proc0 が起動時に 奇数 → 両ブロックのゼロ化 → `ABI` → 偶数 の順に書く。`ABI` が一致していれば前の値（奇数なら 1 つ下の偶数）+ 2、でなければ 2 |
| 0x44 | `ABI` | `"RL"` + 版 `u16 = 3`（版 3 で `PLAT_PCIE` / `PLAT_SCMI` を追加） |
| 0x48 | `PLAT_PCIE` | STATIC: proc0 の `platform::init` が最初に 1（初期化中）を書き、IRQ53 の有効化前に 2（Up）か 4（DetectorOff）を書く。以後は IRQ53 だけが書く（3 = Lost、4 = DetectorOff）。bits 0〜7: 0=未開始、1=初期化中、2=Up、3=Lost、4=DetectorOff。bits 8〜15: DetectorOff の理由（1=武装時に `PCIE_INTF` ≠ 0、2=空の入場 16 回）。bits 16〜31: Lost を検出したときの `PCIE_MONITOR2[31:16]` |
| 0x4c | `PLAT_SCMI` | STATIC: proc0 の init が IRQ57 の有効化前に Serving を公開し、以後 IRQ57 が更新する。bits 0〜7: 0=未開始、1=Serving、2=Quarantined。bits 8〜15: 予約（0）。bits 16〜31: Serving 中の要求数（飽和）、それ以外は 0 |
| 0x50〜0x7f / 0x80〜0xaf | proc0 / proc1 のブロック | 下表 |
| 0xb0〜0xbf | 予約 0 | |

| ブロック内 | 語 | 内容 |
|---|---|---|
| 0x00 | `STATE` | 0 未起動、2 実行中（各コアの `main` の前に rp1-rt が書く）、3 停止（`status::stop`）、4 panic、5 fault。他は予約 |
| 0x04 | `REASON` | 停止のコード / fault の例外番号 |
| 0x08 | `LINE` | panic の行（feature `panic-location` のときだけ。既定は 0）/ fault の積まれた PC |
| 0x0c | `AUX` | fault の CFSR |
| 0x10 | `BEAT` | 最後の `status::beat()` の TIMER µs（下位 32 bit）。変わったかだけに意味がある |
| 0x18〜0x2b | `FILE` | panic のファイル名の末尾 20 バイト（NUL 埋め、`panic-location` のときだけ） |

- 終端の状態（3 / 4 / 5）は、詳細 → barrier → `STATE` の順に、critical section の中で「今 2 か」を見てから書く。上書きしない。
- `beat()` は TIMER の読み出しと store を 1 つの critical section で行う（ハンドラの心拍を古い値で上書きしない）。
- `start` より前（起動コードが `STATE` を実行中にする前）の panic は記録しない。
- panic の位置は既定では記録しない。ハンドラが `PanicInfo` を読むと、全 panic 箇所の `fmt::Arguments` と、
  unwrap したエラーの `Debug` 実装がイメージに残り、2.5〜4.7 KiB 増える（STATIC、2026-11-17、ワークスペースの全ファーム構成の
  `cargo rp1 check` の image_bytes の前後差）。lab の大きい構成は 48 KiB に入らなくなる。余裕のあるファームだけ
  rp1-rt の feature `panic-location` で有効にする（`examples/dual-core`）。
- Linux（`rp1_linux::Monitor`、`Firmware::status` / `health`）:
  - `status()`: `ABI` が一致し `EPOCH` が偶数で前後同じときだけ返す（それ以外は `NotReady`。一度も起動していない場合も同じ）。
    `"RL"` で版が違う `ABI`（版 2 のファームなど）は、両方の版を書いたエラー。`PLAT_PCIE` / `PLAT_SCMI` は rp1-proto の
    `PcieState::from_word` / `ScmiState::from_word`（rp1-hal と共用）で解釈し、`linux-status` は `pcie=Up scmi=Serving(requests=N)` の形で出す。
    `STATE` が終端のときに限り、barrier の後に読んだ詳細は確定した値。実行中の詳細は返さない。2 つのコア・`BEAT`・`SHM_*` が同じ時点の値である保証は無い。
    BAR2 が `0xffff_ffff` を返したらリンク断のエラー。
  - `health(window)`: 終端・未知の状態のコアは `Down`。宣言 `heartbeat` のあるコアの `BEAT` を
    `max(window, 2 × 宣言した周期の最大)` あけて 2 回読み、変わらなければ `Stalled`。宣言どおりに打つコアを `Stalled` と言わない。
  - `Monitor::open()` は `rp1:link` を読み取り専用・ロックなしで開く（HW、2026-11-18: 標準カーネルの実 OS と BusyBox 環境で UIO の読み取り専用 mmap を確認）。


## 10. 実験的 link wake（NOTE v3）

**STATIC · 2026-11-27.** schema 2 で `link_wake = "experimental"` を選ぶと ELF type
`0x52503103`、NOTE descriptor version 3、DT `rp1,abi = <3>` になる。critical TLV
`LINK_WAKE`（`0x24`、16 B）の byte layout は [宣言と NOTE §2.4](declaration-and-note.md#24-実験的-link-wake-note-v3)。
MSI-X vector 29 の owner は 1 コア。v3 の `rp1-to-linux` ring で `notify = "irq"`
を宣言できるのは owner のコアだけ。別コアからこの vector を使うには owner への通知が必要になる。

### 10.1 Link SRAM mailbox

各語は aligned LE `u32` で、初期化後は単一 writer。proc0 の `rp1-rt::status` 初期化は
`0xc0..0xff` をゼロ化する。proc0 は提供 caps を書いてから `BOOT_EPOCH` を公開する。
`BOOT_EPOCH` は status の `EPOCH` のコピーで、mailbox と提供 caps の公開を示す。
B2 は request/result を照合し、起動時は従来の 1 ms / 10 ms 再確認を使う。
startup doorbell と Ready のイベント配送は後続の実装に残る。

| Offset | Field | Writer | Commit |
|---|---|---|---|
| `0xc0` | `BOOT_EPOCH` | proc0 | `BOOT_EPOCH` |
| `0xc4` | `OFFERED_CAPS` | proc0 | `BOOT_EPOCH` |
| `0xc8` | `REQUEST_EPOCH` | Linux | `SHM_REQUEST`（`0x20`、opcode 3） |
| `0xcc` | `REQUEST_SEQ` | Linux | `SHM_REQUEST` |
| `0xd0` | `REQUIRED_CAPS` | Linux | `SHM_REQUEST` |
| `0xd4` | `ACCEPTED_CAPS` | proc0 | `SHM_STATUS`（`0x1c`） |
| `0xd8` | `RESULT_SEQ` | proc0 | `SHM_STATUS` |
| `0xdc` | `WAIT` (arm) | Linux | 同じ語を readback |
| `0xe0` | `SEQ` | vector owner | 同じ語を readback、次いで pulse |
| `0xe4` | `ACK` | Linux | 同じ語を readback |
| `0xe8..0xff` | 予約 0 | proc0 (初期化) | — |

`RESULT_SEQ` は応答した `REQUEST_SEQ` と同じ値を使う。Linux は `SHM_STATUS` の acquire 後、
同じ boot epoch/request の結果を使う。

### 10.2 Ring と ticket

128 B の制御部と slot の位置は §2 と同じ。各語は aligned LE `u32`。表は各部の先頭からの位置。

| 用途 | arm（待機側） | seq（通知側） | ACK（待機側） |
|---|---|---|---|
| RX | `ctrl+64`（読み手） | `data+68`（書き手） | `ctrl+68`（読み手） |
| credit（reject） | `data+72`（書き手） | `ctrl+72`（読み手） | `data+76`（書き手） |

1. 待機側は次の奇数を arm し、同じアドレスの readback を済ませてから条件を再確認する。
2. 通知側は data/tail/terminal の commit を同じ窓の readback で完了し、arm を読む。新しい
   奇数なら `seq = arm` を commit し、同じアドレスを readback してから pulse する。
3. 待機側は現在の arm と一致する seq を観測したときだけ ACK できる。同じアドレスを
   readback する。取消しは `arm + 1` を commit/readback する。現在の arm 以外の seq/ACK
   は効力を持たない。待機側は shared vector の起床ごとに全条件を再確認する。
   Linux の RPC は ready を確認して即座に解除するとき、同じ BAR2 制御窓に ACK → arm
   の順で公開し、最後の arm の readback で両方を完了できる。
4. arm は `0xffff_fffd` から `1` へ回り、現在の seq と直近の ACK に等しい候補を飛ばす。
   同じ世代の reopen は保持した arm から進む。通知側は arm の取得から seq の commit までに
   その待機側が `2^31 - 3` 回の新規 arm を行う前に完了する。通知側が停止している間は seq が
   固定され、ACK は保持値かその seq のみを採れるため、最大 2 候補を飛ばしてもこの範囲で
   arm は再利用されない。
5. 同じ世代で通知側を引き継ぐときは、旧 writer を停止・排出する。条件がまだ ready で、
   現在の奇数 arm に対し `seq == arm && ack != arm` なら、引継ぎ時に一度 recovery kick する。

### 10.3 初期化と失効

DDR header v2 は §1.3 の 64 B 形式で、offset 8 の版は 2、offset 56 / 60 は boot epoch /
accepted caps（いずれも LE `u32`）。ring ticket は `(boot_epoch, generation)`、
mailbox ticket は boot epoch に属する。proc0 は旧 writer を停止し、posted writes と DMA を排出してから
領域を再利用する。新しい boot epoch の世代は 2 から始まり、同じ epoch では未使用の偶数へ進む。
世代の再利用が必要になればその boot の DDR 初期化を拒否する。

proc0 は新世代の直前の奇数を書いて readback する。その間に全 ring の RX/credit の
arm/seq/ACK をゼロ化して各語を readback し、header を書き、最後に偶数世代を commit/readback
する。Linux は Ready の status、NOTE digest、boot epoch、偶数世代、accepted caps を照合する。
奇数世代、別の世代、別の boot epoch は既存の ring handle を失効させる。

### 10.4 Linux のデータ待ち

**STATIC · 2026-11-28.** v3 の Linux session は `rp1:irq` を排他 `flock` と
`O_NONBLOCK` で開き、初回に 4 byte の `write(1)` を行う。WaitSet は
対象の RX ticket と link ticket を arm/readback し、世代・終端・全 RX の
head を再確認してから元の絶対期限まで `ppoll` する。`RPC_REPLY_WAKE` 未合意時の
IRQ RPC もこの ticket 経路を使う。
4 byte の count を受けたら
`write(1)` を完了してから再走査する。空振りでは同じ ticket と期限を使う。
取消し時はその WaitSet の ticket を解除する。

期限時も最初に未読 count を取り込む。可視の終端、世代の無効化、確認した PCIe
断は `PeerDown`。count がなく ready を発見した場合は `late` を加算する。
明示期限の readiness API は `NotifyLost` を返し、データは取得可能なまま残す。
相対期限の readiness/receive と RPC は ready またはデータを返す。count が期限後に
届く場合もある。進捗が見えない場合は `Timeout`。owner が終端を書かずに停止した
場合も含む。相対待機の `None` はタイマーなしで待つ。有限時間での停止検知には
`Monitor` の heartbeat または独立 supervisor を使う。
既知の再起動は host の旧 session と writer を明示的に破棄する。
boot epoch が数値として再出現する場合、この識別には運用側での失効と排出が必要になる。
owner の panic/fault から期限より早く起床するには独立した supervisor が必要。

### 10.5 RPC request 内の返信通知

**STATIC · 2026-11-28.** `RPC_REPLY_WAKE`（bit 3）を合意した IRQ RPC では、
返信通知を要求する slot の seq を `index ^ 0x80000000` として書く。RP1 は
期待する 32-bit index とこの符号化の両方を照合する。通知なしの要求は通常の seq を使う。

Linux は要求を公開する前に listener を初回 enable する。同じ BAR2 SRAM 窓の
世代を readback して要求 head の書き込みを完了し、世代を照合してから force を書く。
RP1 は要求を読んでから返信を公開し、DDR head の readback 後に未消費なら pulse を送る。
Linux はローカル DDR に返信があれば取得する。空なら既存 count を受領して
再 enable し、返信 head を再確認してから同じ期限まで `ppoll` する。
再確認では、既存 count と合流した通知の返信も取得できる。
通知できる owner は終端状態も同じ vector で伝える。Linux は起床時と期限時に検証する。
