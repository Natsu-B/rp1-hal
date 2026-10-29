# ビルドの流れ（設計 E、2026-10-23 草案）

宣言から起動一式までを、利用者は `cargo rp1 build` 1 回で作る。中でやることと、各段階で止める検査を決める。

## 1. 道具

| 道具 | 種類 | 役割 |
|---|---|---|
| `rp1-build` | ライブラリ（ファーム crate の build.rs から呼ぶ） | 宣言とボード表を読み、型・linker script・note・overlay・config.txt・cmdline.txt を生成する |
| `cargo-rp1` | cargo のサブコマンド（ホストで動く） | ファームのビルド、ELF 検査、overlay の試し当て、起動一式の組み立て、配置、実機の起動と表示 |
| `rp1boot` | ローダー（aarch64、no_std） | 起動一式の一部。ビルド済みを配布し、ソースからも作れる（§5） |

## 2. `cargo rp1 build` の段階

```
① 宣言の検査と生成（build.rs → rp1_build::configure）
② ファームのコンパイルとリンク（cargo build --target thumbv7m-none-eabi --release）
③ ELF 検査（cargo-rp1）
④ overlay の試し当て（cargo-rp1）
⑤ 起動一式の組み立て（cargo-rp1）
```

### ① 宣言の検査と生成（build.rs）

入力: ファーム crate の `Cargo.toml`（`[package.metadata.rp1]`）、`rp1-build` 同梱のボード表。

| 出力（`OUT_DIR`） | 中身 | 使い道 |
|---|---|---|
| `rp1.rs` | `Proc0Resources` / `Proc1Resources`、shm のハンドル型、note と overlay のバイト列（`#[link_section]` 付き static） | `rp1_rt::bind!()` が include する |
| `rp1-layout.x` | 共有 SRAM（イメージ 48 KiB / link ページ 16 KiB / 予約）、ISRAM・DSRAM（共通イメージ、スタック）、`.note.rp1` と `.rp1.dtbo` を非ロードの節に置く指定、予算の `ASSERT` | `cargo:rustc-link-arg=-T…` |
| `boot/config.txt`、`boot/cmdline.txt` | 固定の行 + 宣言の `config_extra` / `cmdline_extra` | ⑤で起動一式へコピー。ハッシュは note の BOOT レコードへ |
| `cargo:rustc-cfg` | `rp1_log`、`rp1_clock_authority_rp1` など | `log!` 等の条件分岐 |
| `cargo:rerun-if-changed` | Cargo.toml、ボード表 | 再生成の条件 |

- 宣言のエラー（declaration-and-note.md §1.4）はここで止める。
- overlay は rp1-build が FDT を直接書き出す（`dtc` に依存しない）。外部参照は `__fixups__`、内部参照は `__local_fixups__` に入れる。
- 生成物はすべて決定的（同じ宣言・同じボード表なら同じバイト列）。note の FIRMWARE レコードのソースハッシュは git から取り、取れなければ 0。

### ② コンパイルとリンク

- ターゲットは `thumbv7m-none-eabi`、**stable の Rust** で作れることを条件にする（`global_asm!`、`#[unsafe(naked)]` は stable）。
- `rp1-rt` の `link.x` が `rp1-layout.x` を INCLUDE する。
- 予算超過（共有 SRAM のイメージ 48 KiB、DSRAM のスタック + static、ISRAM の共通イメージ 8 KiB）は linker の `ASSERT` で止まる。

### ③ ELF 検査（確実に分かることだけ、design.md §5.2）

| 検査 | 方法 | 結果 |
|---|---|---|
| 予算 | 節のアドレスと大きさを領域と比べ、使用量を表で表示 | 超過はエラー（② と同じ内容を分かりやすく） |
| 共有 SRAM 上の LDREX/STREX | 逆アセンブルし、LDREX/STREX のアドレスレジスタが同じ基本ブロック内で 0x2000_0000..0x2001_0000 の定数（`movw/movt`、リテラル）から来ていればエラー。出所が分からないものは数えて表示 | 型と配置が主な保証なので、これは補助 |
| `#[shared]` の節の中身 | `.shared*` 節のシンボルが `SharedSafe` の型だけから来ているか（rp1-rt のマクロが付ける印で確認） | 印の無いものはエラー |
| note と overlay | `.note.rp1` の CRC・形式、`.rp1.dtbo` が FDT として読めること | 壊れていればエラー |
| 禁止事項 | watchdog TRIGGER（0x4015_4000 bit31）などの禁止アドレスへのリテラル | 警告（AGENTS.md の禁止事項に対応） |

