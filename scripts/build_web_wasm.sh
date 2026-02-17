#!/usr/bin/env bash
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
WASM_TARGET_DIR="$ROOT_DIR/target/wasm32-unknown-emscripten/release"
WEB_WASM_DIR="$ROOT_DIR/web/public/wasm"

mkdir -p "$WEB_WASM_DIR"

if ! command -v emcc >/dev/null 2>&1; then
  EMSDK_ENV_SH="/home/xks/apps/emsdk/emsdk_env.sh"
  if [[ -f "$EMSDK_ENV_SH" ]]; then
    # shellcheck source=/dev/null
    source "$EMSDK_ENV_SH"
  fi
fi

if ! command -v emcc >/dev/null 2>&1; then
  echo "emcc not found. Please source your emsdk env first." >&2
  exit 1
fi

export CARGO_PROFILE_RELEASE_CODEGEN_UNITS=1
export CARGO_PROFILE_RELEASE_LTO=true

# Force new Wasm exception handling (Rust 1.93+ default) for highs-sys
# to avoid __resumeException linker errors
export CXXFLAGS="${CXXFLAGS:-} -fwasm-exceptions"

WASM_STACK_SIZE="${END_WEB_WASM_STACK_SIZE:-1048576}"
RUSTFLAGS_ARGS=(
  "-Copt-level=z"
  "-Cstrip=symbols"
  "-Clink-arg=-sMALLOC=emmalloc"
  "-Clink-arg=-sALLOW_MEMORY_GROWTH=1"
  "-Clink-arg=-sINITIAL_MEMORY=67108864"
  "-Clink-arg=-sSTACK_SIZE=${WASM_STACK_SIZE}"
  "-Clink-arg=-sINCOMING_MODULE_JS_API=[\"noInitialRun\",\"printErr\",\"locateFile\"]"
  "-Clink-arg=-sFILESYSTEM=0"
  "-Clink-arg=-sMODULARIZE=1"
  "-Clink-arg=-sEXPORT_NAME=createEndWebModule"
  "-Clink-arg=-sENVIRONMENT=web,worker"
  "-Clink-arg=-sEXPORTED_FUNCTIONS=[\"_end_web_bootstrap\",\"_end_web_solve_from_aic_toml\",\"_end_web_free_c_string\"]"
  "-Clink-arg=-sEXPORTED_RUNTIME_METHODS=[\"ccall\",\"UTF8ToString\"]"
)

if [[ "${END_WEB_WASM_DEBUG:-0}" == "1" ]]; then
  echo "Building wasm with debug diagnostics flags (END_WEB_WASM_DEBUG=1)."
  echo "WASM stack size: ${WASM_STACK_SIZE} bytes."
  RUSTFLAGS_ARGS+=(
    "-Clink-arg=-sASSERTIONS=2"
    "-Clink-arg=-sSAFE_HEAP=1"
    "-Clink-arg=-sSTACK_OVERFLOW_CHECK=2"
  )
else
  RUSTFLAGS_ARGS+=("-Clink-arg=-sASSERTIONS=0")
fi

printf -v RUSTFLAGS "%s " "${RUSTFLAGS_ARGS[@]}"
export RUSTFLAGS="${RUSTFLAGS% }"

cargo build --target wasm32-unknown-emscripten --release -p end-web --bin end-web

cp "$WASM_TARGET_DIR/end-web.js" "$WEB_WASM_DIR/end_web.js"
cp "$WASM_TARGET_DIR/end_web.wasm" "$WEB_WASM_DIR/end_web.wasm"

echo "Copied wasm artifacts to $WEB_WASM_DIR"
