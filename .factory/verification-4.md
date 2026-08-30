# Independent verification 4 — FAIL

**Date:** 2026-08-30

**Work order:** `math-circle-board-verify-4`

**Candidate:** `ce03a065b85c5b5713e17d65d9e7a5370d2da414`

**Live URL:** https://math-circle-board.sociobot.in
**Verdict:** **FAIL — do not promote.**

This was a fresh verification from the clean candidate checkout. No product
source was changed. `GET /health` returned the exact candidate SHA, and the
live JS/CSS asset names and sizes matched a fresh local production build.

## Mandatory first checks

### Claims gate — FAIL

`.factory/claims.json` is missing. Therefore no listed claim tests exist to
run through the demo entry point. The claims contract makes a missing manifest
release-blocking by itself.

There are also many unlisted claims in visitor copy and README, including
“Private by default”, “No ads”, offline reload, JSON export, no analytics or
runtime CDN scripts, the 40-read/8-write rate-limit bursts, and a printable
recap “in minutes”. None has the required `@claim:<id>` test registration.

### Cold first-read and demo gate — FAIL

Cold desktop and 390 px visits show:

- headline: “Follow every line of thinking.”
- supporting sentence: “Sequence open-ended problems, catch partial
  strategies, and leave each gathering with a learning record worth keeping.”
- only primary action: “Sign in with Microsoft”

The screen does not plainly name the volunteer facilitator of a small math
circle. The headline is metaphorical rather than the job in the user's words.
Most importantly, there is no visible **Try it with sample data** action.
`/demo` returns HTTP 404 and renders the same sign-in/setup gate, with no sample
workspace, demo banner, reset, or start-for-real action. `.factory/demo.md` is
also missing. This independently requires FAIL.

## Release-blocking defects

### P1 — required legal routes return 404 and fail accessibility

Both linked legal pages render client-side text but their document responses
are HTTP **404**. Chromium logs `Failed to load resource: the server responded
with a status of 404` on each page. This makes both required footer links dead
to crawlers and causes console errors.

Axe found a serious `color-contrast` violation on both `/privacy` and `/terms`:
the 12 px `.eyebrow` uses `#d36b3f` on `#f5f0e4`, measured by axe at **3.1:1**
where 4.5:1 is required. The repository E2E visits these pages but does not
assert the navigation response or console errors and does not run axe there.

### P1 — routing and keyboard focus do not meet the app contract

Authenticated navigation is transient DOM state rather than real navigation:

- activating **Learners** leaves the URL at `/`;
- reloading returns to the Board view;
- opening `/#learners` directly also opens Board;
- after keyboard activation, focus falls to `<body>` rather than the new page
  heading; back/forward cannot restore the view.

The skip link changes the hash to `#main` but leaves focus on `<body>`, so it
does not move screen-reader/keyboard focus to main content. On the public 390 px
screen, the wordmark is 38 px tall and Privacy/Terms are about 20 px tall,
below the required 44 px touch target. Global yellow focus rings have only
**1.38:1** contrast against the paper surface and **1.54:1** against inputs,
below the required 3:1 when controls on light app surfaces receive focus.

### P1 — claim-like promises have no claim coverage

This is broader than the missing file: the public privacy, offline, export,
rate-limit, payment, and no-tracking promises are not tied to observable demo
tests. The existing four Playwright tests are untagged and cannot be selected
using the mandatory claim commands. A verifier cannot prove any promise from a
fresh demo sandbox because that sandbox does not exist.

## Other findings

### P2 — required public site structure and discovery metadata are absent

The signed-out landing page has no product preview, three-step “How it works”,
plain privacy/non-goals section, or $39 paid-tier section. Those sections are
unreachable until a real board owner signs in. There is no normal page header
or navigation landmark on the landing page.

The live document has no canonical URL, Open Graph metadata, Twitter card, or
apple-touch icon. `robots.txt` and `sitemap.xml` return 404. There is no designed
404 route: an arbitrary missing path returns status 404 but renders the normal
setup gate. Runtime titles use “task — Math Circle Board” rather than the
required “Math Circle Board — what it does” form. `.factory/copy-audit.md` is
missing.

### P2 — deletion controls do not cover the whole private board

Facilitators can export everything and delete sessions, learners, and photos,
but there is no way to delete the board itself, including facilitator name,
circle name, and owner identifier. For a minors-adjacent product whose brief
requires deletion controls, a confirmed full-board deletion path is needed.

### P2 — Dockerfile violates the required Rust base-image contract

The Dockerfile uses `FROM rust:1.89-alpine`; the backend contract explicitly
requires the moving stable tag `rust:1-alpine` or `rust:1-slim` and forbids a
pinned minor. The image could not be executed here because Docker is not
installed, but the exact frontend and optimized Rust builds passed locally and
the live deployment is healthy.

## Evidence that passed

### Clean install, tests, checks, and production artifacts

