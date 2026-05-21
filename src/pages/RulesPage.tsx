function RulesPage() {
  return (
    <div>
      <h1 style={{ fontSize: '24px', fontWeight: '700', marginBottom: '24px' }}>规则预览</h1>
      <div className="card">
        <div className="card-header">当前规则</div>
        <p style={{ color: 'var(--color-text-secondary)' }}>暂无规则</p>
      </div>
      <div style={{ display: 'flex', gap: '8px' }}>
        <button className="btn btn-secondary">预览规则</button>
        <button className="btn btn-primary">应用规则</button>
      </div>
    </div>
  );
}

export default RulesPage;