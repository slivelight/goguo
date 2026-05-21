import type { RecoveryProgressResponse } from '../../lib/types';

interface RecoveryOverlayProps {
  progress: RecoveryProgressResponse;
  onCancel?: () => void;
}

function RecoveryOverlay({ progress, onCancel }: RecoveryOverlayProps) {
  if (!progress.has_task) return null;

  const percentage = Math.round((progress.completed_count / progress.total_items) * 100);

  return (
    <div className="recovery-overlay">
      <div className="recovery-card">
        <h3 className="recovery-title">恢复进度</h3>
        <div className="recovery-progress-bar">
          <div className="recovery-progress-fill" style={{ width: `${percentage}%` }} />
        </div>
        <div className="recovery-stats">
          <span>已完成: {progress.completed_count}/{progress.total_items}</span>
          <span>成功: {progress.succeeded}</span>
          <span>失败: {progress.failed}</span>
        </div>
        <div className="recovery-status">
          状态: {progress.status || '进行中'}
        </div>
        {onCancel && progress.status === 'in_progress' && (
          <button className="btn btn-secondary" onClick={onCancel}>
            取消恢复
          </button>
        )}
      </div>
    </div>
  );
}

export default RecoveryOverlay;