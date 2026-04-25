# Agent Instructions

## Commands
- Rust crate root is `mls-sim-rs/`; run Cargo commands there, not at repo root.
- Check: `cargo check`
- Build: `cargo build --release`
- Run: `cargo run -- --script-dir "D:/path/to/script"`
- Health check after run: `curl http://127.0.0.1:5000/api/health`

## Architecture
- App entrypoint: `mls-sim-rs/src/main.rs`.
- HTTP routes and static web fallback are wired in `mls-sim-rs/src/server.rs`.
- API handlers live under `mls-sim-rs/src/api/`.
- Room and Lua runtime behavior live under `mls-sim-rs/src/room/`.
- `mls-sim-rs/web/` is embedded into the binary via `rust-embed`; edit these files for the built-in Web UI.
- `参考/mls-master/` is platform reference material, not simulator source.

## Commit Attribution
AI commits MUST include:

    Co-Authored-By: <model name> <noreply@provider-domain>

## Git Remotes
- `origin`: personal Fork for daily development.
- `upstream`: original repo; create clean upstream contribution branches from `upstream/main`, then cherry-pick or port only suitable changes.

## Docs Governance
- docs-governance: managed
- Formal docs live under `docs/`; start at `docs/README.md`.
- Use `docs-governance` for docs system changes and docs impact review.
- Follow `docs/standards/` before writing or moving docs; use `docs/templates/` for new docs.
- Update parent `README.md` files when moving, adding, or deleting docs.

## Local Files
- Keep local-only notes in `.local/`.
- Do not commit `.vscode/`, `.env`, `mls-sim-rs/config.json`, `archives/`, or `mls-sim-rs/archives/`.
- Commit `mls-sim-rs/config.example.json` when changing shared config shape.
