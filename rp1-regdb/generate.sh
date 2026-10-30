#!/usr/bin/env bash
set -euo pipefail

root=$(cd "$(dirname "$0")/.." && pwd)
cd "$root"

cargo run -p rp1-regdb -- validate
cargo run -p rp1-regdb -- generate
cargo run -p rp1-regdb -- generate --check
cargo run -p rp1-regdb -- generate-pinmux
cargo run -p rp1-regdb -- generate-pinmux --check
cargo fmt -p rp1-regdb -p rp1-pac -p rp1-pinmux
