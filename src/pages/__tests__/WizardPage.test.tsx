import { describe, it, expect, vi, beforeEach } from 'vitest';
import { render, screen, fireEvent } from '@testing-library/react';
import WizardPage from '../WizardPage';
import * as wizardStore from '../../stores/wizard-store';
import * as notifStore from '../../stores/notif-store';
import * as ipc from '../../lib/tauri-ipc';

const mockNavigate = vi.fn();

vi.mock('react-router-dom', () => ({
  useNavigate: () => mockNavigate,
}));

vi.mock('../../stores/wizard-store', () => ({
  useWizardStore: vi.fn(),
}));

vi.mock('../../stores/notif-store', () => ({
  useNotifStore: vi.fn(),
}));

vi.mock('../../lib/tauri-ipc', () => ({
  detectDeploymentMode: vi.fn(),
  startInitialAssessment: vi.fn(),
  confirmBaseline: vi.fn(),
  applyPresetTemplate: vi.fn(),
  addTargetSite: vi.fn(),
}));

describe('WizardPage', () => {
  beforeEach(() => {
    vi.clearAllMocks();
    vi.mocked(wizardStore.useWizardStore).mockReturnValue({
      currentStep: 'welcome',
      deploymentMode: null,
      hasBaseline: false,
      selectedSites: [],
      completedSteps: [],
      nextStep: vi.fn(),
      prevStep: vi.fn(),
      setDeploymentMode: vi.fn(),
      setHasBaseline: vi.fn(),
      selectSites: vi.fn(),
      markComplete: vi.fn(),
    } as unknown as ReturnType<typeof wizardStore.useWizardStore>);
    
    vi.mocked(notifStore.useNotifStore).mockReturnValue({
      addNotification: vi.fn(),
    } as unknown as ReturnType<typeof notifStore.useNotifStore>);
    
    vi.mocked(ipc.detectDeploymentMode).mockResolvedValue({
      mode: 'windows_only',
      detected: 'windows_only',
      is_auto: true,
    });
  });

  it('renders wizard title', () => {
    render(<WizardPage />);
    expect(screen.getByText('欢迎使用 GoGuo')).toBeDefined();
  });

  it('shows 7 step indicators', () => {
    render(<WizardPage />);
    const steps = screen.getAllByText(/[1-7]/);
    expect(steps.length).toBeGreaterThanOrEqual(7);
  });

  it('shows welcome step content', () => {
    render(<WizardPage />);
    expect(screen.getByText('网络可达性诊断工具')).toBeDefined();
    expect(screen.getByText(/本向导将引导您完成初始配置/)).toBeDefined();
  });

  it('shows next button', () => {
    render(<WizardPage />);
    expect(screen.getByText('下一步')).toBeDefined();
  });

  it('hides prev button on welcome step', () => {
    render(<WizardPage />);
    expect(screen.queryByText('上一步')).toBeNull();
  });

  it('calls nextStep when next button clicked on welcome step', () => {
    const nextStep = vi.fn();
    const markComplete = vi.fn();
    vi.mocked(wizardStore.useWizardStore).mockReturnValue({
      currentStep: 'welcome',
      deploymentMode: null,
      hasBaseline: false,
      selectedSites: [],
      completedSteps: [],
      nextStep,
      prevStep: vi.fn(),
      setDeploymentMode: vi.fn(),
      setHasBaseline: vi.fn(),
      selectSites: vi.fn(),
      markComplete,
    } as unknown as ReturnType<typeof wizardStore.useWizardStore>);

    render(<WizardPage />);
    fireEvent.click(screen.getByText('下一步'));
    expect(nextStep).toHaveBeenCalled();
  });

  it('shows deployment mode step', () => {
    vi.mocked(wizardStore.useWizardStore).mockReturnValue({
      currentStep: 'deployment-mode',
      deploymentMode: null,
      hasBaseline: false,
      selectedSites: [],
      completedSteps: [],
      nextStep: vi.fn(),
      prevStep: vi.fn(),
      setDeploymentMode: vi.fn(),
      setHasBaseline: vi.fn(),
      selectSites: vi.fn(),
      markComplete: vi.fn(),
    } as unknown as ReturnType<typeof wizardStore.useWizardStore>);

    render(<WizardPage />);
    expect(screen.getByText('选择部署模式')).toBeDefined();
    expect(screen.getByText('Windows Only')).toBeDefined();
    expect(screen.getByText('WSL Only')).toBeDefined();
  });

  it('shows deployment mode options', () => {
    vi.mocked(wizardStore.useWizardStore).mockReturnValue({
      currentStep: 'deployment-mode',
      deploymentMode: null,
      hasBaseline: false,
      selectedSites: [],
      completedSteps: [],
      nextStep: vi.fn(),
      prevStep: vi.fn(),
      setDeploymentMode: vi.fn(),
      setHasBaseline: vi.fn(),
      selectSites: vi.fn(),
      markComplete: vi.fn(),
    } as unknown as ReturnType<typeof wizardStore.useWizardStore>);

    render(<WizardPage />);
    expect(screen.getByText('Linux Only')).toBeDefined();
    expect(screen.getByText('Coordinated')).toBeDefined();
  });

  it('shows prev button on deployment-mode step', () => {
    vi.mocked(wizardStore.useWizardStore).mockReturnValue({
      currentStep: 'deployment-mode',
      deploymentMode: null,
      hasBaseline: false,
      selectedSites: [],
      completedSteps: [],
      nextStep: vi.fn(),
      prevStep: vi.fn(),
      setDeploymentMode: vi.fn(),
      setHasBaseline: vi.fn(),
      selectSites: vi.fn(),
      markComplete: vi.fn(),
    } as unknown as ReturnType<typeof wizardStore.useWizardStore>);

    render(<WizardPage />);
    expect(screen.getByText('上一步')).toBeDefined();
  });

  it('shows initial-assessment step', () => {
    vi.mocked(wizardStore.useWizardStore).mockReturnValue({
      currentStep: 'initial-assessment',
      deploymentMode: 'windows_only',
      hasBaseline: false,
      selectedSites: [],
      completedSteps: ['deployment-mode'],
      nextStep: vi.fn(),
      prevStep: vi.fn(),
      setDeploymentMode: vi.fn(),
      setHasBaseline: vi.fn(),
      selectSites: vi.fn(),
      markComplete: vi.fn(),
    } as unknown as ReturnType<typeof wizardStore.useWizardStore>);

    render(<WizardPage />);
    expect(screen.getByText('初始网络评估')).toBeDefined();
    expect(screen.getByText('尚未评估')).toBeDefined();
  });

  it('shows baseline-confirm step', () => {
    vi.mocked(wizardStore.useWizardStore).mockReturnValue({
      currentStep: 'baseline-confirm',
      deploymentMode: 'windows_only',
      hasBaseline: false,
      selectedSites: [],
      completedSteps: ['deployment-mode', 'initial-assessment'],
      nextStep: vi.fn(),
      prevStep: vi.fn(),
      setDeploymentMode: vi.fn(),
      setHasBaseline: vi.fn(),
      selectSites: vi.fn(),
      markComplete: vi.fn(),
    } as unknown as ReturnType<typeof wizardStore.useWizardStore>);

    render(<WizardPage />);
    expect(screen.getByText('确认 Baseline')).toBeDefined();
  });

  it('shows site-selection step', () => {
    vi.mocked(wizardStore.useWizardStore).mockReturnValue({
      currentStep: 'site-selection',
      deploymentMode: 'windows_only',
      hasBaseline: true,
      selectedSites: [],
      completedSteps: ['deployment-mode', 'initial-assessment', 'baseline-confirm'],
      nextStep: vi.fn(),
      prevStep: vi.fn(),
      setDeploymentMode: vi.fn(),
      setHasBaseline: vi.fn(),
      selectSites: vi.fn(),
      markComplete: vi.fn(),
    } as unknown as ReturnType<typeof wizardStore.useWizardStore>);

    render(<WizardPage />);
    expect(screen.getByText('选择目标站点')).toBeDefined();
    expect(screen.getByText('GitHub')).toBeDefined();
    expect(screen.getByText('npm')).toBeDefined();
  });

  it('shows finish step', () => {
    vi.mocked(wizardStore.useWizardStore).mockReturnValue({
      currentStep: 'finish',
      deploymentMode: 'windows_only',
      hasBaseline: true,
      selectedSites: ['github', 'npm'],
      completedSteps: ['deployment-mode', 'initial-assessment', 'baseline-confirm', 'site-selection', 'rule-preview'],
      nextStep: vi.fn(),
      prevStep: vi.fn(),
      setDeploymentMode: vi.fn(),
      setHasBaseline: vi.fn(),
      selectSites: vi.fn(),
      markComplete: vi.fn(),
    } as unknown as ReturnType<typeof wizardStore.useWizardStore>);

    render(<WizardPage />);
    expect(screen.getByText('🎉 配置完成！')).toBeDefined();
    expect(screen.getByText('开始使用')).toBeDefined();
  });

  it('navigates to dashboard on finish', () => {
    const markComplete = vi.fn();
    vi.mocked(wizardStore.useWizardStore).mockReturnValue({
      currentStep: 'finish',
      deploymentMode: 'windows_only',
      hasBaseline: true,
      selectedSites: ['github'],
      completedSteps: ['deployment-mode', 'initial-assessment', 'baseline-confirm', 'site-selection', 'rule-preview'],
      nextStep: vi.fn(),
      prevStep: vi.fn(),
      setDeploymentMode: vi.fn(),
      setHasBaseline: vi.fn(),
      selectSites: vi.fn(),
      markComplete,
    } as unknown as ReturnType<typeof wizardStore.useWizardStore>);

    render(<WizardPage />);
    fireEvent.click(screen.getByText('开始使用'));
    expect(markComplete).toHaveBeenCalled();
    expect(mockNavigate).toHaveBeenCalledWith('/dashboard');
  });

  it('applies developer template when all selected sites are developer subset', async () => {
    const markComplete = vi.fn();
    const nextStep = vi.fn();
    vi.mocked(wizardStore.useWizardStore).mockReturnValue({
      currentStep: 'site-selection',
      deploymentMode: 'windows_only',
      hasBaseline: true,
      selectedSites: ['github', 'npm'],
      completedSteps: ['deployment-mode', 'initial-assessment', 'baseline-confirm'],
      nextStep,
      prevStep: vi.fn(),
      setDeploymentMode: vi.fn(),
      setHasBaseline: vi.fn(),
      selectSites: vi.fn(),
      markComplete,
    } as unknown as ReturnType<typeof wizardStore.useWizardStore>);

    vi.mocked(ipc.applyPresetTemplate).mockResolvedValue({
      added_count: 2,
      failed_count: 0,
      sites: ['github', 'npm'],
    });

    render(<WizardPage />);
    fireEvent.click(screen.getByText('下一步'));

    await vi.waitFor(() => {
      expect(ipc.applyPresetTemplate).toHaveBeenCalledWith('developer');
    });
  });

  it('adds sites individually when not matching template', async () => {
    const markComplete = vi.fn();
    vi.mocked(wizardStore.useWizardStore).mockReturnValue({
      currentStep: 'site-selection',
      deploymentMode: 'windows_only',
      hasBaseline: true,
      selectedSites: ['custom-site'],
      completedSteps: ['deployment-mode', 'initial-assessment', 'baseline-confirm'],
      nextStep: vi.fn(),
      prevStep: vi.fn(),
      setDeploymentMode: vi.fn(),
      setHasBaseline: vi.fn(),
      selectSites: vi.fn(),
      markComplete,
    } as unknown as ReturnType<typeof wizardStore.useWizardStore>);

    vi.mocked(ipc.addTargetSite).mockResolvedValue({
      success: true,
      rules_generated: 1,
      verification_passed: true,
    });

    render(<WizardPage />);
    fireEvent.click(screen.getByText('下一步'));

    await vi.waitFor(() => {
      expect(ipc.addTargetSite).toHaveBeenCalledWith('custom-site');
    });
  });

  it('shows impact description for deployment modes', () => {
    vi.mocked(wizardStore.useWizardStore).mockReturnValue({
      currentStep: 'deployment-mode',
      deploymentMode: null,
      hasBaseline: false,
      selectedSites: [],
      completedSteps: [],
      nextStep: vi.fn(),
      prevStep: vi.fn(),
      setDeploymentMode: vi.fn(),
      setHasBaseline: vi.fn(),
      selectSites: vi.fn(),
      markComplete: vi.fn(),
    } as unknown as ReturnType<typeof wizardStore.useWizardStore>);

    render(<WizardPage />);
    expect(screen.getByText(/仅监控和管理 Windows 系统的代理配置/)).toBeDefined();
  });
});