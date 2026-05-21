import { describe, it, expect, beforeEach } from 'vitest';
import { useWizardStore } from '../wizard-store';

describe('wizard-store', () => {
  beforeEach(() => {
    useWizardStore.getState().reset();
  });

  it('initial state is correct', () => {
    const state = useWizardStore.getState();
    expect(state.currentStep).toBe('welcome');
    expect(state.completedSteps).toEqual([]);
    expect(state.deploymentMode).toBeNull();
    expect(state.hasBaseline).toBe(false);
    expect(state.selectedSites).toEqual([]);
  });

  it('nextStep advances step', () => {
    useWizardStore.getState().nextStep();
    expect(useWizardStore.getState().currentStep).toBe('deployment-mode');
  });

  it('nextStep stops at finish', () => {
    useWizardStore.getState().goToStep('finish');
    useWizardStore.getState().nextStep();
    expect(useWizardStore.getState().currentStep).toBe('finish');
  });

  it('prevStep goes back', () => {
    useWizardStore.getState().goToStep('baseline-confirm');
    useWizardStore.getState().prevStep();
    expect(useWizardStore.getState().currentStep).toBe('initial-assessment');
  });

  it('prevStep stops at welcome', () => {
    useWizardStore.getState().prevStep();
    expect(useWizardStore.getState().currentStep).toBe('welcome');
  });

  it('goToStep sets specific step', () => {
    useWizardStore.getState().goToStep('site-selection');
    expect(useWizardStore.getState().currentStep).toBe('site-selection');
  });

  it('setDeploymentMode updates mode', () => {
    useWizardStore.getState().setDeploymentMode('windows_only');
    expect(useWizardStore.getState().deploymentMode).toBe('windows_only');
  });

  it('setHasBaseline updates flag', () => {
    useWizardStore.getState().setHasBaseline(true);
    expect(useWizardStore.getState().hasBaseline).toBe(true);
  });

  it('selectSites updates sites', () => {
    useWizardStore.getState().selectSites(['github', 'npm']);
    expect(useWizardStore.getState().selectedSites).toEqual(['github', 'npm']);
  });

  it('markComplete adds to completedSteps', () => {
    useWizardStore.getState().markComplete('welcome');
    useWizardStore.getState().markComplete('deployment-mode');
    expect(useWizardStore.getState().completedSteps).toEqual(['welcome', 'deployment-mode']);
  });

  it('reset restores initial state', () => {
    useWizardStore.getState().goToStep('finish');
    useWizardStore.getState().setDeploymentMode('coordinated');
    useWizardStore.getState().reset();

    const state = useWizardStore.getState();
    expect(state.currentStep).toBe('welcome');
    expect(state.deploymentMode).toBeNull();
  });
});