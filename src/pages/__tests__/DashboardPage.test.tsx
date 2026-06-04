import { describe, it, expect, vi, beforeEach } from 'vitest';
import { render, screen, waitFor, act } from '@testing-library/react';
import DashboardPage from '../DashboardPage';
import * as serviceStore from '../../stores/service-store';
import * as baselineStore from '../../stores/baseline-store';
import * as diagStore from '../../stores/diag-store';
import * as notifStore from '../../stores/notif-store';
import * as ipc from '../../lib/tauri-ipc';

vi.mock('../../stores/service-store', () => ({
  useServiceStore: vi.fn(),
  initializeServiceStore: vi.fn(),
}));

vi.mock('../../stores/baseline-store', () => ({
  useBaselineStore: vi.fn(),
  initializeBaselineStore: vi.fn(),
}));

vi.mock('../../stores/diag-store', () => ({
  useDiagStore: vi.fn(),
}));

vi.mock('../../stores/notif-store', () => ({
  useNotifStore: vi.fn(),
}));

vi.mock('../../lib/tauri-ipc', () => ({
  startInitialAssessment: vi.fn(),
  confirmBaseline: vi.fn(),
  triggerReadjustment: vi.fn(),
  stopService: vi.fn(),
  getWslStatus: vi.fn(),
  getNetworkMode: vi.fn(),
}));

describe('DashboardPage', () => {
  beforeEach(() => {
    vi.clearAllMocks();
    vi.mocked(serviceStore.useServiceStore).mockReturnValue({
      mihomoRunning: true,
      proxyGuardRestartCount: 0,
      fetchServiceStatus: vi.fn(),
    } as unknown as ReturnType<typeof serviceStore.useServiceStore>);

    vi.mocked(baselineStore.useBaselineStore).mockReturnValue({
      hasBaseline: true,
      items: [],
      getDeviatedCount: () => 0,
      getMatchCount: () => 5,
      fetchBaselineStatus: vi.fn(),
    } as unknown as ReturnType<typeof baselineStore.useBaselineStore>);

    vi.mocked(diagStore.useDiagStore).mockReturnValue({
      reachability: [],
      fetchReachability: vi.fn(),
    } as unknown as ReturnType<typeof diagStore.useDiagStore>);

    vi.mocked(notifStore.useNotifStore).mockReturnValue({
      notifications: [],
    } as unknown as ReturnType<typeof notifStore.useNotifStore>);
  });

  it('renders dashboard title', async () => {
    await act(async () => { render(<DashboardPage />); });
    expect(screen.getByText('仪表盘')).toBeDefined();
  });

  it('shows service status card', async () => {
    await act(async () => { render(<DashboardPage />); });
    expect(screen.getByText('服务状态')).toBeDefined();
  });

  it('shows baseline status card', async () => {
    await act(async () => { render(<DashboardPage />); });
    expect(screen.getByText('Baseline 状态')).toBeDefined();
  });

  it('shows reachability summary card', async () => {
    await act(async () => { render(<DashboardPage />); });
    expect(screen.getByText('可达性摘要')).toBeDefined();
  });

  it('shows restore button', async () => {
    await act(async () => { render(<DashboardPage />); });
    expect(screen.getByText('立即恢复')).toBeDefined();
  });

  it('shows service as running', async () => {
    await act(async () => { render(<DashboardPage />); });
    expect(screen.getByText('运行中')).toBeDefined();
  });

  it('shows baseline as confirmed', async () => {
    await act(async () => { render(<DashboardPage />); });
    expect(screen.getByText('已确认')).toBeDefined();
  });

  it('shows environment info card', async () => {
    vi.mocked(ipc.getWslStatus).mockResolvedValue({
      is_wsl: true,
      distro_name: 'Ubuntu',
      distro_version: '22.04',
      network_mode: 'mirrored',
      reachable: true,
    });
    vi.mocked(ipc.getNetworkMode).mockResolvedValue({
      mode: 'mirrored',
      proxy_strategy: 'auto',
    });

    await act(async () => { render(<DashboardPage />); });
    await waitFor(() => {
      expect(screen.getByText('环境信息')).toBeDefined();
      expect(screen.getByText(/WSL/)).toBeDefined();
    });
  });
});
