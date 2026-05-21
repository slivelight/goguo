function DashboardPage() {
  return (
    <div>
      <h1 style={{ fontSize: '24px', fontWeight: '700', marginBottom: '24px' }}>仪表盘</h1>
      <div style={{ display: 'grid', gridTemplateColumns: 'repeat(3, 1fr)', gap: '16px' }}>
        <div className="card">
          <div className="card-header">服务状态</div>
          <div>
            <span className="status-badge running">运行中</span>
            <p style={{ marginTop: '8px', color: 'var(--color-text-secondary)' }}>Mihomo 代理服务</p>
          </div>
        </div>
        <div className="card">
          <div className="card-header">Baseline 状态</div>
          <div>
            <span className="status-badge running">已确认</span>
            <p style={{ marginTop: '8px', color: 'var(--color-text-secondary)' }}>版本: v1</p>
          </div>
        </div>
        <div className="card">
          <div className="card-header">部署模式</div>
          <div>
            <span className="status-badge running">Windows Only</span>
            <p style={{ marginTop: '8px', color: 'var(--color-text-secondary)' }}>当前平台</p>
          </div>
        </div>
      </div>
      <div className="card">
        <div className="card-header">可达性摘要</div>
        <p style={{ color: 'var(--color-text-secondary)' }}>加载中...</p>
      </div>
      <div className="card">
        <div className="card-header">最近通知</div>
        <p style={{ color: 'var(--color-text-secondary)' }}>暂无通知</p>
      </div>
    </div>
  );
}

export default DashboardPage;