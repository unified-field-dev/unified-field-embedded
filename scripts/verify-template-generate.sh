#!/usr/bin/env bash
# Generate a scratch host from this Liquid template and run hygiene checks.
# Template maintainers: use this instead of cargo fmt/test at the Liquid root.
set -euo pipefail

ROOT="$(cd "$(dirname "$0")/.." && pwd)"
PROJECT_NAME="${PROJECT_NAME:-my-uf-app}"
APP_TITLE="${APP_TITLE:-My App}"
GH_OWNER="${GH_OWNER:-unified-field-dev}"
GH_REPO="${GH_REPO:-$PROJECT_NAME}"
DEST="${DEST:-$(mktemp -d "${TMPDIR:-/tmp}/uf-embedded-gen.XXXXXX")}"
KEEP_DEST="${KEEP_DEST:-0}"
STAGE="${STAGE:-$(mktemp -d "${TMPDIR:-/tmp}/uf-embedded-stage.XXXXXX")}"

cleanup() {
  rm -rf "$STAGE"
  if [[ "$KEEP_DEST" != "1" && -d "$DEST" && "$DEST" == *uf-embedded-gen* ]]; then
    rm -rf "$DEST"
  fi
}
trap cleanup EXIT

command -v cargo-generate >/dev/null 2>&1 || cargo install cargo-generate --locked

# Stage a clean template copy so local `target/` / sqlite / node_modules are never walked.
echo "Staging template (excluding build artifacts)…"
rsync -a \
  --exclude '.git/' \
  --exclude 'target/' \
  --exclude 'target-docs-idio/' \
  --exclude 'data/' \
  --exclude 'end2end/node_modules/' \
  --exclude 'end2end/playwright-report/' \
  --exclude 'end2end/test-results/' \
  --exclude '*.sqlite3' \
  --exclude '*.sqlite3-*' \
  --exclude '.github/workflows/template-ci.yml' \
  "$ROOT"/ "$STAGE"/

echo "Generating into $DEST (project-name=$PROJECT_NAME)…"
cargo generate \
  --path "$STAGE" \
  --destination "$DEST" \
  --name "$PROJECT_NAME" \
  --define "app_title=$APP_TITLE" \
  --define "gh_owner=$GH_OWNER" \
  --define "gh_repo=$GH_REPO" \
  --force \
  --allow-commands \
  --vcs none

GEN_ROOT="$DEST/$PROJECT_NAME"
if [[ ! -d "$GEN_ROOT" ]]; then
  GEN_ROOT="$DEST"
fi

echo "GEN_ROOT=$GEN_ROOT"

# Fail if Liquid leftovers remain in build-critical paths.
leftovers="$(
  grep -RIn --exclude-dir=target --exclude-dir=.git --exclude-dir=node_modules \
    -e '{{[[:space:]]*project-name' \
    -e '{{[[:space:]]*app_title' \
    -e '{{[[:space:]]*gh_owner' \
    -e '{{[[:space:]]*gh_repo' \
    "$GEN_ROOT" \
    --include='*.rs' --include='*.toml' --include='*.sh' --include='*.yml' \
    || true
)"
if [[ -n "$leftovers" ]]; then
  echo "$leftovers" >&2
  echo "error: leftover Liquid placeholders in generated tree" >&2
  exit 1
fi

leptos_name="$(awk '/\[\[workspace.metadata.leptos\]\]/{f=1} f&&/^name =/{gsub(/"/,"",$3); print $3; exit}' "$GEN_ROOT/Cargo.toml")"
if [[ "$leptos_name" != "$PROJECT_NAME" ]]; then
  echo "error: leptos name '$leptos_name' != project-name '$PROJECT_NAME'" >&2
  exit 1
fi
if ! grep -q "LEPTOS_OUTPUT_NAME=$PROJECT_NAME" "$GEN_ROOT/scripts/serve-local.sh"; then
  echo "error: serve-local.sh LEPTOS_OUTPUT_NAME mismatch" >&2
  exit 1
fi
if ! grep -q "data/${PROJECT_NAME}.sqlite3" "$GEN_ROOT/server/src/valence_bootstrap.rs"; then
  echo "error: DEFAULT_SQLITE_PATH stem mismatch" >&2
  exit 1
fi

cd "$GEN_ROOT"
if [[ "${GENERATE_LOCKFILE:-0}" == "1" ]]; then
  cargo generate-lockfile
fi
if [[ "${GENERATE_NPM_LOCK:-0}" == "1" && -d end2end ]]; then
  (cd end2end && npm install --package-lock-only >/dev/null 2>&1 || true)
fi

echo "Generated host hygiene OK: $GEN_ROOT"
if [[ "${RUN_CHECK:-0}" == "1" ]]; then
  if [[ ! -f Cargo.lock ]]; then
    cargo generate-lockfile
  fi
  CARGO_BUILD_JOBS="${CARGO_BUILD_JOBS:-1}" cargo check -p server --features server-embedded
fi
if [[ "${RUN_TEST_LAB:-0}" == "1" ]]; then
  if [[ ! -f Cargo.lock ]]; then
    cargo generate-lockfile
  fi
  cargo test -p "${PROJECT_NAME}-e2e" -- --test-threads=1
fi

echo "$GEN_ROOT" > "${DEST}/.gen_root_path"
