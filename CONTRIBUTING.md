# Contributing to Vidi

Thanks for taking an interest. This document covers the dev environment, the
commands you'll run during a typical cycle, and the PR workflow.

## Prerequisites

- [Node.js](https://nodejs.org/) 20 or newer
- [pnpm](https://pnpm.io/) 10 or newer
- [Rust](https://www.rust-lang.org/tools/install) 1.77 or newer (stable toolchain)
- Platform-specific Tauri dependencies — see the
  [Tauri prerequisites](https://v2.tauri.app/start/prerequisites/) page for
  your OS. On Ubuntu/Debian you'll need `libwebkit2gtk-4.1-dev`,
  `libappindicator3-dev`, `librsvg2-dev`, and a few related packages.

## Setup

```sh
git clone https://github.com/ipcrm/vidi
cd vidi
pnpm install
```

## Everyday commands

| Command | What it does |
|---|---|
| `pnpm tauri:dev` | Run the full Tauri app with hot reload (Rust + Svelte) |
| `pnpm dev` | Frontend only, in a browser at `localhost:1420` (Tauri APIs unavailable) |
| `pnpm test` | Frontend Vitest suite |
| `pnpm typecheck` | `svelte-check` across `src/` |
| `cargo test --manifest-path src-tauri/Cargo.toml --lib` | Rust tests |
| `cargo clippy --manifest-path src-tauri/Cargo.toml --all-targets -- -D warnings` | Rust lints |
| `cargo fmt --manifest-path src-tauri/Cargo.toml` | Rust formatter |
| `pnpm tauri:build` | Produce a platform bundle in `src-tauri/target/release/bundle/` |

The CI workflow runs the test/typecheck/lint set on every PR — PRs should be
green before being merged.

## Project layout

```
.
├── docs/                  # User-facing docs site (deployed to GitHub Pages)
├── src/                   # Svelte 5 frontend
│   ├── App.svelte
│   ├── main.ts
│   └── lib/
│       ├── components/    # Reader, Sidebar, TabBar, FindBar, panels, dialogs
│       ├── render/        # KaTeX + Mermaid lazy-loaders
│       ├── stores/        # Reactive app state (session, tabs, theme, panels)
│       ├── styles/        # Design tokens + prose typography
│       └── util/          # Debounce, measure helpers
├── src-tauri/             # Rust backend + Tauri config
│   ├── src/
│   │   ├── commands/      # Tauri command handlers (render, folder, remote, search, …)
│   │   ├── markdown/      # pulldown-cmark pipeline + extensions + sanitize
│   │   ├── search/        # tantivy full-text search
│   │   ├── sources/       # Local walker, GitHub URL normalization, remote fetcher
│   │   ├── persistence/   # Atomic JSON KV store (settings, recents, bookmarks)
│   │   └── model.rs       # Shared serde types
│   ├── Cargo.toml
│   └── tauri.conf.json
└── .github/workflows/     # CI, release, docs-deploy, security scans
```

## Commits and PRs

- Branch from `main`; PR back into `main`.
- Follow [Conventional Commits](https://www.conventionalcommits.org/):
  - `feat:` new feature
  - `fix:` bug fix
  - `docs:` documentation only
  - `refactor:` code change without behavior change
  - `perf:` performance improvement
  - `test:` test additions / fixes
  - `chore:` build / tooling / deps
- Release notes are auto-generated from these subjects, so a tidy log yields a
  tidy changelog.
- Keep PRs small where it's natural to — reviewers appreciate focused changes
  over multi-topic ones.

## Release

Releases are tag-driven:

```sh
git tag v0.1.0
git push --tags
```

The `build.yml` workflow picks up the tag, patches the three version
manifests (`tauri.conf.json`, `package.json`, `Cargo.toml`) from the tag
value, builds macOS arm64, macOS x64, Linux x64, and Windows x64 bundles, and
publishes a GitHub Release with SHA256 checksums and SLSA build provenance.

Docs redeploy to [ipcrm.github.io/vidi](https://ipcrm.github.io/vidi) on
the same trigger.
