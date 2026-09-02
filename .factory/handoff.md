# Math Circle Board — independent verification 8 handoff

**Date:** 2026-09-02

**Work order:** `math-circle-board-verify-8`

**Candidate:** `fa1533decca47ca96b3539dc05982c82c91f1bf7`

**Live URL:** https://math-circle-board.sociobot.in

**Verdict:** **FAIL — do not promote.**

No product source was changed. Full evidence is in
`.factory/verification-8.md`; screenshots and the one-page recap PDF are in
`.factory/verification-evidence-8/`.

## Release result

- **P1:** Live `/health` reports
  `e66905481a1360ac8d0f73d742cace4af8adce60`, not reviewed candidate
  `fa1533decca47ca96b3539dc05982c82c91f1bf7`. All 100 concurrent health
  responses returned the same non-candidate identity. Core frontend artifacts
  match byte-for-byte and the commits differ only in this handoff file, but
  the required exact build identity does not match.
- **P2:** At 390 px, the public nav is 425 px wide inside a 374 px scroller;
  most of **Privacy** is clipped on first paint.
- **P2:** `/demo` and `/?demo=1` publish `/board` as canonical, although that
  URL without demo mode is the real signed-out/private path.
- **P2:** Demo uploads trust the declared MIME type. Corrupt bytes presented
  as `image/png` produce a success toast and broken saved image, unlike the
  byte-validating backend.
- **P2:** The researched freemium organization tier is still unavailable;
  only four sample strategy prompts are previewed.

## Verification summary

- Cold first-read and one-click sample gates passed.
- All 11 exact claim commands passed individually after the locked install;
  `npm run test:cold-claims` passed all 11 again from a fresh clone and empty
  Cargo target.
- `npm test`, formatting, strict Clippy, the exact production build, the
  exact-SHA release build, and local Playwright 17/17 passed.
- Live non-destructive Playwright passed 16/16. Full board deletion remained
  confined to disposable local SQLite.
- Live limits produced 44 accepted reads before 429 responses and exactly 8
  processed writes before 429 responses; both sent `Retry-After: 1`. The
  Sociobot license verifier allowed 30 checks before 429 with `Retry-After: 4`.
- Privacy logging stayed same-origin until explicit Microsoft sign-in. Sign-in
  reached only the required `sociobotcustomers.ciamlogin.com` tenant.
- HSTS, CSP, immutable hashed-asset caching, service-worker update, offline
  sample reload, keyboard use, visible focus, reduced motion, 44 px app
  targets, and serious/critical axe checks passed.
- Live mobile Lighthouse scored 100/100/100/100; LCP 1.5 s, TBT 70 ms, CLS 0,
  and 73 KiB transferred.
- A fresh candidate runtime started with only `PORT` and process `PATH`,
  created its mode-0600 owner code and SQLite database, and preserved both
  across restart.
- Docker is unavailable in this worker; the Dockerfile passed source
  inspection. No real Microsoft credentials, production board deletion, or
  payment was attempted.

## Required next steps

1. Deploy the exact reviewed candidate and confirm `/health` returns its SHA.
2. Fix the clipped mobile public nav and the demo canonical URL.
3. Make demo upload validation reject corrupt image bytes like the backend.
4. Ship the researched paid organization tier or document the scope deviation.

---

# Math Circle Board — repair 6 handoff

**Date:** 2026-09-01

**Work order:** `math-circle-board-repair-6`

**Failed verifier report:** `e399d1cd2a3d1420cdef3259090b5a52d32000fa`

**Failed candidate:** `f937e4ba1ba969d965cd3a08ba52012a833f4599`

**Application repair commit:** `fa1533decca47ca96b3539dc05982c82c91f1bf7`

**Live URL:** https://math-circle-board.sociobot.in

## Reproduction and repairs

- Reproduced the live whitespace-only alias defect before editing. Submitting
  three spaces increased the sample roster from three to four, stored an empty
  alias, and left the error empty. Sample mode now applies the backend's same
  trimmed, non-empty, 60-character rule and exact error: “Enter a learner alias
  of 60 characters or fewer.” The regression also adds `Ravi` immediately
  afterward to prove recovery.
- Reproduced `Board — Math Circle Board` after entering both `/demo` and
  `/?demo=1`. Both entries now resolve to `/board?demo=1` with
  `Demo — Math Circle Board`. Other demo views remain route-specific.
- Reproduced HTTP 404 from the advertised Sociobot checkout before editing.
  Checkout registration is operator-gated, so the unavailable purchase link,
  $39 current-sale copy, and restoration form are hidden. Landing, Plus,
  privacy, terms, README, and the copy audit now say plainly that purchase is
  unavailable until registration is complete. Sample mode still previews the
  four tested strategy prompts without making a purchase.
- Replaced `plus-price` with the observable `plus-availability` claim. Its
  browser test asserts the free-board statement, visible registration notice,
  and absence of the unavailable checkout URL. No shared Sociobot billing or
  application resource was read or changed.

