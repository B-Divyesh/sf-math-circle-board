# Sample demo

- Entry point and canonical URL: `https://math-circle-board.sociobot.in/demo` (local: `http://localhost:8080/demo`). `?demo=1` remains a supported direct-entry URL and canonicalizes to `/demo`.
- Setup: none. The first rendered screen is the working board.
- Sample: Saturday Problem Circle with two dated sessions, three learner aliases, four open problems, and four partial/shared attempts. Attempts include strategies and facilitator-only notes.
- Isolation: the browser adapter handles every board operation. It never calls `/api/*`. State uses only the `demo:math-circle-board:*` namespace in `sessionStorage`; it cannot read or write the real SQLite board.
- Reset: use **Reset demo** in the persistent yellow banner or in Settings. This replaces all demo changes with the shipped sample.
- Exit: use **Start for real**. It clears all `demo:math-circle-board:*` keys and returns to the signed-out landing page.
- Offline: after the service worker controls the page, the sample shell and session-storage data reload offline.

Run the isolation regression from a clean checkout after `npm ci`:

```sh
npm run test:claims -- --grep @claim:demo-isolation
```
