# Inkly

Inkly is a **personal document library** you run locally: a single web app to collect pages, sort them into folders, search them, and read them in the browser.

Built with Rust (`axum`), Tantivy for search, and a React + Tailwind SPA.

## What you can do

- **Sign in** with the username and password you configure for the server. After login, the browser remembers your session for later visits.
- **Browse the library** using the sidebar: folders and document titles reflect how you organized content (paths and tags).
- **Add documents** from the “new document” flow: paste text, upload a file, or point at a URL, plus optional title, folder path, tags, and private notes.
- **Search** full text from the header: optional filters include limiting results to the **current folder**, setting a **result cap**, and narrowing by **tags** (comma-separated in search settings).
- **Open a document** to read it: HTML content is shown in a safe reading view; other content is rendered as rich text where appropriate.
- **Edit or delete** a document from its page when you need to fix metadata or retire a clipping.

Your data (index and stored files) lives under the directory set by `DATA_DIR` in `.env` (see `.env.example`).

Optional **automatic summarization** of indexed content can be turned on in `.env` with `SUMMARIZE_ENABLED`; see comments in `.env.example` for build and runtime notes.

## Prerequisites

- Rust toolchain (Cargo)
- Node.js (for frontend dev, or for the automatic production build step)

## Run the app

1. **Environment**  
   Copy `.env.example` to `.env` and set at least `USERNAME` and `PASSWORD` (these are what you type on the login page). Adjust `HOST` if the default bind address or port should change (`.env.example` uses `127.0.0.1:15173`; if you omit `HOST`, the server falls back to `127.0.0.1:8080`).

2. **Start the server**  
   From the repo root:

   ```bash
   cargo run -p inkly
   ```

3. **Open the app**  
   In your browser, go to the address in `HOST` (or `http://127.0.0.1:8080` if you did not set `HOST`).

**Single binary:** the backend embeds the built frontend at compile time, so one process serves the UI and the backend. During `cargo build` / `cargo run`, Cargo normally runs `npm run build` in `frontend/` unless you disable that.

- To **skip** rebuilding the frontend on each backend build: `SKIP_FRONTEND_BUILD=1`.

## Develop the frontend with hot reload

When you want to edit UI code with Vite’s dev server instead of embedding the production bundle:

1. `cd frontend && npm install`
2. Keep the backend running (`cargo run -p inkly`).
3. In another terminal: `npm run dev`

The dev server proxies API routes to the backend URL from this repo’s `.env` (`HOST`), so the SPA can use same-origin URLs while both processes run.

## Login and accounts

Signing in uses the same **username** and **password** as in your `.env`. Today the server is configured for one user; that username also scopes your documents and search so they stay private to that account.
