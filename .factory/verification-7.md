# Independent verification 7 — FAIL

**Date:** 2026-09-01

**Work order:** `math-circle-board-verify-7`

**Candidate:** `f937e4ba1ba969d965cd3a08ba52012a833f4599`

**Live URL:** https://math-circle-board.sociobot.in
**Verdict:** **FAIL — do not promote.**

This was a fresh independent product-QA run against the researched brief and
factory acceptance contract. No product source was changed.

## Release-blocking finding

### P1 — the advertised Circle Plus purchase cannot be opened

The landing page shows **Buy Circle Plus through Sociobot** and links to the
documented product checkout URL. A fresh direct request to that live URL
returned:

```text
GET https://api.sociobot.in/api/v1/products/math-circle-board/checkout
HTTP 404
{"error":"enabled factory product","status":404}
```

The invalid-license check route itself is available and returned HTTP 200 with
`{"valid":false,"reason":"invalid","expires_at":null}`. The failure is
specific to opening the registered checkout. A visitor therefore cannot buy
the advertised `$39` one-time option.

This also exposes a claim-test gap. `@claim:plus-price` checks only that the
button has the expected URL. It does not open that URL and confirm a working
checkout outcome. The claims contract requires the observable result rather
than the presence of a link. The claim test passes while the live claim is
false, so this is release-blocking independently of the otherwise passing
suite.

## Other findings

### P2 — sample mode accepts a blank learner alias made of spaces

On a fresh live `/learners?demo=1` board, entering three spaces and choosing
**Add learner** changed the roster from three to four entries. The new stored
record had `alias: ""`, and the form showed no error. The real backend trims
and rejects the same value, so sample and real input behavior do not agree.

The user can recover: adding `Ravi` works, and **Reset demo** restores the
original three learners. The invalid blank record should be rejected before it
is stored, with the same useful message as the backend.

### P2 — the demo entry does not have a demo-specific document title

Both `/demo` and `/?demo=1` enter `/board?demo=1` with the title
`Board — Math Circle Board`. The site-structure contract requires a demo route
title such as `Demo — Math Circle Board`. The other checked routes have
specific titles, one `<h1>`, and one `<main>`.

## Mandatory first checks

### First-read and one-click sample — PASS

A cold desktop load returned HTTP 200 and showed, in the first viewport:

- audience: “For volunteer math circle facilitators”;
- job: “Plan and record small math-circle sessions” and the 6–12 learner
  context;
- first action: **Try it with sample data**, with the explanation that it
  opens a filled board and changes stay in the demo.

One click opened a populated working board. The persistent banner says
“Demo — sample data, nothing is saved” and provides **Reset demo** and
**Start for real**. Evidence:
`.factory/verification-evidence-7/live-first-read-desktop.png` and
`.factory/verification-evidence-7/live-demo-mobile.png`.

### Declared claims — all commands pass, but one assertion is insufficient

The repository was initially clean at the requested commit. The literal claim
commands cannot start before dependencies are installed (`vite: not found`),
which is expected for a clone without `node_modules`. After the documented
locked install, every exact manifest command passed individually. The
repository's `npm run test:cold-claims` then independently cloned the commit,
installed dependencies, used an empty Cargo target, and passed all 11 commands:

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
| `plus-price` | PASS assertion, **FAIL live outcome** |
| `plus-strategy-palette` | PASS |
| `full-delete` | PASS |

The paid-link problem above means the claims gate does not support acceptance
despite all command exit codes being zero.

## Local quality gates

All checks used candidate `f937e4b` after `npm ci`:

```text
npm ci
PASS — 60 packages installed; 0 reported vulnerabilities

npm test
PASS — TypeScript; Vitest 3/3; Rust 11/11

cargo fmt --all -- --check
PASS

cargo clippy --all-targets --all-features -- -D warnings
PASS

npm run build
PASS — dist/ created

BUILD_SHA=f937e4ba1ba969d965cd3a08ba52012a833f4599 cargo build --release
PASS

npm run test:e2e
PASS — Playwright 1.58.2, 15/15

npm run test:cold-claims
PASS — all 11 exact claim commands from a fresh clone and empty Rust target
```

The production output was 45.03 KB raw / 14.86 KB gzip entry JavaScript,
268.93 KB raw / 67.30 KB gzip lazy Microsoft identity JavaScript, and 25.54 KB
raw / 6.27 KB gzip CSS. The initial page does not load the identity chunk.
The mobile hero is 22.52 KB, and no font file is downloaded.

Docker is unavailable in this worker. Source inspection confirms a multi-stage
build, `rust:1-alpine`, `ARG BUILD_SHA`, no `.git` dependency, a non-root
runtime user, port 8080, and `/data` storage.

## End-to-end behavior and recovery

The 15-test local browser run confirms the normal facilitator flow on a fresh
SQLite directory: create the adult-owned board, add a session and open
problem, add a learner, save a partial attempt/status/private note, add a valid
WebP, export, open the recap, confirm the private note is absent, and delete
the complete board.

