function SitesPage() {
  return (
    <div>
      <h1 style={{ fontSize: '24px', fontWeight: '700', marginBottom: '24px' }}>站点管理</h1>
      <div className="card">
        <div className="card-header">站点列表</div>
        <p style={{ color: 'var(--color-text-secondary)' }}>暂无已添加站点</p>
      </div>
      <div className="card">
        <div className="card-header">预设模板</div>
        <div style={{ display: 'flex', gap: '8px' }}>
          <button className="btn btn-secondary">开发者模板</button>
          <button className="btn btn-secondary">办公模板</button>
        </div>
      </div>
    </div>
  );
}

export default SitesPage;