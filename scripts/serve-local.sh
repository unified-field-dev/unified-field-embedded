#!/usr/bin/env bash
# Start the embedded host with the env Leptos expects, restoring /pkg if missing.
#
# Why this exists: `cargo build -p server` does not emit hydrate JS/WASM.
# Only `cargo leptos build|serve --split` fills target/site/pkg/. Starting the binary
# while pkg is empty/cleared mid-build produces /pkg/{{project-name}}.js 404.
#
# Always pass `--split` (rewrites `__wasm_split_placeholder__`). Without it, hydrate
# fails in the browser with unresolved `__wasm_split_placeholder__` imports.
# Plain `wasm-bindgen` must NOT be used after a split build — it leaves broken
# placeholder imports and overwrites the split-fixed JS.
#
# Starter profile: `server-embedded` (backends + counter/auth/tag). Hosted product: unified-field-site.
set -euo pipefail
ROOT="$(cd "$(dirname "$0")/.." && pwd)"
cd "$ROOT"

PKG="$ROOT/target/site/pkg"
WASM_SRC="$ROOT/target/front/wasm32-unknown-unknown/debug/frontend.wasm"
JS="$PKG/{{project-name}}.js"
MAIN_WASM="$PKG/{{project-name}}.wasm"
BG_WASM="$PKG/{{project-name}}_bg.wasm"
SPLIT_MANIFEST="$PKG/__wasm_split_manifest.json"

pkg_ready() {
  [[ -f "$JS" ]] || return 1
  [[ -f "$MAIN_WASM" || -f "$BG_WASM" ]] || return 1
  # Split builds must keep rewritten imports (never bare placeholders).
  if [[ -f "$SPLIT_MANIFEST" ]] && grep -q '__wasm_split_placeholder__' "$JS"; then
    return 1
  fi
  return 0
}

ensure_pkg() {
  if pkg_ready; then
    return 0
  fi

  if [[ -f "$SPLIT_MANIFEST" ]] || grep -q '__wasm_split_placeholder__' "$JS" 2>/dev/null; then
    echo "error: split hydrate assets are missing or broken (placeholder imports in $JS)." >&2
    echo "Re-run a split build; do not restore with plain wasm-bindgen:" >&2
    echo "  CARGO_BUILD_JOBS=1 cargo leptos build --split --bin-features ssr,server-embedded --lib-features hydrate" >&2
    exit 1
  fi

  if [[ ! -f "$WASM_SRC" ]]; then
    echo "error: missing $JS / wasm and no $WASM_SRC to regenerate from." >&2
    echo "Build: CARGO_BUILD_JOBS=1 cargo leptos build --split --bin-features ssr,server-embedded --lib-features hydrate" >&2
    exit 1
  fi

  echo "Restoring non-split target/site/pkg from frontend.wasm via wasm-bindgen…"
  echo "warning: for lazy routes use cargo leptos build --split instead." >&2
  mkdir -p "$PKG"
  local tmp
  tmp="$(mktemp -d)"
  wasm-bindgen --target web --out-name "{{project-name}}" --out-dir "$tmp" "$WASM_SRC"
  cp -a "$tmp"/. "$PKG"/
  rm -rf "$tmp"
  if [[ -f "$BG_WASM" && ! -e "$MAIN_WASM" ]]; then
    ln -sf "{{project-name}}_bg.wasm" "$MAIN_WASM"
  fi
}

ensure_pkg

export PHOTON_TRANSPORT_KEY="${PHOTON_TRANSPORT_KEY:-cGhvdG9uLWRldi10cmFuc3BvcnQta2V5LTMyYnl0ZXM=}"
export VALENCE_OWNERSHIP_UNIFIED_FETCH="${VALENCE_OWNERSHIP_UNIFIED_FETCH:-0}"
export LEPTOS_OUTPUT_NAME={{project-name}}
export LEPTOS_SITE_ROOT=target/site
export LEPTOS_SITE_PKG_DIR=pkg
export LEPTOS_SITE_ADDR="${LEPTOS_SITE_ADDR:-0.0.0.0:3000}"

exec ./target/debug/server
