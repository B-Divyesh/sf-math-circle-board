# Math Circle Board — verification 10 handoff

**Work order:** `math-circle-board-verify-10`

**Candidate:** `eeeeeef2519a0a98aa49611dbf8774ba2d69caba`

**Live URL:** <https://math-circle-board.sociobot.in>

**Date:** 2 September 2026

**Result:** **FAIL**

Independent QA confirmed that the deployed container identifies itself as the
candidate and that the core product works. All 11 exact claim commands, the
full local test suite, the release build, strict Rust lint, build identity, 19
local browser tests, and 18 safe live browser tests passed.

Release acceptance still fails on three defects:

1. **High:** landing/README claims are missing or incompletely represented in
   `.factory/claims.json`, including the 6–12 learner range, full sample counts,
   first-boot/runtime storage behavior, and container/runtime assertions.
2. **High:** three actual-throttling mobile Lighthouse runs measured LCP at
   2.696 s, 2.532 s, and 2.663 s, all above the `< 2.5 s` budget. The h1 waits
   on the initial JavaScript/API chain.
3. **Medium:** at 390 px the app navigation is 410 px wide inside a 374 px
   viewport and clips about 36 px of the Settings destination.

The cold first-read gate passed. The first screen names the facilitator, job,
and first action, and one click opens an isolated sample. Live privacy,
same-origin requests, secure headers, Microsoft CIAM authority, offline reload,
service-worker update, keyboard focus, axe, 404/legal routes, recap privacy,
export, invalid-input recovery, immutable asset caching, and API throttling all
passed. Observed live allowance: 43/100 reads passed before/refilling around the
40-request burst and 8/30 writes reached validation; the rest returned 429 with
`Retry-After: 1`.

Detailed commands, evidence, and remediation are in
[`.factory/verification-10.md`](verification-10.md). Evidence is in
`.factory/verification-evidence-10/`.

## Reproduce

```sh
npm ci
npm test
npm run build
cargo fmt --check
cargo clippy --all-targets --all-features -- -D warnings
cargo build --release
npm run test:e2e
npm run test:identity
PLAYWRIGHT_BASE_URL=https://math-circle-board.sociobot.in \
  npx playwright test --workers=1 --grep-invert '@claim:full-delete'
```

No product code, deployment, DNS, billing, or cloud resource was modified by
this verification. Only this report, handoff, and verification evidence were
added.
