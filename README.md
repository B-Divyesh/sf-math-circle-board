# Math Circle Board

Math Circle Board is a private, adult-owned workspace for volunteer facilitators of small exploratory math circles. It sequences open-ended problems, records each learner’s partial thinking and strategy tags, keeps facilitator-only notes and paper-work photos, and produces a printable session recap in minutes.

The product is intentionally not a classroom LMS, gradebook, public child profile, or chat network. Learner aliases are sufficient; no learner email is requested.

## Run locally

Requirements: Node 22+, Rust 1.89+, and SQLite support.

```sh
npm install
npm run build
cargo run
```

Open `http://localhost:8080`. The server runs with no required environment variables and creates `./data/board.db` plus `./data/uploads/` on first boot. Optional configuration:

- `PORT` — HTTP port, default `8080`
- `DATA_DIR` — persistent SQLite/upload directory, default `./data`
- `DIST_DIR` — built frontend directory, default `./dist`

For live frontend development, run `cargo run` and `npm run dev` in separate terminals, then open Vite’s URL.

## Test and build

```sh
npm test          # Vitest unit tests + Rust tests
npm run build     # reproducible frontend output in dist/
npm run test:e2e  # with the app running on http://127.0.0.1:4173
```

The end-to-end suite covers initial adult setup, a complete session/learner/attempt/recap path, mobile sign-in, legal pages, and serious/critical axe violations.

## Container deployment

```sh
docker build -t math-circle-board .
docker run --rm -p 8080:8080 -v mcb-data:/data math-circle-board
```

The multi-stage image builds the Vite frontend and Rust server, runs as a non-root user, serves on `PORT`, and persists state in `/data`. Back up that volume and use the in-app JSON export regularly.

## Privacy and billing

All board routes require the facilitator passphrase. Session tokens are random, HttpOnly, and SameSite=Strict; uploaded images are served only after authorization. The app contains no analytics, advertising, remote fonts, or runtime CDN scripts. `/privacy` and `/terms` provide the product policies.

The free board includes the complete core workflow and export. Circle Plus is a $39 one-time license that unlocks a reusable strategy palette and future organization controls through Sociobot’s hosted billing API; no payment provider is embedded here.

Visual direction, asset provenance, and design tokens are in [`.factory/design.md`](.factory/design.md). Licensed under the MIT License; see [`LICENSE`](LICENSE).
