import { describe, it, expect, vi, beforeEach } from 'vitest';
import { render, screen, fireEvent, waitFor, act } from '@testing-library/react';
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
  importSubscription: vi.fn(),
  getSubscriptionSources: vi.fn(),
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

    vi.mocked(ipc.getSubscriptionSources).mockResolvedValue([]);
  });

  it('renders settings title', async () => {
    await act(async () => { render(<SettingsPage />); });
    expect(screen.getByText('设置')).toBeDefined();
  });

  it('shows deployment mode card', async () => {
    await act(async () => { render(<SettingsPage />); });
    expect(screen.getByText('部署模式')).toBeDefined();
  });

  it('shows all deployment mode options', async () => {
    await act(async () => { render(<SettingsPage />); });
    expect(screen.getByText('Windows Only')).toBeDefined();
    expect(screen.getByText('WSL Only')).toBeDefined();
    expect(screen.getByText('Linux Only')).toBeDefined();
    expect(screen.getByText('Coordinated')).toBeDefined();
  });

  it('shows impact description for deployment modes', async () => {
    await act(async () => { render(<SettingsPage />); });
    expect(screen.getByText(/仅监控和管理 Windows 系统的代理配置/)).toBeDefined();
    expect(screen.getByText(/仅监控和管理 WSL Linux 环境的代理配置/)).toBeDefined();
    expect(screen.getByText(/仅监控和管理原生 Linux 系统的代理配置/)).toBeDefined();
    expect(screen.getByText(/同时管理 Windows 和 WSL 两侧的代理配置/)).toBeDefined();
  });

  it('shows subscription source card', async () => {
    await act(async () => { render(<SettingsPage />); });
    expect(screen.getByText('订阅源')).toBeDefined();
  });

  it('shows system info card', async () => {
    await act(async () => { render(<SettingsPage />); });
    expect(screen.getByText('系统信息')).toBeDefined();
  });

  it('shows import subscription button', async () => {
    await act(async () => { render(<SettingsPage />); });
    expect(screen.getByText('导入订阅')).toBeDefined();
  });

  it('shows empty state when no subscription sources', async () => {
    await act(async () => { render(<SettingsPage />); });
    await waitFor(() => {
      expect(screen.getByText('暂无订阅源配置')).toBeDefined();
    });
  });

  it('shows subscription source list when sources exist', async () => {
    vi.mocked(ipc.getSubscriptionSources).mockResolvedValue([
      { name: 'Test Source', url: 'https://sub.example.com/link', enabled: true },
    ]);

    await act(async () => { render(<SettingsPage />); });
    await waitFor(() => {
      expect(screen.getByText('Test Source')).toBeDefined();
    });
  });

  it('shows import dialog when import button clicked', async () => {
    await act(async () => { render(<SettingsPage />); });
    fireEvent.click(screen.getByText('导入订阅'));
    expect(screen.getByPlaceholderText('https://example.com/subscription')).toBeDefined();
  });
});
