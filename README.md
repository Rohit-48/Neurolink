# NeuroLink Monorepo v2.0

Two separate apps now live in this repo:

- `neurolink` (Express, Node.js) in `apps/neurolink`
- `neurolinkrs` / `neurolinkd` (Rust, Axum) in `apps/neurolinkrs`

## Run

Express app (`neurolink`, default `3000`):

```bash
npm run dev:neurolink
```

Rust app (`neurolinkrs`, default `3001`):

```bash
npm run dev:neurolinkrs
```

Rust daemon command (`neurolinkd`, default `3001`):

```bash
npm run dev:neurolinkd
```

## Build

Rust release build:

```bash
npm run build:neurolinkrs
```

## CLI Banners

When you launch commands in a terminal, each CLI prints a colored ASCII banner:

- `neurolink` -> `NEUROLINK · Express Runtime`
- `neurolinkrs` -> `NEUROLINKRS 2.0 - Rust Service`
- `neurolinkd` -> `NEUROLINKD 2.0 - Rust Daemon`

## Core parity (both apps)

- Upload batches via web UI
- List uploaded files and batches
- Download individual files
- Download batch archive
- Download file chunk by index/size

## Common endpoints (both apps)

- `GET /`
- `GET /health`
- `GET /files`
- `GET /uploads`
- `GET /shared/:filename`
- `GET /download/batch/:batch_id`
- `GET /download/chunk/:filename?index=<n>&chunk_size=<bytes>`
- `POST /transfer/init`
- `POST /transfer/chunk`
- `POST /transfer/complete`
