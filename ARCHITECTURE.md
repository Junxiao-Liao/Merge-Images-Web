# MergeImages v2 — Technical Architecture (No PWA)

Version: 0.4 (adaptive output-size guardrails; PWA removed)  
Target hosting: GitHub Pages (Project Pages)  
Runtime: Modern desktop + mobile browsers

## 1. Architectural overview

The application is a static web app with a thin UI and a compute-heavy engine compiled to WebAssembly. All merge computation runs in a dedicated Web Worker to keep the main thread responsive.

**High-level flow:**
1. UI collects input image files + user options.
2. UI sends an ordered file list and options to the Worker.
3. Worker reads bytes via `file.arrayBuffer()` per request.
4. Worker calls into Rust/WASM to decode, normalize orientation, scale, merge, and encode.
5. Worker transfers result bytes back to UI.
6. UI previews and downloads the result.

**Explicit non-PWA stance (v0.4):**
- No Service Worker registration.
- No Web App Manifest.
- No offline guarantee beyond standard browser caching behavior.

## 2. Technology stack

### 2.1 Frontend
- SvelteKit (static adapter)
- Vite
- Playwright for E2E
- `<img>`-based preview rendering (main thread) in a scrollable container

### 2.2 WASM engine (Rust)
- `wasm-bindgen` + `wasm-bindgen-futures`
- `image` crate (decode/resize/encode)
- Minimal EXIF parsing (orientation) for formats that carry EXIF (primarily JPEG/TIFF)
- Deterministic scaling (fixed filters + deterministic rounding)
- **Fail-fast sizing checks** using a caller-supplied `max_out_pixels` threshold (enables mobile-only enforcement)

### 2.3 Worker messaging
- Web Worker for CPU-heavy processing
- Transfer lists for large `ArrayBuffer` results to avoid copies

## 3. Repository layout (suggested)

- `/app` — SvelteKit UI
- `/engine` — Rust crate compiled to WASM
- `/.github/workflows` — CI

## 4. Runtime components and responsibilities

### 4.1 UI (main thread)
- File import (input elements + drag-and-drop)
- Input validation (reject HEIC/HEIF early; best-effort for other formats)
- Thumbnail generation (browser decode permitted here)
- Reordering UX
- Option controls:
  - direction, background
- Merge initiation + progress display
- Preview rendering (object URL + `<img>`) + download (with Safari fallback)
  - Safari fallback: if `<a download>` does not trigger a save flow (notably on iOS), open the blob URL in a new tab and instruct the user to Save/Share.

**Adaptive output-size guardrails (FR-5)**
- UI (or Worker) performs device classification (`mobile` vs `desktop`) and determines `max_out_pixels` using the v0.4 mapping.
- The chosen `max_out_pixels` is passed down to the Worker/WASM layer and echoed in error reporting to produce a useful user message.

### 4.2 Worker (compute orchestration)
Responsibilities:
- Accept input file list + options
- Read file bytes (`arrayBuffer`)
- Compute/confirm the effective `max_out_pixels` guardrail
- Call into WASM engine
- Post result bytes (and structured errors) back to UI

Design principles:
- Avoid main-thread blocking
- Minimize copies across boundaries (use transfer lists for the output buffer)
- Provide coarse progress events (optional but recommended for large merges)

### 4.3 WASM engine (pure merge core)
The engine must be deterministic for a given set of inputs and options.

Inputs:
- list of image byte arrays
- options: direction (`vertical`/`horizontal`), background color
- `max_out_pixels: u64` (supplied by caller; enables different caps by device/profile)

Outputs:
- encoded output bytes (PNG by default)
- structured error code and details on failure

## 5. Engine contract (v0.4)

### 5.1 Decode and normalize
- Decode inputs using `image` crate decoders included in the build.
- Best-effort EXIF orientation normalization:
  - If EXIF orientation is present and parseable, apply the corresponding transform.
  - Otherwise, treat orientation as “no transform”.

