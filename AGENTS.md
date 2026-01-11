# AGENTS.md — MergeImages v2

## Project structure
- `/engine` — Rust/WASM merge engine
- `/app` — SvelteKit frontend

## Prerequisites
- Rust toolchain (`rustup`)
- wasm-pack: `cargo install wasm-pack`
- Node.js 22+

## Build commands

### Engine (Rust/WASM)
```bash
cd engine
cargo check          # Type check
cargo test           # Run tests
cargo fmt            # Format code
cargo clippy         # Lint
wasm-pack build --target web --out-dir ../app/static/wasm  # Build WASM
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
```

### Full local build
```bash
# From repo root
cd engine && wasm-pack build --target web --out-dir ../app/static/wasm
cd ../app && npm run build
```

## Current guidance
- Read `ARCHITECTURE.md` for intended architecture (SvelteKit UI + Rust/WASM engine, no PWA).
- Read `REQUIREMENTS.md` for functional and testing requirements.
- Do not add Service Workers or a Web App Manifest.
- **Project management must use CLI commands** (`cargo`, `npm`, `wasm-pack`, etc.) rather than manually editing `Cargo.toml`, `package.json`, or other config files.

Keep this file updated as the project grows.