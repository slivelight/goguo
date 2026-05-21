function SettingsPage() {
  const deploymentModes = [
    { id: 'windows_only', label: 'Windows Only', desc: '仅在 Windows 上运行' },
    { id: 'wsl_only', label: 'WSL Only', desc: '仅在 WSL 上运行' },
    { id: 'linux_only', label: 'Linux Only', desc: '仅在 Linux 上运行' },
    { id: 'coordinated', label: 'Coordinated', desc: 'Windows + WSL 协调模式' },
  ];

  return (
    <div>
      <h1 style={{ fontSize: '24px', fontWeight: '700', marginBottom: '24px' }}>设置</h1>
      <div className="card">
        <div className="card-header">部署模式</div>
        <div style={{ display: 'grid', gridTemplateColumns: 'repeat(2, 1fr)', gap: '12px' }}>
          {deploymentModes.map((mode) => (
            <div
              key={mode.id}
              style={{
                padding: '12px',
                border: '1px solid var(--color-border)',
                borderRadius: '8px',
                cursor: 'pointer',
              }}
            >
              <div style={{ fontWeight: '600' }}>{mode.label}</div>
              <div style={{ fontSize: '12px', color: 'var(--color-text-secondary)' }}>
                {mode.desc}
              </div>
            </div>
          ))}
        </div>
      </div>
      <div className="card">
        <div className="card-header">订阅源</div>
        <p style={{ color: 'var(--color-text-secondary)' }}>暂无订阅源</p>
        <button className="btn btn-secondary" style={{ marginTop: '8px' }}>
          导入订阅
        </button>
      </div>
    </div>
  );
}

export default SettingsPage;