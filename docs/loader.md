# ローダー rp1boot（設計 D、2026-10-23 草案）

BCM2712 の third stage loader。VPU ファームから `kernel=` として起動され、RP1 にファームを載せ、Linux 用の DT を整えて、本来のカーネルへ渡す。

## 0. 今の PoC ローダーから分かったこと（2026-10-23 の記録、2026-10-28 訂正）

| 項目 | 値 | 出典 |
|---|---|---|
| I2C で RP1 へ書き込む時間 | 約 3 秒（62.7 KB のイメージ） | `<lab-log>`（chip id 15:40:20 → image loaded 15:40:23） |
| ローダー自前の TFTP | **訂正（2026-10-28）**: 旧記録の「initramfs に約 36 秒」は誤帰属。36 秒はカーネル取得完了から initramfs 取得完了までで、間にカーネルの gzip 展開と SHA-256 が入る。initramfs の取得そのものは約 4 秒。再測定: 4.35 MB の取得 2.85 s（約 1.46 MiB/s）、512 B stop-and-wait（STATIC） | 同上（15:39:42 → 15:40:14 開始 → 15:40:18）、`<lab-log>`（HW+STATIC） |
| ローダーの遅さの本当の内訳 | ソフトウェア gzip 展開 約 17.8 s、展開後 Image の検査・SHA-256 約 13.0 s、initramfs の SHA-256 約 2.4 s、I2C 約 3.4 s（64 B チャンク、DIV=2500、実 SCL は未測定）。キャッシュ属性の影響は HYP | 同上 `timing-20261028.json`（HW） |
| TFTP の通信路 | RP1 内蔵の Ethernet（GEM）。RP1 を再ロードすると一度切れる。PoC は「先に全部取る → GEM を止める → RP1 再ロード → GEM を再初期化」 | `net_boot.rs` |
| 配置 | カーネル・initramfs・DTB の置き場所が固定アドレス | `placement.rs` |
| 実験用 feature | 約 50 個。封印（ハッシュ固定）とアドレス直書きが多い | `Cargo.toml` |

## 1. 提案: 大きなファイルは VPU ファームに読ませる

VPU ファームは SD・TFTP（・USB・NVMe）からファイルを読む機能を最初から持っている。TFTP は実測 **約 2.9 MiB/s**（16 MiB 5.48 s、32 MiB 11.02 s、HW 2026-10-28）で、PoC ローダー自前の TFTP（約 1.46 MiB/s）より速い。
ファームの `initramfs` 指定は中身を解釈せずにメモリへ置き、場所を DTB の `/chosen` に書くだけなので、ここに「起動一式の束」を入れる。

**検証済み（HW 2026-10-28、`<lab-log>`・`01-blob32/`）:**

- cpio でない 16 MiB・32 MiB のバイナリをそのまま置き、全域のパターン照合で不一致 0。最大サイズは未確定（**少なくとも 32 MiB**）。
- 置き場所はファームが決める（16 MiB で `[0x2e000000,0x2f000000)`、32 MiB で `[0x2d000000,0x2f000000)`、終端が 0x2f000000 付近）。`followkernel` は「ローダー直後」を意味しない。ローダーは必ず `/chosen/linux,initrd-start/end` を読む。
- `linux,initrd-start/end` は今回 BE32 の 1 セル。4 byte と 8 byte の両方を扱う。
- ローダーは EL2、DTB は 0x20000000 で渡された。
- SD 経路は未試験。

ただし束にしても、ローダー内のソフトウェア展開と SHA-256 が大きな待ち時間として残り得る（§0）。gzip のカーネルを避ける・SHA を速くする（キャッシュ有効化など、HYP）は別途。

```
config.txt（生成）
  kernel=rp1boot.img
  initramfs rp1-bundle.img followkernel
  auto_initramfs=0
  arm_64bit=1 ...（固定の行） + config_extra
cmdline.txt（生成）
  console=... uio_pdrv_genirq.of_id=generic-uio ... rp1.bundle=<束の SHA-256 先頭 16 桁> + cmdline_extra

rp1-bundle.img（生成。cargo rp1 build が組み立てる）
  ヘッダー（magic、版、各要素の位置・長さ・SHA-256）
  rp1.elf          … ファーム（.note.rp1、.rp1.dtbo 入り）
  Image(.gz)       … 本来の Linux カーネル（利用者が指定したファイルを取り込む）
  initramfs        … 省略可（利用者が指定したファイルを取り込む）
```

これでローダーは SD・TFTP・Ethernet の処理を持たなくてよい。SD 起動とネットブートの違いは VPU ファームが吸収するので、
「SD と TFTP を素直な形で残す」は、ファームの通常の仕組みそのものになる。