### 5.2 Scaling rule (fixed)
- Vertical merge:
  - Target width = maximum width among normalized inputs.
  - Each image is resized to that target width, preserving aspect ratio with deterministic rounding.
- Horizontal merge:
  - Target height = maximum height among normalized inputs.
  - Each image is resized to that target height, preserving aspect ratio with deterministic rounding.
- Upscaling is allowed/expected.
- Resampling filters are fixed to ensure deterministic results.

### 5.3 Composition and background
- The engine composites each resized image onto the output canvas in order.
- Transparent pixels are flattened against the configured background fill color (default: white).

### 5.4 Fail-fast limits (caller-supplied threshold)
Before allocating an output canvas, compute:
- output width/height
- total pixels = width * height

If total pixels exceeds `max_out_pixels` (supplied by the caller), return a structured error immediately.

Rationale:
- Mobile browsers often have substantially tighter memory budgets and may terminate the tab during large allocations.

### 5.5 Error policy
- v0.4 contract: the entire merge fails if any input required for the merge fails decode/processing.
- Error payload includes file index/name (if available) to enable a useful UI message.

## 6. Worker protocol (message schema)

### 6.1 Requests
`MERGE_REQUEST`:
- `files: File[]` (ordered)
- `options: { direction: "vertical"|"horizontal", background: { r,g,b,a } }`
- `limitsOverride?: { maxOutPixels?: number }` (optional; primarily for deterministic tests)

Notes:
- In production, the Worker (or UI) computes the effective `maxOutPixels` using FR-5 defaults and runtime hints.
- In tests, `limitsOverride.maxOutPixels` may be set to a low value to validate TOO_LARGE behavior without huge fixtures.

### 6.2 Responses
`MERGE_PROGRESS` (optional):
- coarse stage + percent estimate

`MERGE_SUCCESS`:
- `mime: "image/png"` (or `"image/jpeg"` if enabled)
- `bytes: ArrayBuffer` (transferable)
- `width`, `height`

`MERGE_ERROR`:
- `code: "UNSUPPORTED"|"DECODE_FAILED"|"TOO_LARGE"|"INTERNAL_ERROR"|...`
- `message: string`
- `details?: { fileName?: string, fileIndex?: number, width?: number, height?: number, outPixels?: number, maxOutPixels?: number }`

## 7. GitHub Pages (Project Pages) deployment details

### 7.1 SvelteKit static build configuration
- Use SvelteKit static adapter with `paths.base` set to `/<repo-name>` for Project Pages.
- Ensure SPA fallback for deep links (e.g., copy `index.html` to `404.html` post-build).

### 7.2 Asset path correctness
- Ensure WASM and other static assets are referenced with the base path.
- Prefer hashed asset filenames from the bundler to avoid stale caches.

### 7.3 Non-PWA enforcement
- Do not include `manifest.webmanifest`.
- Do not register a Service Worker in application code.
- Ensure build does not emit SW artifacts (no PWA plugins).

## 8. Testing strategy (enforced by CI)

### 8.1 Rust tests
- Unit tests for:
  - target dimension computation
  - EXIF orientation normalization transforms
  - deterministic rounding of resized dimensions
  - output buffer sizing and offset computations
  - fail-fast checks using a caller-supplied `max_out_pixels`

### 8.2 WASM boundary tests
- `wasm-bindgen-test` for:
  - protocol correctness
  - minimal end-to-end merge with small fixtures

### 8.3 Frontend tests
- Playwright E2E:
  - import, reorder, merge, preview, download
  - error scenarios (unsupported/decode fail/too-large)
  - deterministic output verification on fixtures (pixel-identical)

## 9. CI/CD outline (GitHub Actions)

- Build Rust WASM artifacts
- Build SvelteKit static site
- Run tests (Rust + frontend + E2E)
- Deploy static artifacts to GitHub Pages

## 10. Known limitations (documented behavior)
- Best-effort EXIF orientation (only when metadata is present/parseable)
- Animated inputs are treated as “first frame only”
- Very large merges may exceed memory constraints on some browsers even below the pixel caps (documented as a practical constraint)
