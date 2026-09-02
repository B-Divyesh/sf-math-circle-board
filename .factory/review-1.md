# Adversarial first-read review 1 — Math Circle Board

**Reviewed:** 2026-09-02  
**Live URL:** <https://math-circle-board.sociobot.in>  
**Verdict:** **FAIL**

The core workflow is clear and tryable. This review fails on four copy and claims-contract defects. The acceptance rule requires zero findings for PASS.

## Cold first read

Fresh desktop (1440 px) and mobile (390 px) visits answered all three questions before scrolling.

- **What it does:** plans and records small math-circle sessions, partial attempts, private notes, and recaps.
- **For whom:** volunteer math-circle facilitators with 6–12 learners.
- **First action:** **Try it with sample data**; adjacent text says a filled board opens and changes stay in the demo.

The 390 px first screen has no horizontal overflow (390 px scroll width / 390 px client width). Its blue-hour, warm-paper, lantern-room visual system is distinct and product-specific rather than generic SaaS styling.

## Findings

### F-1-1 — Minor — landing heading is a slogan, not a section name

**Location and quote:** landing Privacy and limits section: **“Keep only the record you need”**.

**Why:** heard alone in a heading list, it does not name the section or say what information follows. It is a mood/slogan construction prohibited by the plain-words contract.

**Fix:** replace it with **“Data kept on the board”**. The following copy can then explain aliases, the non-gradebook boundary, and privacy.

### F-1-2 — Minor — README sentence exceeds the 22-word cap

**Location and quote:** README Test and build: **“The browser suite covers desktop and 390 px layouts, keyboard routing and focus, axe checks, legal status codes, a designed 404, offline reload, privacy, rate limits, full-board deletion, and the sample workflow.”** (32 words.)

**Why:** it combines many independent ideas and exceeds the hard 22-word copy limit.

**Fix:** replace it with: **“The browser tests check desktop and 390 px layouts, keyboard routing, focus, accessibility, legal routes, 404, and offline reload. They also check privacy, rate limits, deletion, and the sample workflow.”**

### F-1-3 — Minor — README uses unexplained test jargon

**Location and quote:** README Test and build: **“npm run test:claims -- --grep @claim:demo-isolation runs one manifest claim from a clean server.”**

**Why:** “manifest claim” has no plain meaning for a reader. They must infer that this runs a named verification check.

**Fix:** replace it with: **“npm run test:claims -- --grep @claim:demo-isolation runs the named demo-isolation check on a fresh local server.”**

### F-1-4 — Minor — landing footer makes an unlisted claim

**Location and quote:** landing footer: **“v0.1.0 · Original AI-assisted environmental art”**.

**Why:** this visitor-facing provenance claim has no matching entry or observable test in .factory/claims.json. The claims contract requires untestable claims to leave visitor copy. Required asset provenance can remain in the design document.

**Fix:** remove **“Original AI-assisted environmental art”** from the public footer, or add a meaningful observable provenance claim and test.

## Copy audit

Counts treat hyphenated terms and 6–12 as one word. Navigation labels, headings, buttons, captions, and footer copy are included. The four entries marked with a finding are the only flags.

### Landing page

