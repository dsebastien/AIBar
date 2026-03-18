# AIBar

Cross-platform system tray application that monitors AI usage quotas across multiple providers. Keeps your Claude, Codex, Cursor, Gemini, Copilot, and 17+ other AI service limits visible with live refresh and reset countdowns.

## Features

- **22+ AI providers** -- Claude, Codex, Cursor, Gemini, Copilot, Augment, Amp, Kimi, z.ai, MiniMax, Factory, JetBrains AI, Kilo, Kiro, Vertex AI, Ollama, OpenRouter, Warp, and more
- **System tray** -- lives in the notification area, click to show usage dashboard
- **Dynamic tray icon** -- colored usage meter bars update in real time
- **Multiple auth methods** -- API tokens, browser cookies, OAuth, CLI integration
- **Usage tracking** -- session and weekly usage windows with reset countdowns
- **Cost tracking** -- 30-day rolling cost history with per-model token breakdowns
- **Usage pace** -- see if you're on track, ahead, or behind your limits
- **Provider status** -- incident badges from StatusPage.io
- **Auto-refresh** -- configurable cadence (1m, 2m, 5m, 15m, or manual)
- **CLI tool** -- `aibar-cli` for scripts and CI usage
- **Auto-updates** -- built-in update mechanism

## Platforms

- **Windows** (10+)
- **Linux** (Ubuntu 22.04+, Fedora 38+, Arch)

## Installation

### Download

Download the latest release from the [GitHub Releases](https://github.com/topoffunnel/AIBar/releases) page:

- **Windows**: `.msi` or NSIS installer
- **Linux**: `.AppImage` or `.deb`

### CLI Only

```bash
# Download the CLI binary from releases
# Linux
tar xzf aibar-cli-*-linux-x64.tar.gz
./aibar-cli providers
```

## Development

### Prerequisites

- [Rust](https://rustup.rs/) (stable)
- [Bun](https://bun.sh/) (latest)
- Linux: `sudo apt install libwebkit2gtk-4.1-dev libappindicator3-dev librsvg2-dev patchelf libsecret-1-dev`

### Setup

```bash
bun install
cargo check
```

### Dev

```bash
bun run tauri dev
```

### Build

```bash
bun run tauri build
```

### CLI

```bash
cargo run --bin aibar-cli -- providers
cargo run --bin aibar-cli -- usage --provider ollama
```

### Quality

```bash
bun run check:all    # lint + format + tsc + clippy + fmt
bun run test         # frontend + backend tests
bun run validate     # all checks + tests
```

## Tech Stack

- **Backend**: Rust, Tauri v2, tokio
- **Frontend**: React 19, TypeScript (strict), Tailwind CSS v4, Zustand
- **Build**: Vite, bun
- **Testing**: Vitest + React Testing Library (frontend), cargo test (backend)

## Architecture

```
Cargo workspace
├── crates/aibar-providers/   # Shared provider engine (no Tauri dependency)
│   ├── models, traits        # ProviderId, FetchStrategy, FetchPipeline
│   ├── auth/                 # Cookie decryption, OAuth, JWT, Stronghold
│   └── providers/            # 22+ provider implementations
├── src-tauri/                # Tauri v2 app (system tray, commands, managers)
└── src-cli/                  # Standalone CLI binary
```

## License

MIT
