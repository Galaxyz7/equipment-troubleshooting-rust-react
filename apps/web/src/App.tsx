import { BrowserRouter, Routes, Route, Navigate } from 'react-router-dom';
import { lazy, Suspense } from 'react';
import { ErrorBoundary } from './components/ErrorBoundary';

// Lazy load page components for code splitting
const LandingPage = lazy(() => import('./pages/LandingPage'));
const TroubleshootPage = lazy(() => import('./pages/TroubleshootPage'));
const ConclusionPage = lazy(() => import('./pages/ConclusionPage'));
const AdminLoginPage = lazy(() => import('./pages/AdminLoginPage'));
const IssuesListPage = lazy(() => import('./pages/IssuesListPage'));
const AnalyticsPage = lazy(() => import('./pages/AnalyticsPage'));

// Loading fallback component for lazy-loaded routes
function LoadingFallback() {
  return (
    <div className="min-h-screen bg-[#f5f5f5] flex items-center justify-center">
      <div className="text-center">
        <div className="inline-block animate-spin rounded-full h-12 w-12 border-b-2 border-purple-600 mb-4"></div>
        <p className="text-xl font-semibold text-gray-700">Loading...</p>
      </div>
    </div>
  );
}

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
    <ErrorBoundary>
      <BrowserRouter>
        <Suspense fallback={<LoadingFallback />}>
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
        </Suspense>
      </BrowserRouter>
    </ErrorBoundary>
  );
}

export default App;
