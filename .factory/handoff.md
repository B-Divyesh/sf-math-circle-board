# Math Circle Board — polish 1 handoff

**Work order:** `math-circle-board-polish-1`
**Date:** 2 September 2026
**Result:** PASS — every review 1 and controller finding is resolved.

## Shipped

- Replaced the landing slogan with the informative heading **Data kept on the board**.
- Split the long README test sentence and replaced “manifest claim” with a plain explanation.
- Removed the public provenance claim. Asset provenance remains in `.factory/design.md`.
- Added a verb-first, 85-character catalog description.
- Added `npm run test:copy`. It protects the copy fixes, catalog limit, unique claim IDs, and one browser test per claim.
- Kept the lantern-room identity, isolated demo storage, route titles, 404, legal pages, focus behavior, and mobile layout intact.
- Updated `.factory/claims.json` so the catalog capabilities point to their observable tests.

## Verification

- `npm test`: PASS — TypeScript, copy contract, 3 Vitest tests, and 11 Rust tests.
- `npm run build`: PASS — JS 14.38 + 67.30 KB gzip; CSS 6.28 KB gzip.
- `npm run test:e2e`: PASS — 19/19 browser tests.
- `npm run test:cold-claims`: PASS from a temporary clean clone. All 11 declared claim commands passed independently.
- `npm run test:identity`: PASS — `/health` returned implementation SHA `d158859ed0ff97adb8f6b59c73a0f3a38740a6a6` from the local container.
- Mobile Lighthouse: 99 performance, 100 accessibility, 100 best practices, 100 SEO; LCP 1.8 s, CLS 0, TBT 20 ms.
- Fleet deployment: PASS with durable `/data`, one replica, and image `sf-math-circle-board:d158859ed0ff`.
- Cold live checks: root and `/?demo=1` returned 200 with no console errors. Titles, `lang`, one h1, main landmark, and alt coverage passed.
- Live Playwright: 8/8 public, demo, offline, privacy, route, focus, 404, and mobile regressions passed.
- Live health returned `{ "build": "d158859ed0ff97adb8f6b59c73a0f3a38740a6a6", "ok": true }` for the reviewed production revision.

## Run and verify

```sh
npm ci
npm test
npm run build
npm run test:e2e
npm run test:cold-claims
npm run test:identity
```

Live product: <https://math-circle-board.sociobot.in>
Direct sample: <https://math-circle-board.sociobot.in/?demo=1>

## Known gaps and next steps

No review finding or required acceptance item remains. No follow-up is required for this polish round.
