#!/usr/bin/env bash
set -euo pipefail

repo_dir="$(cd "$(dirname "$0")/.." && pwd)"
test_data_dir="$(mktemp -d)"
test_port="${MCB_TEST_PORT:-18081}"
server_log="$test_data_dir/server.log"
server_pid=""
backend_build_timeout="${MCB_BACKEND_BUILD_TIMEOUT_SECONDS:-600}"

if [[ -n "${CARGO_TARGET_DIR:-}" ]]; then
  cargo_target_dir="$CARGO_TARGET_DIR"
  if [[ "$cargo_target_dir" != /* ]]; then
    cargo_target_dir="$repo_dir/$cargo_target_dir"
  fi
else
  cargo_target_dir="$repo_dir/target"
fi
backend_bin="$cargo_target_dir/debug/math-circle-board"

cleanup() {
  if [[ -n "$server_pid" ]]; then
    kill "$server_pid" 2>/dev/null || true
    wait "$server_pid" 2>/dev/null || true
  fi
  rm -rf "$test_data_dir"
}
trap cleanup EXIT

cd "$repo_dir"
npm run build

printf 'Building the browser-test backend before the startup deadline...\n'
if ! timeout "$backend_build_timeout" cargo build --quiet --features test-auth --bin math-circle-board; then
  printf 'Backend build did not finish within %s seconds.\n' "$backend_build_timeout" >&2
  exit 1
fi

PORT="$test_port" \
DATA_DIR="$test_data_dir/data" \
DIST_DIR="$repo_dir/dist" \
MCB_OWNER_INVITE="adult-setup-code-0123456789" \
MCB_TEST_AUTH_TOKEN="integration-test-entra-token" \
BUILD_SHA="browser-test" \
"$backend_bin" >"$server_log" 2>&1 &
server_pid="$!"

for _ in $(seq 1 120); do
  if curl --silent --fail "http://127.0.0.1:$test_port/health" >/dev/null; then
    break
  fi
  if ! kill -0 "$server_pid" 2>/dev/null; then
    sed -n '1,240p' "$server_log"
    exit 1
  fi
  sleep 0.25
done

curl --silent --fail "http://127.0.0.1:$test_port/health" >/dev/null || {
  sed -n '1,240p' "$server_log"
  exit 1
}

PLAYWRIGHT_BASE_URL="http://127.0.0.1:$test_port" \
MCB_TEST_OWNER_CODE="adult-setup-code-0123456789" \
MCB_TEST_AUTH_TOKEN="integration-test-entra-token" \
MCB_TEST_BACKEND_BIN="$backend_bin" \
npx playwright test --workers=1 "$@"
