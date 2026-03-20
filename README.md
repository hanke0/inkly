# Inkly (Axum + Tantivy + React)

This repository contains a starter framework for:
- Backend: Rust + `axum` (JWT auth, typed DTOs)
- Search/storage: `tantivy` (tenant-scoped indexing + searching)
- Frontend: React (Vite) + Tailwind CSS SPA

## Prerequisites
- Rust toolchain (with Cargo)
- Node.js (for the frontend build/dev server)

## Run backend
1. Configure environment:
   - Copy `.env.example` to `.env` and set `INKLY_JWT_SECRET`.
2. Start the server:
   - `cargo run -p inkly-backend`

Notes:
- The backend binary embeds the frontend `frontend/dist` at compile time.
- During `cargo build`/`cargo run`, Cargo will run `npm run build` inside `frontend/` (unless disabled).
- To skip rebuilding the frontend during backend builds, set `INKLY_SKIP_FRONTEND_BUILD=1`.

Default:
- Bind: `127.0.0.1:8080`
- Tantivy index directory: `./data/tantivy`

### JWT requirements (HS256)
The backend expects `Authorization: Bearer <token>`.
The JWT must be signed with `INKLY_JWT_SECRET` (HS256) and contain:
- `sub`: user id (string)
- `tenant_id`: tenant id (string)
- `exp`: expiration time (unix seconds)

## API
All protected endpoints require JWT auth.

- `GET /healthz`
  - Response: `{ "status": "ok" }`

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

Optional env:
- `VITE_API_BASE_URL` (defaults to `http://127.0.0.1:8080`)

The UI supports:
- Saving a JWT to `localStorage` (`inkly.jwt`)
- Indexing a single document
- Searching within the tenant from the JWT claims

Single-binary mode:
- After building the backend, the compiled frontend is served from the backend at `/` (SPA fallback).

