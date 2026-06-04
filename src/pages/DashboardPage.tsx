import { useEffect, useState, useRef } from 'react';
import { useServiceStore, initializeServiceStore } from '../stores/service-store';
import { useBaselineStore, initializeBaselineStore } from '../stores/baseline-store';
import { useNotifStore } from '../stores/notif-store';
import { useDiagStore } from '../stores/diag-store';
import { useRecoveryStore } from '../stores/recovery-store';
import StatusBadge from '../components/shared/StatusBadge';
import NotifBar from '../components/shared/NotifBar';
import ConfirmDialog from '../components/shared/ConfirmDialog';
import RecoveryOverlay from '../components/shared/RecoveryOverlay';
import BaselineReviewDialog from '../components/shared/BaselineReviewDialog';
import { confirmBaseline, triggerReadjustment, stopService, getWslStatus, getNetworkMode } from '../lib/tauri-ipc';
import type { WslStatusResponse, NetworkModeResponse } from '../lib/types';

function DashboardPage() {
  const [showConfirmDialog, setShowConfirmDialog] = useState(false);
  const [confirmAction, setConfirmAction] = useState<'restore' | 'stop' | null>(null);
  const [showReviewDialog, setShowReviewDialog] = useState(false);
  const [isLoading, setIsLoading] = useState(false);
  const [wslInfo, setWslInfo] = useState<WslStatusResponse | null>(null);
  const [networkInfo, setNetworkInfo] = useState<NetworkModeResponse | null>(null);
  const pollRef = useRef<ReturnType<typeof setInterval> | null>(null);

  const { mihomoRunning, proxyGuardRestartCount, fetchServiceStatus } = useServiceStore();
  const { hasBaseline, itemCount, stateSummary, snapshotItems, getDeviatedCount, getMatchCount, fetchBaselineStatus, startAssessment, resetAssessment } = useBaselineStore();
  const { notifications } = useNotifStore();
  const { reachability, fetchReachability } = useDiagStore();
  const { isRestoring, progress, fetchRecoveryStatus } = useRecoveryStore();

  useEffect(() => {
    initializeServiceStore();
    initializeBaselineStore();
    fetchServiceStatus();
    fetchBaselineStatus();
    fetchReachability();
    fetchRecoveryStatus();
    loadEnvironmentInfo();
  }, []);

  const loadEnvironmentInfo = async () => {
    try {
      const [wsl, network] = await Promise.all([getWslStatus(), getNetworkMode()]);
      setWslInfo(wsl);
      setNetworkInfo(network);
    } catch {
      // environment info is non-critical
    }
  };

  // Poll recovery status when restoring
  useEffect(() => {
    if (isRestoring) {
      pollRef.current = setInterval(() => {
        fetchRecoveryStatus();
      }, 1000);
    } else if (pollRef.current) {
      clearInterval(pollRef.current);
      pollRef.current = null;
    }
    return () => {
      if (pollRef.current) clearInterval(pollRef.current);
    };
  }, [isRestoring]);

  const handleRestoreClick = () => {
    if (isRestoring) return;
    setConfirmAction('restore');
    setShowConfirmDialog(true);
  };

  const handleStopClick = () => {
    if (isRestoring) return;
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
    fetchRecoveryStatus();
  };

  const reachableCount = reachability.filter(r => r.reachable).length;
  const totalSites = reachability.length;

  return (
    <div>
      {isRestoring && progress && <RecoveryOverlay progress={progress} />}

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
              <button
                id="btn-assess"
                className="btn btn-secondary"
                disabled={isRestoring || isLoading}
                onClick={async () => {
                  const btn = document.getElementById('btn-assess') as HTMLButtonElement | null;
                  try {
                    if (btn) { btn.textContent = '评估中...'; btn.disabled = true; }
                    setIsLoading(true);
                    await new Promise<void>(r => setTimeout(r, 100));
                    await startAssessment();
                    setShowReviewDialog(true);
                  } catch (err) {
                    console.error('[Dashboard] startAssessment failed:', err);
                    alert(`评估失败: ${err instanceof Error ? err.message : String(err)}`);
                  } finally {
                    if (btn) { btn.textContent = '开始评估'; btn.disabled = false; }
                    setIsLoading(false);
                  }
                }}>
                开始评估
              </button>
            )}
            {!hasBaseline && (itemCount ?? 0) > 0 && (
              <button className="btn btn-primary" disabled={isRestoring} onClick={() => setShowReviewDialog(true)}>
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

      {wslInfo && (
        <div className="card">
          <div className="card-header">环境信息</div>
          <div style={{ display: 'grid', gridTemplateColumns: 'repeat(2, 1fr)', gap: '8px', fontSize: '14px' }}>
            <div>
              <span style={{ color: 'var(--color-text-secondary)' }}>运行环境: </span>
              <span style={{ fontWeight: '600' }}>{wslInfo.is_wsl ? `WSL${wslInfo.distro_name ? ` (${wslInfo.distro_name})` : ''}` : '原生 Linux/Windows'}</span>
            </div>
            <div>
              <span style={{ color: 'var(--color-text-secondary)' }}>网络模式: </span>
              <span style={{ fontWeight: '600' }}>{wslInfo.network_mode}</span>
            </div>
            {networkInfo && (
              <div>
                <span style={{ color: 'var(--color-text-secondary)' }}>代理策略: </span>
                <span style={{ fontWeight: '600' }}>{networkInfo.proxy_strategy}</span>
              </div>
            )}
          </div>
        </div>
      )}

      <div className="card">
        <div className="card-header">快捷操作</div>
        <div style={{ display: 'flex', gap: '12px' }}>
          <button className="btn btn-primary" onClick={handleRestoreClick} disabled={isRestoring || !hasBaseline || (mihomoRunning && getDeviatedCount() === 0)}>
            立即恢复
          </button>
          {mihomoRunning && (
            <button className="btn btn-danger" onClick={handleStopClick} disabled={isRestoring}>
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

      {stateSummary && (
        <BaselineReviewDialog
          isOpen={showReviewDialog}
          summary={stateSummary}
          items={snapshotItems}
          onConfirm={async () => {
            try {
              await confirmBaseline();
            } catch (err) {
              console.error('[Dashboard] confirmBaseline failed:', err);
              alert(`确认失败: ${err instanceof Error ? err.message : String(err)}`);
            }
            setShowReviewDialog(false);
            fetchBaselineStatus();
          }}
          onCancel={() => {
            setShowReviewDialog(false);
            resetAssessment();
          }}
        />
      )}
    </div>
  );
}

export default DashboardPage;