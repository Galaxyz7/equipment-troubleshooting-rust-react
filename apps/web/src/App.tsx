import { BrowserRouter, Routes, Route, Navigate } from 'react-router-dom';
import LandingPage from './pages/LandingPage';
import TroubleshootPage from './pages/TroubleshootPage';
import ConclusionPage from './pages/ConclusionPage';
import AdminLoginPage from './pages/AdminLoginPage';
import IssuesListPage from './pages/IssuesListPage';
import AnalyticsPage from './pages/AnalyticsPage';

// Protected route component
function ProtectedRoute({ children }: { children: React.ReactNode }) {
  const token = localStorage.getItem('token');

  if (!token) {
    return <Navigate to="/login" replace />;
  }

  return <>{children}</>;
}

function App() {
  return (
    <BrowserRouter>
      <Routes>
        {/* Public Routes */}
        <Route path="/" element={<LandingPage />} />
        <Route path="/troubleshoot" element={<TroubleshootPage />} />
        <Route path="/troubleshoot/:category" element={<TroubleshootPage />} />
        <Route path="/conclusion" element={<ConclusionPage />} />

        {/* Admin Routes */}
        <Route path="/login" element={<AdminLoginPage />} />
        {/* Redirect old /admin/login to new /login */}
        <Route path="/admin/login" element={<Navigate to="/login" replace />} />
        <Route
          path="/admin"
          element={
            <ProtectedRoute>
              <IssuesListPage />
            </ProtectedRoute>
          }
        />
        <Route
          path="/admin/analytics"
          element={
            <ProtectedRoute>
              <AnalyticsPage />
            </ProtectedRoute>
          }
        />

        {/* Catch all - redirect to home */}
        <Route path="*" element={<Navigate to="/" replace />} />
      </Routes>
    </BrowserRouter>
  );
}

export default App;
