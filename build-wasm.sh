#!/usr/bin/env bash
# build-wasm.sh
# Build WASM package like wasm-pack, but simpler (no wasm-pack).
# Based on: https://fourteenscrews.com/essays/look-ma-no-wasm-pack/
# Requires: cargo, wasm-bindgen-cli (cargo install wasm-bindgen-cli)
# Optional: wasm-opt (from Binaryen)
set -euo pipefail

BUILD_TYPE="release"
TARGET="wasm32-unknown-unknown"
OUTDIR="pkg"
SKIP_WASM_OPT=0

while (( "$#" )); do
  case "$1" in
    --debug) BUILD_TYPE="debug"; shift ;;
    --release) BUILD_TYPE="release"; shift ;;
    --target) TARGET="$2"; shift 2 ;;
    --out) OUTDIR="$2"; shift 2 ;;
    --skip-wasm-opt) SKIP_WASM_OPT=1; shift ;;
    *) echo "Unknown arg: $1"; exit 1 ;;
  esac
done

log() { echo -e "\033[1;34m==>\033[0m $*"; }
err() { echo -e "\033[1;31merror:\033[0m $*" >&2; }

# Step 1: check environment
[ -f Cargo.toml ] || { err "Run this from your crate root"; exit 1; }

log "Checking for wasm target..."
rustup target add "$TARGET" >/dev/null 2>&1 || true

log "Building ($BUILD_TYPE)..."
cargo build --lib --target "$TARGET" ${BUILD_TYPE:+--$BUILD_TYPE}

CRATE_NAME=$(sed -n 's/^name\s*=\s*"\(.*\)".*/\1/p' Cargo.toml | head -n1)
WASM_PATH="target/${TARGET}/${BUILD_TYPE}/${CRATE_NAME}.wasm"
[ -f "$WASM_PATH" ] || { err "Couldn't find $WASM_PATH"; exit 1; }

log "Running wasm-bindgen..."
mkdir -p "$OUTDIR"
wasm-bindgen "$WASM_PATH" --out-dir "$OUTDIR" --typescript --target bundler

if (( ! SKIP_WASM_OPT )) && command -v wasm-opt >/dev/null 2>&1; then
  log "Optimizing with wasm-opt..."
  for f in "$OUTDIR"/*.wasm; do
    wasm-opt "$f" -O -o "$f"
  done
else
  log "Skipping wasm-opt"
fi

log "Copying template package.json..."
if [ -f _package.json ]; then
  cp _package.json "$OUTDIR/package.json"
else
  err "_package.json not found"
fi

log "Copying README and LICENSE (if exist)..."
cp -n README* LICENSE* "$OUTDIR"/ 2>/dev/null || true

log "✅ Done. Output in $OUTDIR"
