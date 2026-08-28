# Math Circle Board — build handoff

## Shipped

- A production Rust/axum service with SQLite persistence, first-run adult ownership, hashed passphrases, random 30-day HttpOnly sessions, secure response headers, structured logs, graceful shutdown, and `/health` build metadata.
- A complete private facilitator flow: learner aliases, dated sessions, ordered open problem cards, per-learner status, evolving attempt text, strategy tags, private notes, and up to four protected JPEG/PNG/WebP uploads per attempt.
- A compact printable session recap organized by learner and strategy; private notes never appear in print.
- Data controls: full JSON export including base64 photo payloads; confirmed learner/session/photo deletion; deleted records also remove stored upload files.
- First-class initial, empty, error, offline/draft, saving, and license-invalid states. The interface works at 390 px and with keyboard/native controls.
- Circle Plus paid-unlock integration using the Sociobot checkout/verify contract. The $39 one-time tier adds the reusable strategy palette; core records, print, export, safety, and accessibility remain free. No product ID is hardcoded.
- `/privacy` and `/terms`, PWA shell caching, responsive generated hero art, a non-generic “lantern room” visual system, and full provenance in `.factory/design.md`.
- Multi-stage non-root container packaging; it serves the Vite build and API together on `PORT` (default 8080) and persists under `/data`.

## Run and verify

```sh
npm install
npm test
npm run build
cargo run
```

For the browser suite, run the built app on port 4173 and then:

```sh
npm run test:e2e -- --workers=1
```

Verification completed on 2026-08-28:

- `npm test`: 3 frontend unit tests + 2 Rust tests passed.
- Playwright 1.58.2: 2 end-to-end tests passed, covering setup through recap, private image upload, complete data export, 390 px sign-in, and legal pages.
- Axe in the authenticated recap: zero serious or critical violations.
- `npm run build`: passed; output is exactly `dist/` with `index.html` at its root.
- Production transfer sizes: JS 27.33 KB raw / 9.60 KB gzip; CSS 19.86 KB raw / 5.20 KB gzip; mobile hero 22 KB WebP; desktop hero 63 KB WebP.
- Lighthouse mobile: Performance 100, Accessibility 100, Best Practices 100, SEO 100; LCP 1.7 s, CLS 0, TBT 0 ms.
- Load smoke: 100 concurrent-batch `/health` requests completed successfully in 440 ms locally.
- Runtime start was verified with only `PORT`; both the database/upload location and frontend directory correctly used their defaults.

## Known gaps / next steps

- Docker CLI was not installed in the worker image, so the Dockerfile could not be executed locally (`docker: command not found`). Rust and frontend release inputs were compiled independently and the Docker stages use their lockfiles.
- Hosted checkout and a real paid license require the factory’s later product registration; the expected production Sociobot endpoints are already wired.
- Circle Plus currently unlocks the reusable strategy palette. Organization membership/role controls are intentionally left for a later shared-auth release.
- SQLite is appropriate for one private circle deployment. Multi-organization hosting would require tenant isolation and PostgreSQL.
