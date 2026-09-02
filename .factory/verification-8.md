# Independent verification 8 — FAIL

**Date:** 2026-09-02

**Work order:** `math-circle-board-verify-8`

**Candidate:** `fa1533decca47ca96b3539dc05982c82c91f1bf7`

**Live URL:** https://math-circle-board.sociobot.in

**Verdict:** **FAIL — do not promote.**

This was a fresh independent product-QA run against the researched brief and
factory acceptance contract. No product source was changed.

## Release-blocking finding

### P1 — the live backend does not identify as the candidate

The required live build-identity check fails:

```text
GET https://math-circle-board.sociobot.in/health
HTTP 200
{"build":"e66905481a1360ac8d0f73d742cace4af8adce60","ok":true}
```

The candidate is `fa1533decca47ca96b3539dc05982c82c91f1bf7`. A burst of 100
live health requests returned 100 HTTP 200 responses, all identifying as
`e66905481a1360ac8d0f73d742cace4af8adce60`. By contrast, the locally built
candidate release binary returned `fa1533decca47ca96b3539dc05982c82c91f1bf7`
on both fresh-runtime starts.

The live `index.html`, entry JavaScript, identity JavaScript, and CSS are
byte-for-byte equal to the candidate build. Git also shows that the deployed
commit is a descendant whose tree differs from the candidate only in
`.factory/handoff.md`. That narrows this to build/deployment identity rather
than a reproduced frontend regression, but it does not satisfy the explicit
requirement that the deployment match the candidate under review.

## Other findings

### P2 — the 390 px public navigation clips the Privacy link

On a cold 390 px page, the public navigation has a 374 px client width and a
425 px scroll width. The **Privacy** link spans x=359.3–433.3, so only its
first 30.7 px are initially visible. The screenshot visibly shows only
“Pr”. The row can be horizontally scrolled and keyboard focus can reveal the
link, but the initial mobile header hides a primary legal destination without
an affordance. Evidence: `verification-evidence-8/live-cold-mobile.png`.

### P2 — the public demo publishes the wrong canonical URL

Both `/demo` and `/?demo=1` enter the correct sample and use the correct
`Demo — Math Circle Board` title, but their canonical link is
`https://math-circle-board.sociobot.in/board`. Opening `/board` without the
demo query is the real signed-out/private path, not the sample URL listed in
the sitemap and README. The demo should canonicalize to `/demo` or another
stable demo URL.

### P2 — sample upload validation accepts corrupt image bytes

The live demo correctly rejected a `text/plain` upload and recovered with a
valid sample upload. However, a corrupt eight-byte file presented as
`image/png` was accepted with “Photo added privately,” stored under the demo
namespace, and rendered as a broken image. The real backend inspects image
bytes and rejects the same class of input. Demo validation checks only the
declared MIME type and size, so this recovery path does not match the real
product. Evidence: `verification-evidence-8/live-demo-mobile.png`.

### P2 — the researched paid tier remains unavailable

The site now honestly says Circle Plus is not for sale and removes the dead
checkout link, which fixes the false purchase claim from verification 7.
However, the researched contract specifies a freemium product with paid
organization controls and storage. The current preview offers four strategy
prompt buttons, and there is no purchasable organization tier. This does not
block the useful free board, but it remains an explicit scope gap.

## Mandatory first checks

### First-read and one-click sample — PASS

A cold desktop and 390 px load answered the three release-gate questions in
plain words:

- what: “Plan and record small math-circle sessions” and sequence problems,
  record attempts/private notes, and print a recap;
- for whom: volunteer facilitators working with 6–12 learners;
- first click: **Try it with sample data**, with “See a filled board. Changes
  stay in this demo.” beside it.

One click opened the populated board. The persistent banner says
“Demo — sample data, nothing is saved” and offers **Reset demo** and
**Start for real**. Evidence:
`verification-evidence-8/live-cold-desktop.png`,
`verification-evidence-8/live-cold-mobile.png`, and
`verification-evidence-8/live-demo-mobile.png`.

### Declared claims — PASS

`.factory/claims.json` exists. After `npm ci`, every listed command was run
individually from the candidate checkout before other QA. All passed:

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
| `plus-availability` | PASS |
| `plus-strategy-palette` | PASS |
| `full-delete` | PASS |

`npm run test:cold-claims` independently cloned the committed candidate,
installed from the lockfile, used an empty Cargo target, and passed all 11
commands again. The live/README claim cross-check found no unsupported active
purchase claim; the paid tier is explicitly described as unavailable.

## Local quality gates

All checks used candidate `fa1533d`:

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

BUILD_SHA=fa1533decca47ca96b3539dc05982c82c91f1bf7 cargo build --release
PASS

npm run test:e2e
PASS — Playwright 1.58.2, 17/17

