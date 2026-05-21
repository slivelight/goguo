import { useNotifStore } from '../../stores/notif-store';

function NotifBar() {
  const { notifications, unreadCount, markAllAsRead, clearAll } = useNotifStore();

  if (notifications.length === 0) {
    return (
      <div className="notif-bar-empty">
        <span>暂无通知</span>
      </div>
    );
  }

  return (
    <div className="notif-bar">
      <div className="notif-bar-header">
        <span className="notif-count">
          {unreadCount > 0 ? `${unreadCount} 条未读` : '全部已读'}
        </span>
        <div className="notif-bar-actions">
          {unreadCount > 0 && (
            <button className="btn btn-secondary" onClick={markAllAsRead}>
              全部标记已读
            </button>
          )}
          <button className="btn btn-secondary" onClick={clearAll}>
            清空
          </button>
        </div>
      </div>
      <div className="notif-list">
        {notifications.slice(0, 5).map((notif) => (
          <div key={notif.id} className={`notif-item ${notif.type} ${notif.read ? 'read' : 'unread'}`}>
            <div className="notif-title">{notif.title}</div>
            <div className="notif-message">{notif.message}</div>
            <div className="notif-time">{new Date(notif.timestamp).toLocaleString()}</div>
          </div>
        ))}
      </div>
      {notifications.length > 5 && (
        <div className="notif-more">
          还有 {notifications.length - 5} 条通知...
        </div>
      )}
    </div>
  );
}

export default NotifBar;