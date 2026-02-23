# NeuroLink v2.0

Single-binary Rust file sharing service with built-in web UI.

## What Changed
- One command only: `neurolink`
- Built-in browser UI at `/`
- Built-in file serving at `/shared/`
- Colored elephant ASCII banner on startup
- Unsafe browser ports are auto-rejected with fallback to `3000`

## Run
```bash
cargo run --bin neurolink
```

Or with options:
```bash
cargo run --bin neurolink -- --port 3000 --storage ./shared
```

## Open In Browser
- Local: `http://localhost:3000`
- Mobile on same Wi-Fi: `http://<your-computer-lan-ip>:3000`
- Do not open `0.0.0.0` in browser (bind address only)

## Web UI
At `/` you can:
- select a file
- upload with progress
- get a direct link to uploaded file in `/shared/<filename>`

## API
- `POST /transfer/init`
- `POST /transfer/chunk`
- `POST /transfer/complete`
- `GET /transfer/:id/status`
- `GET /health`
- `GET /shared/<filename>`

## Port Safety
If you start with a browser-blocked port, NeuroLink logs a warning and falls back to `3000`.

## Release Build
```bash
cargo build --release --bin neurolink
./target/release/neurolink
```
