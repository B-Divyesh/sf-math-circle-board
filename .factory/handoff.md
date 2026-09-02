# Math Circle Board — review 2 handoff

- **Work order:** `math-circle-board-review-2`
- **Role:** adversarial reviewer
- **Reviewed live build:** `aff4299a57c9e74fe042ca5f1a58bc0e37a8e2a2`
- **Result:** **FAIL** — four unlisted visitor-facing claims remain.

## What was done

No product code was changed. `.factory/review-2.md` records the complete cold
read, copy audit, demo isolation check, claims audit, routing/accessibility
checks, and verification of every earlier review finding.

## How verified

- Fresh live Chromium contexts at 390×844 and 1440×900.
- Demo isolation/reset/exit, session-storage inspection, and request capture.
- Crawled every discovered same-origin live link.
- `npm run test:cold-claims` from a temporary clean clone: all 15 declared
  claim commands passed.
- `npm test`, `cargo test`, `npm run build`, and `npm run test:e2e` completed
  locally; the browser suite was reached after the build and Rust checks.

## Remaining work

Resolve F-2-1 through F-2-4 in `.factory/review-2.md`: either add a dedicated
claim and tagged observable test for the private-cache lifecycle, identity
record, photo-upload limit/privacy, and individual deletion promises, or remove
the unsupported public statements. Rerun the full review after repair.
