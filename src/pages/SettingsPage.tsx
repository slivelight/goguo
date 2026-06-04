import { useEffect, useState } from 'react';
import { getDeploymentMode, setDeploymentMode as setDeploymentModeIpc, getWslStatus, importSubscription, getSubscriptionSources } from '../lib/tauri-ipc';
import { useNotifStore } from '../stores/notif-store';
import type { DeploymentModeResponse, SubscriptionSource } from '../lib/types';
import ConfirmDialog from '../components/shared/ConfirmDialog';
import ImportSubscriptionDialog from '../components/shared/ImportSubscriptionDialog';
import StatusBadge from '../components/shared/StatusBadge';

const deploymentModes = [
  {
    id: 'windows_only',
    label: 'Windows Only',
    desc: '仅在 Windows 上运行',
    impact: '仅监控和管理 Windows 系统的代理配置（hosts、系统代理、PAC）。适用于纯 Windows 环境或 WSL 网络由 Windows 代理覆盖的场景。',
  },
  {
    id: 'wsl_only',
    label: 'WSL Only',
    desc: '仅在 WSL 上运行',
    impact: '仅监控和管理 WSL Linux 环境的代理配置（环境变量、Git 代理、DNS）。适用于所有工作都在 WSL 内完成的场景。',
  },
  {
    id: 'linux_only',
    label: 'Linux Only',
    desc: '仅在 Linux 上运行',
    impact: '仅监控和管理原生 Linux 系统的代理配置。适用于纯 Linux 桌面环境。',
  },
  {
    id: 'coordinated',
    label: 'Coordinated',
    desc: 'Windows + WSL 协调模式',
    impact: '同时管理 Windows 和 WSL 两侧的代理配置，确保双环境网络一致性。适用于 Windows + WSL 混合开发场景。',
  },
];