**「必ず生成した config.txt / cmdline.txt を使う」の確かめ方**（前案の「ローダーが config.txt を読み直してハッシュ照合」を置き換える）:

| 確かめること | 方法 |
|---|---|
| `kernel=rp1boot.img` が使われた | ローダー自身が動いていること |
| `initramfs rp1-bundle.img` が使われた | `/chosen/linux,initrd-start/end` の先に束のヘッダーがあり、ハッシュが合う |
| 生成した cmdline.txt が使われた | DTB の `/chosen/bootargs` を空白で区切った**引数トークン**の中に `rp1.bundle=<束のハッシュ>` があり、束と一致する。ファームは自分の引数を前に足すので、bootargs 全体の一致では照合しない（HW 2026-10-28: 値はそのまま残る） |
| config.txt の他の行（dtparam、別の overlay）が RP1 の所有物とぶつかっていない | overlay を最後に当てた後の衝突検査（declaration-and-note.md §3.2） |

config.txt の中身そのものは照合しないが、「使われたかどうか」と「結果として衝突していないか」は確かめられる。
config.txt を手で書き換えても、束と cmdline が合っていて衝突が無ければ通る（害が無い変更は許す）。

## 2. 手順

```
VPU ファーム: config.txt / cmdline.txt を処理、rp1-bundle.img を initramfs としてメモリへ、DTB を作る
  → rp1boot（EL2、x0 = DTB）
  1. ログ開始（UART10）。DTB を読む
  2. /chosen の initrd の範囲（1/2 セルどちらも）から束を見つけ、ヘッダーと各要素の SHA-256 を検査する。bootargs のトークン rp1.bundle= と照合する
  3. rp1.elf の .note.rp1 を検査する（CRC、知らない critical レコード、BOOT レコード）
  4. 作業用の DTB を広い領域に作り（fdt_open_into）、.rp1.dtbo を当てる（libfdt の fdt_overlay_apply）。当てた後の衝突を検査する
  5. /chosen の initrd を束の中の initramfs の範囲に書き換える（initramfs が無ければ initrd のプロパティを消す）
     メモリの空き（/memory − reserved-memory − 束 − DTB − ローダー自身）を DTB から計算し、カーネルの置き場所を決める
  6. RP1: I2C で bootrom に戻す → チップ ID を確かめる → イメージを書き込む → 共有メモリの物理アドレスなどのハンドオフ語を
     共有 SRAM に書く（同じく I2C）→ 起動（EEPROM の bootmain と同じ起動手順）
  7. 確認（既定で有効）: ローダーが PCIe を初期化し直し、ファームの準備完了の印を BAR2 で待つ（上限 5 秒）
  8. カーネルを置く（.gz なら展開）。2 MiB 境界。キャッシュを掃き出し、EL2 のまま x0 = DTB でジャンプ
```

- 手順 6 の I2C 書き込みは約 3 秒かかる（HW）。v0.1 はそのまま受け入れる。短くする案（I2C で小さな stub だけ書き、残りは PCIe の BAR2 経由で書く）は v0.2 で検討。
- 手順 7 は、Linux が起動してから RP1 の不調に気づく（SCMI のクロック待ちで止まる等）のを避けるため。既存 PoC で動いている経路。

## 3. 失敗したとき（公式のブートローダーに合わせる）

公式の EEPROM ブートローダーは、起動方法が失敗すると `BOOT_ORDER` の次の起動方法へ進み、`0xf`（RESTART）なら最初からやり直し、
`0xe`（STOP）ならエラー表示を出して止まる（DOC: Raspberry Pi の BOOT_ORDER の説明。boot.conf のコメントが参照している）。
この CM5 の現在の起動順は `order f1`（NETWORK → SD → RESTART、UART10 の `Boot mode: NETWORK (02) order f1`、HW）。

ローダーの失敗は「この起動方法の失敗」とみなし、同じ規則に従いたい。ただし **`/chosen/bootloader` に残りの起動順は無い**
（HW 2026-10-28: 全 10 属性 `boot-mode=2`, `version`, `capabilities`, `update-timestamp`, `build-timestamp`, `tryboot`, `rsts`,
`partition`, `arg1`, `count`。BOOT_ORDER 相当は無い。`<lab-log>`）。
今の起動方法（`boot-mode`）は分かるが、「次」はローダーが自分では知らない。そこで:

1. 理由を UART10 に出し、ACT LED でエラーを示す。形式は公式に合わせ「長い点滅 n 回 → 短い点滅 m 回、通常 2 秒休んで繰り返す」
   （DOC: Raspberry Pi「LED warning flash codes」。例 0/3 一般的な起動失敗、4/3 RP1 が無い）。EL2 からの LED 制御は未実装・未試験。
