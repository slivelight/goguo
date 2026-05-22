import { create } from 'zustand';
import type { RecoveryProgressResponse } from '../lib/types';
import { getIsRestoring, getRecoveryProgress } from '../lib/tauri-ipc';

interface RecoveryState {
  isRestoring: boolean;
  progress: RecoveryProgressResponse | null;
  isLoading: boolean;
}

interface RecoveryActions {
  fetchRecoveryStatus: () => Promise<void>;
  reset: () => void;
}

const initialState: RecoveryState = {
  isRestoring: false,
  progress: null,
  isLoading: false,
};

export const useRecoveryStore = create<RecoveryState & RecoveryActions>((set) => ({
  ...initialState,

  fetchRecoveryStatus: async () => {
    set({ isLoading: true });
    try {
      const [isRestoring, progress] = await Promise.all([
        getIsRestoring(),
        getRecoveryProgress(),
      ]);
      set({ isRestoring, progress, isLoading: false });
    } catch {
      set({ isLoading: false });
    }
  },

  reset: () => set(initialState),
}));
