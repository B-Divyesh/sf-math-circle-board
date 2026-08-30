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