```text
npm ci                                      PASS (60 packages, 0 vulnerabilities)
npm test                                    PASS
  TypeScript                                PASS
  Vitest                                    PASS (3/3)
  Rust                                      PASS (6/6)
cargo fmt --all -- --check                  PASS
cargo clippy --all-targets --all-features -- -D warnings
                                             PASS
npm run build                               PASS; dist/ produced
BUILD_SHA=<candidate> cargo build --release PASS
npm run test:e2e -- --workers=1             PASS (4/4)
```

Vite emitted 32,027 B raw / 11.41 KB gzip initial JS and 20,156 B raw /
5.27 KB gzip CSS. The lazy Microsoft identity chunk is 268,932 B raw /
67.30 KB gzip. The 390 px hero is 22,518 B. These meet the initial-load
budgets.

### End-to-end product and recovery

The isolated test-auth workflow completed adult setup, session, problem,
learner, partial attempt, strategy, private note, valid WebP upload, export,
and print recap. The generated PDF was one A4 page; it contained learner
thinking and excluded the facilitator-only note. Export contained the private
note and one attachment. A server restart retained the group, learner,
session, problem, attempt, private note, and attachment.

Independent invalid/boundary checks produced the expected responses:

- blank and 61-character learner aliases: 400;
- case-insensitive duplicate alias: 409;
- malformed and impossible dates: 400;
- unknown attempt status and 13 strategy tags: 400;
- fake PNG bytes: 400;
- request over 6 MiB: 413;
- a subsequent valid board read remained 200 with unchanged valid data.

The release binary started in a fresh directory with only `PORT` plus process
`PATH`, used `./data`, generated `owner-invite.txt` with mode 0600, served the
frontend, and returned the candidate SHA from `/health`.

### Deployment identity, concurrency, and limits

- Live `/health`: 200 with build
  `ce03a065b85c5b5713e17d65d9e7a5370d2da414`.
- 100 concurrent live health requests: 100 × 200 in 465 ms.
- Live read burst: configured burst 40; a 160-concurrent request probe yielded
  47 × 200 and 113 × 429 as tokens replenished, with `Retry-After: 1`.
- Live write burst: 8 requests reached authentication and 32 were limited,
  with 429 and `Retry-After: 1`.
- Sociobot license verification burst: 30 × 200 and 70 × 429, with
  `Retry-After: 4`; observed allowance was 30 in the burst window.

### Identity, billing, privacy, PWA, and headers

The sign-in action redirects to only the configured identity authority,
`sociobotcustomers.ciamlogin.com`, using the expected tenant, client ID,
authorization-code flow, and PKCE S256. The hosted Microsoft page loads normal
Microsoft first-party assets, but no alternate identity provider is offered.

Cold public requests were same-origin only. The authenticated local workflow
made only product-origin requests (plus product-created `blob:` URLs). Supplying
an invalid license in the live URL stored it under
`sb_license:math-circle-board`, stripped it from the URL, called only the
documented `api.sociobot.in` verification endpoint, and cached the invalid
verdict. No analytics, ads, remote fonts, or CDN scripts loaded.

The service worker registered, updated, controlled the next reload, and used
cache `mcb-shell-v3`. The repository's fresh authenticated 390 px test passed
offline reload with cached board data and drafts. Public offline reload served
the shell and a clear connection error because no authenticated board was
cached.

Live responses include HSTS, CSP, `nosniff`, frame denial, same-origin referrer
policy, and a restrictive Permissions-Policy. Hashed assets use one-year
immutable caching. The successful factory URL verifier found a title,
`lang=en`, one h1, main, image alt text, labeled buttons, and no console error
on `/`.

### Performance and baseline accessibility

Fresh live mobile Lighthouse: Performance 100, Accessibility 100, Best
Practices 100, SEO 100; LCP 1.4 s, CLS 0, TBT 50 ms, total transfer 76 KiB.
The public `/` page had zero axe violations at desktop and 390 px, no horizontal
overflow, one h1/main, and no console/page errors. At 200% root text size the
390 px page still had no horizontal overflow. Reduced-motion CSS collapses
animation and transition durations to 0.01 ms. The legal-route axe and routing
failures above remain release-blocking despite the landing score.

## Required remediation before another candidate

1. Add `.factory/claims.json`; register exactly one tagged demo test for every
   live/README claim and run all listed commands successfully.
2. Add a one-click, isolated `/demo` with realistic sample data, persistent
   demo banner, reset, start-for-real action, and `.factory/demo.md`.
3. Rewrite the first screen in plain words to identify the volunteer
   facilitator and small math circle, with the demo as the primary action.
4. Make `/privacy` and `/terms` return 200, fix their 3.1:1 eyebrow contrast,
   and include them in console and axe coverage.
5. Implement real view URLs/history, deep-link reload, back/forward restoration,
   route focus/announcement, a functional skip target, 44 px targets, and
   3:1 focus indicators on both dark and light surfaces.
6. Add the required public skeleton, metadata, robots/sitemap, and designed 404.
7. Add confirmed full-board deletion and change the Rust base image to
   `rust:1-alpine` or `rust:1-slim`.
