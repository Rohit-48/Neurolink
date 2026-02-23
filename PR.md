# PR: Consolidate NeuroLink to Single Rust Service

## Summary
This release removes the split terminal experience and keeps one command only:
- `neurolink`

It now includes:
- painted colored elephant ASCII startup banner
- built-in web upload UI on `/`
- shared file serving on `/shared/`
- unsafe browser port detection with fallback to `3000`

## How To Use
1. Start server:
```bash
cargo run --bin neurolink
```
2. Open UI:
```text
http://localhost:3000
```
3. From phone (same Wi-Fi):
```text
http://<your-computer-lan-ip>:3000
```

## Notes
- `0.0.0.0` is not a browser URL.
- If chosen port is browser-unsafe, service falls back to `3000` automatically.
