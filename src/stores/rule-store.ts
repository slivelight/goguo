import { create } from 'zustand';
import { previewRules, applyRules } from '../lib/tauri-ipc';

interface RuleState {
  rules: string[];
  previewData: string[];
  isLoading: boolean;
  error: string | null;
}

interface RuleActions {
  preview: () => Promise<void>;
  apply: (confirm: boolean) => Promise<void>;
  reset: () => void;
}

const initialState: RuleState = {
  rules: [],
  previewData: [],
  isLoading: false,
  error: null,
};

export const useRuleStore = create<RuleState & RuleActions>((set) => ({
  ...initialState,

  preview: async () => {
    set({ isLoading: true, error: null });
    try {
      const rules = await previewRules();
      set({
        previewData: rules,
        isLoading: false,
      });
    } catch (err) {
      set({
        isLoading: false,
        error: err instanceof Error ? err.message : 'Failed to preview rules',
      });
    }
  },

  apply: async (confirm: boolean) => {
    set({ isLoading: true, error: null });
    try {
      const response = await applyRules(confirm);
      if (response.success) {
        set((state) => ({
          rules: state.previewData,
          isLoading: false,
        }));
      } else {
        set({
          isLoading: false,
          error: response.error || 'Failed to apply rules',
        });
      }
    } catch (err) {
      set({
        isLoading: false,
        error: err instanceof Error ? err.message : 'Failed to apply rules',
      });
    }
  },

  reset: () => set(initialState),
}));