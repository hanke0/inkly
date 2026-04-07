import { Suspense, lazy } from 'react';
import { BrowserRouter, Navigate, Route, Routes } from 'react-router-dom';

const Dashboard = lazy(() => import('./Dashboard'));
const DocumentView = lazy(() => import('./DocumentView'));
const LoginPage = lazy(() => import('./LoginPage'));
const RequireAuth = lazy(() => import('./RequireAuth'));

export default function App() {
  return (
    <div className="flex min-h-0 min-w-0 flex-1 flex-col overflow-hidden">
      <BrowserRouter>
        <Suspense
          fallback={
            <div className="flex min-h-0 min-w-0 flex-1 items-center justify-center bg-inkly-shell text-inkly-muted">
              <div className="h-6 w-6 animate-spin rounded-full border-2 border-inkly-border border-t-inkly-accent" />
            </div>
          }
        >
          <Routes>
            <Route path="/login" element={<LoginPage />} />
            <Route element={<RequireAuth />}>
              <Route path="/" element={<Dashboard />} />
              <Route path="/doc/:docId" element={<DocumentView />} />
            </Route>
            <Route path="*" element={<Navigate to="/" replace />} />
          </Routes>
        </Suspense>
      </BrowserRouter>
    </div>
  );
}