2. 次の起動方法を決める。候補（どれも未試験）:
   - **案 A（DOC/HYP）: 一回限りの起動順で再起動する。** ファームの `set_reboot_order`（config.txt の説明。実行時は mailbox
     property tag `0x0003808b`、4 byte の新しい BOOT_ORDER）。再起動に耐えるレジスタに置かれ、使われたら消える（DOC）。
     値は宣言（例 `boot.fallback_order`、束のヘッダーに入れる）と現在の `boot-mode` から作る。EL2 の third stage loader から
     この mailbox が使えるかは HYP。
   - **案 B（HYP）: EEPROM 側の BOOT_ORDER をそのまま次へ進める。** 再起動後は EEPROM が BOOT_ORDER を最初から辿るので、
     普通に再起動すると同じ方法を再試行するだけになる（= RESTART 相当）。EEPROM の BOOT_ORDER をローダーが読む手段
     （mailbox で bootloader 設定を読む等）は未確認。読めれば「現在の `boot-mode` の次の nibble」を案 A に渡せる。
   - どちらも使えない場合は普通に再起動する（結果として RESTART と同じくやり直しになる）。
3. 次が STOP（`0xe`）なら停止する（案 A/B で次が分かるときだけ判断できる）。

| どこで | RP1 の状態 | 動作 |
|---|---|---|
| 手順 1〜5（RP1 に触る前） | EEPROM が載せた標準の RP1 ファームのまま | 上記 1〜3 |
| 手順 6〜7（RP1 を再ロードした後） | ファームが無い / 動いていない | 上記 1〜3（再起動で EEPROM が RP1 を標準ファームに戻す） |
| 手順 8 | — | 上記 1〜3 |

- 宣言 `boot.on_failure = "stock"` を書いた場合だけ、手順 1〜5 の失敗では overlay を当てずに元の DTB で本来のカーネルを起動する（素の Raspberry Pi OS として動く）。既定ではない。
- 同じ失敗で再起動を繰り返し続けるのは公式の RESTART と同じ振る舞い。止めたい場合は起動順を STOP で終える（公式と同じ設定で制御する）。

## 4. 部品

| 部品 | 中身 | 出どころ |
|---|---|---|
| `boot/rp1boot`（rp1-hal リポジトリの crate） | lib（ホストで試験）: 束の検査、ELF → イメージ、Image ヘッダー、配置の重なり。bin（aarch64）: 入口・例外、MMU、UART10、RP1 の I2C 起動、Linux への引き渡し | PoC の `rp1_bootstrap.rs` / `bcm2712_i2c.rs` / `bcm2712_aon.rs` / `rp1_image.rs` / `linux.rs` を必要な分だけ移植（**実装 2026-10-29**） |
| BCM2712 の I2C・AON GPIO（RUN ピン）・UART10・タイマー | rp1boot 内の数十行の MMIO（`arch_hal` は使わない）。PCIe RC は触らない（§6） | — |
| DTB の読み書き・overlay の適用・最終検査 | **`rp1-dt`（純 Rust、no_std + alloc）**。libfdt の `fdt_overlay_apply` と同じ意味で、`cargo rp1 check --dtb` と同じコード | 決定（§6）: cc + libfdt はやめた |
| SHA-256 | `sha2`（no_std）。gzip 展開は持たない（カーネルはホストで展開して束に入れる） | — |
| SD・FAT・TFTP・GEM・DHCP | **持たない** | — |

## 5. 決定（2026-10-23）

1. **大きなファイルを VPU ファームに読ませる（束を `initramfs` として渡す）**。ローダーから SD・TFTP・Ethernet の処理が消え、起動も速くなる見込み。
   前に決めた「ローダーが config.txt を読み直してハッシュ照合」は、§1 の確かめ方（束・cmdline のハッシュ・衝突検査）に置き換える。
2. **失敗時は公式のブートローダーと同じ規則**（§3）。BOOT_ORDER の次へ進むか、STOP なら止まる。`on_failure = "stock"` は任意の選択肢。
   2026-10-28 修正: DTB から残りの起動順は読めない（HW）。次の方法は `set_reboot_order`（案 A、DOC/HYP）か EEPROM の BOOT_ORDER（案 B、HYP）で決め、どちらも検証してから確定する。
3. **手順 7（PCIe を初期化し直して準備完了を待つ）を既定で有効**にする。
4. **I2C の約 3 秒は v0.1 では受け入れる**。PoC ローダーが遅い原因は 2026-10-28 に調べた（§0）: 主因は TFTP ではなくソフトウェアの gzip 展開と SHA-256。

