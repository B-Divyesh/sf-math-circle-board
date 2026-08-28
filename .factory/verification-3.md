# Independent verification 3 — FAIL

**Date:** 2026-08-28  
**Candidate:** `f2aae0d02c9b39188f52e6ab5944b1450304af5a`  
**Live URL:** https://math-circle-board.sociobot.in  
**Verdict:** **FAIL — do not promote.**

This was a fresh verification from a clean candidate checkout. No
product source was changed. The live deployment is conclusively this candidate:
`GET /health` returned
`{"build":"f2aae0d02c9b39188f52e6ab5944b1450304af5a","ok":true}`, and the
SHA-256 values of live and freshly built `index.html`, JS, CSS, and `sw.js`
were identical.

## Release-blocking defects

### P1 — no API rate limiting or `Retry-After`

The mandatory backend contract requires every server-side endpoint (apart from
health) to rate limit per forwarded client IP and, when exceeded, return `429`
with `Retry-After`.

Fresh live evidence: a burst of **120** `GET /api/status` requests, at up to
40 concurrent requests, returned **120 × 200**, **0 × 429**, and no
`Retry-After` header. Thus no threshold was observed through 120 requests.
Source corroborates this: `src/main.rs` constructs the complete `/api` router
without a governor/rate-limit layer, and `Cargo.toml` has no such dependency.
This leaves both public setup/login and authenticated write routes without the
required abuse control.

### P1 — the required Entra External ID sign-in is absent

The acceptance instruction requires a product that requires sign-in to use
only the Sociobot Microsoft Entra External ID tenant
`https://sociobotcustomers.ciamlogin.com`. This app requires the facilitator
to use its local passphrase sign-in form (`/api/login`) and creates its own
`mcb_session` cookie. There is no Entra authority, OIDC client, or Microsoft
identity flow in source or live network activity (`rg` found no `ciamlogin` or
Microsoft identity reference; live initial load made only same-origin requests).
The local passphrase system is not the requested Entra identity integration.

## Checks that passed

### Clean install, quality gates, and production build

```sh
npm ci
npm test
cargo fmt --check
cargo clippy --all-targets -- -D warnings
npm run build
cargo build --release
```

All passed. `npm test` ran TypeScript checking, 3 Vitest tests, and 4 Rust
tests. The production output is `dist/` with 28,282 B raw / 9,891 B gzip JS,
19,856 B raw / 5,201 B gzip CSS, and a 22,518 B mobile hero image — within the
stated budgets. Docker and Lighthouse executables were not installed in this
verification image, so no container execution or Lighthouse score is claimed.

### Core workflow, invalid input, and persistence

Against a fresh release binary and SQLite data directory, Playwright 1.58.2
passed all 3 repository tests:

```sh
MCB_TEST_OWNER_CODE=adult-setup-code-0123456789 \
PLAYWRIGHT_BASE_URL=http://127.0.0.1:18081 \
npm run test:e2e -- --workers=1
```

This exercised adult-code setup, desktop session → problem → learner → partial
attempt/strategy/private note → private WebP upload → export/print recap;
mobile sign-in; 390 px authenticated offline reload; keyboard skip link; and
authenticated-recap axe. The subsequent direct API smoke confirmed export held
1 learner, 1 session, 1 problem, 1 attempt, and 1 attachment. The same
authenticated data remained available after a server restart. A local
100-request concurrent `/health` smoke returned 100 × 200.

Recovery and privacy checks returned the expected results: invalid calendar
date 400, blank learner alias 400, case-insensitive duplicate alias 409,
unknown attempt status 400, unauthenticated board/file requests 401, and a
non-image uploaded as `image/png` 400. Authenticated attachment delivery was
`image/webp` with `Cache-Control: private, max-age=3600`. The sign-in response
set `HttpOnly; Secure; SameSite=Strict`.

With only `PORT=18080` (apart from the shell PATH) the release binary started,
served `/health`, created its generated owner invite at mode `0600`, and logged
configuration state without logging the secret.

### Live deployment, privacy, accessibility, PWA, and response policy

- Live public board was unconfigured; a deliberately wrong adult setup code
  returned 403 and `/api/status` remained unclaimed.
- Desktop and 390 px public screens had no console/page errors, one `h1`, one
  `main`, `lang=en`, no horizontal overflow at 390 px, and a working service
  worker controller. The first keyboard Tab reached “Skip to main content”
  with a visible `rgb(243, 201, 105) solid 3px` focus outline. Reduced-motion
  transition/animation duration computed to `0.01ms`.
- Axe on both desktop and 390 px live public screen reported zero serious or
  critical findings; the authenticated local recap did likewise. Repository
  e2e also confirmed offline reload from its versioned service worker cache.
  The shipped worker has a versioned cache (`mcb-v2`) and uses `skipWaiting`
  plus `clients.claim` for update takeover.
- Browser request capture during live initial load was same-origin only. Source
  and network inspection found no analytics, ads, remote fonts, or runtime CDN
  scripts. The only intentional external endpoint is the Sociobot billing API.
- Live security headers include HSTS (`max-age=63072000; includeSubDomains`),
  CSP, `nosniff`, `DENY` framing, same-origin referrer policy, and a restrictive
  Permissions-Policy. An untrusted-origin preflight to `/api/login` returned
  405 without a CORS grant. Hashed live JS uses
  `Cache-Control: public, max-age=31536000, immutable`.

## Required remediation

1. Add a per-client-IP rate limiter to **all** `/api` endpoints, using the
   first `X-Forwarded-For` hop behind ingress; verify a burst yields `429` and
   a meaningful `Retry-After` header. Use stricter limits for setup, login,
   writes, and uploads.
2. Replace the local passphrase identity flow with the required Sociobot
   Microsoft Entra External ID flow at `sociobotcustomers.ciamlogin.com`, and
   verify no other sign-in authority is used.
