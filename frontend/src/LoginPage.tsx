import { useEffect, useState } from "react";
import { useNavigate } from "react-router-dom";

import {
  clearStoredCredentials,
  fetchSession,
  hasStoredCredentials,
  storeCredentials,
  verifyLogin,
} from "./api";

export default function LoginPage() {
  const navigate = useNavigate();
  const [username, setUsername] = useState("");
  const [password, setPassword] = useState("");
  const [error, setError] = useState("");
  const [loading, setLoading] = useState(false);
  const [checking, setChecking] = useState(() => hasStoredCredentials());

  useEffect(() => {
    if (!hasStoredCredentials()) {
      return;
    }

    let cancelled = false;
    void (async () => {
      try {
        await fetchSession();
        if (!cancelled) {
          navigate("/", { replace: true });
        }
      } catch {
        clearStoredCredentials();
        if (!cancelled) {
          setChecking(false);
        }
      }
    })();

    return () => {
      cancelled = true;
    };
  }, [navigate]);

  async function onSubmit(e: React.FormEvent) {
    e.preventDefault();
    setError("");
    if (!username.trim()) {
      setError("Enter a username.");
      return;
    }

    setLoading(true);
    try {
      const ok = await verifyLogin(username, password);
      if (!ok) {
        setError("Invalid username or password.");
        return;
      }
      storeCredentials(username, password);
      navigate("/", { replace: true });
    } catch {
      setError("Could not reach the server.");
    } finally {
      setLoading(false);
    }
  }

  if (checking) {
    return (
      <div className="flex min-h-screen items-center justify-center bg-inkly-shell text-inkly-muted">
        <div className="text-sm">Checking sign-in…</div>
      </div>
    );
  }

  return (
    <div className="min-h-screen bg-inkly-shell text-inkly-ink">
      <div className="mx-auto flex min-h-screen max-w-md flex-col justify-center px-4 py-12">
        <div className="mb-8 text-center">
          <h1 className="font-serif text-3xl font-medium tracking-tight text-inkly-ink">Inkly</h1>
          <p className="mt-2 text-sm leading-relaxed text-inkly-muted">
            Your personal web archive, self-hosted
          </p>
        </div>

        <form
          onSubmit={onSubmit}
          className="rounded-xl border border-inkly-border bg-inkly-paper p-6 shadow-sm"
        >
          {error ? (
            <div className="mb-4 rounded-lg border border-red-200 bg-red-50 p-3 text-sm text-red-800">{error}</div>
          ) : null}

          <label className="block text-sm text-inkly-muted">Username</label>
          <input
            type="text"
            name="username"
            autoComplete="username"
            className="mt-2 w-full rounded-lg border border-inkly-border bg-white p-2 text-sm text-inkly-ink shadow-sm outline-none focus:border-inkly-accent focus:ring-1 focus:ring-inkly-accent"
            value={username}
            onChange={(e) => setUsername(e.target.value)}
            disabled={loading}
          />

          <label className="mt-4 block text-sm text-inkly-muted">Password</label>
          <input
            type="password"
            name="password"
            autoComplete="current-password"
            className="mt-2 w-full rounded-lg border border-inkly-border bg-white p-2 text-sm text-inkly-ink shadow-sm outline-none focus:border-inkly-accent focus:ring-1 focus:ring-inkly-accent"
            value={password}
            onChange={(e) => setPassword(e.target.value)}
            disabled={loading}
          />

          <button
            type="submit"
            disabled={loading}
            className="mt-6 w-full rounded-lg bg-inkly-accent py-2 text-sm font-medium text-white shadow-sm transition-colors hover:bg-inkly-accent-hover disabled:opacity-50"
          >
            {loading ? "Signing in…" : "Sign in"}
          </button>
        </form>
      </div>
    </div>
  );
}
