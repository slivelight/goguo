function WizardPage() {
  return (
    <div style={{ maxWidth: '600px', margin: '0 auto' }}>
      <h1 style={{ fontSize: '24px', fontWeight: '700', marginBottom: '24px', textAlign: 'center' }}>
        首次引导设置
      </h1>
      <div className="card">
        <div style={{ textAlign: 'center', padding: '32px' }}>
          <p style={{ color: 'var(--color-text-secondary)' }}>
            欢迎使用 GoGuo！让我们开始配置您的网络可达性工具。
          </p>
          <button className="btn btn-primary" style={{ marginTop: '24px' }}>
            开始配置
          </button>
        </div>
      </div>
    </div>
  );
}

export default WizardPage;