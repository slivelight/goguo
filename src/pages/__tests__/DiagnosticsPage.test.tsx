import { describe, it, expect, vi, beforeEach } from 'vitest';
import { render, screen } from '@testing-library/react';
import { MemoryRouter } from 'react-router-dom';
import DiagnosticsPage from '../DiagnosticsPage';
import * as diagStore from '../../stores/diag-store';
import * as notifStore from '../../stores/notif-store';

vi.mock('../../stores/diag-store', () => ({
  useDiagStore: vi.fn(),
}));

vi.mock('../../stores/notif-store', () => ({
  useNotifStore: vi.fn(),
}));

function renderWithRouter(ui: React.ReactElement) {
  return render(<MemoryRouter>{ui}</MemoryRouter>);
}

describe('DiagnosticsPage', () => {
  beforeEach(() => {
    vi.clearAllMocks();
    vi.mocked(diagStore.useDiagStore).mockReturnValue({
      reachability: [],
      nodePool: { total_nodes: 0, available_nodes: 0, nodes: [] },
      auditLog: { total_count: 0, records: [] },
      fetchReachability: vi.fn(),
      fetchNodePool: vi.fn(),
      fetchAuditLog: vi.fn(),
      diagnoseSite: vi.fn(),
      isLoading: false,
    } as unknown as ReturnType<typeof diagStore.useDiagStore>);
    
    vi.mocked(notifStore.useNotifStore).mockReturnValue({
      addNotification: vi.fn(),
    } as unknown as ReturnType<typeof notifStore.useNotifStore>);
  });

  it('renders diagnostics title', () => {
    renderWithRouter(<DiagnosticsPage />);
    expect(screen.getByText('诊断')).toBeDefined();
  });

  it('shows node pool status card', () => {
    renderWithRouter(<DiagnosticsPage />);
    expect(screen.getByText('节点池状态')).toBeDefined();
  });

  it('shows site reachability card', () => {
    renderWithRouter(<DiagnosticsPage />);
    expect(screen.getByText('站点可达性')).toBeDefined();
  });

  it('shows audit log card', () => {
    renderWithRouter(<DiagnosticsPage />);
    expect(screen.getByText('审计日志')).toBeDefined();
  });

  it('shows empty state when no data', () => {
    renderWithRouter(<DiagnosticsPage />);
    expect(screen.getByText('暂无站点数据')).toBeDefined();
    expect(screen.getByText('暂无审计记录')).toBeDefined();
  });

  it('shows node pool stats', () => {
    renderWithRouter(<DiagnosticsPage />);
    expect(screen.getByText('总节点:')).toBeDefined();
    expect(screen.getByText('可用:')).toBeDefined();
  });

  it('shows reachability data when available', () => {
    vi.mocked(diagStore.useDiagStore).mockReturnValue({
      reachability: [
        { site_id: 'github', reachable: true, response_time_ms: 100 },
        { site_id: 'npm', reachable: false },
      ],
      nodePool: { total_nodes: 5, available_nodes: 3, current_node: 'node-1', nodes: [] },
      auditLog: { total_count: 0, records: [] },
      fetchReachability: vi.fn(),
      fetchNodePool: vi.fn(),
      fetchAuditLog: vi.fn(),
      diagnoseSite: vi.fn(),
      isLoading: false,
    } as unknown as ReturnType<typeof diagStore.useDiagStore>);

    renderWithRouter(<DiagnosticsPage />);
    expect(screen.getByText('github')).toBeDefined();
    expect(screen.getByText('npm')).toBeDefined();
  });
});