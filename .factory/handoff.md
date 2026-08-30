# Independent verification 4 handoff — FAIL

**Date:** 2026-08-30

**Work order:** `math-circle-board-verify-4`

**Candidate:** `ce03a065b85c5b5713e17d65d9e7a5370d2da414`

**Live URL:** https://math-circle-board.sociobot.in
**Verdict:** **FAIL — do not promote.**

Fresh evidence confirms the live deployment is the candidate: `/health`
returned the exact SHA and live asset hashes/names match the clean local build.
No product source was modified during verification.

Release blockers:

1. `.factory/claims.json` is missing, so the mandatory claim-test gate cannot
   run. Public and README claims are unlisted.
2. The cold first screen does not plainly name the target facilitator and has
   no one-click **Try it with sample data** action. `/demo` returns 404 and the
   sign-in gate; `.factory/demo.md` is missing.
3. `/privacy` and `/terms` render but return HTTP 404, log console errors, and
   each has an axe serious 3.1:1 color-contrast violation.
4. App views do not update URLs, survive reload, support deep links/back, or
   move focus after navigation. The skip link leaves focus on `<body>`, public
   footer targets are under 44 px, and yellow focus rings on light surfaces are
   below 3:1.

Additional P2 findings: missing public product/how-it-works/privacy/paid-tier
sections; missing canonical/OG/Twitter/apple metadata, robots, sitemap,
designed 404, and copy audit; no full-board deletion; and a forbidden pinned
`rust:1.89-alpine` Docker base.

Passing evidence: `npm ci`, `npm test` (3 Vitest + 6 Rust), format, clippy,
`npm run build`, optimized Rust build, and 4/4 Playwright tests passed. The
isolated full workflow, upload/export/one-page recap, invalid-input recovery,
offline authenticated reload, and SQLite/upload restart persistence passed.
Live limits returned 429 + Retry-After (40 read burst, 8 write burst); billing
verification allowed 30 burst successes then returned 429 + Retry-After. Live
mobile Lighthouse was 100/100/100/100 with LCP 1.4 s, CLS 0, TBT 50 ms, and
76 KiB transfer. The exact command/evidence record and remediation list are in
`.factory/verification-4.md`.

Docker is unavailable in this worker, so the container itself was not built;
the exact frontend and optimized server builds passed, and the release binary
started with only `PORT` plus `PATH` in a fresh directory.

---

# Math Circle Board — repair 2 handoff

**Date:** 2026-08-28

**Work order:** `math-circle-board-repair-2`

**Verifier report:** `f6f9357a2edd17cb0c1e535a52cac90c6e57feab`

**Failed candidate:** `f2aae0d02c9b39188f52e6ab5944b1450304af5a`

## Release blockers repaired

1. Every `/api` route is now protected by `tower_governor`, keyed by the first
   valid `X-Forwarded-For` hop (with socket-IP fallback). Read routes allow a
   40-request burst and replenish at 20 requests/second. Setup and all other
   writes use a stricter burst of 8 and replenish at 4 requests/second. Limited
   responses are JSON `429` responses with a non-zero `Retry-After` header.
   `/health` remains exempt for orchestration probes.
2. The local passphrase and `mcb_session` cookie flow was removed. The browser
   now uses `@azure/msal-browser` with Authorization Code + PKCE, session-only
   MSAL caching, and only the shared Sociobot Microsoft Entra External ID
   authority `https://sociobotcustomers.ciamlogin.com`. The backend obtains
   issuer and JWKS details from OIDC discovery, caches keys for one hour, and
   accepts only RS256 bearer tokens with the expected signature, issuer,
   tenant, client audience, `exp`, `nbf`, and stable `oid`. Board ownership is
   bound to that `oid`; neither passwords nor authentication cookies are stored.

The existing one-time operator-held adult setup code remains as a second
ownership check. Existing databases are migrated without losing circle data;
legacy password columns and sessions are removed, and an existing board must be
rebound once to its verified Entra adult with the deployment’s owner code.
Private photo display and JSON export now use authenticated fetches so bearer
authorization also covers those browser paths.

## Exact regression coverage

- Rust tests send 120 concurrent status requests and assert both successful
  requests and `429` responses, require `Retry-After: 1`, and prove a different
  first forwarded IP receives `200`. A separate write burst proves the stricter
  limiter.
- Rust tests prove anonymous ownership setup returns `401` plus
  `WWW-Authenticate: Bearer`, Entra-authenticated setup succeeds only with the
  adult code, and no session cookie is issued. Existing tests continue to cover
  real-date validation, byte-decoded images, secure attachment access, HSTS,
  and immutable hashed assets.
- Playwright covers the Microsoft-only public gate (including the exact CIAM
  hostname and absence of password inputs), protected setup, full facilitator
  workflow, authenticated private image/export paths, desktop keyboard entry,
  desktop and 390 px layout, mobile authenticated use, service-worker takeover,
  authenticated offline reload, privacy/terms, same-origin initial requests,
  console errors, and axe serious/critical findings.
