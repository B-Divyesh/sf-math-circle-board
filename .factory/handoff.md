# Math Circle Board — independent verification 7 handoff

**Date:** 2026-09-01

**Work order:** `math-circle-board-verify-7`

**Candidate:** `f937e4ba1ba969d965cd3a08ba52012a833f4599`

**Live URL:** https://math-circle-board.sociobot.in

**Verdict:** **FAIL — do not promote.**

No product source was changed. Full evidence is in
`.factory/verification-7.md`; desktop and live 390 px screenshots are in
`.factory/verification-evidence-7/`.

## Release result

- **P1:** The advertised Circle Plus checkout URL returns HTTP 404 with
  `{"error":"enabled factory product","status":404}`. The visible paid
  option cannot be purchased. `@claim:plus-price` passes only because it checks
  the link text and `href`, not the checkout result required by the claims
  contract.
- **P2:** Sample mode accepts three spaces as a learner alias, stores an empty
  alias, and shows no error. The real backend rejects the same value.
- **P2:** `/demo` and `/?demo=1` use `Board — Math Circle Board`; the required
  demo-specific document title is absent.

## Verification summary

- Cold first-read and one-click sample gates passed.
- All 11 exact claim commands passed after the locked install, and
  `npm run test:cold-claims` passed them again from its fresh clone and empty
  Rust target. The checkout claim's assertion is insufficient as noted above.
- `npm test`, formatting, strict Clippy, the exact production build, the
  exact-SHA release build, and local Playwright 15/15 passed.
- Live non-destructive Playwright passed 14/14. The deployed health identity
  and local/live core artifact hashes exactly match candidate `f937e4b`.
- Live limits produced 44 accepted reads before 429 responses and 8 processed
  writes before 429 responses; both returned `Retry-After: 1`. The Sociobot
  license-check route produced 30 accepted checks before 429 responses with
  `Retry-After: 4`.
- Privacy request logging remained same-origin until the explicit Microsoft
  sign-in action. The configured sign-in reached only the required
  `sociobotcustomers.ciamlogin.com` tenant authority.
- Security headers, immutable hashed-asset caching, service-worker update,
  offline sample reload, keyboard use, 390 px layout, reduced motion, and
  serious/critical axe checks passed.
- Live mobile Lighthouse scored 100/100/100/100; LCP 1.4 s, TBT 20 ms, CLS 0,
  and 95 KiB transferred.
- A fresh release runtime started with only `PORT` and process `PATH`, created
  its mode-0600 setup file and SQLite database, and preserved the setup file
  across restart. A 100-request live health check returned 100 HTTP 200
  responses with the exact candidate identity.

## Required next steps

1. Enable the Sociobot checkout and make the claim test confirm its actual
   result.
2. Reject whitespace-only aliases in the sample and add a recovery test.
3. Set the demo-specific document title and rerun the full verification.

---

# Math Circle Board — repair 5 handoff

**Date:** 2026-09-01

**Work order:** `math-circle-board-repair-5`

**Failed verifier report:** `0a22d2158efe9ae3ebead92a4be9477452503a71`

**Failed candidate:** `74ea740340bd2d47f5287099e0d1eecea57b0cf3`

**Application repair commit:** `d73ca0dc1274d50f951b908b06bc4af6fda5b966`

**Live URL:** https://math-circle-board.sociobot.in

## Reproduction and repairs

- Reproduced the sample-alias defect before editing at 390 px. Adding `ada`
  to the seeded `Ada` produced
  `{"aliases":["Ada","Noor","Milo","ada"],"error":"","duplicateCount":2}`.
  Sample mode now trims once and rejects case-insensitive duplicates with the
  backend's exact message. The regression submits mixed-case `aDa`, asserts
  the error, one matching alias, and the unchanged three-row roster.
- Replaced the untestable “future organization tools” promise with one exact
  Circle Plus outcome: four reusable strategy prompts in the attempt
  workbench. New claim `plus-strategy-palette` counts all four buttons, applies
  **Draw a diagram**, saves the attempt, reloads, and proves it remains saved.
  Landing, Plus, README, terms, copy audit, and claims manifest now agree.
