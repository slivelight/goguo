import { BrowserRouter, Routes, Route, Navigate } from 'react-router-dom';
import Layout from './components/layout/Layout';
import DashboardPage from './pages/DashboardPage';
import SitesPage from './pages/SitesPage';
import RulesPage from './pages/RulesPage';
import DiagnosticsPage from './pages/DiagnosticsPage';
import SettingsPage from './pages/SettingsPage';
import WizardPage from './pages/WizardPage';

function App() {
  return (
    <BrowserRouter>
      <Routes>
        <Route path="/" element={<Layout />}>
          <Route index element={<Navigate to="/dashboard" replace />} />
          <Route path="dashboard" element={<DashboardPage />} />
          <Route path="sites" element={<SitesPage />} />
          <Route path="rules" element={<RulesPage />} />
          <Route path="diagnostics" element={<DiagnosticsPage />} />
          <Route path="settings" element={<SettingsPage />} />
          <Route path="wizard" element={<WizardPage />} />
        </Route>
      </Routes>
    </BrowserRouter>
  );
}

export default App;