interface RecoveryAckDialogProps {
  isOpen: boolean;
  failedItems: string[];
  onAcknowledge: () => void;
  onRetry?: () => void;
}

function RecoveryAckDialog({
  isOpen,
  failedItems,
  onAcknowledge,
  onRetry,
}: RecoveryAckDialogProps) {
  if (!isOpen) return null;

  return (
    <div className="confirm-dialog-overlay">
      <div className="recovery-ack-dialog">
        <h3 className="recovery-ack-title">恢复任务失败项确认</h3>
        <p className="recovery-ack-message">
          以下状态项恢复失败，请确认是否手动处理：
        </p>
        <div className="recovery-ack-failed-list">
          {failedItems.map((item) => (
            <div key={item} className="failed-item">
              <span className="failed-item-icon">✗</span>
              <span className="failed-item-name">{item}</span>
            </div>
          ))}
        </div>
        <div className="recovery-ack-actions">
          {onRetry && (
            <button className="btn btn-secondary" onClick={onRetry}>
              重试
            </button>
          )}
          <button className="btn btn-primary" onClick={onAcknowledge}>
            确认
          </button>
        </div>
      </div>
    </div>
  );
}

export default RecoveryAckDialog;