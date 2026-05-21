function DiagnosticsPage() {
  return (
    <div>
      <h1 style={{ fontSize: '24px', fontWeight: '700', marginBottom: '24px' }}>诊断</h1>
      <div className="card">
        <div className="card-header">节点池状态</div>
        <p style={{ color: 'var(--color-text-secondary)' }}>总节点: 0 | 可用: 0</p>
      </div>
      <div className="card">
        <div className="card-header">可达性诊断</div>
        <p style={{ color: 'var(--color-text-secondary)' }}>加载中...</p>
      </div>
      <div className="card">
        <div className="card-header">审计日志</div>
        <p style={{ color: 'var(--color-text-secondary)' }}>暂无日志</p>
      </div>
    </div>
  );
}

export default DiagnosticsPage;