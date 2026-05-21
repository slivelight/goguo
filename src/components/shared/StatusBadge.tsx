type StatusType = 'running' | 'stopped' | 'error' | 'warning' | 'success';

interface StatusBadgeProps {
  status: StatusType;
  label?: string;
}

function StatusBadge({ status, label }: StatusBadgeProps) {
  const statusLabels: Record<StatusType, string> = {
    running: '运行中',
    stopped: '已停止',
    error: '异常',
    warning: '警告',
    success: '成功',
  };

  return (
    <span className={`status-badge ${status}`}>
      {label || statusLabels[status]}
    </span>
  );
}

export default StatusBadge;