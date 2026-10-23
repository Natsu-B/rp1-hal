# rp1-hal

Raspberry Pi CM5 の RP1 にある 2 つの Cortex-M3（proc0 / proc1）向けの、rp-hal 風の Rust HAL と、
BCM2712 上の Linux との AMP 連携、および RP1 ファームを載せる BCM2712 側の third stage loader をまとめたリポジトリ。

状態: 設計段階。設計は [docs/design.md](docs/design.md)。

旧資産（参照のみ、ここへは段階的に移植する）:

- `<local>` — 旧スタンドアロン系列（regdb / PAC 生成器 / rt）
- `<local>` — PoC HAL（OpenAMP 系）と診断ブランチ群
- `<local>` — PoC chainboot loader
