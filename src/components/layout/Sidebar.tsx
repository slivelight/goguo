import { NavLink } from 'react-router-dom';

function Sidebar() {
  const navItems = [
    { path: '/dashboard', label: '仪表盘', icon: '📊' },
    { path: '/sites', label: '站点管理', icon: '🌐' },
    { path: '/rules', label: '规则预览', icon: '📋' },
    { path: '/diagnostics', label: '诊断', icon: '🔍' },
    { path: '/settings', label: '设置', icon: '⚙️' },
  ];

  return (
    <aside className="sidebar">
      <div className="sidebar-header">
        <h1>GoGuo</h1>
        <p style={{ fontSize: '12px', color: 'var(--color-text-secondary)' }}>网络可达性诊断工具</p>
      </div>
      <nav className="sidebar-nav">
        {navItems.map((item) => (
          <NavLink
            key={item.path}
            to={item.path}
            className={({ isActive }) => `nav-item ${isActive ? 'active' : ''}`}
          >
            <span style={{ marginRight: '12px' }}>{item.icon}</span>
            {item.label}
          </NavLink>
        ))}
      </nav>
    </aside>
  );
}

export default Sidebar;