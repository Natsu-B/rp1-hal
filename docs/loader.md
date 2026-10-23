# ローダー rp1boot（設計 D、2026-10-23 草案）

BCM2712 の third stage loader。VPU ファームから `kernel=` として起動され、RP1 にファームを載せ、Linux 用の DT を整えて、本来のカーネルへ渡す。

## 0. 今の PoC ローダーから分かったこと（2026-10-23 の記録）

| 項目 | 値 | 出典 |
|---|---|---|
| I2C で RP1 へ書き込む時間 | 約 3 秒（62.7 KB のイメージ） | `<lab-log>`（chip id 15:40:20 → image loaded 15:40:23） |
| ローダー自前の TFTP | initramfs 3.9 MB の取得完了まで約 36 秒（同じ秒に完了していたカーネルとの差） | 同上（15:39:42 → 15:40:18） |
| TFTP の通信路 | RP1 内蔵の Ethernet（GEM）。RP1 を再ロードすると一度切れる。PoC は「先に全部取る → GEM を止める → RP1 再ロード → GEM を再初期化」 | `net_boot.rs` |
| 配置 | カーネル・initramfs・DTB の置き場所が固定アドレス | `placement.rs` |
| 実験用 feature | 約 50 個。封印（ハッシュ固定）とアドレス直書きが多い | `Cargo.toml` |

## 1. 提案: 大きなファイルは VPU ファームに読ませる

VPU ファームは SD・TFTP（・USB・NVMe）からファイルを読む機能を最初から持っていて、ローダー自前の TFTP より速い見込み（未測定）。
ファームの `initramfs` 指定は中身を解釈せずにメモリへ置き、場所を DTB の `/chosen` に書くだけなので、ここに「起動一式の束」を入れる。

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
| 生成した cmdline.txt が使われた | DTB の `/chosen/bootargs` に `rp1.bundle=<束のハッシュ>` があり、束と一致する |
| config.txt の他の行（dtparam、別の overlay）が RP1 の所有物とぶつかっていない | overlay を最後に当てた後の衝突検査（declaration-and-note.md §3.2） |

config.txt の中身そのものは照合しないが、「使われたかどうか」と「結果として衝突していないか」は確かめられる。
config.txt を手で書き換えても、束と cmdline が合っていて衝突が無ければ通る（害が無い変更は許す）。

## 2. 手順

```
VPU ファーム: config.txt / cmdline.txt を処理、rp1-bundle.img を initramfs としてメモリへ、DTB を作る
  → rp1boot（EL2、x0 = DTB）
  1. ログ開始（UART10）。DTB を読む
  2. /chosen の initrd の範囲から束を見つけ、ヘッダーと各要素の SHA-256 を検査する。bootargs の rp1.bundle= と照合する
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

ローダーの失敗は「この起動方法の失敗」とみなし、同じ規則に従う:

1. 理由を UART10 に出し、ACT LED でエラーを示す（公式と同じ点滅の形式にそろえる。形式は未確認）。
2. 残りの起動順を DTB の `/chosen/bootloader` から読む（読めるかは未確認）。
3. 次が STOP（`0xe`）なら停止する。
4. それ以外は、次の起動方法から始まるよう起動順を指定して再起動する（ファームの `set_reboot_order` の仕組みが使えるかは未確認）。
   使えなければ普通に再起動する（結果として RESTART と同じくやり直しになる）。

| どこで | RP1 の状態 | 動作 |
|---|---|---|
| 手順 1〜5（RP1 に触る前） | EEPROM が載せた標準の RP1 ファームのまま | 上記 1〜4 |
| 手順 6〜7（RP1 を再ロードした後） | ファームが無い / 動いていない | 上記 1〜4（再起動で EEPROM が RP1 を標準ファームに戻す） |
| 手順 8 | — | 上記 1〜4 |

- 宣言 `boot.on_failure = "stock"` を書いた場合だけ、手順 1〜5 の失敗では overlay を当てずに元の DTB で本来のカーネルを起動する（素の Raspberry Pi OS として動く）。既定ではない。
- 同じ失敗で再起動を繰り返し続けるのは公式の RESTART と同じ振る舞い。止めたい場合は起動順を STOP で終える（公式と同じ設定で制御する）。

## 4. 部品

| 部品 | 中身 | 出どころ |
|---|---|---|
| `rp1-boot`（rp1-hal リポジトリの crate、no_std） | RP1 の I2C bootstrap、ELF → イメージ、note の検査、束の検査、ハンドオフ語 | PoC の `rp1_bootstrap.rs` / `rp1_image.rs` / `rp1_note.rs` を整理して移す |
| BCM2712 の I2C・AON GPIO（RUN ピン）・PCIe RC・UART10・タイマー | 既存の `arch_hal` | `aarch64_type1_hypervisor` の crate を rev 固定で使う（ユーザーのリポジトリ） |
| DTB の読み書き・overlay の適用 | libfdt（`fdt_open_into`、`fdt_overlay_apply`、`fdt_pack`） | C のソースを `cc` で組み込む |
| gzip 展開、SHA-256 | `miniz_oxide`、`sha2` | PoC と同じ |
| SD・FAT・TFTP・GEM・DHCP | **持たない** | — |

## 5. 決定（2026-10-23）

1. **大きなファイルを VPU ファームに読ませる（束を `initramfs` として渡す）**。ローダーから SD・TFTP・Ethernet の処理が消え、起動も速くなる見込み。
   前に決めた「ローダーが config.txt を読み直してハッシュ照合」は、§1 の確かめ方（束・cmdline のハッシュ・衝突検査）に置き換える。
2. **失敗時は公式のブートローダーと同じ規則**（§3）。BOOT_ORDER の次へ進むか、STOP なら止まる。`on_failure = "stock"` は任意の選択肢。
3. **手順 7（PCIe を初期化し直して準備完了を待つ）を既定で有効**にする。
4. **I2C の約 3 秒は v0.1 では受け入れる**。PoC ローダーが遅い原因（TFTP、I2C）は別途調べる（TODO）。