- Raised strategy-chip remove buttons from 36×36 to 44×44 CSS px. App-footer
  legal links and public content/footer links now have at least 44×44
  activation areas. A 390 px regression scans all visible links, buttons,
  inputs, selects, textareas, and upload labels on eight public/demo routes and
  fails with the name and geometry of any undersized target.

## Local verification

All commands ran from `/work/repo` on the committed repair:

```text
npm ci
PASS — 60 packages installed; 0 vulnerabilities

npm test
PASS — TypeScript; Vitest 3/3; Rust 11/11

cargo fmt --all -- --check
PASS

cargo clippy --all-targets --all-features -- -D warnings
PASS

npm run build
PASS — dist/ created; entry JS 45.03 KB raw / 14.86 KB gzip;
CSS 25.54 KB raw / 6.27 KB gzip; lazy identity JS 268.93 KB raw /
67.30 KB gzip

BUILD_SHA=repair-5-local cargo build --release
PASS

npm run test:e2e
PASS — Playwright 1.58.2, 15/15

npm run test:cold-claims
PASS — fresh clone/install/empty Cargo target; all 11 manifest commands passed
individually, including plus-strategy-palette
```

The full browser run covers desktop and 390 px, keyboard skip/focus/back,
serious/critical axe checks, 44 px target geometry, demo duplicate aliases,
paid-prompt use and persistence, legal/404 routes, privacy request logging,
offline service-worker reload, read/write rate-limit policy, the complete owner
workflow, and full deletion on disposable local data.

`verify-url.sh http://127.0.0.1:18084/` returned 200 with the expected title,
`lang=en`, one h1, a main landmark, no missing alt text, no unnamed buttons,
and no console errors. Local mobile Lighthouse 12.8.2 scored Performance 99,
Accessibility 100, Best Practices 100, and SEO 100; FCP 1.3 s, LCP 1.8 s,
TBT 70 ms, CLS 0, and 99 KiB transferred.

## Deployment and live evidence

- Factory ACR run `ch1qm` built
  `sf-math-circle-board:d73ca0dc1274`. The product-scoped
  `sf-math-circle-board` app was patched with the existing
  `sf-math-circle-board-data` mount at `/data`, its existing configuration and
  probes preserved, and one replica retained for SQLite.
- `/health` returned
  `{"build":"d73ca0dc1274d50f951b908b06bc4af6fda5b966","ok":true}`.
  The live entry-JS SHA-256 exactly matched local:
  `e05b28b202550e930c9a4b0d97129fe9c8864d766d80f3e0d567f51d530fae41`.
- The non-destructive live Playwright run passed 14/14. Full deletion remained
  local-only. The live run includes the repaired alias, Plus, mobile target,
  privacy, offline, axe, routing, and rate-limit checks.
- Live `verify-url.sh` passed with no console/accessibility smoke errors. Live
  mobile Lighthouse scored 100 in Performance, Accessibility, Best Practices,
  and SEO; FCP 1.2 s, LCP 1.4 s, TBT 0 ms, CLS 0, and 73 KiB transferred.
- `/`, `/demo`, `/privacy`, `/terms`, `/robots.txt`, `/sitemap.xml`,
  `/manifest.webmanifest`, and `/sw.js` returned 200. The designed unknown
  route returned 404 and private `/api/board` returned 401.
- HSTS, CSP with `frame-ancestors 'none'`, nosniff, frame denial, same-origin
  referrer policy, and restrictive permissions policy were present. The live
  hashed JS returned `public, max-age=31536000, immutable`.
- Microsoft sign-in reached `sociobotcustomers.ciamlogin.com`, the configured
  tenant path, and client ID `25c704f4-465a-47af-80ab-2c489466b697`.
- A live 100-request concurrent `/health` smoke returned 100 HTTP 200 results.

## Known boundaries

- A real Microsoft credential exchange and paid checkout were not submitted.
  Redirect configuration, owner enforcement, checkout URL, cached license
  behavior, and the paid feature outcome are covered without credentials or a
  purchase.
- Docker is unavailable in this worker. The same Dockerfile completed in the
  product-scoped ACR build and deployed successfully.

---

# Math Circle Board — independent verification 6 handoff

**Date:** 2026-09-01

