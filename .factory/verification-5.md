# Independent verification 5 — FAIL

**Date:** 2026-09-01  
**Work order:** `math-circle-board-verify-5`  
**Candidate:** `9b865187a20becba99263c262f547f08849e176f`  
**Live URL:** https://math-circle-board.sociobot.in  
**Verdict:** **FAIL — do not promote.**

No product source was modified during this verification.

## Release-blocking findings

### P1 — the 404 page has a serious contrast failure

Live mobile Playwright + axe found a serious `color-contrast` result at
`/not-a-real-page`. The decorative `<div class="not-found-mark" aria-hidden="true">404</div>`
renders foreground `#303533` on `#101d2a`, a **1.36:1** ratio; axe requires
3:1 for this 80 px text. The page is otherwise a working 404 with recovery
links, but the accessibility contract requires no serious or critical axe
findings. The same live check found no serious/critical findings on the landing
demo, `/privacy`, or `/terms`.

### P1 — the first declared claim command fails from a cold build

After `npm ci` in this clean checkout, the first required command,
`npm run test:claims -- --grep @claim:demo-isolation`, exited **1** before a
test ran. `scripts/run-e2e.sh` allows 30 seconds for `cargo run` to make
`/health` available, while the uncached Rust build took about 70 seconds.
This makes the required demo-entry claim test fail on a clean first run. The
claims contract says any failing claim test is release-blocking.

After the backend was compiled, all ten individual manifest commands passed;
that confirms the feature assertions but does not make the cold command pass.

## Deployment identity and first read

- `GET /health` returned HTTP 200 and
  `{"build":"9b865187a20becba99263c262f547f08849e176f","ok":true}`.
- SHA-256 values for live `index.html`, `assets/index-BRwFmSsQ.js`, and
  `assets/index-CSO0TFcb.css` exactly matched the candidate's local production
  build.
- Cold live first read passed: “Plan and record small math-circle sessions”
  explains the job; its next sentence names volunteer facilitators and 6–12
  learners; and **Try it with sample data** is visible with “See a filled
  board. Changes stay in this demo.”

## Checks that passed

### Local quality gates

- `npm ci`: passed (60 packages; 0 reported vulnerabilities).
- `npm test`: passed — TypeScript check, 3 Vitest tests, 11 Rust tests.
- `npm run typecheck`: passed.
- `npm run build`: passed; `dist/` produced. Initial JavaScript was 82.17 KB
  gzip (14.87 KB + 67.30 KB), and CSS was 6.21 KB gzip.
- `npm run test:e2e`: passed — 12/12 Playwright tests.
- `BUILD_SHA=9b865187a20becba99263c262f547f08849e176f cargo build --release`:
  passed. Docker was unavailable in this QA container, so a container-image
  build was not run.
- Once compilation was warm, every one of the ten explicit commands in
  `.factory/claims.json` passed separately: `demo-isolation`, `attempt-record`,
  `recap-privacy`, `json-export`, `offline-reload`, `no-tracking`,
  `owner-access`, `rate-limits`, `plus-price`, and `full-delete`.

### Live product, privacy, accessibility, and resilience

- Desktop and 390 px sample use worked: a blank learner alias was blocked by
  native validation (“Please fill out this field”), then adding `Ravi`
  recovered normally. Saving a partial attempt and private note worked; recap
  showed the thinking and omitted the private note.
- On the public/sample flow, Playwright recorded only
  `https://math-circle-board.sociobot.in` requests. There were no console or
  page errors. No remote font, analytics, advertisement, or runtime script was
  loaded.
- Fresh keyboard use reached the visible skip link, moved focus to `main`, and
  activating Learners moved focus to the new page `<h1>`. The 390 px board had
  a 390 px document width. Reduced motion computed `scroll-behavior: auto` and
  `transition-duration: 0.00001s`.
- Response headers include HSTS, `X-Content-Type-Options: nosniff`,
  `X-Frame-Options: DENY`, `Referrer-Policy: same-origin`, CSP with
  `frame-ancestors 'none'`, and a restrictive permissions policy. Hashed JS
  uses `Cache-Control: public, max-age=31536000, immutable`.
- `/privacy` and `/terms` return 200; the designed unknown route returns 404.
- Live rate-limit confirmation: 100 concurrent reads with one forwarded client
  address produced **42 × 200** and **58 × 429**, each limited response with
  `Retry-After: 1` (the configured burst is 40; two tokens replenished during
  the burst). Thirty deliberately invalid write requests from a separate
  address produced **8 × 422** followed by **22 × 429**, again with
  `Retry-After: 1`. Thus the observed write allowance was 8 and read allowance
  was approximately 40 per burst.

## Required follow-up

1. Change or remove the low-contrast decorative 404 mark and add `/not-a-real-page`
   to the serious/critical axe regression coverage.
2. Make `scripts/run-e2e.sh` reliable from a cold checkout, for example by
   building the test binary before its 30-second health wait or by using an
   appropriate bounded startup allowance. Re-run every manifest command from
   a truly cold build before requesting verification again.
