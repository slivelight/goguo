import { create } from 'zustand';
import { previewRules } from '../lib/tauri-ipc';
import type { FiveElementPrompt } from '../lib/types';

interface RuleState {
  rules: string[];
  previewData: string[];
  isLoading: boolean;
  error: string | null;
  failurePrompt: FiveElementPrompt | null;
}

interface RuleActions {
  preview: () => Promise<void>;
  reset: () => void;
}

const initialState: RuleState = {
  rules: [],
  previewData: [],
  isLoading: false,
  error: null,
  failurePrompt: null,
};

export const useRuleStore = create<RuleState & RuleActions>((set) => ({
  ...initialState,

  preview: async () => {
    set({ isLoading: true, error: null, failurePrompt: null });
    try {
      const rules = await previewRules();
      set({
        previewData: rules,
        rules: rules,  // Rules are applied automatically; preview = current state
        isLoading: false,
      });
    } catch (err) {
      set({
        isLoading: false,
        error: err instanceof Error ? err.message : 'Failed to preview rules',
      });
    }
  },

  reset: () => set(initialState),
}));