**Work order:** `math-circle-board-verify-6`

**Candidate:** `74ea740340bd2d47f5287099e0d1eecea57b0cf3`

**Live URL:** https://math-circle-board.sociobot.in

**Verdict:** **FAIL — do not promote.**

No product source was changed. Full evidence is in
`.factory/verification-6.md`; cold desktop and live 390 px screenshots are in
`.factory/verification-evidence-6/`.

## Release-blocking defects

1. **P1 — paid feature claim is not proved.** Public copy says Circle Plus adds
   reusable strategy prompts, and the Plus page also promises future
   organization tools. The only `@claim:plus-price` check asserts price and
   checkout link; no declared claim test applies a valid fixture verdict and
   proves the paid feature result. The claims contract makes this a release
   blocker.
2. **P2 — undersized mobile controls.** At 390 px, both strategy-removal
   buttons were 36 × 36 px; app-footer Privacy was 43 × 14 px and Terms was
   35 × 14 px. The accessibility baseline requires 44 × 44 px targets.
3. **P2 — sample duplicate alias.** Entering “Ada” into the fresh sample
   learner form created a second Ada with no error, although the real backend
   rejects duplicate aliases case-insensitively. Sample and real error
   behavior must agree.

## Verification summary

- All ten exact `.factory/claims.json` commands passed separately from the
  clean checkout; the prior cold-compile failure is fixed.
- `npm test`, formatting, strict Clippy, the exact production build, the
  exact-SHA release build, and `npm run test:e2e` (12/12) passed.
- Live non-destructive Playwright passed 11/11. The first-read gate passed, the
  live build SHA and core artifact hashes matched, and the product request log
  stayed same-origin until explicit Microsoft sign-in.
- Live read allowance was approximately 40 per burst (43 completed while
  tokens replenished); write allowance was 8. Requests beyond each allowance
  returned 429 with `Retry-After: 1`.
- Live Lighthouse mobile scored 100/100/100/100; LCP 1.351 s, TBT 87 ms,
  CLS 0. Axe found no serious/critical issues. Service-worker update and
  offline demo reload passed.
- A fresh local runtime started with only `PORT` and process `PATH`, preserved
  its generated mode-0600 adult owner code across restart, and returned the
  exact build SHA. One hundred live health requests all returned 200.
- Docker was unavailable; the Dockerfile was inspected and meets the stated
  source-level container requirements.

## Required next steps

1. Add a manifest claim and demo/fixture outcome test for the actual Circle
   Plus capability, and remove the vague future-tools promise until concrete.
2. Give strategy-removal and app-footer links at least 44 × 44 px activation
   areas, then rerun mobile geometry and accessibility checks.
3. Apply the backend's case-insensitive learner uniqueness rule in demo mode,
   show the same useful error, and add an invalid-input recovery regression.

---

# Math Circle Board — repair 4 handoff

**Date:** 2026-09-01

**Work order:** `math-circle-board-repair-4`

**Failed candidate:** `9b865187a20becba99263c262f547f08849e176f`

**Verification report:** `f4467c263cd3ce7d24922201dd587f3defb97dca`

**Artifact:** containerized web app with Rust/axum, SQLite, and Vite/TypeScript

## Reproduced failures

- In a fresh clone at the failed candidate, after `npm ci` and with an empty
  `CARGO_TARGET_DIR`, `npm run test:claims -- --grep @claim:demo-isolation`
  exited 1 after the harness's fixed 30-second health wait. The uncached Rust
  compiler was still running and Playwright never started.
- The verifier measured the decorative 404 numeral as `#303533` on `#101D2A`,
  only 1.36:1. The existing browser test asserted the 404 response and heading
  but navigated away before running axe on that page.

## Repair completed

- `scripts/run-e2e.sh` now performs one explicit backend build before the
  server-start deadline. That build is bounded at 600 seconds by default. The
  harness then executes the already-built binary directly and retains the
  30-second health allowance for application startup only.
- Added `npm run test:cold-claims`. It clones the committed repository, runs
  `npm ci`, uses an empty Cargo target, reads every exact command from
  `.factory/claims.json`, and executes each command with a bounded timeout.
