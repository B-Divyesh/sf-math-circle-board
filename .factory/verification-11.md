# Independent verification 11 — PASS

**Candidate:** `aff4299a57c9e74fe042ca5f1a58bc0e37a8e2a2`  
**Live URL:** <https://math-circle-board.sociobot.in>  
**Verified:** 2 September 2026

## Result

**PASS.** The deployed site identifies itself as the requested candidate, all
declared claims passed from the clean checkout, the complete local suite
passed, and independent live checks found no release-blocking defect.

## First-read and demo gate

Cold-loading the live landing page gave this plainly readable first screen:

- **What:** “Plan and record small math-circle sessions.”
- **For whom:** volunteer math-circle facilitators working with 6–12 learners.
- **First action:** “Try it with sample data,” followed by “See a filled board.
  Changes stay in this demo.”

That action opens a working, filled sample board with the persistent “Demo —
sample data, nothing is saved” banner, Reset demo, and Start for real actions.
This passes the plain-words and one-click sandbox gates.

## Clean-checkout checks

- `npm ci` — PASS: 60 packages installed; audit reported zero vulnerabilities.
- Every exact command declared in `.factory/claims.json` — PASS, invoked in
  manifest order from the clean candidate checkout: all 15 claim tests,
  including demo isolation/counts, learner limit, attempt/recap/export,
  offline reload, no tracking, owner access, first boot, container contract,
  rate limits, release scope, strategy prompts, and full deletion.
- `npm test` — PASS: TypeScript, copy/claims mapping, Vitest 3/3, and Rust
  12/12.
- `cargo fmt --check` — PASS.
- `cargo clippy --all-targets --all-features -- -D warnings` — PASS.
- `cargo build --release` — PASS.
- `npm run test:identity` — PASS: an isolated runtime with only `PATH` and
  `PORT` returned the candidate build identity from `/health`.
- `npm run build` — PASS; `dist/` produced. The public shell is 43.95 KB raw
  / 14.46 KB gzip JS and 25.77 KB raw / 6.30 KB gzip CSS. The lazy Microsoft
  identity chunk is 67.30 KB gzip, keeping the first-load JS well below budget.
- `npm run test:e2e` — PASS: 26/26 Playwright regressions; the final result is
  `{"status":"passed","failedTests":[]}`.

Docker/Podman is unavailable in this verifier container, so image packaging
was not repeated locally. The Docker recipe is nevertheless covered by the
passing container-runtime claim (non-root user, `PORT`, `/data`, build args and
health identity), and the live service was tested directly.

## Live deployment and product QA

- `GET /health` returned
  `{"build":"aff4299a57c9e74fe042ca5f1a58bc0e37a8e2a2","ok":true}`.
- Fresh `dist/index.html`, `main-CGO_1TiV.css`, and `main-dOwzNLCm.js` matched
  their live counterparts byte-for-byte by SHA-256.
- Desktop and 390 px reduced-motion demo checks found zero console/page errors,
  zero serious/critical axe violations, same-origin-only requests, visible
  solid keyboard focus, and no horizontal navigation clipping. At 390 px the
  app navigation measured `clientWidth: 374`, `scrollWidth: 374`; all four
  destinations are fully visible.
- Keyboard-only smoke test reached the skip link first, moved focus to main,
  and traversed all main navigation controls. Public, demo, privacy and terms
  links returned HTTP 200; an unknown route returns the designed HTTP 404.
- The PWA is controlled by its service worker, accepts `registration.update()`,
  and reloaded `/demo` offline at 390 px with its sample banner and “The coin
  trail” visible (`scrollWidth: 390`).
- Privacy traffic check across landing, demo, and demo navigation observed only
  `https://math-circle-board.sociobot.in`. The anonymous private-board request
  is covered by the passing claim and returns 401 with Bearer authentication;
  the public sign-in path uses only the documented
  `sociobotcustomers.ciamlogin.com` authority.
- Response headers include HSTS, `X-Content-Type-Options: nosniff`, frame
  denial, same-origin referrer policy, restrictive permissions policy, and a
  response-header CSP with `frame-ancestors 'none'`. Hashed JS/CSS has
  `Cache-Control: public, max-age=31536000, immutable`.
- Live rate limiting was independently exercised with fixed forwarded client
  IPs. Of 100 concurrent reads, 48 returned 200 and 52 returned 429. Of 30
  invalid setup writes, 8 returned validation 422 and 22 returned 429. Both
  limited response sets had `Retry-After: 1`, confirming the documented
  40-read burst with refill and 8-write burst.
- Three independent live 390 px DevTools-throttled LCP measurements were
  2.000 s, 2.136 s, and 2.000 s. All are below the 2.5 s budget; the LCP
  element was the landing h1. This closes the LCP failure reported in
  verification 10.

## Defects

None found. There are no release-blocking findings.
