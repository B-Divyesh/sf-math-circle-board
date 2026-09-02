# Math Circle Board — polish round 2 handoff

- **Work order:** `math-circle-board-polish-2`
- **Repair commit:** `187a3ba61b9b92d29c676c62469111ed694b9c13`
- **Deployed source:** `6b669f7f859dcba703d690761247ea70fa984e2f`
- **Live URL:** <https://math-circle-board.sociobot.in>
- **Result:** PASS — no review finding remains.

## What changed

- Registered and tested private-cache lifecycle, photo-upload limits/privacy,
  and individual deletion promises in `.factory/claims.json`.
- Removed the untestable claim about the exact account record stored by the
  service. The remaining Microsoft sign-in statement is covered by owner access.
- Made the documented “under 5 MB” limit exact in both private and demo uploads.
- Preserved the one-click `?demo=1` sandbox, first screen, legal routes,
  metadata, 404, keyboard focus, mobile layout, and the full facilitator board
  workflow.

## Run and verify

```sh
npm ci
npm test
npm run build
npm run test:e2e
npm run test:cold-claims
```

The container starts with `PORT` alone. It uses `./data` locally and `/data`
in the deployed container. `npm run test:cold-claims` starts from a temporary
clean clone and runs every manifest claim individually.

Local evidence is in `.factory/polish-evidence-2/`. The full finding map is
in `.factory/polish-2.md`.

## Verification evidence

- `npm test`: passed TypeScript, copy contract, 3 Vitest tests, and 12 Rust
  tests.
- `npm run build`: passed; the initial application JS is 14.44 KB gzip.
- `npm run test:e2e`: passed all 26 browser tests, including axe serious/
  critical checks, mobile, routing, offline demo, rate limiting, and board
  data controls.
- `npm run test:cold-claims`: passed all 18 declared commands from a clean
  clone.
- `verify-url.sh` passed locally for root and demo with no console errors and
  complete title/lang/main/alt/button checks.
- Live root and demo passed the same `verify-url.sh` check. A fresh live
  Playwright run passed the public first-screen and demo-isolation tests 2/2.
  See `.factory/polish-evidence-2/live/` and `live-demo/`.

## Known gaps

None. The product intentionally has no paid tier or organization controls in
this release; that scoped limitation is stated and tested.