## 6. v0.1 の実装（2026-10-29、HW hwrun5）

実装: `boot/rp1boot`（ローダー）、`rp1-dt`（DTB）、`rp1-proto::bundle`（束の形式）、`cargo rp1 bundle`（起動一式）、
`lab/build-rp1boot.sh`（lab の組）。証拠: `<lab>`。

| 項目 | 決めたこと | 根拠 |
|---|---|---|
| ツールチェーン | **stable** の Rust、rustup 配布の `aarch64-unknown-none-softfloat` の core/alloc。build-std も nightly 機能も使わない | STATIC（ビルド）+ HW（cold boot で Linux まで） |
| 束 | `RP1BNDL` ヘッダー（各要素の位置・長さ・SHA-256）+ `rp1.elf` + **非圧縮の arm64 Image** [+ initramfs]、4 KiB 境界。束の id はヘッダーの SHA-256、cmdline.txt の末尾に `rp1.bundle=<id の先頭 8 byte の hex>` | ローダーにソフトウェア gzip を持たせない。要素ごとの SHA-256 で壊れた束を拒否（HW: カーネル中の 1 bit 反転を検出） |
| ハッシュ | 束全体（28.7 MB）の SHA-256 を 183 ms（EL2 で恒等写像の MMU・キャッシュを有効にした後） | HW。PoC はキャッシュ無しで Image の SHA-256 に約 13 s |
| overlay | `rp1-dt::overlay::apply`（純 Rust）。fixture と実ボードの DTB で `fdtoverlay` と同じ木になる（ホスト試験）。当てた後に `rp1_dt::check::check`（`gpio-reserved-ranges` の縮小 + `final_conflicts`）。実機では適用と検査で 1 ms | STATIC + HW |
| initrd | `/chosen/linux,initrd-*` を束の中の initramfs の範囲（8 byte）に書き換える。コピーしない | HW（Linux が initramfs から `~ #`） |
| 配置 | ローダー 0x80000（ファームの表示は「offset 0x200000」だが実際は 0x80000 で動く、HW）、カーネル 0x06000000、最終 DTB 0x20200000。ローダー・束・元の DTB・shm・最終 DTB の予約と重なれば停止 | HW |
| RP1 | PoC と同じ I2C 手順。scratch の stack は note ENTRY の初期 MSP（0x10004000、ファームの reset は自分で MSP を設定する）。I2C 27,800 B で 822 ms | HW |
| PCIe（§2 手順 7） | **ローダーは RC を触らない**。VPU ファームは OS 起動前に PERSTn を下げたままにするので、RP1 の冷起動は Linux の brcm-pcie probe の PERSTn 解除（RP1 起動から約 2.5 s）を待ってエンドポイントを立てる（`rp1_hal::cold` の待ちを 1.5 s → 20 s）。手順 7（ローダーでの再初期化と準備完了待ち）は v0.1 では実装しない | HW（perstn_wait 2.4〜2.5 s、link 8.2 ms、SCMI・ドアベル・shm 成立） |
| 失敗時（§3） | v0.1 は理由を UART10 に出して停止（例外も ESR/ELR/FAR を出して停止）。案 A（`set_reboot_order`）・LED は未実装（TODO） | HW（負例） |

実機で分かった注意:

- ファームは **DTB に既にある `/chosen/bootargs` を cmdline.txt の前に足し**、bootargs を約 1966 B で切る（HW）。
  ローダーは末尾のトークンが欠けたら「切り詰め」と出して止まる。`cargo rp1 bundle` は 880 B を超える cmdline を拒否する。
- 基準 DTB: ファームが選ぶ標準の `bcm2712-rpi-cm5l-cm5io.dtb`（+ `bcm2712d0.dtbo`、lab は `dtoverlay=uart1-pi5`）。
  `device_tree=` は使わない。SCMI の transport・クロック利用者の付け替え・予約は `.rp1.dtbo` が自分で足す
  （declaration-and-note.md §3.1）。標準 DTB の `/chosen/bootargs`（約 150 B）がファームにより cmdline.txt の前に付くが、
  880 B の上限内で収まる（HW: bootargs 568 B）。HW（<lab-log> boot 2 回）。
- 起動時間（電源 → BusyBox の `~ #`、UART の 1 s 精度）: **約 31 s**（PoC 84〜87 s）。内訳: VPU ファーム（束 28.7 MB の
  TFTP 約 10 s を含む）約 26 s、rp1boot 約 1.1 s、Linux 約 3 s。
- SD 起動の経路は未試験。
