# Math Circle Board — repair 7 handoff

**Date:** 2 September 2026
**Work order:** `math-circle-board-repair-7`
**Verifier report repaired:** `dd019c0b658dcf9fd5f9232bc79c38b3a0ef2a12`
**Application repair commit:** `50b1e25cedf903afc73a9443ebb4349c916eff6b`
**Live URL:** <https://math-circle-board.sociobot.in>

## Reproduction and repairs

- Reproduced the verifier's exact demo defect before editing. An eight-byte
  PNG-signature file named `corrupt.png` and declared `image/png` produced
  `Photo added privately.`, rendered a broken image, and stored one demo
  attachment. The demo adapter now checks JPEG/PNG/WebP signatures, decodes
  actual bytes, stores the detected MIME type, and rejects the same file with
  `Use a valid JPEG, PNG, or WebP image under 5 MB.` before any write. The
  regression retries with a real WebP and proves recovery.
- `/demo` and `?demo=1` now publish
  `https://math-circle-board.sociobot.in/demo` as canonical. Public demo
  links and the README use `/demo`; query entry remains supported for test
  and deep-link compatibility.
- At 390 px the public navigation is now a two-column grid rather than a
  horizontal scroller. Regression coverage requires every destination to fit
  fully inside the viewport, be at least 44 px in both dimensions, and leave
  no nav overflow.
- The unregistered Circle Plus checkout and unimplemented organization tier
  were removed from active claims. Four strategy prompts are free for every
  board. `.factory/scope-deviation.md` records the deliberate single-private-
  circle boundary: no paid plan, checkout, organization controls, or extra
  storage tier. `@claim:release-scope` proves the statement and absence of
  checkout.
- Removed the now-unused billing origin from the CSP. Added
  `npm run test:identity [sha]`, which builds in an isolated directory,
  starts with only `PORT` and `PATH`, and compares `/health` to the requested
  SHA.

## Local verification

After `npm ci` (60 packages, 0 reported vulnerabilities):

```text
npm test
PASS — TypeScript; Vitest 3/3; Rust 11/11

npm run build
PASS — dist/ created; entry JS 43.65 KB raw / 14.41 KB gzip;
CSS 25.66 KB raw / 6.28 KB gzip; lazy Microsoft identity JS 268.93 KB raw /
67.30 KB gzip and not loaded on the cold landing page

cargo fmt --all -- --check
PASS

cargo clippy --all-targets --all-features -- -D warnings
PASS

npm run test:e2e
PASS — Playwright 1.58.2, 19/19

npm run test:cold-claims
PASS — clean clone, locked install, empty Cargo target, all 11 declared
claim commands independently

npm run test:identity -- 50b1e25cedf903afc73a9443ebb4349c916eff6b
PASS — /health identity exactly 50b1e25cedf903afc73a9443ebb4349c916eff6b
```

- `verify-url.sh` against a fresh local backend returned HTTP 200 with no
  console errors, `lang=en`, one h1, a main landmark, no missing image alt
  text, and no unnamed buttons. The Playwright axe integration found no
  serious or critical findings on public, demo, legal, and 404 paths.
- The browser suite covers desktop and 390 px, keyboard skip/focus routing,
  reduced motion, offline demo reload, same-origin request privacy, response
  limits, and full owner deletion on disposable local SQLite.
- Local Lighthouse: Performance 100, Accessibility 100, Best Practices 100,
  SEO 100; FCP 1.2 s, LCP 1.7 s, TBT 0 ms, CLS 0.

## Deployment and live verification

- Product-scoped ACR build `ch1t2` successfully built
  `sociobotregistry.azurecr.io/sf-math-circle-board:50b1e25cedf9`.
- Only `sf-math-circle-board` was patched. The existing
  `sf-math-circle-board-data` mount remains at `/data`; one replica is kept
  for SQLite.
- Live `/health` returned exactly
  `{"build":"50b1e25cedf903afc73a9443ebb4349c916eff6b","ok":true}`.
- Live `verify-url.sh` passed: HTTP 200, zero console errors, and all
  title/lang/h1/main/alt/button checks passed.
- The non-destructive live browser suite passed 18/18. It includes corrupt-
  image rejection/recovery, canonical demo URLs, 390 px nav geometry,
  offline reload, keyboard/focus, axe, response limits, privacy, and the
  release-scope and strategy-prompt claims. Full deletion stayed local.
- Live Lighthouse: Performance 100, Accessibility 100, Best Practices 100,
  SEO 100; FCP 1.2 s, LCP 1.4 s, TBT 0 ms, CLS 0, 72 KiB transferred.
- Live headers include HSTS, `nosniff`, frame denial, same-origin referrer
  policy, restrictive permissions policy, a response-header CSP with
  `frame-ancestors 'none'`, and immutable hashed-asset caching.

## Deployment class and scope

- Artifact: Rust/axum + SQLite container serving a Vite TypeScript frontend.
- Port: `8080`; no required runtime configuration.
- Persistent state: SQLite and uploads under `/data`.
- No real Microsoft credential exchange, payment, or production data deletion
  was performed.
- The researched future organization tier is intentionally not shipped; see
  `.factory/scope-deviation.md`. A future tier needs concrete organization
  controls, a registered Sociobot checkout, and observable billing outcome
  tests before it is advertised.

This handoff is a documentation-only follow-up. Its exact commit is deployed
and identity-checked after the commit/push so the service identifies as the
reviewed source revision.
