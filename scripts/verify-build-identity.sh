#!/usr/bin/env bash
set -euo pipefail

repo_dir="$(cd "$(dirname "$0")/.." && pwd)"
expected_sha="${1:-$(git -C "$repo_dir" rev-parse HEAD)}"
work_dir="$(mktemp -d)"
target_dir="$work_dir/target"
port="${MCB_IDENTITY_PORT:-18083}"
server_pid=""

cleanup() {
  if [[ -n "$server_pid" ]]; then
    kill "$server_pid" 2>/dev/null || true
    wait "$server_pid" 2>/dev/null || true
  fi
  rm -rf "$work_dir"
}
trap cleanup EXIT

cd "$repo_dir"
npm run build >/dev/null
BUILD_SHA="$expected_sha" CARGO_TARGET_DIR="$target_dir" cargo build --release --quiet
ln -s "$repo_dir/dist" "$work_dir/dist"

(
  cd "$work_dir"
  exec env -i PATH="$PATH" PORT="$port" "$target_dir/release/math-circle-board"
) >"$work_dir/server.log" 2>&1 &
server_pid="$!"

for _ in $(seq 1 80); do
  if curl --silent --fail "http://127.0.0.1:$port/health" >"$work_dir/health.json"; then
    break
  fi
  if ! kill -0 "$server_pid" 2>/dev/null; then
    cat "$work_dir/server.log" >&2
    exit 1
  fi
  sleep 0.25
done

actual_sha="$(node -e 'const fs=require("fs"); console.log(JSON.parse(fs.readFileSync(process.argv[1],"utf8")).build)' "$work_dir/health.json")"
if [[ "$actual_sha" != "$expected_sha" ]]; then
  printf 'Expected /health build %s, got %s\n' "$expected_sha" "$actual_sha" >&2
  exit 1
fi
printf 'PASS /health build identity: %s\n' "$actual_sha"