| Copy unit | Words | Result |
|---|---:|---|
| Math Circle Board | 3 | Pass |
| Demo | 1 | Pass |
| Product preview | 2 | Pass |
| How it works | 3 | Pass |
| Privacy | 1 | Pass |
| For volunteer math circle facilitators | 5 | Pass |
| Plan and record small math-circle sessions | 6 | Pass |
| Sequence open problems, record partial attempts, keep private notes, and print a recap for 6–12 learners. | 16 | Pass |
| Try it with sample data | 5 | Pass |
| See a filled board. | 5 | Pass |
| Changes stay in this demo. | 5 | Pass |
| Sign in with Microsoft | 4 | Pass |
| Set up a board with the adult owner code. | 9 | Pass |
| Private boards require the owner’s Microsoft sign-in. | 7 | Pass — owner-access |
| Sample mode reloads offline after its first visit. | 8 | Pass — offline-reload |
| All current board tools are free. | 6 | Pass — release-scope |
| Sample problem cards on a facilitator’s table. | 7 | Pass |
| See one session at a glance | 6 | Pass |
| The board keeps the problem order, learner status, and next discussion prompt together. | 13 | Pass — attempt-record |
| Open this sample board | 4 | Pass |
| Invariants in motion · Aug 29 | 5 | Pass |
| The coin trail | 3 | Pass |
| 2 of 3 attempts recorded | 5 | Pass |
| Corner cuts | 2 | Pass |
| 1 of 3 attempts recorded | 5 | Pass |
| Switching lamps | 2 | Pass |
| No attempts yet | 3 | Pass |
| Run the session in three steps | 6 | Pass |
| Sequence problems | 2 | Pass |
| Add prompts in the order you plan to discuss them. | 10 | Pass |
| Record attempts | 2 | Pass |
| Save partial ideas, strategy tags, and a private facilitator note. | 10 | Pass — attempt-record |
| Print the recap | 3 | Pass |
| Make a session record that leaves private notes out. | 9 | Pass — recap-privacy |
| Privacy and limits | 3 | Pass |
| Keep only the record you need | 6 | **F-1-1** |
| Use aliases instead of learner emails. | 6 | Pass — owner-access |
| The board is not a gradebook, chat service, or public learner profile. | 12 | Pass |
| Read the privacy details | 4 | Pass |
| Release scope | 2 | Pass |
| This release is for one private circle | 8 | Pass — release-scope |
| It has no paid plan, checkout, organization controls, or extra storage tier. | 12 | Pass — release-scope |
| Use four free strategy prompts | 5 | Pass — strategy-palette |
| Plan and record small math-circle sessions. | 6 | Pass |
| Terms | 1 | Pass |
| Built by Param Factory | 3 | Pass |
| v0.1.0 · Original AI-assisted environmental art | 6 | **F-1-4** |

### README prose

Command blocks and configuration keys are code, not sentences. Every prose sentence and explanatory list item follows.

| Sentence or explanatory item | Words | Result |
|---|---:|---|
| Plan and record small math-circle sessions. | 6 | Pass |
| The board is for volunteer facilitators working with private groups of 6–12 learners. | 13 | Pass |
| Facilitators can sequence open problems, record partial attempts and strategy tags, keep private notes, and print a recap. | 18 | Pass |
| The roster uses learner aliases and does not ask for learner email addresses. | 13 | Pass |
| The board is not a gradebook, chat service, or public learner profile. | 12 | Pass |
| Try the isolated sample board. | 5 | Pass |
| It works without an account, uses only demo: session-storage keys, and never calls the private board API. | 17 | Pass |
| Reset restores the original two sessions, three learners, four problems, and four attempts. | 13 | Pass |
| Requirements: Node 22+, stable Rust, and SQLite support. | 8 | Pass |
| The server starts with no required environment variables. | 8 | Pass |
| It stores SQLite data and uploads in ./data locally and /data in the deployed container. | 14 | Pass |
| The server creates a 48-character adult setup code in ./data/owner-invite.txt on first boot. | 12 | Pass |
| Give that code only to the adult who will own the board. | 12 | Pass |
| Microsoft Entra handles sign-in; private board API routes reject anonymous access. | 11 | Pass |
| PORT — HTTP port, default 8080. | 5 | Pass |
| DATA_DIR — persistent SQLite and upload directory, default ./data. | 8 | Pass |
| DIST_DIR — built frontend directory, default ./dist. | 6 | Pass |
| MCB_OWNER_INVITE — optional adult setup code override. | 7 | Pass |
| ENTRA_TENANT_ID, ENTRA_TENANT_SUBDOMAIN, ENTRA_CLIENT_ID — optional Microsoft identity overrides. | 10 | Pass |
| npm run test:e2e builds the frontend, starts an isolated test server and SQLite directory, runs Playwright 1.58.2, then removes the test data. | 19 | Pass |
| npm run test:claims -- --grep @claim:demo-isolation runs one manifest claim from a clean server. | 15 | **F-1-3** |
| npm run test:cold-claims clones the committed checkout, installs dependencies, creates an empty Cargo target, and runs every declared claim command. | 21 | Pass |
| This covers first-run backend compilation separately from the bounded server-start deadline. | 11 | Pass |
| The browser suite covers desktop and 390 px layouts, keyboard routing and focus, axe checks, legal status codes, a designed 404, offline reload, privacy, rate limits, full-board deletion, and the sample workflow. | 32 | **F-1-2** |
| Every public claim and its clean-clone command are listed in .factory/claims.json. | 10 | Pass |
| The multi-stage image uses the moving stable Rust toolchain, runs as a non-root user, serves PORT, and persists records under /data. | 21 | Pass |
| /health returns the build SHA. | 5 | Pass |
| Private board data requires the signed-in owner. | 7 | Pass — owner-access |
| The public landing page and sample flow load no analytics, ads, remote fonts, or third-party runtime scripts. | 17 | Pass — no-tracking |
| Read routes use a 40-request burst and writes use an 8-request burst; limited responses return 429 with Retry-After. | 18 | Pass — rate-limits |
| Settings exports the board record as JSON. | 7 | Pass — json-export |
| A private facilitator note is not included in the printable recap. | 11 | Pass — recap-privacy |
| The owner can delete individual records or the complete private board. | 11 | Pass — full-delete |
| All current board tools are free, including four reusable strategy prompts. | 11 | Pass — release-scope and strategy-palette |
| This release has no paid plan, checkout, organization controls, or extra storage tier. | 13 | Pass — release-scope |
| See /privacy, /terms, and the scope decision. | 7 | Pass |
| Visual direction and original-asset provenance are in .factory/design.md. | 7 | Pass |
| Licensed under the MIT License; see LICENSE. | 7 | Pass |

