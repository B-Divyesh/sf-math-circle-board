#!/usr/bin/env bash
set -euo pipefail

repo_dir="$(cd "$(dirname "$0")/.." && pwd)"
test_data_dir="$(mktemp -d)"
test_port="${MCB_TEST_PORT:-18081}"
server_log="$test_data_dir/server.log"
server_pid=""

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
PORT="$test_port" \
DATA_DIR="$test_data_dir/data" \
DIST_DIR="$repo_dir/dist" \
MCB_OWNER_INVITE="adult-setup-code-0123456789" \
MCB_TEST_AUTH_TOKEN="integration-test-entra-token" \
BUILD_SHA="browser-test" \
cargo run --quiet --features test-auth >"$server_log" 2>&1 &
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
npx playwright test --workers=1 "$@"
