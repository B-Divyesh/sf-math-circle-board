# Polish round 1 — finding evidence

**Reviewed source:** `60d4178a186f8c97a7ac45c2ece0131b4107147c`  
**Implementation commit:** `d158859ed0ff97adb8f6b59c73a0f3a38740a6a6`  
**Live URL:** <https://math-circle-board.sociobot.in>  
**Result:** PASS — zero findings remain.

No earlier `.factory/review-*.md` or `.factory/polish-*.md` existed. Every item in `.factory/review-1.md` and the controller addendum is mapped below.

| Finding | Change made | Evidence |
|---|---|---|
| F-1-1 — slogan heading | Replaced “Keep only the record you need” with **Data kept on the board**. | `npm run test:copy`; Playwright **plain public first screen and metadata are complete**; [cold mobile screenshot](polish-evidence-1/root/screenshot-mobile.png); live root check. |
| F-1-2 — 32-word README sentence | Replaced it with two sentences of 16 and 11 words. | `npm run test:copy`; `.factory/copy-audit.md`; committed README. |
| F-1-3 — manifest-claim jargon | Rewrote it as “runs the named demo-isolation check on a fresh local server.” | `npm run test:copy`; committed README. |
| F-1-4 — unlisted provenance claim | Removed the claim from every public footer. Kept provenance only in the internal design record. | `npm run test:copy`; Playwright **plain public first screen and metadata are complete** asserts the phrase is absent; [cold mobile screenshot](polish-evidence-1/root/screenshot-mobile.png); live root check. |
| Controller — informative section name | Same complete repair as F-1-1. | Same F-1-1 evidence. |
| Controller — split README sentence | Same complete repair as F-1-2. | Same F-1-2 evidence. |
| Controller — replace manifest jargon | Same complete repair as F-1-3. | Same F-1-3 evidence. |
| Controller — remove or register provenance | Chose removal because provenance is not an observable product capability. | Same F-1-4 evidence; `.factory/design.md` keeps the source record. |

## Required product regressions

- One-click demo and `?demo=1`: **@claim:demo-isolation** passed locally, from a clean clone, and live. The banner, reset, and exit are visible in the [live demo screenshot](polish-evidence-1/demo/screenshot-mobile.png).
- Claims: all 11 `.factory/claims.json` commands passed separately from a clean clone. `npm run test:copy` confirms each ID is unique and maps to exactly one tagged browser test.
- Titles, metadata, routing, focus, 404, and legal links: the full browser suite passed. The selected live suite passed 8/8.
- Mobile: 390 px navigation and all visible 44×44 targets passed locally and live. Both cold screenshots were captured at 390×844.
- Accessibility: Playwright axe found no serious or critical issues. Fleet verification found one h1, `lang="en"`, a main landmark, no missing alt text, and no unlabeled buttons.
- Privacy and offline: **@claim:no-tracking** and **@claim:offline-reload** passed locally, from a clean clone, and live.
- Backend: 11 Rust tests passed, including 429 plus `Retry-After`, first forwarded-IP handling, migrations, private deletion, and true 404 responses.

## Live cold evidence

- Root: HTTP 200, title **Math Circle Board — Plan small math-circle sessions**, 678 ms verifier load, zero console errors.
- Demo: HTTP 200, title **Demo — Math Circle Board**, 535 ms verifier load, zero console errors.
- `/health`: build `d158859ed0ff97adb8f6b59c73a0f3a38740a6a6`, `ok: true` on the reviewed deployment.
- Visual inspection confirmed the lantern-room identity, readable first screen, informative privacy heading, compact footer, persistent demo banner, and usable 390 px board.