## Demo and sandbox

- The landing action opens an already-filled Saturday Problem Circle board.
- The persistent banner says **“Demo — sample data, nothing is saved”** and includes **Reset demo** and **Start for real**.
- A fresh browser context showed only demo:math-circle-board:board in session storage. The isolation test edits a learner, resets it away, and asserts no API request.
- A fresh 390 px context received service-worker control, went offline, reloaded, and retained the banner and filled board.
- Request capture across landing, demo entry, and demo navigation saw only the product origin.

## Claims gate

All eleven declared commands completed through npm run test:cold-claims from a temporary clean clone: demo-isolation, attempt-record, recap-privacy, json-export, offline-reload, no-tracking, owner-access, rate-limits, release-scope, strategy-palette, and full-delete.

The local browser suite passed all 19 tests, including every claim tag, accessibility, legal and 404 routes, mobile controls, routing, focus, and offline behavior. F-1-4 is the sole unlisted public claim found.

## Structure and history

- Root, demo, privacy, and terms returned 200. The designed missing route returned HTTP 404 and supplied return actions. All public anchor targets returned 200.
- Public, demo, privacy, terms, 404, and demo sub-routes each had one h1, a description, route title, and canonical link. Root supplied OG/Twitter metadata, icon assets, robots, sitemap, and manifest.
- Skip link, route focus, browser back, 44 px mobile controls, reduced motion, and serious/critical axe checks passed. Normal public, demo, and legal loads had no script console errors.
- Live headers include HSTS, nosniff, frame denial, same-origin referrer policy, permissions policy, and response-header CSP with frame-ancestors none.
- No prior review or polish files exist. I read every verification and handoff record. Their previously reported rate limits, Entra sign-in, claims/demo, legal/accessibility/routing, full deletion, upload/mobile, and paid-tier defects all remain fixed, confirmed by the current code and tests. None regressed.

## Missed leverage

No additional AI, import, export, or sync feature is implied strongly enough to require it. JSON export, printable recap, session sequencing, partial-attempt records, and strategy prompts cover the facilitator job. Adding AI would send sensitive learner-thinking data without improving the core workflow.

## What would make this perfect

Repair F-1-1 through F-1-4, rerun the copy audit and clean-clone claims, then repeat the 390 px cold read. That would leave a clear first screen, an honest demo, and complete observable claim coverage.

