import { Outlet } from 'react-router-dom';
import Sidebar from './Sidebar';
import Header from './Header';
import StatusBar from './StatusBar';
import OfflineIndicator from '../shared/OfflineIndicator';

function Layout() {
  return (
    <div className="app-container">
      <Sidebar />
      <div className="main-content">
        <Header />
        <div className="page-content">
          <Outlet />
        </div>
        <StatusBar />
      </div>
      <OfflineIndicator />
    </div>
  );
}

export default Layout;