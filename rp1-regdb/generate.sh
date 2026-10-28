#!/usr/bin/env bash
set -euo pipefail

root=$(cd "$(dirname "$0")/.." && pwd)
cd "$root"

cargo run -p rp1-regdb -- validate
cargo run -p rp1-regdb -- generate
cargo run -p rp1-regdb -- generate --check
cargo fmt -p rp1-regdb -p rp1-pac