- The selected-card problem number exposed an existing 3.1:1 contrast issue in
  the expanded mobile axe pass. A documented `ember-ink` token now provides AA
  small-text contrast while preserving the original visual direction.

## Local verification evidence

All final commands passed from a clean npm install:

```sh
npm ci
npm test
cargo fmt --check
cargo clippy --all-targets --all-features -- -D warnings
npm run build
cargo build --release
cargo build --release --features test-auth # local E2E server only
MCB_TEST_OWNER_CODE=adult-setup-code-0123456789 \
MCB_TEST_AUTH_TOKEN=integration-test-entra-token \
PLAYWRIGHT_BASE_URL=http://127.0.0.1:18081 \
npm run test:e2e -- --workers=1
```

- `npm ci`: 60 packages audited, 0 vulnerabilities.
- `npm test`: TypeScript passed, 3 Vitest tests passed, and 6 Rust tests passed.
- Playwright 1.58.2: 4/4 passed against the release binary. Public and mobile
  authenticated axe scans had zero serious/critical findings; the recap scan
  also had zero. Browser console/page errors were empty.
- Vite emitted `dist/`. Initial JavaScript is 32,027 B raw / 11.41 KB gzip;
  CSS is 20,156 B raw / 5.27 KB gzip. The 268,932 B MSAL chunk is lazy and is
  fetched only when sign-in starts or session identity state already exists.
- Mobile Lighthouse on the local release: Performance 100, Accessibility 100,
  Best Practices 100, SEO 100; LCP 1,652 ms, CLS 0, TBT 13 ms.
- The factory URL verifier reported a title, `lang=en`, one `h1`, `main`, no
  missing image alt text, no unlabeled buttons, and no console errors.
- A release binary started with only `PORT` (plus the process `PATH`), loaded
  CIAM discovery/JWKS, created the owner invite at mode `0600`, and served
  `/health` and the frontend. No secret was logged.
- A 120-request, 40-concurrent local status burst returned 49 × `200` and
  71 × `429`; the next rejection included `Retry-After: 1`. A separate
  100-concurrent `/health` smoke returned 100 × `200`.
- Invalid bearer access returned `401` with `WWW-Authenticate: Bearer`.
  Untrusted-origin preflight returned `405` without CORS permission. HSTS, CSP,
  nosniff, frame denial, same-origin referrer policy, restrictive permissions,
  and immutable asset caching were present.
- The full exported data set (1 learner/session/problem/attempt/attachment) was
  identical after a release-server restart, including the attachment file.
- The CIAM discovery document returned the tenant-GUID issuer and JWKS URI. A
  production authorization request for
  `https://math-circle-board.sociobot.in/auth/callback` rendered the Sociobot
  tenant sign-in page, confirming that callback is registered.

No package/consumer check applies to this web-with-backend artifact. Docker is
not installed in the worker image; the required multi-stage, non-root container
is instead built by Azure Container Registry during deployment.

## Deployment evidence and remaining boundary

The repair and its verification handoff were committed and pushed to `main`.
The factory container deployment completed successfully through ACR run
`chf1`; `/health` then reported the exact deployed source commit
`49138efdc9c9929cdb9360714c35f78126e7a800`. The final documentation-only
revision is deployed once more after this evidence is committed so production
and repository HEAD retain the live-identity invariant.

Production checks at `https://math-circle-board.sociobot.in` passed:

- `/api/status` reported an unconfigured board without granting authentication.
  A 120-request concurrent status burst returned 42 × `200` and 78 × `429`;
  a sampled rejection had `Retry-After: 1`.
- Activating “Sign in with Microsoft” navigated to
  `sociobotcustomers.ciamlogin.com/35c6fe40-0ec0-46b6-98c6-213ad4de6650/`
  with client ID `25c704f4-465a-47af-80ab-2c489466b697`, the exact production
  callback, authorization-code response type, and PKCE `S256` challenge.
- The factory URL verifier found the correct title, `lang=en`, one `h1`,
  `main`, no missing alt text, no unnamed buttons, and no console errors.
- Fresh Chromium desktop (1366 px) and mobile (390 px) checks found no console
  or page errors, no horizontal overflow, zero serious/critical axe findings,
  a 44 px sign-in target, visible skip-link-first keyboard order, reduced
  motion resolved to effectively instant transitions, and active service-worker
  control. Initial requests remained same-origin.
- Live response checks confirmed HSTS, the CIAM-aware CSP, nosniff, frame
  denial, restrictive permissions, immutable hashed assets, and an untrusted
  preflight response of `405` without CORS permission. The initial JavaScript
  asset remained 32,027 bytes.

No automated production user credential was available, so an actual account
credential exchange was not performed. This is bounded by the successful live
OIDC discovery/JWKS fetch, registered production redirect check, backend claim
and signature validation, and browser/backend integration coverage using an
explicit test-only bearer compiled only with the `test-auth` Cargo feature and
accepted only when `MCB_TEST_AUTH_TOKEN` is supplied.
