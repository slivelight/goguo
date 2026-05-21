import { lazy, Suspense, useEffect } from 'react';
import { BrowserRouter, Routes, Route, Navigate } from 'react-router-dom';
import Layout from './components/layout/Layout';

const DashboardPage = lazy(() => import('./pages/DashboardPage'));
const SitesPage = lazy(() => import('./pages/SitesPage'));
const RulesPage = lazy(() => import('./pages/RulesPage'));
const DiagnosticsPage = lazy(() => import('./pages/DiagnosticsPage'));
const SettingsPage = lazy(() => import('./pages/SettingsPage'));
const WizardPage = lazy(() => import('./pages/WizardPage'));

function LoadingFallback() {
  return (
    <div style={{ display: 'flex', alignItems: 'center', justifyContent: 'center', minHeight: '200px' }}>
      <div style={{ color: 'var(--color-text-secondary)' }}>加载中...</div>
    </div>
  );
}

function App() {
  useEffect(() => {
    const stores = [
      './stores/service-store',
      './stores/baseline-store',
      './stores/notif-store',
    ];
    stores.forEach(async (storePath) => {
      const store = await import(storePath);
      if (store.initializeServiceStore) {
        store.initializeServiceStore();
      }
      if (store.initializeBaselineStore) {
        store.initializeBaselineStore();
      }
      if (store.initializeNotifStore) {
        store.initializeNotifStore();
      }
    });
  }, []);

  return (
    <BrowserRouter>
      <Routes>
        <Route path="/" element={<Layout />}>
          <Route index element={<Navigate to="/dashboard" replace />} />
          <Route path="dashboard" element={<Suspense fallback={<LoadingFallback />}><DashboardPage /></Suspense>} />
          <Route path="sites" element={<Suspense fallback={<LoadingFallback />}><SitesPage /></Suspense>} />
          <Route path="rules" element={<Suspense fallback={<LoadingFallback />}><RulesPage /></Suspense>} />
          <Route path="diagnostics" element={<Suspense fallback={<LoadingFallback />}><DiagnosticsPage /></Suspense>} />
          <Route path="settings" element={<Suspense fallback={<LoadingFallback />}><SettingsPage /></Suspense>} />
          <Route path="wizard" element={<Suspense fallback={<LoadingFallback />}><WizardPage /></Suspense>} />
        </Route>
      </Routes>
    </BrowserRouter>
  );
}

export default App;