# AGENTS.md — MergeImages v2

## Project structure
- `/engine` — Rust/WASM merge engine
- `/app` — SvelteKit frontend
  - `/src/lib/components` — UI components (ImageMerger, EmptyState, ImageList, etc.)
  - `/src/lib/utils` — Utilities (deviceClass, pixelLimits, download, workerManager, formats)
  - `/tests` — Playwright E2E tests
  - `/src/lib/workers` — Web Worker for WASM integration
  - `/src/lib/mergerState.svelte.ts` — Global state management using Svelte 5 Runes
  - `/src/routes/preview` — Dedicated full-page preview route

## Prerequisites
- Rust toolchain (`rustup`)
- wasm-pack: `cargo install wasm-pack`
- Node.js 22+

## Build commands

### Engine (Rust/WASM)
```bash
cd engine
cargo check          # Type check
cargo test           # Run unit tests
cargo fmt            # Format code
cargo clippy         # Lint
wasm-pack build --target web --out-dir ../app/static/wasm  # Build WASM
wasm-pack test --headless --chrome  # Run WASM boundary tests
```

### App (SvelteKit)
```bash
cd app
npm install          # Install dependencies
npm run dev          # Dev server
npm run build        # Production build
npm run preview      # Preview production build
npm run check        # Type check
npm run lint         # Lint and format (fixes)
npm run test         # Run Playwright E2E tests
npm run test:ui      # Run Playwright tests with UI
```

### Full local build
```bash
# From repo root
cd engine && wasm-pack build --target web --out-dir ../app/static/wasm
cd ../app && npm run build
```

### Run all tests
```bash
# Engine tests (unit + WASM boundary)
cd engine && cargo test
# Note: WASM boundary tests require: wasm-pack test --headless --chrome

# App tests (E2E)
cd app && npm run test
```

## Current guidance
- Read `ARCHITECTURE.md` for intended architecture (SvelteKit UI + Rust/WASM engine, no PWA).
- Read `REQUIREMENTS.md` for functional and testing requirements.
- Do not add Service Workers or a Web App Manifest.
- **Project management must use CLI commands** (`cargo`, `npm`, `wasm-pack`, etc.) rather than manually editing `Cargo.toml`, `package.json`, or other config files.

Keep this file updated as the project grows.