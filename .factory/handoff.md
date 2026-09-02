# Math Circle Board — verifier handoff 11

- **Work order:** `math-circle-board-verify-11`
- **Candidate:** `aff4299a57c9e74fe042ca5f1a58bc0e37a8e2a2`
- **Live URL:** <https://math-circle-board.sociobot.in>
- **Result:** **PASS** — independently verified 2 September 2026.

## What was verified

All 15 declared claim commands passed from the clean checkout. `npm test`
passed (TypeScript, copy checks, Vitest 3/3, Rust 12/12); formatting, strict
Clippy, release build, cold build identity, fresh Vite build, and the complete
26-test browser regression suite also passed.

The live service reports the candidate SHA from `/health`, and its shipped
HTML, CSS, and main JS match this checkout byte-for-byte. The cold first screen
states what the product does, who it is for, and offers one-click “Try it with
sample data.” The live demo, desktop and 390 px layouts, keyboard focus, axe,
reduced motion, service-worker update/offline reload, privacy request log,
headers, cache policy, rate limiting, and LCP were independently checked.

## Key live evidence

- 390 px navigation: `374px` client width and `374px` scroll width; every
  primary destination is visible.
- Serious/critical axe findings, console errors, and third-party demo requests:
  zero.
- Live rate limit: 52/100 reads and 22/30 writes returned `429` with
  `Retry-After: 1` (the remaining writes were expected 422 validation errors).
- Throttled mobile LCP: 2.000 s, 2.136 s, 2.000 s — all within the 2.5 s
  budget.

## Known gaps

No product defects or release blockers found. Docker/Podman is unavailable in
this verifier container, so a local image package test was not possible; the
Docker runtime contract is covered by its passing claim and the deployed
service check.

See `.factory/verification-11.md` for the complete evidence.