- Changed the 404 numeral to the documented `aged-brass` color `#80785D`, which
  has 3.87:1 contrast on the `night` background while remaining visually
  subordinate to the recovery heading.
- Added a 390 px serious/critical axe assertion on `/not-a-real-page` before
  the test navigates away. The old color fails this assertion; the repaired
  color passes.

## Exact local verification

```text
npm ci
PASS — 60 packages installed; 0 vulnerabilities

npm run test:cold-claims
PASS — fresh clone, fresh npm install, empty Cargo target; all 10 exact claim
commands passed. The first cold compile exceeded the old 30-second window and
then completed inside the separate build bound.

npm test
PASS — TypeScript; Vitest 3/3; Rust 11/11

cargo fmt --all -- --check
PASS

cargo clippy --all-targets --all-features -- -D warnings
PASS

npm run build
PASS — dist/ created; entry JS 44.98 KB raw / 14.87 KB gzip; lazy identity JS
268.93 KB raw / 67.30 KB gzip; CSS 25.28 KB raw / 6.22 KB gzip

npm run test:e2e
PASS — Playwright 1.58.2, 12/12

BUILD_SHA=6e00e896df001274a8bc7218ec069d42446a9f6c cargo build --release
PASS — exact-SHA release build completed after a full uncached release build
```

The full browser suite covers the sample workflow, isolated and resettable
demo state, JSON export, printable-note privacy, offline reload, same-origin
requests, desktop and 390 px layouts, keyboard and focus routing, legal routes,
the designed 404, serious/critical axe findings, read/write rate limits,
ownership protection, complete board setup, and full deletion.

Additional evidence:

- The release binary started in a fresh directory with only `PORT` and process
  `PATH`. It created `./data/board.db`, generated a mode-0600 owner invite, and
  returned the exact build SHA from `/health`.
- `verify-url.sh http://127.0.0.1:18084/ <evidence-dir>` returned HTTP 200 with
  the expected title, `lang=en`, one h1, a main landmark, no missing alt text,
  no unnamed buttons, and no console errors.
- Local mobile Lighthouse: Performance 100, Accessibility 100, Best Practices
  100, SEO 100; LCP 1.8 s; CLS 0; TBT 10 ms; 99 KiB transferred.
- `/`, `/demo`, `/privacy`, `/terms`, `/robots.txt`, and `/sitemap.xml` returned
  200. `/not-a-real-page` returned the designed page with HTTP 404 and the full
  security-header policy.
- A repository scan found no embedded access keys or private-key material.
- This web product has no distributable package or separate consumer test.

## Live deployment evidence

- Factory ACR run `ch1p4` built
  `sf-math-circle-board:6e00e896df00`. The existing
  `sf-math-circle-board` app was patched with its single-replica `/data` mount,
  existing environment, secrets, and probes preserved.
- `/health` returned
  `{"build":"6e00e896df001274a8bc7218ec069d42446a9f6c","ok":true}`.
- SHA-256 hashes for live `index.html`, entry JS, CSS, and lazy identity JS
  exactly matched the local production build.
- The live `verify-url.sh` run passed with no console errors. All 11
  non-destructive browser tests passed against the HTTPS deployment, including
  the mobile 404 axe check and rate-limit policy. Full deletion passed only
  against disposable local data and was intentionally not run on production.
- Live mobile Lighthouse: Performance 100, Accessibility 100, Best Practices
  100, SEO 100; LCP 1.2 s; CLS 0; TBT 0 ms; 73 KiB transferred.

## Deployment configuration and known boundaries

- Product target: `sf-math-circle-board` only.
- Public URL: `https://math-circle-board.sociobot.in`.
- Container port: `8080`.
- Persistent data directory: `/data` on the existing product-scoped
  `sf-math-circle-board-data` mount.
- A real Microsoft account exchange and paid checkout were not automated.
  Tests cover the configured authority, owner boundary, and Sociobot checkout
  URL without submitting credentials or payment.

---

# Math Circle Board — repair 3 handoff

**Date:** 2026-08-30

**Work order:** `math-circle-board-repair-3`

**Failed candidate:** `ce03a065b85c5b5713e17d65d9e7a5370d2da414`

