function StatusBar() {
  return (
    <footer className="status-bar">
      <div>
        <span>状态: </span>
        <span className="status-badge running">运行中</span>
      </div>
      <div>
        <span>版本 0.1.0</span>
      </div>
    </footer>
  );
}

export default StatusBar;