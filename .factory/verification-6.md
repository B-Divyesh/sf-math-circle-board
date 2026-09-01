# Independent verification 6 — FAIL

**Date:** 2026-09-01  
**Work order:** `math-circle-board-verify-6`  
**Candidate:** `74ea740340bd2d47f5287099e0d1eecea57b0cf3`  
**Live URL:** https://math-circle-board.sociobot.in  
**Verdict:** **FAIL — do not promote.**

This was a fresh independent product-QA run against the researched brief and
the factory acceptance contract. Product source was not changed.

## Release-blocking findings

### P1 — the paid feature claim has no outcome test

The landing page and README say that Circle Plus adds reusable strategy
prompts. The Plus page also says that future organization tools are included
with the license. Neither outcome is declared and proved in
`.factory/claims.json`.

The sole Plus check, `@claim:plus-price`, only confirms the `$39` text and the
Sociobot checkout URL (`tests/e2e.spec.ts:151-156`). It never applies a valid
recorded license verdict, confirms the strategy palette becomes available, or
uses a prompt. “Future organization tools” is not concrete enough to verify.
This is an unlisted/untested public claim, which the claims contract defines as
release-blocking. The existing demo already exposes the Plus palette, so its
observable result can be covered without making a purchase.

### P2 — several mobile touch targets are smaller than 44 × 44 CSS px

At 390 px in the live demo, browser geometry measured:

- both “Remove strategy” buttons at **36 × 36 px**;
- app-footer “Privacy” at **43 × 14 px**;
- app-footer “Terms” at **35 × 14 px**.

The visually hidden file input was excluded because its 44 px label is the
actual control. Axe reports no serious/critical issue, but target size is a
separate non-negotiable accessibility requirement. These controls need a
44 × 44 px activation area.

### P2 — sample mode accepts a duplicate learner alias

In a fresh live sample, the learner list contained one “Ada.” Entering “Ada”
again produced two “Ada” rows and no error. A blank alias was correctly stopped
by native required-field validation, and the user could recover.

The real backend rejects aliases case-insensitively with `409`, but
`frontend/src/demo.ts:76-79` inserts the duplicate without applying that rule.
This makes the one-click sample behave differently on an important record
integrity path and can produce ambiguous learner recaps.

## Mandatory first-read result — PASS

Cold live desktop load returned HTTP 200. The first viewport said:

- audience: “For volunteer math circle facilitators”;
- job: “Plan and record small math-circle sessions” and sequence problems,
  record partial attempts/private notes, and print a recap for 6–12 learners;
- first action: “Try it with sample data,” followed by “See a filled board.
  Changes stay in this demo.”

The action entered a populated board in one click. The persistent banner said
“Demo — sample data, nothing is saved” and offered Reset demo and Start for
real. Screenshots are in `verification-evidence-6/`.

## Declared claims gate — all listed commands PASS

After `npm ci` in the clean candidate checkout, every exact command in
`.factory/claims.json` ran separately through the demo entry point:

| Claim | Result |
|---|---|
| `demo-isolation` | PASS |
| `attempt-record` | PASS |
| `recap-privacy` | PASS |
| `json-export` | PASS |
| `offline-reload` | PASS |
| `no-tracking` | PASS |
| `owner-access` | PASS |
| `rate-limits` | PASS |
| `plus-price` | PASS |
| `full-delete` | PASS |

The first command completed from a cold Rust build, so the prior fixed-startup
failure is resolved. The P1 above is a cross-check failure: paid capability
copy exists outside the outcomes covered by those ten entries.

## Local quality gates

- `npm ci`: PASS — 60 packages installed; 0 vulnerabilities.
- `npm test`: PASS — TypeScript check, 3/3 Vitest tests, 11/11 Rust tests.
- `cargo fmt --all -- --check`: PASS.
- `cargo clippy --all-targets --all-features -- -D warnings`: PASS.
- `npm run build`: PASS — `dist/` created; entry JS 44.98 KB raw / 14.87 KB
  gzip; lazy identity JS 268.93 KB raw / 67.30 KB gzip; CSS 25.28 KB raw /
  6.22 KB gzip. The mobile hero is 22,518 bytes.
- `BUILD_SHA=74ea740340bd2d47f5287099e0d1eecea57b0cf3 cargo build --release`:
  PASS.
- `npm run test:e2e`: PASS — 12/12, including the disposable full owner
  workflow and complete deletion.
