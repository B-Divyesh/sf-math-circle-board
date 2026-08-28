# Math Circle Board — repair handoff

## Release repair

This repair addresses every failure in the independent verification report for
candidate `16daa451f39a897929e0094725ec5623b17022a3`.

- A fresh deployment is no longer publicly claimable. On first boot the server
  creates a CSPRNG 48-character, one-time owner invite at
  `/data/owner-invite.txt` with mode `0600` (or accepts an optional
  installer-supplied `MCB_OWNER_INVITE`). It is never served or logged. Setup
  requires that code plus an adult-responsibility confirmation, and removes the
  generated file after the first successful claim. The deployment operator must
  transfer the code only through an authenticated adult support channel.
- Session creation and deletion cookies now carry `HttpOnly; Secure;
  SameSite=Strict`; all responses set two-year HSTS with subdomains.
- Session dates are checked as real Gregorian `YYYY-MM-DD` calendar dates.
- Uploads are decoded from their bytes and accepted only as valid JPEG, PNG, or
  WebP; the served MIME value comes from the decoded format, not multipart
  metadata.
- Authenticated board data is retained in local storage for offline reloads;
  drafts remain local and sign-out clears the cached board. The privacy notice
  states this explicitly. The PWA cache version was advanced to `v2`.
- Hashed `/assets/` responses now use `Cache-Control: public,
  max-age=31536000, immutable`.
- TypeScript is repaired and enforced through `npm run typecheck`, which is
  part of `npm test`.

## Regression coverage

`src/main.rs` contains endpoint-level regression coverage that proves a wrong
adult code is forbidden, a correct setup response has a secure cookie, an
impossible date is rejected, forged `image/png` multipart content is neither
accepted nor stored, HSTS is present, and a hashed asset is immutable-cached.
It also unit-tests leap dates and byte-level image detection. Playwright covers
the protected setup, keyboard skip link, facilitator workflow, 390 px sign-in,
390 px authenticated offline reload, privacy/terms, console errors, and Axe
serious/critical violations.

## Verification run — 2026-08-28

All commands below passed in this repair workspace.

```sh
npm ci
npm test
cargo fmt --check
cargo clippy --all-targets -- -D warnings
npm run build
cargo build --release
MCB_TEST_OWNER_CODE=adult-setup-code-0123456789 \
  PLAYWRIGHT_BASE_URL=http://127.0.0.1:18081 \
  npm run test:e2e -- --workers=1
```

Results:

- `npm ci`: 58 packages installed; 0 vulnerabilities.
- `npm test`: TypeScript check, 3 Vitest tests, and 4 Rust tests passed.
- Production build: `dist/` emitted; JavaScript is 28.28 KB raw / 9.92 KB
  gzip and CSS is 19.86 KB raw / 5.20 KB gzip.
- Release binary starts with only `PORT`: `/health` returned
  `{"build":"development","ok":true}` and the generated owner invite was
  mode `600`; no secret value was logged.
- Playwright 1.58.2: 3/3 passed. This includes desktop, 390 px mobile,
  keyboard, authenticated cached offline reload, and Axe (zero serious or
  critical violations in the authenticated recap).
- Local mobile Lighthouse: Performance 100, Accessibility 100, Best Practices
  100, SEO 100; LCP 0.3 s and CLS 0.
- Response-policy smoke confirmed HSTS, CSP, nosniff, frame denial, same-origin
  referrer policy, and immutable hashed-asset caching. An untrusted-origin
  `OPTIONS /api/login` returned `405` with no CORS grant.
- 100 concurrent local `/health` requests completed successfully.

## Run and deployment

```sh
npm ci && npm run build
cargo run
```

The image remains the original multi-stage non-root Rust/Axum + Vite container
and listens on `PORT` (default `8080`). With no optional environment variables,
the adult setup code is generated under the persistent `/data` directory; read
it only via the deployment operator’s authenticated control plane and provide
it to the verified adult owner. See `README.md` for a reproducible local
browser command and the optional installer override.

## Known operational note

The product deliberately cannot infer adulthood from a name or checkbox. The
one-time deployment-held code prevents anonymous first-visitor takeover; the
factory operator must complete the human verification and secure code transfer
when provisioning the private group. No payment, analytics, remote font, or
other third-party tracking was added.
