import { useEffect, useState } from 'react';
import { useServiceStore, initializeServiceStore } from '../stores/service-store';
import { useBaselineStore, initializeBaselineStore } from '../stores/baseline-store';
import { useNotifStore } from '../stores/notif-store';
import { useDiagStore } from '../stores/diag-store';
import StatusBadge from '../components/shared/StatusBadge';
import NotifBar from '../components/shared/NotifBar';
import ConfirmDialog from '../components/shared/ConfirmDialog';
import { startInitialAssessment, confirmBaseline, triggerReadjustment, stopService } from '../lib/tauri-ipc';

function DashboardPage() {
  const [showConfirmDialog, setShowConfirmDialog] = useState(false);
  const [confirmAction, setConfirmAction] = useState<'restore' | 'stop' | null>(null);

  const { mihomoRunning, proxyGuardRestartCount, fetchServiceStatus } = useServiceStore();
  const { hasBaseline, items, getDeviatedCount, getMatchCount, fetchBaselineStatus } = useBaselineStore();
  const { notifications } = useNotifStore();
  const { reachability, fetchReachability } = useDiagStore();

  useEffect(() => {
    initializeServiceStore();
    initializeBaselineStore();
    fetchServiceStatus();
    fetchBaselineStatus();
    fetchReachability();
  }, []);

  const handleRestoreClick = () => {
    setConfirmAction('restore');
    setShowConfirmDialog(true);
  };

  const handleStopClick = () => {
    setConfirmAction('stop');
    setShowConfirmDialog(true);
  };

  const handleConfirm = async () => {
    if (confirmAction === 'restore') {
      await triggerReadjustment();
    } else if (confirmAction === 'stop') {
      await stopService();
    }
    setShowConfirmDialog(false);
    setConfirmAction(null);
    fetchServiceStatus();
    fetchBaselineStatus();
  };

  const reachableCount = reachability.filter(r => r.reachable).length;
  const totalSites = reachability.length;

  return (
    <div>
      <h1 style={{ fontSize: '24px', fontWeight: '700', marginBottom: '24px' }}>仪表盘</h1>
      
      <div style={{ display: 'grid', gridTemplateColumns: 'repeat(3, 1fr)', gap: '16px' }}>
        <div className="card">
          <div className="card-header">服务状态</div>
          <div style={{ marginBottom: '12px' }}>
            <StatusBadge status={mihomoRunning ? 'running' : 'stopped'} />
          </div>
          <p style={{ color: 'var(--color-text-secondary)', fontSize: '14px' }}>
            Mihomo 代理服务
          </p>
          {proxyGuardRestartCount > 0 && (
            <p style={{ color: 'var(--color-warning)', fontSize: '12px' }}>
              自动恢复尝试: {proxyGuardRestartCount}
            </p>
          )}
        </div>

        <div className="card">
          <div className="card-header">Baseline 状态</div>
          <div style={{ marginBottom: '12px' }}>
            <StatusBadge status={hasBaseline ? 'success' : 'warning'} 
              label={hasBaseline ? '已确认' : '待确认'} />
          </div>
          <p style={{ color: 'var(--color-text-secondary)', fontSize: '14px' }}>
            偏离: {getDeviatedCount()} / 匹配: {getMatchCount()}
          </p>
          <div style={{ marginTop: '12px', display: 'flex', gap: '8px' }}>
            {!hasBaseline && (
              <button className="btn btn-secondary" onClick={async () => {
                await startInitialAssessment();
                fetchBaselineStatus();
              }}>
                开始评估
              </button>
            )}
            {!hasBaseline && items.length > 0 && (
              <button className="btn btn-primary" onClick={async () => {
                await confirmBaseline();
                fetchBaselineStatus();
              }}>
                确认 Baseline
              </button>
            )}
          </div>
        </div>

        <div className="card">
          <div className="card-header">可达性摘要</div>
          <div style={{ marginBottom: '12px' }}>
            <StatusBadge 
              status={totalSites === 0 ? 'warning' : reachableCount === totalSites ? 'success' : 'error'}
              label={totalSites === 0 ? '无站点' : reachableCount === totalSites ? '全部可用' : `${totalSites - reachableCount} 个需关注`}
            />
          </div>
          <p style={{ color: 'var(--color-text-secondary)', fontSize: '14px' }}>
            已添加站点: {totalSites}
          </p>
        </div>
      </div>

      <div className="card">
        <div className="card-header">快捷操作</div>
        <div style={{ display: 'flex', gap: '12px' }}>
          <button className="btn btn-primary" onClick={handleRestoreClick}>
            立即恢复
          </button>
          {mihomoRunning && (
            <button className="btn btn-danger" onClick={handleStopClick}>
              停止服务
            </button>
          )}
        </div>
      </div>

      {notifications.length > 0 && (
        <div className="card">
          <div className="card-header">最近通知</div>
          <NotifBar />
        </div>
      )}

      <ConfirmDialog
        isOpen={showConfirmDialog}
        title={confirmAction === 'restore' ? '确认恢复' : '确认停止服务'}
        message={confirmAction === 'restore' 
          ? '将恢复到 Baseline 状态，确认执行？' 
          : '停止服务将恢复到 Baseline，确认？'}
        confirmText="确认"
        danger={confirmAction === 'stop'}
        onConfirm={handleConfirm}
        onCancel={() => {
          setShowConfirmDialog(false);
          setConfirmAction(null);
        }}
      />
    </div>
  );
}

export default DashboardPage;