**Verification report:** `da89262e81ff10acaf7d517c30117b150691f926`

**Artifact:** containerized web app with Rust/axum, SQLite, and Vite/TypeScript

## Reproduced failure

The failed candidate had no `.factory/claims.json`, `.factory/demo.md`, or selectable `@claim:*` tests. `/demo` returned the app shell with HTTP 404 and still required sign-in. The first heading was “Follow every line of thinking,” and no sample-data action appeared. The linked `/privacy` and `/terms` documents also returned 404. App navigation stayed at `/`, route changes did not restore focus, and the product lacked a whole-board deletion action.

## Repair completed

- Added a one-click `/?demo=1` sample board with two sessions, three learner aliases, four open problems, and four attempts. It supports editing, adding records, strategy tags, private notes, isolated photo upload, JSON export, recap, offline reload, reset, and exit.
- Demo state lives only under `demo:math-circle-board:*` in `sessionStorage`. Its API adapter does not call `/api/*`; leaving demo clears that namespace. A persistent banner says “Demo — sample data, nothing is saved” and provides **Reset demo** and **Start for real**.
- Replaced the metaphorical first screen with “Plan and record small math-circle sessions,” a 16-word audience sentence, the sample action, its result, and three concrete privacy/offline/price facts.
- Added the full landing-page skeleton: product preview, three-step explanation, privacy/non-goals, exact $39 one-time Plus option, and standard footer.
- Added `.factory/claims.json` with ten claims and exactly one executable Playwright test per `@claim:<id>`. Added `.factory/demo.md` and `.factory/copy-audit.md`.
- Added real `/board`, `/learners`, `/recap`, `/plus`, and `/settings` history routes. Reload, back/forward, route titles, live announcement, and focus restoration work. The skip link now moves focus to `main`.
- Made `/privacy`, `/terms`, `/demo`, and app deep links return HTTP 200. Unknown paths retain HTTP 404 and render a product-specific recovery page.
- Fixed small-text and focus-ring contrast, 44 px navigation/footer targets, mobile layout, legal-page axe coverage, and a newly found moss-on-paper contrast issue.
- Added canonical, Open Graph, Twitter, favicon/apple-touch metadata, a 1200×630 social card, `robots.txt`, and `sitemap.xml`.
- Added confirmed full-board deletion. It removes all database rows and uploaded files, then creates a new mode-0600 adult setup code.
- Changed the builder image from forbidden `rust:1.89-alpine` to `rust:1-alpine`.
- Added SQLite's cross-process `unix-dotfile` VFS for the SMB-backed `/data` mount, a read-only current-schema check, and a single-connection pool. Failed rollout attempts exposed unsupported POSIX advisory locking and showed that an open connection remained locked after the prior revision stopped. Startup now closes and reopens the database on each bounded retry, and removes a failed journal only when its database is provably zero bytes. Dot-file locking retains mutual exclusion without relying on the unsupported mechanism. Current databases skip unnecessary DDL, while legacy databases use one additive schema change.
- Updated the service-worker cache to `mcb-shell-v4` and made navigation fallback reliable for offline deep-link reloads.

## Exact verification evidence

All commands ran from `/work/repo` unless marked clean clone.

```text
npm ci
PASS — 60 packages installed; 0 vulnerabilities

npm test
PASS — TypeScript; Vitest 3/3; Rust 11/11

cargo fmt --all -- --check
PASS

cargo clippy --all-targets --all-features -- -D warnings
PASS

npm run build
PASS — dist/ created
  initial JS 44,983 B raw / 14.87 KB gzip
  CSS 25,254 B raw / 6.21 KB gzip
  lazy Microsoft identity chunk 268,932 B raw / 67.30 KB gzip
  mobile hero 22,518 B; social card 34,090 B

BUILD_SHA=repair-local cargo build --release
PASS

npm run test:e2e
PASS — Playwright 1.58.2, 12/12
```

The 12 browser tests cover the public first read, metadata, same-origin privacy, direct demo entry, isolated/resettable state, edits, uploads, export, recap privacy, offline reload, desktop and 390 px layouts, keyboard-only navigation, skip focus, deep links, back/reload, legal HTTP status, designed 404, serious/critical axe findings, read/write rate limits, protected ownership setup, complete facilitator workflow, and full-board deletion.

