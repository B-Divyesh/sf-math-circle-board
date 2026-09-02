# Independent verification 10 — FAIL

**Candidate:** `eeeeeef2519a0a98aa49611dbf8774ba2d69caba`

**Live URL:** <https://math-circle-board.sociobot.in>

**Verified:** 2 September 2026

## Result

**FAIL.** The core product works, all eleven declared claim commands pass, and
the live deployment is the candidate. Release acceptance still fails because
public claims are missing from `.factory/claims.json` and actual-throttling
mobile Lighthouse misses the explicit LCP budget in three of three runs. The
390 px app navigation also clips part of its final destination.

## First-read gate — PASS

On a cold load, the first screen says this board helps volunteer math-circle
facilitators plan and record sessions for 6–12 learners. It names problem
sequencing, partial attempts, private notes, and printable recaps. The first
action is **Try it with sample data**, with the adjacent explanation “See a
filled board. Changes stay in this demo.” One click opens a working sample with
the persistent “Demo — sample data, nothing is saved” banner.

Evidence: [desktop first read](verification-evidence-10/live-first-read-desktop.png)
and [390 px demo](verification-evidence-10/live-demo-mobile.png).

## Candidate and clean-checkout gates

- The checkout began clean at the requested candidate.
- `npm ci` — PASS; 60 packages installed and zero vulnerabilities reported.
- Every exact command in `.factory/claims.json` — PASS, run separately before
  broader QA. Each command reported one passing Playwright test:
  `demo-isolation`, `attempt-record`, `recap-privacy`, `json-export`,
  `offline-reload`, `no-tracking`, `owner-access`, `rate-limits`,
  `release-scope`, `strategy-palette`, and `full-delete`.
- `npm test` — PASS: TypeScript, copy contract, Vitest 3/3, Rust 11/11.
- `npm run build` — PASS and produced `dist/`. Output was 81.68 KB gzip JS
  across the app and lazy identity chunks, plus 6.28 KB gzip CSS. The cold
  public page requested only the 43.60 KB raw app chunk.
- `cargo fmt --check` — PASS.
- `cargo clippy --all-targets --all-features -- -D warnings` — PASS.
- `cargo build --release` — PASS from a cold target in 7m 01s.
- `npm run test:identity` — PASS; an environment containing only `PATH` and
  `PORT` returned the candidate SHA from `/health`.
- `npm run test:e2e` — PASS, 19/19 local Playwright tests.
- Docker/Podman is not installed in this verifier container, so the Dockerfile
  could not be packaged locally. The release binary, identity path, and live
  container were checked independently.

## Live end-to-end and platform checks

- `/health` returned
  `{"build":"eeeeeef2519a0a98aa49611dbf8774ba2d69caba","ok":true}`.
  Local and live SHA-256 hashes matched byte-for-byte for `index.html`, CSS,
  the app JS, and the lazy Microsoft identity JS.
- The worker `verify-url.sh` passed: HTTP 200, correct title, `lang="en"`, one
  h1, a main landmark, complete alt attributes, and no console/page errors.
  See [verify.json](verification-evidence-10/verify.json).
- 18/18 non-destructive repository Playwright tests passed directly against
  production. They covered the demo, duplicate and blank aliases, corrupt
  image recovery, attempt save/reload, private-note recap exclusion, JSON
  export, offline reload, same-origin requests, anonymous API rejection,
  limits, legal/404 routes, keyboard focus, 390 px targets, and route history.
  Full-board deletion passed only against the isolated local SQLite database.
- The sample workflow produced a one-page A4 recap and excluded its private
  facilitator notes. See [recap PDF](verification-evidence-10/live-demo-recap.pdf).
- `/`, `/demo`, `/privacy`, `/terms`, and the designed HTTP 404 had no serious
  or critical axe findings, no horizontal page overflow, one h1, and no
  console/page errors. Reduced motion cut the maximum transition/animation to
  `0.00001s`; visible controls met 44 px after excluding the intentionally
  invisible file input.
- The service worker accepted `registration.update()`, controlled `/demo`,
  and reloaded the complete sample offline at 390 px.
- Every same-origin link discovered from the landing, demo, privacy, and terms
  routes returned 200. The unknown route returned the designed HTTP 404.
- Public and demo flows made only same-origin requests. The sign-in action
  redirected only to `https://sociobotcustomers.ciamlogin.com/.../oauth2`.
- Response headers include HSTS, `nosniff`, frame denial, same-origin referrer
  policy, restrictive permissions policy, and a response-header CSP containing
  `frame-ancestors 'none'`. Hashed JS/CSS returns one-year immutable caching.
  Evidence: [root headers](verification-evidence-10/root-headers.txt) and
  [asset headers](verification-evidence-10/asset-headers.txt).
- A single forwarded client sent 100 concurrent read requests: 43 returned
  200 and 57 returned 429. A different client sent 30 invalid writes: 8
  reached validation (422) and 22 returned 429. Both limited responses had
  `Retry-After: 1`. This confirms the documented 40-read burst with refill and
  the 8-write burst.

## Findings

### High — public claims are absent from the required claims manifest

The claims contract says any landing-page or README claim without a manifest
entry fails verification. All listed entries pass, but the cross-check found
unlisted or incompletely asserted claims:

- The landing and README say the board is for **6–12 learners**; no claim test
  asserts the quantitative range.
- The README promises that reset restores exactly two sessions, three
  learners, four problems, and four attempts. Tests assert the session and
  learner counts, but not all four published quantities.
- The README says the server needs no environment variables, writes SQLite and
  uploads to the documented persistence paths, creates a 48-character setup
  code on first boot, and uses Microsoft Entra. These are not manifest claims.
- The README says the container runs non-root, serves `PORT`, persists under
  `/data`, and returns the build SHA from `/health`. These are tested elsewhere
  but absent from `.factory/claims.json`, which is still a contract violation.

This finding alone makes the candidate non-releasable under the work order.
Either remove the claims or add exact manifest entries whose tagged tests
assert the complete observable statements.

### High — throttled mobile LCP exceeds the 2.5 second budget

Three Lighthouse 12.8.2 mobile runs using DevTools throttling for a Moto G
class profile measured LCP at **2.696 s, 2.532 s, and 2.663 s** (median
**2.663 s**). The budget is `< 2.5 s`. The h1 is the LCP element; 97% of its
time is render delay while the JavaScript/API dependency chain completes.
Performance scores were 92, 96, and 93, CLS was 0, and transfer was about
73 KB, so this is an LCP timing defect rather than a bundle-size defect.

For transparency, three standard Lighthouse simulated-throttling runs scored
100, 98, and 100 with LCP between 1.350 and 1.455 s and accessibility 100.
Those do not cancel the three reproducible actual-throttling budget misses.
Raw reports are in `verification-evidence-10/lighthouse-mobile*.json`.

### Medium — the primary app navigation clips at 390 px

At 390 px, the demo app navigation has `clientWidth: 374` and
`scrollWidth: 410`. The Settings link ends at x=417.92 while the navigation
ends at x=382, so about 36 px is hidden on first paint. Horizontal touch or
keyboard scrolling can reach it, but all four primary destinations are not
fully visible. The clipping is visible in the saved mobile screenshot.

## Scope

The shipped release deliberately omits the researched future paid organization
tier. It truthfully has no checkout or paid-storage claim, and
`.factory/scope-deviation.md` documents that decision. The smallest useful
adult-owned circle workflow is otherwise present. No AI feature is necessary
for this job; structured export already covers the obvious portability need.
