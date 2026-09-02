# Adversarial first-read review 2 — Math Circle Board

**Reviewed:** 2026-09-02<br>
**Live URL:** https://math-circle-board.sociobot.in<br>
**Live build:** aff4299a57c9e74fe042ca5f1a58bc0e37a8e2a2<br>
**Verdict:** **FAIL**

The first-read, demo, declared-claim, and structure checks pass. This review fails because four visitor-facing promises have no entry and no matching tagged observable test in .factory/claims.json. Acceptance requires zero findings.

## Cold first read

Fresh Chromium contexts loaded the live root at 390×844 and 1440×900 without horizontal overflow or console errors. Before scrolling, the page answered:

- **What it does:** “Plan and record small math-circle sessions.” It sequences problems, records partial attempts, keeps private notes, and prints a recap.
- **For whom:** “For volunteer math circle facilitators,” with “6–12 learners” in the supporting sentence.
- **What to click first:** **Try it with sample data**. Its adjacent text says “See a filled board. Changes stay in this demo.”

The 390 px action was visible at y=585 with a 346×44 px target. The mobile page was 390/390 px and desktop 1440/1440 px (client/scroll width). The lantern-room art, dark slate shell, warm paper work surfaces, and trail-like problem sequence are product-specific rather than generic SaaS styling.

## Findings

### F-2-1 — High — private-board offline cache promise is not a declared claim

**Location and quote:** /privacy, “The last opened board and unsaved drafts may stay in this browser for offline use. Signing out clears the cached board.”

**Why:** offline-reload promises and tests only **sample mode**. No manifest entry or tagged test verifies an authenticated private-board cache, draft retention, or cache removal on sign-out. A facilitator may rely on this when deciding whether to put private learner thinking on the device.

**Fix:** replace the statements with the already-tested sample-only statement, or add private-cache-lifecycle with a tagged test that creates an authenticated board, checks documented cache/draft behavior, signs out, and asserts the documented keys are removed.

### F-2-2 — High — account-record privacy statement is not a declared claim

**Location and quote:** /privacy, “Microsoft handles the adult sign-in. The board stores the account identifier, not the account password.”

**Why:** owner-access checks anonymous API rejection, sign-in controls, aliases, and absent email/password fields. It does not inspect what the server persists or prove it never persists a password. This is a specific security and privacy statement without a claim entry.

**Fix:** add identity-records with a tagged isolated-auth test that inspects the created SQLite record and asserts identifier-only storage with no password value; or remove the storage-model sentence from public copy.

### F-2-3 — High — private-photo type and size promise is not a declared claim

**Location and quote:** demo attempt workbench, “Private images of paper work only. JPEG, PNG, or WebP under 5 MB.”

**Why:** no claim entry covers private uploads, formats, or the 5 MB boundary. The untagged regression checks corrupt bytes and one valid WebP; it does not prove the stated size boundary or the privacy/access property.

**Fix:** add photo-upload-limits with a tagged test accepting each listed format below 5 MB, rejecting an over-limit/non-image file, checking recovery copy, and confirming anonymous retrieval fails. Otherwise remove the format, size, and privacy promises.

### F-2-4 — Medium — individual-deletion promise is not a declared claim

**Location and quote:** README: “The owner can delete individual records or the complete private board.” /privacy says facilitators can “remove photos and learners, delete sessions, or delete the entire private board.”

**Why:** full-delete tests only deletion of the complete board. No declared claim tests deletion of an individual photo, learner, or session, despite these being presented as data controls.

**Fix:** add individual-delete with a tagged authenticated test that creates and removes each offered individual record, asserting its API/storage and UI absence; or narrow the copy to complete-board deletion.

## Copy audit

Counts treat hyphenated compounds and 6–12 as one word. Navigation labels, headings, actions, captions, alt text, and footer copy are included. No item exceeds 22 words or uses banned marketing language, unexplained jargon, inconsistent core terms, mood headings, or non-result-naming actions. The claim-contract issues above are the only flags.

### Landing page