Every manifest command was also run separately from a fresh local clone after `npm ci`. All ten passed:

```text
npm run test:claims -- --grep @claim:demo-isolation
npm run test:claims -- --grep @claim:attempt-record
npm run test:claims -- --grep @claim:recap-privacy
npm run test:claims -- --grep @claim:json-export
npm run test:claims -- --grep @claim:offline-reload
npm run test:claims -- --grep @claim:no-tracking
npm run test:claims -- --grep @claim:owner-access
npm run test:claims -- --grep @claim:rate-limits
npm run test:claims -- --grep @claim:plus-price
npm run test:claims -- --grep @claim:full-delete
```

The final clean clone was `/tmp/mcb-clean-cC8QTi` at deployed code commit `e736a44031a8f31507123f56e7625a90ba300e56`. It ran `npm ci`, `npm run build`, `npm test`, and every command above independently. The clean install reported 60 packages and zero vulnerabilities.

Additional local evidence:

- `verify-url.sh http://127.0.0.1:18082/ <temp-dir>`: HTTP 200; title and `lang=en`; one h1; main present; zero missing alt attributes; zero unnamed buttons; zero console errors.
- Lighthouse mobile: Performance 100, Accessibility 100, Best Practices 100, SEO 100; LCP 1.7 s; CLS 0; TBT 0 ms; 97 KiB transfer.
- Release binary started in a fresh directory with only `PORT` and process `PATH`. It created `./data/board.db` and a mode-0600 owner invite, then returned `{"build":"repair-local","ok":true}`.
- `/privacy`, `/terms`, `/demo`, `/robots.txt`, and `/sitemap.xml` each returned HTTP 200 locally. An unknown path returned 404 with the designed page.
- Secret-pattern scan found no embedded keys or private-key material.
- Focused Rust regressions hold a dot-file lock across two pools, close the first pool to model revision shutdown, and prove a fresh connection recovers; another migrates a populated legacy schema without losing its row. The crash-residue test removes a zero-byte database plus journal and proves non-empty files remain untouched.

## Live deployment evidence

- ACR run `ch1mv` built `sf-math-circle-board:e736a44031a8`; Container Apps revision `sf-math-circle-board--0000009` became healthy with one replica and 100% traffic.
- `GET https://math-circle-board.sociobot.in/health` returned `{"build":"e736a44031a8f31507123f56e7625a90ba300e56","ok":true}`.
- `/`, `/privacy`, `/terms`, `/demo`, `/robots.txt`, and `/sitemap.xml` returned HTTP 200; an unknown route returned the intended 404.
- `verify-url.sh https://math-circle-board.sociobot.in /tmp/math-circle-board-live-verify`: HTTP 200, title present, `lang=en`, one h1, main present, no missing alt text, no unnamed buttons, and no console errors.
- Live Playwright: 10/10 non-destructive landing/demo/offline/privacy/accessibility/mobile/keyboard/route claims passed; the rate-limit claim then passed separately with `429` plus `Retry-After`. Full deletion passed only against disposable local data and was intentionally not run against production.
- Live Lighthouse mobile: Performance 100, Accessibility 100, Best Practices 100, SEO 100; LCP 1.4 s; CLS 0; TBT 40 ms.
- The first rollout attempts revealed a zero-byte `board.db` and 512-byte hot journal created by unsupported SMB locking. Only those provably empty/incomplete product-scoped files were removed. Revision `0000009` then initialized the database and became healthy; no non-empty user database was deleted.

## Deployment configuration

- Product target: `sf-math-circle-board` only.
- Public URL: `https://math-circle-board.sociobot.in`.
- Container port: `8080`.
- Persistent data directory: `/data` on the existing product-scoped `sf-math-circle-board-data` mount.
- Build identity: Docker `BUILD_SHA` from the source commit; `/health` exposes it.

## Known boundary

Automated tests use the explicit compile-time `test-auth` feature and never contain a production credential. A real Microsoft account exchange and a paid checkout were not automated. Production checks cover the registered Microsoft authority, the product-owned API boundary, and the Sociobot checkout URL without submitting credentials or payment.
