# Independent verification — FAIL

**Date:** 2026-08-28  
**Candidate:** `16daa451f39a897929e0094725ec5623b17022a3`  
**Live URL:** https://math-circle-board.sociobot.in  
**Verdict:** **FAIL — do not promote.**

This is a fresh, independent verification against the researched brief and the
factory acceptance contract. Product source was not changed.

## Blocking defects

### P1 — the public, unclaimed deployment has no adult ownership verification

`GET /api/status` on the live URL returned:

```json
{"authenticated":false,"configured":false,"facilitator":null}
```

The public setup form accepts an arbitrary facilitator name, group name, and
8-character passphrase and writes the sole `settings` row. There is no
invitation, email/adult verification, existing-administrator approval, or
deployment secret. Therefore the first anonymous visitor can permanently claim
the board. This violates the brief's required **verified adult-owned groups**
and is unsafe for a product intended to hold minor learners' records.

### P1 — the available TypeScript type check fails with nine errors

Fresh-install command `npx tsc --noEmit` fails in `frontend/src/main.ts` at
lines 55, 118, and 131. The errors are unsafe `EventTarget | null` use and
access to `querySelector`/`value` on `EventTarget`. The repository has no
`typecheck` script, but TypeScript is installed and this is its configured
project check; the quality-gate requirement that type checks available in the
repository pass is not met.

### P1 — authenticated private-record session cookies omit `Secure`

After local setup the exact header was:

```
set-cookie: mcb_session=<redacted>; HttpOnly; SameSite=Strict; Path=/; Max-Age=2592000
```

It lacks `Secure`. The live HTTP endpoint redirects to HTTPS but has no HSTS
header; a non-Secure cookie can still be sent with an HTTP request before that
redirect. Private records for minors require a `Secure` session cookie (and
HSTS should be supplied at the HTTPS boundary).

## Other defects

### P2 — invalid session dates are stored

Authenticated `POST /api/sessions` with
`{"title":"Impossible date","session_date":"2026-99-99","focus":"Boundary"}`
returned `200 {"id":1}`. Server validation only checks that the value is ten
characters long. This allows impossible dates into the session sequence and
recap record.

### P2 — uploads trust the client-declared MIME type, not image bytes

Authenticated upload of the non-image `/etc/hostname` with multipart type
`image/png` returned `200 {"id":1}` and was subsequently served as
`Content-Type: image/png`. The API must decode/inspect JPEG, PNG, or WebP
bytes before accepting an upload.

### P2 — PWA offline reload is not usable

With a registered service worker controlling the live page, setting the
browser offline and reloading produced only:

`The board room is out of reach. The server may still be starting.`

The cached shell then calls `/api/status` and replaces itself with a connection
error. It does not restore the board/drafts or offer the stated offline retry
behaviour for an existing facilitator.

### P2 — hashed static assets have no HTTP cache policy

Live `/assets/index-BGRd9295.js` returns `200` with `Last-Modified` but no
`Cache-Control`, including no immutable long-lived policy. This misses the
factory caching contract for hashed assets. The service worker cache does not
replace correct HTTP cache behaviour for a first or repeat normal navigation.

## What passed

### Clean local build and checks

- Detached clean worktree checked out exactly
  `16daa451f39a897929e0094725ec5623b17022a3`.
- `npm ci`: passed; 59 packages audited, 0 vulnerabilities.
- `npm test`: passed — 3 Vitest tests and 2 Rust tests.
- `npm run build`: passed; Vite emitted `dist/`.
- `cargo build --release`: passed.
- `cargo fmt --check`: passed.
- `cargo clippy --all-targets -- -D warnings`: passed.
- Browser suite against a fresh local backend/data directory:
  `PLAYWRIGHT_BASE_URL=http://127.0.0.1:18081 npm run test:e2e -- --workers=1`
  passed (2/2).

### End-to-end and backend evidence

- Started the release binary with only `PORT=18080`; it created default
  `./data`, reported configuration readiness, and served `/health`.
- Normal flow passed: setup; learner alias; session; problem; attempt with
  strategy/private note; photo upload; recap/export; restart persistence;
  sign-in; and learner deletion. Deleting the learner removed its attempt and
  attachment from `/api/board` and `/api/export`.
- Rejection/recovery checks passed for a short passphrase (`400`), blank alias
  (`400`), duplicate alias case-insensitively (`409`), unknown attempt status
  (`400`), and unauthenticated board/file access (`401`).
- Authenticated attachments returned `Cache-Control: private, max-age=3600`;
  unauthenticated requests were denied.
- 100 concurrent local `/health` requests completed successfully in 737 ms.

### Browser, accessibility, performance, privacy, and deployment evidence

- Live `/health` returns the exact candidate build:
  `{"build":"16daa451f39a897929e0094725ec5623b17022a3","ok":true}`.
- Fresh local build artifacts exactly match live SHA-256 for `index.html`,
  `index-BGRd9295.js`, and `index-83U2kY5Q.css`.
- Desktop and 390 px live smoke: no console/page errors; 390 px
  `scrollWidth === clientWidth === 390`; `lang=en`, one `<h1>`, and `<main>`
  are present. Keyboard Tab reaches the skip link with a visible
  `rgb(243, 201, 105) solid 3px` outline. Reduced motion computes to `0.01s`.
- Axe on the live public/setup screen and the authenticated local recap found
  zero serious or critical violations.
- Browser requests during the public load were same-origin only. Static code
  inspection found no analytics, remote fonts, ads, or runtime CDN scripts;
  the only intentional external runtime endpoint is Sociobot billing and is
  covered by CSP.
- Live CSP, `X-Content-Type-Options: nosniff`, `X-Frame-Options: DENY`,
  `Referrer-Policy: same-origin`, and `Permissions-Policy` were present.
  Cross-origin preflight to `/api/login` returned `405` with no CORS grant.
- Lighthouse mobile run on the live public screen yielded Performance 100,
  Accessibility 100, Best Practices 100, SEO 100; LCP 1351 ms, CLS 0,
  TBT 61 ms. Raw production JS is 27,325 B (9,600 B gzip), CSS 19,856 B
  (5,200 B gzip), and mobile hero image 22,518 B — all within stated budgets.
- Service worker registers and controls a page; it uses `skipWaiting` and
  `clients.claim`. Offline reload behaviour is the P2 failure above.

## Scope notes

Docker is not installed in this verifier image, so the Dockerfile could not be
executed. Its source was inspected: it is multi-stage, non-root, does not use
`.git`, declares `ARG BUILD_SHA=unknown`, and exposes 8080. No library/CLI
consumer test applies to this web-with-backend product.

## Required remediation before re-verification

1. Gate first ownership on a verifiable adult-controlled mechanism and prevent
   anonymous first-visitor takeover.
2. Fix and enforce TypeScript checking in CI (`tsc --noEmit`).
3. Set session cookies `Secure` and provide HSTS at the HTTPS boundary.
4. Validate ISO calendar dates and decode/verify upload bytes.
5. Make authenticated offline reload retain usable cached state/drafts, and set
   immutable cache headers for hashed static assets.
