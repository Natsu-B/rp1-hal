# AGENTS.md

ホスト全体の作業ルール・実機手順は `<local>` に従う。このファイルはこのリポジトリ固有の規則。

## Git

- コミットメッセージは Conventional Commits（`<type>(<scope>): <summary>`）。
  - type: `feat` `fix` `docs` `refactor` `perf` `test` `build` `ci` `chore` `revert`
  - scope は crate 名またはディレクトリ名（例: `feat(rp1-hal): add uart driver`、`docs(design): ...`）。
  - 互換性を壊す変更は `!` を付け、本文に `BREAKING CHANGE:` を書く。
  - summary は英語の命令形・小文字始まり・末尾ピリオドなし。本文は日本語でもよい。
- 1 コミット 1 論点。ビルドが通らない状態をコミットしない。
- `main` へ直接コミットするのは初期化と設計文書だけ。実装は `feat/*` `fix/*` などのブランチで行い、マージする。
- コミットしないもの: ファーム blob、EEPROM、抽出物、`*.elf` `*.img` `*.bin` `*.dtb`、UART ログ、シリアル番号入りの設定、秘密情報。

## 実装の規則

- 実機で確認していないことを「動く」と書かない。文書の主張には根拠の強さ（HW / STATIC / DOC / HYP）を付ける。
- 共有 SRAM（0x2000_0000..）上で RMW atomic（LDREX/STREX）を使うコードを書かない。
- Linux 側はカーネル内部 API に依存しない。DT の変更とユーザープロセスだけで実現する。
- 実験・診断コードは `lab/` に置き、ライブラリ crate に混ぜない。
