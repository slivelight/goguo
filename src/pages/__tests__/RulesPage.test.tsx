import { describe, it, expect, vi, beforeEach } from 'vitest';
import { render, screen } from '@testing-library/react';
import RulesPage from '../RulesPage';
import * as ruleStore from '../../stores/rule-store';
import * as notifStore from '../../stores/notif-store';

vi.mock('../../stores/rule-store', () => ({
  useRuleStore: vi.fn(),
}));

vi.mock('../../stores/notif-store', () => ({
  useNotifStore: vi.fn(),
}));

describe('RulesPage', () => {
  beforeEach(() => {
    vi.clearAllMocks();
    vi.mocked(ruleStore.useRuleStore).mockReturnValue({
      rules: [],
      previewData: [],
      preview: vi.fn(),
      apply: vi.fn(),
      isLoading: false,
    } as unknown as ReturnType<typeof ruleStore.useRuleStore>);
    
    vi.mocked(notifStore.useNotifStore).mockReturnValue({
      addNotification: vi.fn(),
    } as unknown as ReturnType<typeof notifStore.useNotifStore>);
  });

  it('renders rules title', () => {
    render(<RulesPage />);
    expect(screen.getByText('规则预览')).toBeDefined();
  });

  it('shows preview rules card', () => {
    render(<RulesPage />);
    expect(screen.getByText('预览规则')).toBeDefined();
  });

  it('shows applied rules card', () => {
    render(<RulesPage />);
    expect(screen.getByText('已应用规则')).toBeDefined();
  });

  it('shows empty state when no rules', () => {
    render(<RulesPage />);
    expect(screen.getByText('暂无规则，请先添加目标站点')).toBeDefined();
  });

  it('shows refresh preview button', () => {
    render(<RulesPage />);
    expect(screen.getByText('刷新预览')).toBeDefined();
  });

  it('shows rule count', () => {
    render(<RulesPage />);
    expect(screen.getByText('规则总数: 0')).toBeDefined();
  });

  it('shows preview data when available', () => {
    vi.mocked(ruleStore.useRuleStore).mockReturnValue({
      rules: [],
      previewData: ['DOMAIN-SUFFIX,github.com', 'DOMAIN-SUFFIX,npmjs.com'],
      preview: vi.fn(),
      apply: vi.fn(),
      isLoading: false,
    } as unknown as ReturnType<typeof ruleStore.useRuleStore>);

    render(<RulesPage />);
    expect(screen.getByText('规则总数: 2')).toBeDefined();
  });
});