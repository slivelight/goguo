import { create } from 'zustand';

export type WizardStep = 
  | 'welcome'
  | 'deployment-mode'
  | 'initial-assessment'
  | 'baseline-confirm'
  | 'site-selection'
  | 'rule-preview'
  | 'finish';

interface WizardState {
  currentStep: WizardStep;
  completedSteps: WizardStep[];
  deploymentMode: string | null;
  hasBaseline: boolean;
  selectedSites: string[];
  isLoading: boolean;
  error: string | null;
}

interface WizardActions {
  nextStep: () => void;
  prevStep: () => void;
  goToStep: (step: WizardStep) => void;
  setDeploymentMode: (mode: string) => void;
  setHasBaseline: (has: boolean) => void;
  selectSites: (sites: string[]) => void;
  markComplete: (step: WizardStep) => void;
  reset: () => void;
}

const stepOrder: WizardStep[] = [
  'welcome',
  'deployment-mode',
  'initial-assessment',
  'baseline-confirm',
  'site-selection',
  'rule-preview',
  'finish',
];

const initialState: WizardState = {
  currentStep: 'welcome',
  completedSteps: [],
  deploymentMode: null,
  hasBaseline: false,
  selectedSites: [],
  isLoading: false,
  error: null,
};

export const useWizardStore = create<WizardState & WizardActions>((set) => ({
  ...initialState,

  nextStep: () => {
    set((state) => {
      const currentIndex = stepOrder.indexOf(state.currentStep);
      if (currentIndex < stepOrder.length - 1) {
        return { currentStep: stepOrder[currentIndex + 1] };
      }
      return state;
    });
  },

  prevStep: () => {
    set((state) => {
      const currentIndex = stepOrder.indexOf(state.currentStep);
      if (currentIndex > 0) {
        return { currentStep: stepOrder[currentIndex - 1] };
      }
      return state;
    });
  },

  goToStep: (step: WizardStep) => {
    set({ currentStep: step });
  },

  setDeploymentMode: (mode: string) => {
    set({ deploymentMode: mode });
  },

  setHasBaseline: (has: boolean) => {
    set({ hasBaseline: has });
  },

  selectSites: (sites: string[]) => {
    set({ selectedSites: sites });
  },

  markComplete: (step: WizardStep) => {
    set((state) => ({
      completedSteps: [...state.completedSteps, step],
    }));
  },

  reset: () => set(initialState),
}));