function SettingsPage() {
  const [deploymentMode, setDeploymentModeState] = useState<DeploymentModeResponse | null>(null);
  const [wslStatus, setWslStatus] = useState<{ is_wsl: boolean; network_mode: string } | null>(null);
  const [showModeDialog, setShowModeDialog] = useState(false);
  const [selectedMode, setSelectedMode] = useState<string | null>(null);
  const [showImportDialog, setShowImportDialog] = useState(false);
  const [subscriptionSources, setSubscriptionSources] = useState<SubscriptionSource[]>([]);
  const [isImporting, setIsImporting] = useState(false);

  const { addNotification } = useNotifStore();

  useEffect(() => {
    loadDeploymentMode();
    loadWslStatus();
    loadSubscriptionSources();
  }, []);

  const loadDeploymentMode = async () => {
    try {
      const mode = await getDeploymentMode();
      setDeploymentModeState(mode);
    } catch (err) {
      addNotification('error', '加载部署模式失败', err instanceof Error ? err.message : '未知错误');
    }
  };

  const loadWslStatus = async () => {
    try {
      const status = await getWslStatus();
      setWslStatus(status);
    } catch {
      setWslStatus({ is_wsl: false, network_mode: 'unknown' });
    }
  };

  const loadSubscriptionSources = async () => {
    try {
      const sources = await getSubscriptionSources();
      setSubscriptionSources(sources);
    } catch {
      // ignore - sources remain empty
    }
  };

  const handleModeChange = async () => {
    if (!selectedMode) return;
    try {
      const result = await setDeploymentModeIpc(selectedMode);
      setDeploymentModeState(result);
      addNotification('success', '部署模式已更新', `当前模式: ${result.mode}`);
    } catch (err) {
      addNotification('error', '更新失败', err instanceof Error ? err.message : '未知错误');
    }
    setShowModeDialog(false);
    setSelectedMode(null);
  };

  const handleImportSubscription = async (url: string) => {
    setIsImporting(true);
    try {
      const result = await importSubscription(url);
      addNotification('success', '订阅导入完成', `已导入 ${result.imported} 个节点，${result.unsupported} 个不支持`);
      setShowImportDialog(false);
      loadSubscriptionSources();
    } catch (err) {
      addNotification('error', '导入失败', err instanceof Error ? err.message : '未知错误');
    } finally {
      setIsImporting(false);
    }
  };

  return (
    <div>
      <h1 style={{ fontSize: '24px', fontWeight: '700', marginBottom: '24px' }}>设置</h1>

      <div className="card">
        <div className="card-header">部署模式</div>
        <p style={{ color: 'var(--color-text-secondary)', marginBottom: '12px' }}>
          当前模式: {deploymentMode?.mode || '加载中...'}
          {deploymentMode?.is_auto && ' (自动检测)'}
        </p>
        <div style={{ display: 'grid', gridTemplateColumns: 'repeat(2, 1fr)', gap: '12px' }}>
          {deploymentModes.map((mode) => (
            <button
              type="button"
              key={mode.id}
              className="btn"
              style={{
                display: 'block',
                padding: '12px',
                textAlign: 'left',
                border: deploymentMode?.mode === mode.id
                  ? '2px solid var(--color-primary)'
                  : '1px solid var(--color-border)',
                borderRadius: '8px',
                cursor: 'pointer',
                background: deploymentMode?.mode === mode.id
                  ? 'var(--color-bg-tertiary)'
                  : 'transparent',
                width: '100%',
              }}
              onClick={() => {
                setSelectedMode(mode.id);
                setShowModeDialog(true);
              }}
            >
              <div style={{ fontWeight: '600' }}>{mode.label}</div>
              <div style={{ fontSize: '12px', color: 'var(--color-text-secondary)' }}>
                {mode.desc}
              </div>
              <div style={{ fontSize: '11px', color: 'var(--color-text-secondary)', marginTop: '4px', lineHeight: '1.4' }}>
                {mode.impact}
              </div>
            </button>
          ))}
        </div>
      </div>

      {wslStatus && (
        <div className="card">
          <div className="card-header">WSL 状态</div>
          <div style={{ marginBottom: '12px' }}>
            <div>
              <span style={{ color: 'var(--color-text-secondary)' }}>运行环境: </span>
              <span style={{ fontWeight: '600' }}>
                {wslStatus.is_wsl ? 'WSL' : '原生 Linux/Windows'}
              </span>
            </div>
            <div>
              <span style={{ color: 'var(--color-text-secondary)' }}>网络模式: </span>
              <span style={{ fontWeight: '600' }}>{wslStatus.network_mode}</span>
            </div>
          </div>
        </div>
      )}

      <div className="card">
        <div className="card-header">订阅源</div>
        {subscriptionSources.length === 0 ? (
          <p style={{ color: 'var(--color-text-secondary)' }}>暂无订阅源配置</p>
        ) : (
          <div style={{ marginBottom: '8px' }}>
            {subscriptionSources.map((source) => (
              <div key={source.url} style={{ display: 'flex', justifyContent: 'space-between', alignItems: 'center', padding: '8px 0', borderBottom: '1px solid var(--color-border, #eee)' }}>
                <div>
                  <span style={{ fontWeight: '600', fontSize: '14px' }}>{source.name}</span>
                  <span style={{ fontSize: '12px', color: 'var(--color-text-secondary)', marginLeft: '8px' }}>{source.url}</span>
                </div>
                <StatusBadge status={source.enabled ? 'success' : 'stopped'} label={source.enabled ? '已启用' : '已禁用'} />
              </div>
            ))}
          </div>
        )}
        <button className="btn btn-secondary" style={{ marginTop: '8px' }} onClick={() => setShowImportDialog(true)}>
          导入订阅
        </button>
      </div>

      <div className="card">
        <div className="card-header">系统信息</div>
        <div style={{ color: 'var(--color-text-secondary)', fontSize: '14px' }}>
          <p>版本: 0.1.0</p>
          <p>构建: Tauri v2 + React v19</p>
        </div>
      </div>

      <ConfirmDialog
        isOpen={showModeDialog}
        title="切换部署模式"
        message={`确认切换到 "${selectedMode}" 模式？这将改变监控范围。`}
        confirmText="切换"
        onConfirm={handleModeChange}
        onCancel={() => {
          setShowModeDialog(false);
          setSelectedMode(null);
        }}
      />

      <ImportSubscriptionDialog
        isOpen={showImportDialog}
        onConfirm={handleImportSubscription}
        onCancel={() => setShowImportDialog(false)}
        isLoading={isImporting}
      />
    </div>
  );
}

export default SettingsPage;