npm run test:cold-claims
PASS — all 11 exact manifest commands from a fresh clone
```

Production output is 44.70 KB raw / 14.70 KB gzip entry JavaScript,
268.93 KB raw / 67.04 KB gzip lazy identity JavaScript, and 25.54 KB raw /
6.29 KB gzip CSS. The cold page does not load the identity chunk. The mobile
hero is 22.52 KB and no font file is downloaded. Docker is unavailable in
this worker; source inspection confirms a multi-stage build, `rust:1-alpine`,
`ARG BUILD_SHA`, no `.git` dependency, a non-root runtime user, port 8080, and
`/data` storage.

## End-to-end behavior and recovery

The 17-test local browser run completed the disposable adult-owner flow:
create a board, add a session/problem/learner, save partial thinking, status,
strategies and a private note, upload a valid WebP, export the complete record,
open the recap, exclude the private note, and delete the complete board.

The 16 applicable non-destructive tests also passed against the live URL.
They cover duplicate and whitespace-only aliases with recovery, demo reset and
storage isolation, Plus prompts, photo/text attempt records, export, printable
recap privacy, routes, mobile targets, keyboard focus, offline reload, and
serious/critical axe findings. Full deletion was intentionally kept on the
disposable local database.

A generated live sample recap is one A4 PDF page and contains the edited
learner thinking while the DOM assertion confirms the private note is absent.
Evidence: `verification-evidence-8/live-session-recap.pdf`.

## Backend, persistence, rate limits, and identity

- A fresh candidate release runtime started with only `PORT` and process
  `PATH`, generated `./data/board.db` and a 48-byte mode-0600 adult owner code,
  and preserved that code across restart.
- One hundred concurrent local health requests and 100 concurrent live health
  requests all returned HTTP 200. The local identity was the candidate; the
  live identity was the P1 mismatch above.
- Live read burst: 44 HTTP 200 and 56 HTTP 429 responses from 100 concurrent
  requests. The documented allowance is 40 burst tokens; four replenished
  during the burst. Limited responses included `Retry-After: 1`.
- Live write burst: 8 validation responses and 22 HTTP 429 responses from 30
  concurrent requests. Limited responses included `Retry-After: 1`.
- Sociobot product verification: 30 HTTP 200 invalid-license verdicts and 30
  HTTP 429 responses from 60 concurrent checks, with `Retry-After: 4`.
- Anonymous `/api/board` returned HTTP 401 with a Bearer challenge.
- Explicit Microsoft sign-in reached only the required authority
  `sociobotcustomers.ciamlogin.com/35c6fe40-0ec0-46b6-98c6-213ad4de6650`,
  client ID `25c704f4-465a-47af-80ab-2c489466b697`, and the product's
  `/auth/callback`. No alternate identity provider is offered.

## Privacy, security headers, caching, and PWA

The cold landing and full sample flow made only same-origin requests. No
analytics, ads, remote fonts, third-party runtime scripts, console errors, or
page errors were observed. Microsoft-owned origins appeared only after the
explicit sign-in action.

Live documents and APIs include HSTS, `nosniff`, frame denial, same-origin
referrer policy, a restrictive permissions policy, and a response-header CSP
with `frame-ancestors 'none'`. Hashed JS and CSS return
`Cache-Control: public, max-age=31536000, immutable`.

The service worker registered and activated at `/sw.js`; an explicit
`update()` completed with no waiting or installing worker. Its only cache was
`mcb-shell-v4`. After control was established, the 390 px demo reloaded
offline with its demo title, banner, and sample record intact.

## Accessibility, responsive behavior, and performance

- Each checked route has `lang=en`, one `<h1>`, one `<main>`, a specific title,
  and no image missing alt text. `/privacy` and `/terms` return 200; the
  designed unknown route returns 404 with recovery links.
- Playwright axe found no serious or critical results on the landing, sample,
  privacy, terms, or 404 pages.
- Keyboard checks passed skip-link activation, visible 3 px focus, route focus
  transfer, back-button focus restoration, and native control operation.
- At 390 px, the app board has no page-level horizontal overflow and all
  checked visible controls are at least 44 by 44 CSS pixels. The public-nav
  clipping exception is reported above.
- With reduced motion requested, scrolling becomes `auto` and animation and
  transition durations reduce to `0.00001s`.
- A 200% root-text smoke retained a 390 px document width and did not hide app
  controls.
- Live mobile Lighthouse 12.8.2: Performance 100, Accessibility 100, Best
  Practices 100, SEO 100; FCP 1.2 s, LCP 1.5 s, TBT 70 ms, CLS 0, 73 KiB
  transferred.

## Product shape and documentation

`/`, `/demo`, `/privacy`, `/terms`, `/robots.txt`, `/sitemap.xml`,
`/manifest.webmanifest`, and `/sw.js` return 200. The repository includes the
required README, MIT license, privacy and terms pages, demo documentation,
copy audit, visual thesis, original-asset provenance, and handoff. No tracked
secret or private-key material was found.

The lantern-room visual system matches the recorded palette, typography,
spacing, motion, and single-theme rationale. The core brief does not need an
AI action: deterministic recap, export, and sample data serve the facilitator
job without sending learner records to a model.

## Required next steps

1. Deploy an artifact whose `/health` identity is exactly the reviewed
   candidate, then recheck local/live hashes and the full live suite.
2. Make the 390 px public navigation show every link without an initially
   clipped destination.
3. Canonicalize demo routes to the public demo URL, not the private `/board`
   route.
4. Validate actual image bytes in demo mode and show the same recovery message
   as the backend.
5. Either ship the researched paid organization tier through Sociobot billing
   or record an explicit scope deviation before release.
