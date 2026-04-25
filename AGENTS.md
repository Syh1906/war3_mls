# Agent Instructions

## Package Manager
- Rust project: use `cargo` in `mls-sim-rs/`.
- Build: `cargo build --release`.
- Check: `cargo check`.

## Commit Attribution
AI commits MUST include:

    Co-Authored-By: <model name> <noreply@provider-domain>

## Git Remotes
- `origin`: personal Fork, push daily development here.
- `upstream`: original repository, fetch updates and create clean contribution branches from it.

## Docs Governance
- docs-governance: managed
- Formal docs live under `docs/`.
- Use `docs-governance` for docs system changes and docs impact review.
- Project docs rules live in `docs/standards/`.
- Project docs templates live in `docs/templates/`.
- Update parent `README.md` files when moving, adding, or deleting docs.

## Local Files
- Keep local-only notes in `.local/`.
- Do not commit `.vscode/`, `.env`, or `mls-sim-rs/config.json`.
