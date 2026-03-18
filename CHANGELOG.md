# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [0.1.0] - 2026-03-18

### Added

- Cross-platform Tauri v2 application (Windows + Linux)
- System tray with dynamic usage meter icon
- 22+ AI provider integrations (Claude, Codex, Cursor, Gemini, Copilot, Augment, Amp, Kimi, z.ai, MiniMax, Factory, JetBrains, Kilo, Kiro, Vertex AI, Ollama, Synthetic, Warp, OpenRouter, Antigravity, OpenCode)
- Provider fetch pipeline with ordered strategy fallback
- Browser cookie decryption (Chrome DPAPI on Windows, Secret Service on Linux, Firefox SQLite)
- OAuth flow support for Claude and Gemini
- API token authentication with encrypted storage (Stronghold)
- CLI tool (`aibar-cli`) for scripts and CI
- Cost tracking with 30-day JSONL logs
- Usage pace analysis (on track / ahead / behind)
- Provider status polling via StatusPage.io
- Configurable refresh cadence (manual, 1m, 2m, 5m, 15m)
- React 19 frontend with TypeScript strict mode
- Tailwind CSS v4 with custom theme
- Zustand state management
- Auto-updates via Tauri updater plugin
- GitHub Actions CI/CD for Windows and Linux
