#!/usr/bin/env bash
set -euo pipefail

repo_dir="$(cd "$(dirname "$0")/.." && pwd)"
cold_root="$(mktemp -d)"
cold_repo="$cold_root/repo"
cold_target="$cold_root/cargo-target"
test_port="${MCB_COLD_TEST_PORT:-18082}"
test_timeout="${MCB_COLD_TEST_TIMEOUT_SECONDS:-900}"

cleanup() {
  rm -rf "$cold_root"
}
trap cleanup EXIT

if ! git -C "$repo_dir" diff --quiet || ! git -C "$repo_dir" diff --cached --quiet; then
  printf 'Commit or stash changes before running the cold-clone regression.\n' >&2
  exit 1
fi

git clone --quiet --no-local "$repo_dir" "$cold_repo"
npm --prefix "$cold_repo" ci

mapfile -t claim_commands < <(
  cd "$cold_repo"
  node --input-type=module -e \
    "import {readFileSync} from 'node:fs'; for (const claim of JSON.parse(readFileSync('.factory/claims.json','utf8'))) console.log(claim.test)"
)

for claim_command in "${claim_commands[@]}"; do
  printf 'Cold-clone claim: %s\n' "$claim_command"
  (
    cd "$cold_repo"
    timeout "$test_timeout" env \
      CARGO_TARGET_DIR="$cold_target" \
      MCB_TEST_PORT="$test_port" \
      bash -c "$claim_command"
  )
done