| Copy unit | Words | Result |
|---|---:|---|
| Skip to main content | 4 | Pass |
| Math Circle Board | 3 | Pass |
| Demo | 1 | Pass |
| Product preview | 2 | Pass |
| How it works | 3 | Pass |
| Privacy | 1 | Pass |
| For volunteer math circle facilitators | 5 | Pass |
| Plan and record small math-circle sessions | 6 | Pass |
| Sequence open problems, record partial attempts, keep private notes, and print a recap for 6–12 learners. | 16 | Pass — learner/attempt/recap claims |
| Try it with sample data | 5 | Pass — demo isolation |
| See a filled board. | 5 | Pass — demo isolation |
| Changes stay in this demo. | 5 | Pass — demo isolation |
| Sign in with Microsoft | 4 | Pass — owner access |
| Set up a board with the adult owner code. | 9 | Pass — first boot |
| Private boards require the owner’s Microsoft sign-in. | 7 | Pass — owner access |
| Sample mode reloads offline after its first visit. | 8 | Pass — offline reload |
| All current board tools are free. | 6 | Pass — release scope |
| An empty mathematics workshop with problem cards beneath a desk lamp | 10 | Pass — descriptive alt |
| Sample problem cards on a facilitator’s table. | 7 | Pass |
| See one session at a glance | 6 | Pass |
| The board keeps the problem order, learner status, and next discussion prompt together. | 13 | Pass — attempt record |
| Open this sample board | 4 | Pass — demo isolation |
| Invariants in motion · Aug 29 | 5 | Pass — sample label |
| The coin trail | 3 | Pass — sample label |
| 2 of 3 attempts recorded | 5 | Pass — sample label |
| Corner cuts | 2 | Pass — sample label |
| 1 of 3 attempts recorded | 5 | Pass — sample label |
| Switching lamps | 2 | Pass — sample label |
| No attempts yet | 3 | Pass — sample label |
| Run the session in three steps | 6 | Pass |
| Sequence problems | 2 | Pass |
| Add prompts in the order you plan to discuss them. | 10 | Pass |
| Record attempts | 2 | Pass |
| Save partial ideas, strategy tags, and a private facilitator note. | 10 | Pass — attempt record |
| Print the recap | 3 | Pass |
| Make a session record that leaves private notes out. | 9 | Pass — recap privacy |
| Privacy and limits | 3 | Pass — informative section name |
| Data kept on the board | 5 | Pass — informative section name |
| Use aliases instead of learner emails. | 6 | Pass — owner access |
| The board is not a gradebook, chat service, or public learner profile. | 12 | Pass — product boundary |
| Read the privacy details | 4 | Pass |
| Release scope | 2 | Pass — informative section name |
| This release is for one private circle | 8 | Pass — release scope |
| It has no paid plan, checkout, organization controls, or extra storage tier. | 12 | Pass — release scope |
| Use four free strategy prompts | 5 | Pass — strategy palette |
| Plan and record small math-circle sessions. | 6 | Pass |
| Terms | 1 | Pass |
| Built by Param Factory | 4 | Pass |
| v0.1.0 | 1 | Pass |

### README

Command blocks are code rather than sentences. Configuration entries are included as explanatory items.

| Sentence or explanatory item | Words | Result |
|---|---:|---|
| Plan and record small math-circle sessions. | 6 | Pass |
| The board is for volunteer facilitators working with private groups of 6–12 learners. | 13 | Pass — learner range |
| Each board accepts up to 12 learner aliases. | 8 | Pass — learner range |
| Facilitators can sequence open problems, record partial attempts and strategy tags, keep private notes, and print a recap. | 18 | Pass — attempt/recap claims |
| The roster uses learner aliases and does not ask for learner email addresses. | 13 | Pass — owner access |
| The board is not a gradebook, chat service, or public learner profile. | 12 | Pass |
| Try the isolated sample board. | 5 | Pass — demo isolation |
| It works without an account, uses only demo: session-storage keys, and never calls the private board API. | 17 | Pass — demo isolation |
| Reset restores the original two sessions, three learners, four problems, and four attempts. | 13 | Pass — sample counts |
| Requirements: Node 22+, stable Rust, and SQLite support. | 8 | Pass |
| Open http://localhost:8080. | 2 | Pass |
| The server starts with no required environment variables. | 8 | Pass — first boot |
| It stores SQLite data and uploads in ./data locally and /data in the deployed container. | 14 | Pass — runtime claims |
| The server creates a 48-character adult setup code in ./data/owner-invite.txt on first boot. | 12 | Pass — first boot |
| Give that code only to the adult who will own the board. | 12 | Pass |
| Microsoft Entra handles sign-in; private board API routes reject anonymous access. | 11 | Pass — owner access |
| PORT — HTTP port, default 8080. | 5 | Pass |
| DATA_DIR — persistent SQLite and upload directory, default ./data. | 8 | Pass |
| DIST_DIR — built frontend directory, default ./dist. | 6 | Pass |
| MCB_OWNER_INVITE — optional adult setup code override. | 7 | Pass |
| ENTRA_TENANT_ID, ENTRA_TENANT_SUBDOMAIN, ENTRA_CLIENT_ID — optional Microsoft identity overrides. | 10 | Pass |
| npm run test:e2e builds the frontend, starts an isolated test server and SQLite directory, runs Playwright 1.58.2, then removes the test data. | 19 | Pass |
| npm run test:claims -- --grep @claim:demo-isolation runs the named demo-isolation check on a fresh local server. | 16 | Pass |
| npm run test:cold-claims clones the committed checkout, installs dependencies, creates an empty Cargo target, and runs every declared claim command. | 21 | Pass |
| This covers first-run backend compilation separately from the bounded server-start deadline. | 11 | Pass |
| The browser tests check desktop and 390 px layouts, keyboard routing, focus, accessibility, legal routes, 404, and offline reload. | 19 | Pass |
| They also check privacy, rate limits, deletion, and the sample workflow. | 11 | Pass |
| Every public claim and its clean-clone command are listed in .factory/claims.json. | 12 | Pass |
| The multi-stage image uses the moving stable Rust toolchain, runs as a non-root user, serves PORT, and persists records under /data. | 21 | Pass — container runtime |
| /health returns the build SHA. | 5 | Pass — container runtime |
| Private board data requires the signed-in owner. | 7 | Pass — owner access |
| The public landing page and sample flow load no analytics, ads, remote fonts, or third-party runtime scripts. | 17 | Pass — no tracking |
| Read routes use a 40-request burst and writes use an 8-request burst; limited responses return 429 with Retry-After. | 18 | Pass — rate limits |
| Settings exports the board record as JSON. | 7 | Pass — JSON export |
| A private facilitator note is not included in the printable recap. | 11 | Pass — recap privacy |
| The owner can delete individual records or the complete private board. | 11 | F-2-4 |
| All current board tools are free, including four reusable strategy prompts. | 11 | Pass — scope/strategy claims |
| This release has no paid plan, checkout, organization controls, or extra storage tier. | 13 | Pass — release scope |
| See /privacy, /terms, and the scope decision. | 7 | Pass |
| Visual direction and original-asset provenance are in .factory/design.md. | 7 | Pass |
| Licensed under the MIT License; see LICENSE. | 7 | Pass |

