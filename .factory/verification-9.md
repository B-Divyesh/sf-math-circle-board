# Independent verification 9 — PASS

**Candidate:** `3a56cbadd6db7d03e4d2a33937cdaab8f115a054`

**Live URL:** <https://math-circle-board.sociobot.in>
**Verified:** 2026-09-02

## Result

**PASS.** The live deployment identifies itself as the candidate:

```json
{"build":"3a56cbadd6db7d03e4d2a33937cdaab8f115a054","ok":true}
```

Cold first read also passes. The landing page says it lets volunteer math-circle
facilitators plan and record small sessions, describes sequencing problems,
partial attempts, private notes, and recaps for 6–12 learners, and offers a
one-click **Try it with sample data** action that says a filled board opens and
changes remain in the demo.

## Clean-checkout gates

- `npm ci` — PASS; 60 packages, zero reported vulnerabilities.
- Every exact command from `.factory/claims.json` — PASS. The eleven exercised
  claims were `demo-isolation`, `attempt-record`, `recap-privacy`,
  `json-export`, `offline-reload`, `no-tracking`, `owner-access`,
  `rate-limits`, `release-scope`, `strategy-palette`, and `full-delete`.
- `npm test` — PASS: TypeScript, Vitest 3/3, Rust 11/11.
- `npm run build` — PASS; `dist/` produced. Initial application JS is 43.65 KB
  raw / 14.41 KB gzip and CSS is 25.66 KB raw / 6.28 KB gzip. The 67.30 KB
  gzip Microsoft identity chunk is lazy and absent from the cold public load.
- `cargo fmt --all -- --check` — PASS.
- `cargo clippy --all-targets --all-features -- -D warnings` — PASS.
- `cargo build --release` — PASS; release binary produced.
- `npm run test:e2e` — PASS; 19/19 Playwright 1.58.2 tests.
- `npm run test:identity -- 3a56cbadd6db7d03e4d2a33937cdaab8f115a054`
  — PASS; isolated `PORT`/`PATH` startup returned the requested build identity.

Docker is not installed in this verifier container, so a local container-image
build could not be run. This is an environment limitation, not a product
failure; the independently checked live container reports the candidate SHA.

## Independent live QA

- Desktop cold load: HTTP 200, correct plain-language title and one h1, main
  landmark, no page errors or console errors, and only same-origin requests in
  the public/demo flow. The expected Microsoft resources load only after the
  explicit sign-in action.
- The explicit sign-in flow redirects to
  `https://sociobotcustomers.ciamlogin.com/.../authorize`; no alternative
  identity provider is used.
- At 390 px, public/demo/legal/404 routes had no horizontal overflow. All
  interactive targets were covered by the 44 px regression; keyboard-only
  testing reached the skip link and moved focus to `main` after navigation.
  Reduced-motion media settings reduced animation and transition durations to
  effectively zero.
- Axe Playwright scans of `/`, `/demo`, `/privacy`, `/terms`, and the designed
  404 found no serious or critical violations. (Chromium reports the intentional
  HTTP 404 navigation as a failed resource; there was no page-script error.)
- Demo workflow exercised sample isolation/reset, alias validation and recovery,
  invalid image rejection followed by valid image recovery, attempts/strategies/
  notes, JSON export, printable recap exclusion of private notes, and full
  private-board deletion in the isolated test database.
- PWA: `/demo` received an active `sw.js` controller, accepted a registration
  update check, and reloaded offline at 390 px with the demo banner and board
  intact.
- Public links crawled from `/`, `/demo`, `/privacy`, and `/terms` returned
  HTTP 200. `/not-a-real-page` returned the designed HTTP 404.
- Response headers include HSTS, `nosniff`, `DENY` framing, same-origin
  referrer policy, restrictive permissions policy, and a response-header CSP
  with `frame-ancestors 'none'`. Hashed JS/CSS assets return
  `Cache-Control: public, max-age=31536000, immutable`.
- Backend allowance was confirmed live with distinct fixed forwarded client
  IPs: a 100-request `/api/status` burst yielded 46 HTTP 200 and 54 HTTP 429;
  a 30-request invalid `/api/setup` burst yielded 8 HTTP 422 and 22 HTTP 429.
  Both limited responses supplied `Retry-After: 1`. This confirms the stated
  40-request read burst with refill and the stricter 8-write burst.

## Defects

No release-blocking, high, medium, or low product defects were found.

## Scope and known limitations

The product deliberately ships one adult-owned private circle, not the brief's
future paid organization tier. It truthfully exposes no checkout, organization
controls, or paid storage; this is documented in `.factory/scope-deviation.md`
and covered by `@claim:release-scope`.
