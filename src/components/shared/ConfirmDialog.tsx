interface ConfirmDialogProps {
  isOpen: boolean;
  title: string;
  message: string;
  currentValue?: string;
  afterValue?: string;
  onConfirm: () => void;
  onCancel: () => void;
  confirmText?: string;
  cancelText?: string;
  danger?: boolean;
}

function ConfirmDialog({
  isOpen,
  title,
  message,
  currentValue,
  afterValue,
  onConfirm,
  onCancel,
  confirmText = '确认',
  cancelText = '取消',
  danger = false,
}: ConfirmDialogProps) {
  if (!isOpen) return null;

  return (
    <div className="confirm-dialog-overlay">
      <div className="confirm-dialog">
        <h3 className="confirm-dialog-title">{title}</h3>
        <p className="confirm-dialog-message">{message}</p>
        {currentValue && afterValue && (
          <div className="confirm-dialog-value-change">
            <div className="value-change-row">
              <span className="value-label">当前:</span>
              <span className="value-current">{currentValue}</span>
            </div>
            <div className="value-change-arrow">→</div>
            <div className="value-change-row">
              <span className="value-label">恢复后:</span>
              <span className="value-after">{afterValue}</span>
            </div>
          </div>
        )}
        <div className="confirm-dialog-actions">
          <button className={`btn ${danger ? 'btn-danger' : 'btn-primary'}`} onClick={onConfirm}>
            {confirmText}
          </button>
          <button className="btn btn-secondary" onClick={onCancel}>
            {cancelText}
          </button>
        </div>
      </div>
    </div>
  );
}

export default ConfirmDialog;