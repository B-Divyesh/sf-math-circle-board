# Math Circle Board

Plan and record small math-circle sessions. The board is for volunteer facilitators working with private groups of 6–12 learners.

Facilitators can sequence open problems, record partial attempts and strategy tags, keep private notes, and print a recap. The roster uses learner aliases and does not ask for learner email addresses. The board is not a gradebook, chat service, or public learner profile.

[Try the isolated sample board](https://math-circle-board.sociobot.in/demo). It works without an account, uses only `demo:` session-storage keys, and never calls the private board API. Reset restores the original two sessions, three learners, four problems, and four attempts.

## Run locally

Requirements: Node 22+, stable Rust, and SQLite support.

```sh
npm install
npm run build
cargo run
```

Open `http://localhost:8080`. The server starts with no required environment variables. It stores SQLite data and uploads in `./data` locally and `/data` in the deployed container.

The server creates a 48-character adult setup code in `./data/owner-invite.txt` on first boot. Give that code only to the adult who will own the board. Microsoft Entra handles sign-in; private board API routes reject anonymous access.

Optional configuration:

- `PORT` — HTTP port, default `8080`.
- `DATA_DIR` — persistent SQLite and upload directory, default `./data`.
- `DIST_DIR` — built frontend directory, default `./dist`.
- `MCB_OWNER_INVITE` — optional adult setup code override.
- `ENTRA_TENANT_ID`, `ENTRA_TENANT_SUBDOMAIN`, `ENTRA_CLIENT_ID` — optional Microsoft identity overrides.

## Test and build

```sh
npm ci
npm test
npm run build
cargo build --release
npm run test:e2e
npm run test:cold-claims
```

`npm run test:e2e` builds the frontend, starts an isolated test server and SQLite directory, runs Playwright 1.58.2, then removes the test data. `npm run test:claims -- --grep "@claim:demo-isolation"` runs one manifest claim from a clean server.

`npm run test:cold-claims` clones the committed checkout, installs dependencies, creates an empty Cargo target, and runs every declared claim command. This covers first-run backend compilation separately from the bounded server-start deadline.

The browser suite covers desktop and 390 px layouts, keyboard routing and focus, axe checks, legal status codes, a designed 404, offline reload, privacy, rate limits, full-board deletion, and the sample workflow. Every public claim and its clean-clone command are listed in [`.factory/claims.json`](.factory/claims.json).

## Container deployment

```sh
docker build --build-arg BUILD_SHA="$(git rev-parse HEAD)" -t math-circle-board .
docker run --rm -p 8080:8080 -v mcb-data:/data math-circle-board
```

The multi-stage image uses the moving stable Rust toolchain, runs as a non-root user, serves `PORT`, and persists records under `/data`. `/health` returns the build SHA.

## Privacy, limits, and price

Private board data requires the signed-in owner. The public landing page and sample flow load no analytics, ads, remote fonts, or third-party runtime scripts. Read routes use a 40-request burst and writes use an 8-request burst; limited responses return `429` with `Retry-After`.

Settings exports the board record as JSON. A private facilitator note is not included in the printable recap. The owner can delete individual records or the complete private board.

All current board tools are free, including four reusable strategy prompts. This release has no paid plan, checkout, organization controls, or extra storage tier. See [`/privacy`](https://math-circle-board.sociobot.in/privacy), [`/terms`](https://math-circle-board.sociobot.in/terms), and the [scope decision](.factory/scope-deviation.md).

Visual direction and original-asset provenance are in [`.factory/design.md`](.factory/design.md). Licensed under the MIT License; see [`LICENSE`](LICENSE).
