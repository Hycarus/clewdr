# CLAUDE.md

Guidance for working in this repository.

## Overview

ClewdR is a Rust reverse-proxy for Claude (Claude.ai web + Claude Code). It exposes
native Claude and OpenAI-compatible endpoints, manages a pool of upstream cookies, and
ships a Leptos/WASM admin dashboard. Goals: low memory (<10 MB RSS), fast startup,
single static binary across Linux/macOS/Windows/Android.

## Workspace layout

Cargo workspace with three members:

- `.` (`clewdr`) — main binary + lib. Server, routing, providers, state machines.
- `clewdr-types` — shared types (config, cookie status, usage, reasons). Used by both
  backend and frontend so the WASM UI and server agree on schemas.
- `clewdr-frontend` — Leptos CSR (client-side rendered) WASM admin UI, built with Trunk.

## Backend source map (`src/`)

- `main.rs` — entry point: logging setup, update check, bind TCP, build router, serve.
- `lib.rs` — module roots, `Args` (clap CLI), version info, `IS_DEBUG`/`IS_DEV`.
- `router.rs` — `RouterBuilder`; wires all routes, middleware layers, CORS, static serving.
- `api/` — axum handlers: `claude_web`, `claude_code`, `config`, `misc`, `error`.
- `claude_web_state/` — state machine for Claude.ai web upstream (`bootstrap`, `chat`,
  `transform`).
- `claude_code_state/` — state machine for Claude Code upstream (`chat`, `exchange`,
  `organization`).
- `providers/claude/` — `LLMProvider` trait impls; `ClaudeProviders` holds web + code
  providers, each invocation builds a fresh state object.
- `middleware/` — `auth.rs` (`RequireAdminAuth`/`RequireBearerAuth`/`RequireFlexibleAuth`
  extractors); `claude/` (request/response transforms, OAI conversion, stop sequences,
  overload check, usage info).
- `services/` — `cookie_actor.rs` (ractor actor managing the cookie pool),
  `update.rs` (self-update).
- `config/` — `clewdr_config.rs`, `cookie.rs`, `token.rs`, `constants.rs`, `reason.rs`.
  Config loaded via `figment` from `clewdr.toml` + env.
- `types/` — Claude/OAI request/response types.
- `utils/` — helpers (`enabled`, `print_out_json`, etc.).

## Endpoints

| Path | Purpose | Auth |
|------|---------|------|
| `POST /v1/messages` | Claude.ai, native format | flexible |
| `POST /v1/chat/completions` | Claude.ai, OpenAI format | bearer |
| `POST /code/v1/messages` | Claude Code, native format | flexible |
| `POST /code/v1/messages/count_tokens` | token counting | flexible |
| `POST /code/v1/chat/completions` | Claude Code, OpenAI format | bearer |
| `GET /v1/models`, `/code/v1/models` | model list | bearer |
| `/api/*` (cookies, config, auth) | admin API | admin |
| `GET /api/version` | version | none |

Default listen address `127.0.0.1:8484`. Static dashboard served as router fallback.

## Architecture notes

- **Cookie pool** is a single `ractor` actor (`CookieActor`). It tracks valid/exhausted/
  invalid cookies, resets exhausted cookies on session/weekly windows (5h / 7d), and
  serves cookie requests over RPC. All cookie mutation goes through actor messages — do
  not mutate cookie state directly.
- **Providers** implement `LLMProvider` (async trait). A new `ClaudeWebState` /
  `ClaudeCodeState` is constructed per request from shared state + `CookieActorHandle`.
- **Middleware** is layered per route group in `router.rs` via `tower::ServiceBuilder`
  and `map_response`. OpenAI compatibility is a response transform (`to_oai`), not a
  separate code path.
- Config hot-reload supported from the dashboard; `arc-swap` (`CLEWDR_CONFIG`) holds the
  live config.

## Build & run

```bash
cargo build --release        # release profile: opt-level=z, lto, panic=abort
cargo run                     # debug run
cargo test                    # tests in tests/ (color_escape, image_conversion)
cargo clippy
```

Cargo features (mutually exclusive pairs enforced in `build.rs`):

- `external-resource` (default) vs `embed-resource` — serve `static/` from disk vs embed.
- `portable` (default) vs `xdg` — config next to binary vs XDG dirs.
- `tokio-console` — enable tokio-console on `0.0.0.0:6669`.

Frontend (`clewdr-frontend/`): `trunk build` / `trunk serve` (WASM, Leptos CSR). The
built assets go into `static/` which the backend serves.

## Conventions

- Edition 2024.
- Errors: `ClewdrError` (snafu-based). Handlers return `Result<_, ClewdrError>`.
- Logging via `tracing`; structured `[REQ]`/`[FIN]`/`[TOKENS]` info lines on requests.
- `mimalloc` is the global allocator.
- License: AGPL-3.0.
