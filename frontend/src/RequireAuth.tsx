import { useEffect, useState } from "react";
import { Navigate, Outlet, useNavigate } from "react-router-dom";

import { clearStoredCredentials, fetchSession, hasStoredCredentials } from "./api";

export default function RequireAuth() {
  const navigate = useNavigate();
  const [ready, setReady] = useState(false);
  const credsPresent = hasStoredCredentials();

  useEffect(() => {
    if (!credsPresent) {
      return;
    }

    let cancelled = false;
    void (async () => {
      try {
        await fetchSession();
        if (!cancelled) {
          setReady(true);
        }
      } catch {
        clearStoredCredentials();
        if (!cancelled) {
          navigate("/login", { replace: true });
        }
      }
    })();

    return () => {
      cancelled = true;
    };
  }, [credsPresent, navigate]);

  if (!credsPresent) {
    return <Navigate to="/login" replace />;
  }

  if (!ready) {
    return (
      <div className="flex min-h-0 min-w-0 flex-1 items-center justify-center bg-inkly-shell text-inkly-muted">
        <div className="text-sm">Checking sign-in…</div>
      </div>
    );
  }

  return (
    <div className="flex min-h-0 min-w-0 flex-1 flex-col overflow-hidden">
      <Outlet />
    </div>
  );
}
