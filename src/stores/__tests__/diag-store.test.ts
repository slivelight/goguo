import { describe, it, expect, beforeEach, vi } from 'vitest';
import { useDiagStore } from '../diag-store';
import { getSiteReachability, getNodePoolStatus, getDiagnosis, getAuditLog } from '../../lib/tauri-ipc';

vi.mock('../../lib/tauri-ipc', () => ({
  getSiteReachability: vi.fn(),
  getNodePoolStatus: vi.fn(),
  getDiagnosis: vi.fn(),
  getAuditLog: vi.fn(),
}));

describe('diag-store', () => {
  beforeEach(() => {
    useDiagStore.getState().reset();
    vi.clearAllMocks();
  });

  it('initial state is correct', () => {
    const state = useDiagStore.getState();
    expect(state.reachability).toEqual([]);
    expect(state.nodePool.total_nodes).toBe(0);
    expect(state.auditLog.total_count).toBe(0);
    expect(state.isLoading).toBe(false);
    expect(state.error).toBeNull();
  });

  it('fetchReachability updates reachability', async () => {
    vi.mocked(getSiteReachability).mockResolvedValue({
      sites: [
        { site_id: 'github', reachable: true, response_time_ms: 100 },
      ],
    });

    await useDiagStore.getState().fetchReachability();

    const state = useDiagStore.getState();
    expect(state.reachability).toHaveLength(1);
    expect(state.reachability[0].site_id).toBe('github');
  });

  it('fetchNodePool updates nodePool', async () => {
    vi.mocked(getNodePoolStatus).mockResolvedValue({
      total_nodes: 5,
      available_nodes: 3,
      current_node: 'node-1',
    });

    await useDiagStore.getState().fetchNodePool();

    const state = useDiagStore.getState();
    expect(state.nodePool.total_nodes).toBe(5);
    expect(state.nodePool.available_nodes).toBe(3);
    expect(state.nodePool.current_node).toBe('node-1');
  });

  it('fetchAuditLog updates auditLog', async () => {
    vi.mocked(getAuditLog).mockResolvedValue({
      total_count: 10,
      records: [
        { timestamp: '2026-05-21T08:00:00Z', action: 'baseline_confirm', target: 'system', result: 'success' },
      ],
    });

    await useDiagStore.getState().fetchAuditLog(0, 10);

    const state = useDiagStore.getState();
    expect(state.auditLog.total_count).toBe(10);
    expect(state.auditLog.records).toHaveLength(1);
  });

  it('diagnoseSite updates specific site', async () => {
    vi.mocked(getSiteReachability).mockResolvedValue({
      sites: [{ site_id: 'github', reachable: true }],
    });
    vi.mocked(getDiagnosis).mockResolvedValue({
      site_id: 'github',
      reachable: false,
      response_time_ms: undefined,
    });

    await useDiagStore.getState().fetchReachability();
    const result = await useDiagStore.getState().diagnoseSite('github');

    expect(result?.reachable).toBe(false);
    const state = useDiagStore.getState();
    expect(state.reachability[0].reachable).toBe(false);
  });
});