### ④ overlay の試し当て

- ボード表に、対応するカーネル系列ごとの基準 DTB（Raspberry Pi の配布物、`__symbols__` 付き）の SHA-256 と入手先を持たせる。
  手元にあれば libfdt（`fdtoverlay` と同じ実装）で当て、参照が全部解決すること・RP1 所有のノードが無効になることを確かめる。
- 基準 DTB が手元に無ければ警告して先へ進む（CI では必須にする）。

### ⑤ 起動一式の組み立て

```
target/rp1/boot/
  config.txt  cmdline.txt  rp1boot.img  rp1.elf
  manifest.txt   各ファイルの SHA-256、ファームとローダーの版、宣言のハッシュ
```

本来のカーネルと initramfs は利用者が用意する（ファイル名は宣言の `boot.kernel` / `boot.initramfs`）。

## 3. 他のサブコマンド

| コマンド | すること |
|---|---|
| `cargo rp1 check --dtb <file>` | 実際の DTB（例: Linux の `/sys/firmware/fdt` を取ってきたもの）に overlay を当て、ローダーと同じ衝突検査を行う |
| `cargo rp1 deploy --tftp <dir>` / `--sd <mount>` | 起動一式を置く。上書き前に退避し、SHA-256 を表示する |
| `cargo rp1 run` | deploy → 再起動のフック（利用者が設定するコマンド。例: `cm5ctl.py force-boot`）→ UART の表示（`log!` の UART0 と Linux コンソール） |
| `cargo rp1 size` | ③ の予算表だけを表示 |
| `cargo rp1 note <elf>` | note を人が読める形で表示 |

設定（再起動のフック、シリアルポート、TFTP の置き場所）はワークスペースの `.cargo/rp1.toml` に置く（利用者の環境ごとの値で、
ファームの宣言とは分ける）。

## 4. Linux 側のアプリ

- `rp1-linux` は普通の std の crate。ターゲットは `aarch64-unknown-linux-gnu`（または musl）。
- note は実行時に DT のコピー（`/proc/device-tree/rp1-firmware/rp1,note`）から読むので、ビルド時にファームの ELF は要らない。
  型は `protocol` crate を共有するので、食い違いはコンパイル時と open 時（型 ID）に分かる。

## 5. ローダー（rp1boot）の作り方

- 起動一式にはビルド済みの `rp1boot.img` を入れる（`rp1boot-bin` crate に同梱し、版と SHA-256 を固定）。
- ソースから作る場合は `cargo build -p rp1boot --target aarch64-unknown-none-softfloat --release` と
  `llvm-objcopy -O binary`。旧 PoC は nightly と `-Z build-std` を使っていたが、**rp1boot は stable と rustup 配布の
  `aarch64-unknown-none-softfloat` の core/alloc で作れる**（2026-10-29、STATIC + HW）。
- overlay の適用は C の libfdt ではなく純 Rust の `rp1-dt`（`fdtoverlay` との一致をホストで試験）。`cargo rp1 check --dtb` も同じコード。
- 起動一式は今は `cargo rp1 bundle <rp1.elf> --kernel <Image[.gz]> [--initramfs F] [--loader rp1boot.img] [--cmdline ..] [--config-extra ..] --out DIR`
  で作る（`cargo rp1 build` への統合と宣言 `boot.*` からの生成は TODO）。

## 6. CI（ホストだけで回すもの）

- `rp1-proto` の単体試験（リング・SeqLock・overwrite の追い越し・型 ID）。
- `rp1-build` の生成物のゴールデン試験（宣言の例 → rp1.rs / layout / note / overlay / config.txt のバイト列）と、宣言エラーの負例。
- 例のファームを ③ まで通す。④ は基準 DTB を CI のキャッシュに置いて必須にする。
- 実機の試験は CI に入れない（`cargo rp1 run` と手順書で行う）。

## 7. 決めたいこと（設計 E の論点）

1. 道具は「build.rs 用ライブラリ（rp1-build）」と「cargo サブコマンド（cargo-rp1）」の 2 つ（案）。
2. ファームもローダーも stable の Rust で作る（案。ローダーは要確認）。
3. 起動一式には、ビルド済みのローダーを入れる（案）。ソースからのビルドは選択式。
4. 利用者の環境ごとの設定（再起動のフック、シリアルポート、TFTP の場所）は `.cargo/rp1.toml`（案）。
