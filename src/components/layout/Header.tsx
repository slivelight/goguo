function Header() {
  return (
    <header className="header">
      <div>
        <h2 style={{ fontSize: '18px', fontWeight: '600' }}>GoGuo</h2>
      </div>
      <div style={{ display: 'flex', alignItems: 'center', gap: '16px' }}>
        <button className="btn btn-primary">立即恢复</button>
      </div>
    </header>
  );
}

export default Header;