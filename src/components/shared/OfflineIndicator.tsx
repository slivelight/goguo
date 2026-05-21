import { useOffline } from '../../hooks/use-offline';

function OfflineIndicator() {
  const { isOffline, lastKnownStatus } = useOffline();

  if (!isOffline) {
    return null;
  }

  return (
    <div className="offline-indicator" style={{
      position: 'fixed',
      bottom: '48px',
      left: '50%',
      transform: 'translateX(-50%)',
      padding: '12px 24px',
      backgroundColor: 'var(--color-warning)',
      color: 'white',
      borderRadius: '8px',
      fontWeight: '500',
      display: 'flex',
      alignItems: 'center',
      gap: '12px',
    }}>
      <span>⚠️ 服务未运行</span>
      <span style={{ fontSize: '12px' }}>
        上次状态: {lastKnownStatus ? '运行中' : '已停止'}
      </span>
      <button className="btn btn-secondary" style={{ fontSize: '12px', padding: '4px 8px' }}>
        启动服务
      </button>
    </div>
  );
}

export default OfflineIndicator;