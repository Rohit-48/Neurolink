# AGENTS.md

## Cursor Cloud specific instructions

### Project overview

NeuroLink v2.0 is split into two independent apps:
- `apps/neurolink`: Express app (Node.js)
- `apps/neurolinkrs`: Rust app (Axum + Tokio)
See `README.md` for run/build commands.

### System dependencies

- **Rust stable >= 1.85** (needed for `edition2024` support required by transitive dependency `getrandom`). The pre-installed Rust 1.83 is too old; run `rustup update stable && rustup default stable` if builds fail with `feature edition2024 is required`. Note: `rustup default stable` is required because the VM default is pinned to `1.83.0-x86_64-unknown-linux-gnu`, so merely updating the stable toolchain does not switch the active default.
- **libssl-dev** and **pkg-config** are required for compiling test dependencies (`openssl-sys`). Without them, `cargo test` fails but `cargo build` (which uses `rustls`) succeeds.

### Running the servers

```bash
# Express
npm run dev:neurolink

# Rust
npm run dev:neurolinkrs
```

- Express default: `0.0.0.0:3000`
- Rust default: `0.0.0.0:3001`
- Both expose web UI at `/`, health at `/health`, and shared files at `/shared/<filename>`.

### Testing

- **Integration tests** (21 tests, all pass): `cargo test -p neurolinkrs --test issue_tasks_validator`
- Avoid `cargo test` at workspace root if OpenSSL is missing; use the Rust package-targeted test command above.

### Linting

- `cargo clippy -p neurolinkrs` — passes with warnings only (dead code, `manual_div_ceil`).
- `cargo fmt --check` — existing code may contain pre-existing format differences.
