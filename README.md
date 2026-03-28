# Inkly (Axum + Tantivy + React)

This repository contains a starter framework for:
- Backend: Rust + `axum` (HTTP Basic auth, typed DTOs)
- Search/storage: `tantivy` (tenant-scoped indexing + searching)
- Frontend: React (Vite) + Tailwind CSS SPA

## Prerequisites
- Rust toolchain (with Cargo)
- Node.js (for the frontend build/dev server)

## Run backend
1. Configure environment:
   - Copy `.env.example` to `.env` and set `USERNAME` and `PASSWORD` (HTTP Basic credentials).
2. Start the server:
   - `cargo run -p inkly`

Notes:
- The backend binary embeds the frontend `frontend/dist` at compile time.
- During `cargo build`/`cargo run`, Cargo will run `npm run build` inside `frontend/` (unless disabled).
- To skip rebuilding the frontend during backend builds, set `SKIP_FRONTEND_BUILD=1`.

Default:
- Bind: `127.0.0.1:8080`
- Tantivy index directory: `./data/tantivy`

### HTTP Basic auth
Protected endpoints expect `Authorization: Basic <base64(username:password)>`.
The username and password must match `USERNAME` and `PASSWORD` from the environment.
The Basic username is used as the tenant id for search/index isolation (single configured user today).

## API
All protected endpoints require HTTP Basic auth.

- `GET /healthz`
  - Response: `{ "status": "ok" }`

- `GET /v1/session`
  - Validates Basic credentials; response `{ "ok": true }` when valid.

- `POST /v1/documents`
  - Body: `{ "doc_id": "...", "title": "...", "content": "..." }`
  - Response: `{ "indexed": number, "deleted": number }`

- `POST /v1/documents/bulk`
  - Body: `{ "documents": [ { "doc_id": "...", "title": "...", "content": "..." } ] }`
  - Response: `{ "indexed": number, "deleted": number }`

- `GET /v1/search?q=...&limit=10`
  - Response:
    - `{ "total_hits": number, "results": [ { "doc_id", "title", "snippet", "score" } ] }`

## Run frontend
1. Install dependencies:
   - `cd frontend && npm install`
2. Start dev server:
   - `npm run dev`

The dev server proxies `/v1` and `/healthz` to `http://127.0.0.1:8080`, so the SPA can call the API with same-origin relative URLs while the backend runs separately.

The UI supports:
- A `/login` page; signed-in credentials are stored in `localStorage` (`inkly.basic.username`, `inkly.basic.password`) and sent as `Authorization: Basic` on API calls
- Indexing and search on `/` after authentication

Single-binary mode:
- After building the backend, the compiled frontend is served from the backend at `/` (SPA fallback).