Boundary and invalid-value checks confirm backend rejection of invalid dates,
non-image bytes presented as an image, duplicate aliases, anonymous private
API access, and limited request groups. A live duplicate-alias check produced
the useful error and preserved one `Ada`; the whitespace-only sample case in
the P2 finding is the remaining inconsistency. After that invalid case, a
valid learner could still be added and reset restored the sample.

A release binary started in a fresh working directory with only `PORT` and a
process `PATH`. It created `./data/board.db` and a 48-byte mode-0600 adult
setup file. Restarting with the same directory preserved that file. Both
starts returned the exact candidate SHA from `/health`.

## Live deployment, API, and identity

- `/health` returned
  `{"build":"f937e4ba1ba969d965cd3a08ba52012a833f4599","ok":true}`.
- Fresh local and live SHA-256 values matched for `index.html`, entry
  JavaScript, CSS, and `sw.js`.
- The non-destructive live Playwright run passed 14/14. Full deletion remained
  confined to the disposable local SQLite directory.
- A 100-request live `/health` concurrency check returned 100 HTTP 200 results,
  all with the candidate build identity.
- A separate 100-request read group produced 44 HTTP 200 and 56 HTTP 429
  responses. A 30-request write group produced 8 validation responses and 22
  HTTP 429 responses. Both limited groups returned `Retry-After: 1`.
- A 100-request license-verification group at the Sociobot product route
  produced 30 HTTP 200 and 70 HTTP 429 responses, with `Retry-After: 4`.
- Anonymous `/api/board` returned HTTP 401 and a Bearer challenge.
- Choosing **Sign in with Microsoft** reached
  `sociobotcustomers.ciamlogin.com/35c6fe40-0ec0-46b6-98c6-213ad4de6650`
  with client ID `25c704f4-465a-47af-80ab-2c489466b697` and the product's
  `/auth/callback`. No alternative sign-in provider is present.

No real Microsoft credentials were submitted. The paid flow could not be
completed because its live checkout entry returned 404.

## Privacy, headers, caching, and PWA

The landing and complete sample navigation request log contained only
`https://math-circle-board.sociobot.in`. There were no analytics, advertising,
remote fonts, or third-party runtime scripts, and no console or page errors.
External Microsoft origins appeared only after the explicit sign-in action.

Live responses include HSTS, `nosniff`, frame denial, same-origin referrer
policy, a restrictive permissions policy, and a CSP with response-header
`frame-ancestors 'none'`. The entry hashed JavaScript returns
`Cache-Control: public, max-age=31536000, immutable`.

The active service worker is `/sw.js`; an explicit update check completed with
no waiting or installing worker. After control was established, a 390 px
sample reload while offline retained the sample banner and current problem.

## Accessibility, responsive behavior, and performance

- The worker `verify-url.sh` check passed: title, `lang=en`, one `<h1>`, a
  `<main>`, image alternatives, named buttons, and no console errors.
- Playwright axe found no serious or critical results on the landing page,
  sample board, privacy, terms, or designed 404.
- Keyboard-only checks confirm the visible skip link, focus transfer to
  `<main>`, route focus on the new `<h1>`, and back-button focus restoration.
  The skip-link focus indicator is a 3 px designed outline.
- At 390 px, all checked controls are at least 44 by 44 CSS pixels and the
  document width remains 390 px. The inspected sample screenshot has no
  clipped primary controls.
- With reduced motion requested, scroll behavior is `auto` and transition and
  animation durations reduce to `0.00001s`.
- Mobile Lighthouse: Performance 100, Accessibility 100, Best Practices 100,
  SEO 100; FCP 1.2 s, LCP 1.4 s, TBT 20 ms, CLS 0, 95 KiB transferred.

## Route and product-shape checks

`/`, `/demo`, `/privacy`, `/terms`, `/robots.txt`, `/sitemap.xml`,
`/manifest.webmanifest`, and `/sw.js` return HTTP 200. The designed unknown
route returns HTTP 404 with a way home. Same-origin navigation links returned
their expected statuses. The external checkout is the one dead product link.

The repository includes the required README, MIT license, privacy and terms
pages, demo documentation, visual thesis, original-asset provenance, and
handoff. The product-specific lantern-room visual system matches the recorded
palette, typography, spacing, motion, and single-theme rationale. The brief
does not require an AI action; deterministic recap, export, and sample mode
cover the small-group job without an additional model dependency.

## Required next steps

1. Register or enable `math-circle-board` in the Sociobot billing engine so
   the published checkout opens, then change `@claim:plus-price` to confirm the
   live/test checkout outcome rather than only its `href`.
2. Apply the backend's trimmed non-empty alias validation in sample mode and
   add a whitespace-only recovery regression.
3. Give the sample entry a demo-specific title, then rerun route metadata,
   claims, full browser, and live paid-link checks.
