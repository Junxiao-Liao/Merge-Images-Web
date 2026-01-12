# MergeImages v2 — Requirements (No PWA)

Version: 0.4 (adaptive output-size guardrails; PWA removed)  
Target hosting: GitHub Pages (Project Pages)  
Runtime: Modern desktop + mobile browsers

## 1. Purpose

MergeImages v2 is a **static, client-only** web application that merges multiple images into a single output image. It must:
- run with **no backend** (all processing on-device),
- be deployable to **GitHub Pages**,
- work on **desktop and mobile** browsers.

Explicit non-goals for v0.4:
- **No PWA**: no Service Worker, no Web App Manifest, no “installable” requirement, and **no offline guarantee** beyond normal browser caching behavior.

## 2. Users and primary use cases

### 2.1 Primary users
- Individuals who need a quick, local tool to combine multiple images (e.g., stitching screenshots, creating a long image for sharing).

### 2.2 Primary workflows (happy path)
1. User opens the app in a browser.
2. User adds images (drag-and-drop or file picker).
3. User reorders images.
4. User selects direction (vertical/horizontal) and background.
5. User runs merge.
6. User previews the output and downloads it.

## 3. Scope

### 3.1 In scope (v0.4)
- Import multiple images
- Reorder inputs
- Merge direction: vertical or horizontal
- Background fill for transparent inputs
- Deterministic merge behavior (see NFR/Testing)
- Output encoding (PNG; optional JPEG if included)
- Preview + download
- GitHub Pages deployment under a base path
- **Adaptive output-size safeguards** tuned for mobile vs desktop (see FR-5)

### 3.2 Out of scope (initial release)
- Any server-side processing, uploads, or accounts
- Cloud storage integrations
- Persistent project/session saving (no local persistence by default)
- Advanced layout features beyond simple vertical/horizontal stacking (e.g., grids, margins, captions)
- Manual scaling modes/toggles (scaling behavior is fixed/automatic)
- Animated output (animated inputs are treated as first-frame only)
- PWA features (offline mode, installability, background sync, push, etc.)

## 4. Functional requirements

### FR-1: Import images
- The app **MUST** allow adding multiple images via:
  - file picker (multi-select), and
  - drag-and-drop onto the app.
- The app **MUST** provide a clear empty state prompting drag-and-drop.
- The app **SHOULD** support “add more” after initial import.

### FR-2: Reorder images
- The app **MUST** allow users to reorder imported images.
- The app **SHOULD** support both:
  - drag-and-drop reordering, and
  - accessible controls (Move Up/Move Down) to support keyboard and stable E2E automation.

### FR-3: Options (direction + background)
- The app **MUST** let users choose:
  - merge direction: `vertical` or `horizontal`,
  - background fill color used when flattening alpha (default: white).
- The app **MAY** offer additional background presets (e.g., black, transparent) as long as semantics remain consistent.

### FR-4: Merge semantics and scaling rule
The merge engine must:
- decode each input image,
- normalize orientation using EXIF orientation when available (best-effort),
- compute target dimensions such that:
  - in vertical mode: **all inputs are scaled to the maximum width** across inputs,
  - in horizontal mode: **all inputs are scaled to the maximum height** across inputs,
  - upscaling is expected when needed,
- concatenate scaled images in the selected order.

### FR-5: Output size safeguards
The app **MUST NOT** enforce output pixel caps or fail-fast memory checks. Merges should proceed without pre-check size limits.

### FR-6: Error handling and user feedback
- If a file is unsupported or fails decode, the app **MUST** show a user-visible error that identifies the failing file(s).
- The merge operation **MUST** fail as a whole if any required input fails (v0.4 contract), unless a future “skip failures” mode is introduced.

### FR-7: Preview
- The app **MUST** display a preview of the merged output when the merge succeeds.
- The preview **MUST** be rendered using an object URL from the generated output bytes.
- The preview area **SHOULD** be scrollable for large outputs.

### FR-8: Download
- The app **MUST** allow downloading the merged output (default filename: `merged.png`).
- The app **MUST** include a Safari/iOS-compatible fallback:
  - if `<a download>` does not reliably trigger save, open the blob URL in a new tab and instruct the user to save/share.

### FR-9: GitHub Pages (Project Pages) deployment
- The app **MUST** deploy as a static build to GitHub Pages Project Pages.
- The build **MUST** work under a base path `/<repo-name>`.
- Deep links **MUST** work (configure static fallback to `404.html`).

## 5. Non-functional requirements

### NFR-1: Privacy & security
- The app **MUST** not upload images or metadata.
- The app **MUST** not require user accounts or authentication.

### NFR-2: Performance
- The app **SHOULD** handle typical user batches (e.g., 5–50 images) on modern devices.
- The app **SHOULD** avoid unnecessary copies of large byte buffers (prefer transfer lists where applicable).
- The app **MUST** keep the UI responsive by performing merge computation off the main thread (Worker + WASM).

### NFR-3: Compatibility
- The app **MUST** work in modern Chromium, Firefox, and Safari on desktop and mobile to a reasonable degree.
- The app **MUST** not rely on platform-specific APIs that prevent GitHub Pages deployment.

### NFR-4: Determinism
- For a fixed set of input images and options, the merge result **MUST** be deterministic at the pixel level.
- Resize behavior **MUST** use deterministic rounding rules and fixed resampling filters.
- Byte-for-byte output equality is **NOT** required (encoder output may vary), but pixel-identical decoded output is required for fixtures.

### NFR-5: Explicitly non-PWA
- The app **MUST NOT** register a Service Worker.
- The app **MUST NOT** include a Web App Manifest.
- The app **MUST NOT** claim offline support in UI or documentation.

## 6. Testing requirements

### TR-1: End-to-end tests (required)
- Playwright E2E coverage for:
  - importing images,
  - reordering,
  - merging vertically and horizontally,
  - downloading output,
  - error states (unsupported input / failed decode),
  - deterministic output properties for small fixture images:
    - pixel-identical rendering is required;
    - byte-for-byte encoded equality is not required.

### TR-2: Rust unit tests (required)
- Unit tests **MUST** cover:
  - scaling target computation,
  - EXIF orientation normalization mapping,
  - deterministic dimension rounding,
  - merge buffer offset computation.

### TR-3: WASM boundary tests (recommended)
- Include a small set of `wasm-bindgen-test` tests to validate the JS↔WASM contract and a basic end-to-end merge in a browser runtime.

### TR-4: Post-modification checks
- After frontend changes, run build, lint, and tests.
- After WASM changes, run build, lint, tests, and `wasm-bindgen-test`.
- All warnings from these checks must be fixed.

## 7. Acceptance criteria (Definition of Done)

A release is acceptable when:
1. Users can complete the end-to-end workflow on desktop and mobile browsers.
2. The app deploys and runs correctly on GitHub Pages Project Pages under `/<repo-name>`.
3. The app does **not** register a Service Worker and does **not** ship a Web App Manifest.
4. Automated CI runs:
   - Rust unit tests pass,
   - Frontend linting passes,
   - Automated E2E tests pass.
