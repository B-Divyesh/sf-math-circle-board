# Math Circle Board — repair 8 handoff

- **Work order:** `math-circle-board-repair-8`
- **Failed candidate:** `eeeeeef2519a0a98aa49611dbf8774ba2d69caba`
- **Repair implementation:** `3e7395a78226af7e1776dd10d3bfcb3d1ab900b5`
- **Live URL:** <https://math-circle-board.sociobot.in>
**Result:** repaired, pushed, deployed, and verified on 2 September 2026.

## What changed

- The complete landing first screen is now present in the server-delivered HTML. Its h1 and sample action render without JavaScript or `/api/status`. JavaScript updates only the ownership controls after status returns.
- Deep links use a separate Vite-built app shell, preventing landing content from flashing on demo, legal, app, and 404 routes.
- At 390 px, app navigation is a four-column grid. Its measured content width is 374 px, its scroll width is 374 px, and all four 90.5×44 px destinations are visible.
- `.factory/claims.json` now has 15 claims. New tagged checks cover the 6–12 learner range, all four sample counts, first-boot files and setup-code length, environment overrides, and the container runtime contract.
- Both the demo adapter and SQLite backend stop the roster at 12 aliases. A database trigger preserves the limit under concurrent writes.
- The service worker now caches root and app navigations separately, keeping both the landing page and demo usable offline after the split-shell change.
- The Docker web build now includes `app.html`.

## Verification evidence

- `npm ci` — 60 packages, zero reported vulnerabilities.
- `npm test` — TypeScript passed; copy/claim mapping passed for 15 claims; Vitest 3/3; Rust 12/12.
- `npm run build` — produced `dist/`; initial app JS 43.95 KB raw / 14.46 KB gzip; CSS 25.77 KB raw / 6.30 KB gzip. Microsoft identity remains a lazy chunk.
- `cargo fmt --check` — passed.
- `cargo clippy --all-targets --all-features -- -D warnings` — passed.
- `cargo build --release` — passed.
- `npm run test:e2e` — 26/26 local browser regressions passed, including desktop, 390 px, keyboard, axe, offline reload/update, privacy requests, errors, and API policy.
- `npm run test:cold-claims` — all 15 exact claim commands passed from a fresh clone and clean install.
- `npm run test:identity -- 3e7395a78226af7e1776dd10d3bfcb3d1ab900b5` — passed from an empty build target with only `PATH` and `PORT` at runtime.
- Worker URL verification — HTTP 200, correct title/lang, one h1, main landmark, complete alt text, and zero console errors.
- Local Lighthouse 12.8.2 at 390×844 with DevTools throttling: LCP 1.725/1.755/1.736 s; median 1.736 s. All runs scored 99 performance and 100 accessibility/best practices/SEO, with CLS 0 and TBT 0.
- Live Lighthouse with the same profile: LCP 1.768/1.743/1.731 s; median 1.743 s. All runs scored 99/100/100/100, with CLS 0 and TBT 0.
- Live non-destructive Playwright run — 25/25 passed. This includes all claims except isolated full-board deletion, which passed locally.
- Live response policy — HSTS, response-header CSP with `frame-ancestors 'none'`, `nosniff`, frame denial, same-origin referrer policy, and restrictive permissions policy. Hashed assets return one-year immutable caching.

Screenshots and machine-readable results are in `.factory/repair-evidence-8/` and `.factory/repair-evidence-8-live/`.

## Deployment

- Factory ACR build `ch1wx` succeeded for `sociobotregistry.azurecr.io/sf-math-circle-board:3e7395a78226`.
- The factory patched only `sf-math-circle-board`, mounted `sf-math-circle-board-data` at `/data`, kept one replica for SQLite, and bound `math-circle-board.sociobot.in`.
- Live `/health` returns `{"build":"3e7395a78226af7e1776dd10d3bfcb3d1ab900b5","ok":true}`.

## Known gaps and next steps

- No release-blocking gap remains from verification 10 or the controller evidence review.
- Docker is unavailable in the worker container. The production Dockerfile was instead built successfully by ACR and exercised in the deployed Container App.
- The researched paid organization tier remains intentionally outside this release; `.factory/scope-deviation.md` records that earlier product decision.
