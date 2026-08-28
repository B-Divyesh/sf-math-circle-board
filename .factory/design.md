# Math Circle Board — visual thesis

## Direction: cinematic environmental art — “the lantern room”

The board should feel like a quiet room just before a good problem is solved: blue dusk outside, warm lamplight across a slate table, index cards and pencil marks accumulating into a path. This is environmental rather than character-led; it protects children’s privacy and makes the facilitator’s work the hero. The atmosphere is focused, humane, and gently adventurous—not scholastic software and not a generic dashboard.

The product opens with one panoramic original image of a lamplit mathematics table. In the working interface, that world is abstracted into pools of warm paper over an ink-blue field, thin topographic “reasoning trails,” and strategy tags shaped like field labels. Decoration only establishes place or explains continuity; the actual cards remain calm and legible.

## Palette

- `night` #101D2A — painted page background; the room at blue hour.
- `night-raised` #172A3A — navigation and recessed controls.
- `paper` #F5F0E4 — primary working surface, like warm stock under a lamp.
- `paper-deep` #E8DECA — secondary surfaces and quiet separators.
- `ink` #16232B — primary text on paper (contrast > 13:1).
- `mist` #AFC2C8 — secondary text on night (contrast > 7:1).
- `ember` #D36B3F — primary action and active trail; white is not used on it because dark ink gives stronger contrast.
- `lantern` #F3C969 — attention and selected states, always paired with text/icon.
- `moss` #39705A — success/completed, paired with a check or label.
- `danger` #A53E42 — destructive state; on paper with explicit warning text.

This is an explicitly dark environmental shell with light work surfaces; a separate theme switch would weaken the “blue-hour room” metaphor. Every reading surface is painted explicitly, and both surface families meet WCAG AA.

## Type and spacing

No remote fonts. Headings use Georgia (a bookish, exploratory voice); interface and body use the native system stack for speed and crisp small text. The scale is 14 / 16 / 19 / 24 / 34 / 52 px. Body never drops below 16 px. Long-form copy is capped at 68 characters per line with 1.55 leading. Tabular numbers use `font-variant-numeric: tabular-nums`.

Spacing follows a 4/8 rhythm: 4, 8, 12, 16, 24, 32, 48, 64. Independent problems earn card boundaries; controls inside a problem are grouped by proximity rather than nested cards. Touch targets are at least 44×44 px.

## Interaction grammar

- The current session is a horizontal “trail”: numbered problem cards form a clear sequence; on phones it becomes a vertical path.
- Selecting a learner changes a single shared workbench instead of opening many modal layers.
- Status changes are described with icon + words (`○ Not started`, `◐ Exploring`, `✓ Shared`).
- Saving gives immediate inline confirmation. Destructive deletion requires a named confirmation.
- Empty states show one concrete next move. Offline state keeps drafts and says exactly what will retry.
- Keyboard: skip link, native controls, ordered tab path, Escape closes dialogs, focus returns to the originating control.

## Motion

Cards enter from the prior position along the trail at 180–240 ms using opacity and a small transform. Toasts rise 8 px, and selected states cross-fade. Nothing loops. Under `prefers-reduced-motion: reduce`, transforms and smooth scrolling are removed and feedback becomes instantaneous opacity/state changes.

## Original asset plan and provenance

Hero: a wide, unoccupied attic mathematics room at blue hour; slate table, paper problem cards, amber task lamp, chalk arcs and geometric shadows, distant night sky. The room conveys collective exploration without depicting or identifying children.

Prompt sheet: “Cinematic environmental concept art of a quiet attic mathematics workshop at blue hour, unoccupied, long slate table with scattered cream index cards and pencils, a single amber task lamp illuminating hand-drawn geometric diagrams, subtle chalk arcs and number patterns, deep ink-blue walls, rain-soft window light, warm ember and muted moss accents, tactile paper and wood, 35mm lens, low eye level, volumetric but restrained light, detailed painterly realism, generous negative space on the left for interface copy. No people, no faces, no text, no letters, no watermark, no logos, no brands, no screens, no glossy corporate office, no neon gradient.”

Generation: Azure AI Foundry image generation via `/opt/fleet/lib/gen-image.sh`, deployment `factory-image`, 2026-08-28. Generated imagery is original for this product. Source PNG and exact prompt sidecar live in `assets/src/`; optimized WebP/AVIF derivatives ship in the frontend. The footer discloses AI-assisted original environmental art.
