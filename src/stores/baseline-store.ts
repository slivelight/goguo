import { create } from 'zustand';
import type { BaselineConfirmedPayload, BaselineDeviationPayload, ComparisonItem, SnapshotItem, StateSummaryResponse } from '../lib/types';
import { getBaselineStatus, confirmBaseline, startInitialAssessment, triggerReadjustment, getStateSummary, getSnapshotDetails } from '../lib/tauri-ipc';
import { subscribeBaselineConfirmed, subscribeBaselineDeviation } from '../lib/events';

interface BaselineState {
  hasBaseline: boolean;
  items: ComparisonItem[];
  version: number | null;
  itemCount: number | null;
  stateSummary: StateSummaryResponse | null;
  snapshotItems: SnapshotItem[];
  deviatedItems: string[];
  isLoading: boolean;
  error: string | null;
}

interface BaselineActions {
  fetchBaselineStatus: () => Promise<void>;
  confirmBaseline: () => Promise<void>;
  startAssessment: () => Promise<void>;
  triggerReadjustment: () => Promise<void>;
  handleBaselineConfirmed: (payload: BaselineConfirmedPayload) => void;
  handleBaselineDeviation: (payload: BaselineDeviationPayload) => void;
  getDeviatedCount: () => number;
  getMatchCount: () => number;
  resetAssessment: () => void;
  reset: () => void;
}

const initialState: BaselineState = {
  hasBaseline: false,
  items: [],
  version: null,
  itemCount: null,
  stateSummary: null,
  snapshotItems: [],
  deviatedItems: [],
  isLoading: false,
  error: null,
};

export const useBaselineStore = create<BaselineState & BaselineActions>((set, get) => ({
  ...initialState,

  fetchBaselineStatus: async () => {
    set({ isLoading: true, error: null });
    try {
      const status = await getBaselineStatus();
      set({
        hasBaseline: status.has_baseline,
        items: status.items,
        isLoading: false,
      });
    } catch (err) {
      set({
        isLoading: false,
        error: err instanceof Error ? err.message : 'Failed to fetch baseline status',
      });
    }
  },

  confirmBaseline: async () => {
    set({ isLoading: true, error: null });
    try {
      const response = await confirmBaseline();
      set({
        hasBaseline: true,
        version: response.version,
        itemCount: response.item_count,
        isLoading: false,
      });
    } catch (err) {
      set({
        isLoading: false,
        error: err instanceof Error ? err.message : 'Failed to confirm baseline',
      });
    }
  },

  startAssessment: async () => {
    set({ isLoading: true, error: null });
    try {
      const response = await startInitialAssessment();
      const [summary, details] = await Promise.all([
        getStateSummary(),
        getSnapshotDetails(),
      ]);
      set({
        hasBaseline: false,
        version: response.version,
        itemCount: response.item_count,
        stateSummary: summary,
        snapshotItems: details,
        isLoading: false,
      });
    } catch (err) {
      set({
        isLoading: false,
        error: err instanceof Error ? err.message : 'Failed to start assessment',
      });
    }
  },

  triggerReadjustment: async () => {
    set({ isLoading: true, error: null });
    try {
      const response = await triggerReadjustment();
      set({
        version: response.version,
        itemCount: response.item_count,
        isLoading: false,
      });
    } catch (err) {
      set({
        isLoading: false,
        error: err instanceof Error ? err.message : 'Failed to trigger readjustment',
      });
    }
  },

  handleBaselineConfirmed: (payload: BaselineConfirmedPayload) => {
    set({
      hasBaseline: true,
      version: payload.version,
      itemCount: payload.item_count,
    });
  },

  handleBaselineDeviation: (payload: BaselineDeviationPayload) => {
    set({
      deviatedItems: payload.deviated_items,
    });
  },

  getDeviatedCount: () => {
    const state = get();
    return state.items.filter((item) => item.result === 'deviated').length;
  },

  getMatchCount: () => {
    const state = get();
    return state.items.filter((item) => item.result === 'match').length;
  },

  resetAssessment: () => set({
    itemCount: null,
    stateSummary: null,
    snapshotItems: [],
    version: null,
  }),

  reset: () => set(initialState),
}));

export function initializeBaselineStore(): void {
  subscribeBaselineConfirmed((payload) => {
    useBaselineStore.getState().handleBaselineConfirmed(payload);
  });
  subscribeBaselineDeviation((payload) => {
    useBaselineStore.getState().handleBaselineDeviation(payload);
  });
}