- Docker was unavailable in this verifier image. Source inspection confirms a
  multi-stage build, `rust:1-alpine`, no `.git` dependency, non-root runtime,
  `/data`, `EXPOSE 8080`, and defaulted `ARG BUILD_SHA`.
- No tracked access key, private-key material, environment file, database,
  owner-code file, or upload was found.

## Backend and persistence evidence

- The release binary started in a fresh directory with only `PORT` and process
  `PATH`. It created `./data/board.db` and a 48-byte mode-0600
  `owner-invite.txt`.
- Restarting preserved the owner-code SHA-256 and returned
  `{"build":"74ea740340bd2d47f5287099e0d1eecea57b0cf3","ok":true}` both times.
- Unit/integration checks confirmed signed ownership, real calendar-date
  validation, decoded JPEG/PNG/WebP validation, unknown-status rejection,
  cascade/deletion behavior, SQLite lock recovery, and schema migration.
- The local browser flow created a board, session, problem, learner, partial
  attempt, private note, and photo; exported the complete record; printed a
  recap without the private note; then deleted the complete disposable board.
- The live service completed 100 concurrent `/health` requests with 100 HTTP
  200 responses in 513 ms.
- Live allowance check from one client: 100 concurrent reads produced 43 ×
  200 and 57 × 429; the configured burst is 40 and three tokens replenished
  during the run. Thirty deliberately invalid writes produced 8 × 422 and
  22 × 429. Both limited responses included `Retry-After: 1`.

## Live deployment, privacy, and identity

- `/health` reports the exact candidate SHA. SHA-256 values for live
  `index.html`, entry JS, and CSS exactly match the local production build.
- Non-destructive live Playwright: PASS, 11/11. The destructive whole-board
  deletion check ran only on disposable local data.
- `/`, `/?demo=1`, `/demo`, `/privacy`, `/terms`, `/robots.txt`,
  `/sitemap.xml`, `/manifest.webmanifest`, and `/sw.js` returned 200. The
  designed unknown route returned 404; private `/api/board` returned 401 with
  `WWW-Authenticate: Bearer`.
- The public/sample request log contained only the product origin. It loaded no
  analytics, ads, remote fonts, or third-party runtime scripts. Invalid text
  upload returned “Use a JPEG, PNG, or WebP image under 5 MB”; a valid WebP
  upload then recovered normally.
- Root headers included HSTS, CSP with `frame-ancestors 'none'`,
  `X-Content-Type-Options: nosniff`, `X-Frame-Options: DENY`,
  `Referrer-Policy: same-origin`, and a restrictive permissions policy.
  Hashed assets use `public, max-age=31536000, immutable`.
- Selecting Microsoft sign-in navigated to
  `sociobotcustomers.ciamlogin.com/35c6fe40-0ec0-46b6-98c6-213ad4de6650/...`
  with the configured client ID. The remaining identity resources were
  standard Microsoft authentication asset origins; no alternate identity
  provider appeared.

## Accessibility, PWA, and performance

- `/opt/fleet/lib/verify-url.sh`: PASS — title, `lang=en`, one H1 after render,
  main landmark, no missing alt text, no unnamed buttons, no console errors.
- Axe serious/critical: 0 on landing, sample, legal routes, and the designed
  404 at desktop and 390 px. Visible focus was a 3 px lantern-yellow outline.
- Keyboard skip, route focus restoration, back/reload, and 390 px horizontal
  fit passed. With 200% same-origin stylesheet text sizing, document width
  remained 390 px and core content stayed visible.
- Reduced motion computed to `0.00001s` durations and `scroll-behavior: auto`.
- Service-worker `update()` completed with active `mcb-shell-v4`, no waiting
  worker, and no stale cache. A controlled 390 px demo reloaded offline with
  its banner and problem data.
- Lighthouse 12.8.2 mobile: Performance 100, Accessibility 100, Best Practices
  100, SEO 100; FCP 1.201 s, LCP 1.351 s, TBT 87 ms, CLS 0, transfer 96,981
  bytes.
- No console or page errors occurred in the tested landing, demo, mobile,
  reduced-motion, legal, 404, or offline flows.

## Acceptance decision

The deployment identity, cold claim commands, functional core, privacy checks,
backend boundaries, and performance gates pass. The candidate still fails the
acceptance contract because a paid capability is publicly promised without a
declared outcome test, the sample accepts ambiguous duplicate learner records,
and several mobile targets are below the mandatory minimum.

