# Proposed Fix Tasks

## 1) Typo fix task
**Task:** Correct the typo in the legacy mDNS service type from `_nerolink._tcp` to `_neurolink._tcp` in user-facing docs, while preserving any explicit note about backwards compatibility if needed.

- **Why:** The README currently lists `_nerolink._tcp` (missing `u`) as a service type, which is likely a typo and can confuse setup/debugging.
- **Where observed:** `README.md` service types section.
- **Suggested acceptance criteria:**
  - Docs consistently spell the primary service type as `_neurolink._tcp`.
  - If legacy compatibility is intentional, the doc clarifies which value is canonical vs legacy.

## 2) Bug fix task
**Task:** Add server-side validation to reject `chunk_size == 0` during transfer initialization and return a clear `400` error.

- **Why:** Transfer chunk calculations divide by `chunk_size`; zero can trigger division-by-zero panics or undefined behavior in both API and transfer manager paths.
- **Where observed:** `src/rust/api/routes.rs` and `src/rust/transfer/mod.rs` compute `total_chunks` using division by `chunk_size`.
- **Suggested acceptance criteria:**
  - `POST /transfer/init` with `chunk_size: 0` returns a structured validation error.
  - Valid chunk sizes continue to work unchanged.
  - Add regression coverage for zero chunk size.

## 3) Code comment / documentation discrepancy task
**Task:** Align README default port documentation with the Rust service implementation default port.

- **Why:** README repeatedly documents Rust service default as `3030`, but `main.rs` defaults to `8000` when env var is absent.
- **Where observed:** `README.md` configuration and quick-start snippets vs `src/rust/main.rs` default port logic.
- **Suggested acceptance criteria:**
  - Docs and code agree on one default (`3030` or `8000`).
  - All examples/CLI snippets are updated to match the chosen default.

## 4) Test improvement task
**Task:** Add focused unit/integration tests for transfer lifecycle edge cases.

- **Why:** Core transfer behavior has no automated tests in-tree; edge cases (missing chunk, duplicate chunk index, out-of-range index, zero chunk size) are high risk.
- **Where observed:** Rust modules (`transfer`, `api`) lack test modules.
- **Suggested acceptance criteria:**
  - At least one test each for:
    - successful init/chunk/complete flow,
    - completion with missing chunks fails,
    - out-of-range chunk index fails,
    - zero chunk size is rejected.
  - Tests run in CI via `cargo test`.
