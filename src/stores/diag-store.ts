import { create } from 'zustand';
import type { SiteReachability, NodePoolStatus } from '../lib/types';
import { getSiteReachability, getNodePoolStatus, getDiagnosis, getAuditLog } from '../lib/tauri-ipc';
import type { AuditLogResponse } from '../lib/types';

interface DiagState {
  reachability: SiteReachability[];
  nodePool: NodePoolStatus;
  auditLog: AuditLogResponse;
  isLoading: boolean;
  error: string | null;
}

interface DiagActions {
  fetchReachability: () => Promise<void>;
  fetchNodePool: () => Promise<void>;
  fetchAuditLog: (offset?: number, limit?: number) => Promise<void>;
  diagnoseSite: (siteId: string) => Promise<SiteReachability | null>;
  reset: () => void;
}

const initialState: DiagState = {
  reachability: [],
  nodePool: {
    total_nodes: 0,
    available_nodes: 0,
    current_node: undefined,
    nodes: [],
  },
  auditLog: {
    total_count: 0,
    records: [],
  },
  isLoading: false,
  error: null,
};

export const useDiagStore = create<DiagState & DiagActions>((set) => ({
  ...initialState,

  fetchReachability: async () => {
    set({ isLoading: true, error: null });
    try {
      const response = await getSiteReachability();
      set({
        reachability: response.sites,
        isLoading: false,
      });
    } catch (err) {
      set({
        isLoading: false,
        error: err instanceof Error ? err.message : 'Failed to fetch reachability',
      });
    }
  },

  fetchNodePool: async () => {
    try {
      const status = await getNodePoolStatus();
      set({ nodePool: status });
    } catch (err) {
      set({
        error: err instanceof Error ? err.message : 'Failed to fetch node pool',
      });
    }
  },

  fetchAuditLog: async (offset?: number, limit?: number) => {
    try {
      const log = await getAuditLog({ offset, limit });
      set((state) => ({
        auditLog: {
          total_count: log.total_count,
          records: offset && offset > 0
            ? [...state.auditLog.records, ...log.records]
            : log.records,
        },
      }));
    } catch (err) {
      set({
        error: err instanceof Error ? err.message : 'Failed to fetch audit log',
      });
    }
  },

  diagnoseSite: async (siteId: string) => {
    try {
      const result = await getDiagnosis(siteId);
      if (result) {
        set((state) => ({
          reachability: state.reachability.map((r) =>
            r.site_id === siteId ? result : r
          ),
        }));
      }
      return result;
    } catch (err) {
      set({
        error: err instanceof Error ? err.message : 'Failed to diagnose site',
      });
      return null;
    }
  },

  reset: () => set(initialState),
}));