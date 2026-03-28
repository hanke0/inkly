import { BrowserRouter, Navigate, Route, Routes } from "react-router-dom";

import Dashboard from "./Dashboard";
import LoginPage from "./LoginPage";
import RequireAuth from "./RequireAuth";

export default function App() {
  return (
    <BrowserRouter>
      <Routes>
        <Route path="/login" element={<LoginPage />} />
        <Route element={<RequireAuth />}>
          <Route path="/" element={<Dashboard />} />
        </Route>
        <Route path="*" element={<Navigate to="/" replace />} />
      </Routes>
    </BrowserRouter>
  );
}
