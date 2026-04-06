import { BrowserRouter, Navigate, Route, Routes } from 'react-router-dom';

import Dashboard from './Dashboard';
import DocumentView from './DocumentView';
import LoginPage from './LoginPage';
import RequireAuth from './RequireAuth';

export default function App() {
  return (
    <div className="flex min-h-0 min-w-0 flex-1 flex-col overflow-hidden">
      <BrowserRouter>
        <Routes>
          <Route path="/login" element={<LoginPage />} />
          <Route element={<RequireAuth />}>
            <Route path="/" element={<Dashboard />} />
            <Route path="/doc/:docId" element={<DocumentView />} />
          </Route>
          <Route path="*" element={<Navigate to="/" replace />} />
        </Routes>
      </BrowserRouter>
    </div>
  );
}
