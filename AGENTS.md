# AGENTS.md

Guidelines for AI agents working on this repository.

## Project Overview

AIBar is a cross-platform system tray application (Windows + Linux) built with Tauri v2. It monitors AI usage quotas across 22+ providers.

## Tech Stack

- **Backend**: Rust 2024 edition, Tauri v2, tokio async runtime
- **Frontend**: React 19, TypeScript (strict mode), Tailwind CSS v4, Zustand, Vite
- **Package manager**: bun
- **Testing**: Vitest + React Testing Library (frontend), cargo test (backend)

## Architecture

Three Rust crates in a Cargo workspace:

- `crates/aibar-providers/` -- shared provider engine, no Tauri dependency
- `src-tauri/` -- Tauri app with tray, commands, managers
- `src-cli/` -- standalone CLI binary

Frontend in `src/` with Vite root at `src/index.html`.

## Build & Run

```bash
bun install                    # install JS deps
bun run tauri dev              # run in dev mode
cargo check                    # check all Rust crates
cargo test                     # run Rust tests
bun run test:frontend          # run frontend tests
bun run check:all              # lint + format + tsc + clippy
```

## Code Style

### Rust

- Edition 2024
- `cargo fmt` and `cargo clippy -- -D warnings`
- Use `anyhow::Result` for error handling
- Use `async-trait` for async trait methods
- Follow the `FetchStrategy` pattern for new providers

### TypeScript

- Strict mode with all checks enabled
- No semicolons, single quotes, 4-space indent, 100 char width
- Use `@/` path alias for imports
- Zustand stores use `subscribeWithSelector` middleware
- Components are functional with TypeScript props interfaces

### Commit Messages

- Conventional commits: `feat:`, `fix:`, `chore:`, `refactor:`, `docs:`, `test:`
- Scopes: app, providers, auth, tray, ui, settings, charts, cli, ci, deps, docs, config

## Adding a New Provider

1. Create `crates/aibar-providers/src/providers/{name}.rs`
2. Implement `FetchStrategy` trait for each auth strategy
3. Add variant to `ProviderId` enum in `models.rs`
4. Register descriptor in `registry.rs`
5. Add module to `providers/mod.rs`
6. Add provider metadata to `src/lib/constants.ts`
7. Add provider icon to `src/components/icons/`

## Key Patterns

- **Provider pipeline**: Strategies tried in order with fallback logic
- **Tauri commands**: `#[tauri::command]` functions in `src-tauri/src/commands/`
- **State management**: `Arc<RwLock<_>>` in Rust, Zustand stores in React
- **Event bridge**: Rust emits Tauri events, React listens via `@tauri-apps/api/event`