## Demo and sandbox

From a fresh 390 px context, the landing action opened /board?demo=1 immediately with the filled Saturday Problem Circle: two sessions, three aliases, four shipped problems, four attempts, strategy chips, and private facilitator notes. The persistent banner read **“Demo — sample data, nothing is saved”** and exposed **Reset demo** and **Start for real**.

Only demo:math-circle-board:board appeared in session storage; no demo or offline-board key appeared in local storage. Reset restored Ada and the shipped sample. Start for real returned to / and cleared demo storage. Request capture across landing, demo entry, and demo navigation contained only the product origin; no /api/* request occurred in demo mode.

## Claims gate

npm run test:cold-claims completed from a temporary clean clone and ran every manifest command separately. All 15 declared claims passed: demo-isolation, sample-counts, learner-range, attempt-record, recap-privacy, json-export, offline-reload, no-tracking, owner-access, first-boot-runtime, container-runtime, rate-limits, release-scope, strategy-palette, and full-delete.

This is not a failed declared test. F-2-1 through F-2-4 are observable promises that the manifest does not declare, leaving no sandbox test to prove them.

## Structure, accessibility, and routing

- /, /demo, /privacy, and /terms returned 200; the unknown route returned a designed 404 with return-home and sample actions. Every discovered same-origin non-fragment link returned 200.
- Root, demo, privacy, terms, and 404 had one h1, route-specific title, description, canonical, OG title, consistent header/footer, and legal links. Root has favicon, apple-touch icon, social card, robots, and sitemap.
- /demo sets **Demo — Math Circle Board** and canonical /demo. Demo subroutes, browser Back, and keyboard navigation restored state; client route changes focused the h1.
- The live 390 px public/app navigation did not clip. Local regression covers 44 px controls, skip link, focus, back navigation, reduced motion, and serious/critical axe results.
- Normal live root/demo loads had no console errors. The intentional missing route produces only the expected HTTP 404 network error. Headers include response-header CSP with frame-ancestors 'none', HSTS, nosniff, frame denial, same-origin referrer policy, and restrictive permissions policy.

## Earlier findings and history

I read review-1.md, polish-1.md, all prior verification records, and the prior handoff. Every earlier review finding was checked live and in source:

| Earlier id | Current verification |
|---|---|
| F-1-1 | Heading is **Data kept on the board**; the slogan is absent. Fixed. |
| F-1-2 | README now has two short browser-test sentences (19 and 11 words). Fixed. |
| F-1-3 | README says “named demo-isolation check on a fresh local server.” Fixed. |
| F-1-4 | Footer is Built by Param Factory · v0.1.0; public AI-art provenance is absent. Fixed. |

No regression of those four items was found. F-2-1 through F-2-4 are separate claims-contract gaps on the privacy and working-board screens.

## Missed leverage

No additional AI, import, export, or sync is clearly implied by the brief. JSON export, printable recap, problem sequencing, attempt records, and strategy prompts cover the stated facilitator workflow. AI would transmit sensitive learner-thinking content without being needed for the core job.

## What would make this perfect

Declare and test the four existing privacy/upload/deletion promises, or remove them and retain only claims the clean sandbox can prove. Then rerun the clean-clone claims and this full cold-read checklist. The first screen, demo, identity, routing, and declared behavior otherwise support a PASS.
