import { describe, it, expect, vi, beforeEach } from 'vitest';
import { render, screen } from '@testing-library/react';
import SettingsPage from '../SettingsPage';
import * as notifStore from '../../stores/notif-store';
import * as ipc from '../../lib/tauri-ipc';

vi.mock('../../stores/notif-store', () => ({
  useNotifStore: vi.fn(),
}));

vi.mock('../../lib/tauri-ipc', () => ({
  getDeploymentMode: vi.fn(),
  setDeploymentMode: vi.fn(),
  getWslStatus: vi.fn(),
  getNetworkMode: vi.fn(),
}));

describe('SettingsPage', () => {
  beforeEach(() => {
    vi.clearAllMocks();
    vi.mocked(notifStore.useNotifStore).mockReturnValue({
      addNotification: vi.fn(),
    } as unknown as ReturnType<typeof notifStore.useNotifStore>);
    
    vi.mocked(ipc.getDeploymentMode).mockResolvedValue({
      mode: 'windows_only',
      detected: 'windows_only',
      is_auto: true,
    });
    
    vi.mocked(ipc.getWslStatus).mockResolvedValue({
      is_wsl: false,
      network_mode: 'unknown',
      reachable: false,
    });
  });

  it('renders settings title', () => {
    render(<SettingsPage />);
    expect(screen.getByText('设置')).toBeDefined();
  });

  it('shows deployment mode card', () => {
    render(<SettingsPage />);
    expect(screen.getByText('部署模式')).toBeDefined();
  });

  it('shows all deployment mode options', () => {
    render(<SettingsPage />);
    expect(screen.getByText('Windows Only')).toBeDefined();
    expect(screen.getByText('WSL Only')).toBeDefined();
    expect(screen.getByText('Linux Only')).toBeDefined();
    expect(screen.getByText('Coordinated')).toBeDefined();
  });

  it('shows subscription source card', () => {
    render(<SettingsPage />);
    expect(screen.getByText('订阅源')).toBeDefined();
  });

  it('shows system info card', () => {
    render(<SettingsPage />);
    expect(screen.getByText('系统信息')).toBeDefined();
  });

  it('shows import subscription button', () => {
    render(<SettingsPage />);
    expect(screen.getByText('导入订阅')).toBeDefined();
  });
});