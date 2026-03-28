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
      <div className="flex min-h-screen items-center justify-center bg-zinc-950 text-zinc-300">
        <div className="text-sm">Checking sign-in…</div>
      </div>
    );
  }

  return (
    <div className="min-h-screen bg-zinc-950 text-zinc-100">
      <div className="mx-auto flex min-h-screen max-w-md flex-col justify-center px-4 py-12">
        <div className="mb-8 text-center">
          <div className="text-sm text-zinc-400">Inkly</div>
          <h1 className="mt-1 text-2xl font-semibold">Sign in</h1>
          <p className="mt-2 text-sm text-zinc-400">Use the same username and password as the server configuration.</p>
        </div>

        <form onSubmit={onSubmit} className="rounded-xl border border-zinc-800 bg-zinc-900 p-6">
          {error ? (
            <div className="mb-4 rounded-lg border border-red-900 bg-red-950/30 p-3 text-sm text-red-200">{error}</div>
          ) : null}

          <label className="block text-sm text-zinc-400">Username</label>
          <input
            type="text"
            name="username"
            autoComplete="username"
            className="mt-2 w-full rounded-lg border border-zinc-800 bg-zinc-950 p-2 text-sm outline-none focus:border-zinc-700"
            value={username}
            onChange={(e) => setUsername(e.target.value)}
            disabled={loading}
          />

          <label className="mt-4 block text-sm text-zinc-400">Password</label>
          <input
            type="password"
            name="password"
            autoComplete="current-password"
            className="mt-2 w-full rounded-lg border border-zinc-800 bg-zinc-950 p-2 text-sm outline-none focus:border-zinc-700"
            value={password}
            onChange={(e) => setPassword(e.target.value)}
            disabled={loading}
          />

          <button
            type="submit"
            disabled={loading}
            className="mt-6 w-full rounded-lg bg-zinc-200 py-2 text-sm font-medium text-zinc-900 disabled:opacity-50"
          >
            {loading ? "Signing in…" : "Sign in"}
          </button>
        </form>
      </div>
    </div>
  );
}
