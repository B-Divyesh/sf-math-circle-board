# Polish round 2 — finding evidence

**Repair commit:** `187a3ba61b9b92d29c676c62469111ed694b9c13`  
**Live URL:** <https://math-circle-board.sociobot.in>  
**Result:** PASS — all review findings are repaired or deliberately removed from visitor copy.

Every earlier review and polish record was read before this repair. The local
verification screenshots are stored under
`.factory/polish-evidence-2/`. The live recheck is recorded below after the
deployment verification.

| Finding | Change made | Evidence |
|---|---|---|
| F-1-1 | Kept the informative **Data kept on the board** heading. | `npm run test:copy`; `plain public first screen and metadata are complete`; local [mobile root](polish-evidence-2/local/screenshot-mobile.png). |
| F-1-2 | Kept the README browser-test explanation as two short sentences. | `npm run test:copy`; `.factory/copy-audit.md`. |
| F-1-3 | Kept the plain “named demo-isolation check” README wording. | `npm run test:copy`; README. |
| F-1-4 | Kept generated-art provenance out of public footer copy. | `npm run test:copy`; public-screen regression asserts the phrase is absent. |
| F-2-1 | Registered **private-cache-lifecycle**. The signed-in flow now proves board cache and a draft exist, then proves sign-out removes the cached board. | `@claim:private-cache-lifecycle`; `.factory/claims.json`; Playwright full workflow. |
| F-2-2 | Removed the unobservable account-identifier/password storage statement. Public copy now says only that Microsoft handles adult sign-in, which `owner-access` already proves. | `@claim:owner-access`; privacy route check; source search has no identifier/password storage promise. |
| F-2-3 | Registered **photo-upload-limits** and tightened both private and demo limits to reject files of 5 MB or more. The workflow uploads valid JPEG, PNG, and WebP files, rejects over-limit and invalid input, and verifies anonymous image retrieval returns 401. | `@claim:photo-upload-limits`; Rust image validation test; Playwright full workflow. |
| F-2-4 | Registered **individual-delete**. The authenticated workflow removes one photo, learner, and session and verifies related records are gone. | `@claim:individual-delete`; Playwright full workflow. |
| Controller — all four visitor promises | Three remaining useful promises are registered and tested. The untestable identity-storage detail is removed. | `npm run test:cold-claims`; `npm run test:e2e`; claims manifest has 18 one-to-one tags. |
| Controller — demo, routing, mobile, first screen, legal links | Preserved and reran the shipped isolated demo, route/title/focus/404 checks, mobile target checks, legal routes, and first-screen tests. | `@claim:demo-isolation`; `@claim:offline-reload`; `legal, 404, mobile, keyboard, routes, and focus pass regression checks`; local [root](polish-evidence-2/local/verify.json) and [demo](polish-evidence-2/local-demo/verify.json). |

## Verification

- `npm test` passed: TypeScript, copy contract, 3 Vitest tests, and 12 Rust tests.
- `npm run build` passed. The initial product chunk is 14.44 KB gzip; the
  lazy Microsoft sign-in dependency is 67.30 KB gzip.
- `npm run test:e2e` passed all 26 browser scenarios, including serious and
  critical axe checks on public, legal, demo, and mobile paths.
- `npm run test:cold-claims` passed from a temporary clean clone. It installed
  locked dependencies and ran every one of the 18 manifest commands
  separately, including the three new private-board claims.
- `/opt/fleet/lib/verify-url.sh` passed locally for `/` and `/demo`: both had
  route titles, `lang="en"`, one h1, a main landmark, no missing image alt
  text, no unnamed buttons, and no console errors.

## Live cold recheck

After deployment, a new browser context loaded the live root and `/demo`.
`verify-url.sh` found no console errors on either route and verified the
route-specific title, `lang`, one h1, main landmark, image alt text, and named
buttons. The live build identity is
`6b669f7f859dcba703d690761247ea70fa984e2f`.

- `PLAYWRIGHT_BASE_URL=https://math-circle-board.sociobot.in npx playwright
  test --workers=1 --grep 'plain public first screen|@claim:demo-isolation'`
  passed 2/2 from fresh live contexts.
- Live [root report](polish-evidence-2/live/verify.json), [root mobile
  screenshot](polish-evidence-2/live/screenshot-mobile.png), [demo report]
  (polish-evidence-2/live-demo/verify.json), and [demo mobile screenshot]
  (polish-evidence-2/live-demo/screenshot-mobile.png) show the working first
  screen, one-click sample board, and persistent demo controls.
