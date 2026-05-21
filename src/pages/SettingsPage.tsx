import { useEffect, useState } from 'react';
import { getDeploymentMode, setDeploymentMode, getWslStatus } from '../lib/tauri-ipc';
import { useNotifStore } from '../stores/notif-store';
import type { DeploymentModeResponse } from '../lib/types';
import ConfirmDialog from '../components/shared/ConfirmDialog';

function SettingsPage() {
  const [deploymentMode, setDeploymentModeState] = useState<DeploymentModeResponse | null>(null);
  const [wslStatus, setWslStatus] = useState<{ is_wsl: boolean; network_mode: string } | null>(null);
  const [showModeDialog, setShowModeDialog] = useState(false);
  const [selectedMode, setSelectedMode] = useState<string | null>(null);

  const { addNotification } = useNotifStore();

  useEffect(() => {
    loadDeploymentMode();
    loadWslStatus();
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

  const handleModeChange = async () => {
    if (!selectedMode) return;
    try {
      const result = await setDeploymentMode(selectedMode);
      setDeploymentModeState(result);
      addNotification('success', '部署模式已更新', `当前模式: ${result.mode}`);
    } catch (err) {
      addNotification('error', '更新失败', err instanceof Error ? err.message : '未知错误');
    }
    setShowModeDialog(false);
    setSelectedMode(null);
  };

  const deploymentModes = [
    { id: 'windows_only', label: 'Windows Only', desc: '仅在 Windows 上运行' },
    { id: 'wsl_only', label: 'WSL Only', desc: '仅在 WSL 上运行' },
    { id: 'linux_only', label: 'Linux Only', desc: '仅在 Linux 上运行' },
    { id: 'coordinated', label: 'Coordinated', desc: 'Windows + WSL 协调模式' },
  ];

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
            <div
              key={mode.id}
              style={{
                padding: '12px',
                border: deploymentMode?.mode === mode.id 
                  ? '2px solid var(--color-primary)' 
                  : '1px solid var(--color-border)',
                borderRadius: '8px',
                cursor: 'pointer',
                background: deploymentMode?.mode === mode.id 
                  ? 'var(--color-bg-tertiary)' 
                  : 'transparent',
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
            </div>
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
        <p style={{ color: 'var(--color-text-secondary)' }}>暂无订阅源配置</p>
        <button className="btn btn-secondary" style={{ marginTop: '8px' }}>
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
    </div>
  );
}

export default SettingsPage;