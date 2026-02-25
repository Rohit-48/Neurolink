# AGENTS.md

## Cursor Cloud specific instructions

### Project overview

NeuroLink v2.0 is a single-binary Rust file-sharing service (Axum + Tokio) with a built-in cyberpunk-themed web UI. See `README.md` for run/build commands.

### System dependencies

- **Rust stable >= 1.85** (needed for `edition2024` support required by transitive dependency `getrandom`). The pre-installed Rust 1.83 is too old; run `rustup update stable` if builds fail with `feature edition2024 is required`.
- **libssl-dev** and **pkg-config** are required for compiling test dependencies (`openssl-sys`). Without them, `cargo test` fails but `cargo build` (which uses `rustls`) succeeds.

### Running the server

```bash
cargo run --bin neurolink -- --port 3000 --storage ./shared
```

The server listens on `0.0.0.0:3000` and serves the web UI at `/`, health at `/health`, and shared files at `/shared/<filename>`.

### Testing

- **Integration tests** (21 tests, all pass): `cargo test --test issue_tasks_validator`
- **Unit tests** in `src/rust/transfer/tests.rs` have pre-existing compilation errors (unresolved `TransferManager` import and missing type annotations). Running `cargo test` (all tests) will fail due to these. Use `cargo test --test issue_tasks_validator` to run passing tests.

### Linting

- `cargo clippy` — passes with warnings only (dead code, `manual_div_ceil`).
- `cargo fmt --check` — existing code has formatting differences; these are pre-existing.