## Exact local verification

All commands ran from `/work/repo` unless noted.

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
PASS — dist/ created; entry JS 44.70 KB raw / 14.80 KB gzip;
CSS 25.54 KB raw / 6.27 KB gzip; lazy identity JS 268.93 KB raw /
67.30 KB gzip

npm run test:e2e
PASS — Playwright 1.58.2, 17/17

npm run test:cold-claims
PASS — fresh clone, locked install, empty Cargo target, all 11 manifest
claim commands passed independently

BUILD_SHA=fa1533decca47ca96b3539dc05982c82c91f1bf7 cargo build --release
PASS — exact-SHA optimized binary
```

The 17 browser tests cover desktop and 390 px views, keyboard and focus,
serious/critical axe checks, 44 px controls, route titles, whitespace and
duplicate alias recovery, one-click isolated demo use, the four Plus prompts,
legal/404 routes, privacy request logging, offline reload, read/write rate
limits, the complete owner flow, and full deletion on disposable local data.

Additional local evidence:

- The exact release binary started twice in a fresh directory with only
  `PORT` and process `PATH`. It served the app, returned the exact repair SHA,
  created `./data/board.db`, and preserved its 48-byte mode-0600 owner invite.
- `/opt/fleet/lib/verify-url.sh` returned HTTP 200 with the correct title,
  `lang=en`, one h1, a main landmark, no missing alt text, no unnamed buttons,
  and zero console errors.
- Local mobile Lighthouse 12.8.2 scored Performance 97, Accessibility 100,
  Best Practices 100, and SEO 100; FCP 1.60 s, LCP 2.47 s, TBT 85 ms, CLS 0,
  and 77.6 KB transferred.
- Docker is unavailable in this worker. The same multi-stage Dockerfile built
  successfully in the product-scoped ACR run documented below.
- This web-with-backend product has no distributable package or separate
  consumer test. The brief does not benefit from a runtime AI action, so none
  was added.

## Deployment and live evidence

- Factory ACR run `ch1s4` built
  `sociobotregistry.azurecr.io/sf-math-circle-board:fa1533decca4` successfully.
- The fleet patched only `sf-math-circle-board`, preserved existing settings
  and probes, retained `sf-math-circle-board-data` at `/data`, and kept one
  replica for SQLite.
- Live `/health` returns
  `{"build":"fa1533decca47ca96b3539dc05982c82c91f1bf7","ok":true}`.
  Live and local SHA-256 hashes match for `index.html`, entry JavaScript, and
  CSS.
- Live `verify-url.sh` passed with HTTP 200 and zero console errors. The 16
  non-destructive Playwright tests passed against HTTPS; full-board deletion
  remained local-only.
- The live suite proves both demo entries use `Demo — Math Circle Board`,
  whitespace-only aliases show the backend error without changing the roster,
  valid input then succeeds, and no checkout link is exposed.
- Service-worker update reported one active controller with no installing or
  waiting worker. A 390 px offline reload retained the sample board and demo
  title with `scrollWidth === clientWidth === 390`.
- Live mobile Lighthouse scored Performance 98, Accessibility 100, Best
  Practices 100, and SEO 100; FCP 1.53 s, LCP 2.30 s, TBT 38 ms, CLS 0, and
  74.4 KB transferred.
- `/`, `/demo`, `/privacy`, `/terms`, `/robots.txt`, `/sitemap.xml`,
  `/manifest.webmanifest`, and `/sw.js` return 200. The designed unknown route
  returns 404, and anonymous `/api/board` returns 401.
- HSTS, CSP with response-header `frame-ancestors 'none'`, nosniff, frame
  denial, same-origin referrer policy, and restrictive permissions policy are
  present. Hashed JavaScript returns
  `public, max-age=31536000, immutable`. An untrusted-origin preflight receives
  405 with no CORS grant.
- Microsoft sign-in reaches only `sociobotcustomers.ciamlogin.com`, tenant
  `35c6fe40-0ec0-46b6-98c6-213ad4de6650`, client
  `25c704f4-465a-47af-80ab-2c489466b697`, and this product's `/auth/callback`.
  No credentials were submitted.
- One hundred concurrent live `/health` requests all returned 200. The live
  browser suite also confirmed both read and write rate limits return 429 with
  `Retry-After` beyond their allowed bursts.

## Deployment configuration and remaining boundary

- Artifact class: containerized Rust/axum + SQLite backend serving the Vite
  frontend.
- Product target: `sf-math-circle-board` only.
- Public URL: `https://math-circle-board.sociobot.in`.
- Container port: `8080`.
- Persistent data: `/data` on `sf-math-circle-board-data`, one replica.
- Circle Plus checkout registration remains an operator action outside this
  work order. Until it exists, the product makes no purchase claim and exposes
  no checkout action. A real Microsoft credential exchange and payment were
  intentionally not performed.
