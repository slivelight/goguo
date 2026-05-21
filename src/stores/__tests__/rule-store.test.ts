import { describe, it, expect, beforeEach, vi } from 'vitest';
import { useRuleStore } from '../rule-store';
import { previewRules, applyRules } from '../../lib/tauri-ipc';

vi.mock('../../lib/tauri-ipc', () => ({
  previewRules: vi.fn(),
  applyRules: vi.fn(),
}));

describe('rule-store', () => {
  beforeEach(() => {
    useRuleStore.getState().reset();
    vi.clearAllMocks();
  });

  it('initial state is correct', () => {
    const state = useRuleStore.getState();
    expect(state.rules).toEqual([]);
    expect(state.previewData).toEqual([]);
    expect(state.isLoading).toBe(false);
    expect(state.error).toBeNull();
  });

  it('preview updates previewData', async () => {
    vi.mocked(previewRules).mockResolvedValue([
      'DOMAIN-SUFFIX,github.com',
      'DOMAIN-SUFFIX,github.io',
    ]);

    await useRuleStore.getState().preview();

    const state = useRuleStore.getState();
    expect(state.previewData).toHaveLength(2);
    expect(state.previewData[0]).toBe('DOMAIN-SUFFIX,github.com');
  });

  it('preview handles error', async () => {
    vi.mocked(previewRules).mockRejectedValue(new Error('Preview failed'));

    await useRuleStore.getState().preview();

    const state = useRuleStore.getState();
    expect(state.error).toBe('Preview failed');
  });

  it('apply with confirm=true updates rules', async () => {
    vi.mocked(previewRules).mockResolvedValue(['DOMAIN-SUFFIX,test.com']);
    vi.mocked(applyRules).mockResolvedValue({
      success: true,
      rules_generated: 1,
      verification_passed: true,
    });

    await useRuleStore.getState().preview();
    await useRuleStore.getState().apply(true);

    const state = useRuleStore.getState();
    expect(state.rules).toHaveLength(1);
    expect(state.rules[0]).toBe('DOMAIN-SUFFIX,test.com');
  });

  it('apply with confirm=false does not update rules', async () => {
    vi.mocked(applyRules).mockResolvedValue({
      success: false,
      rules_generated: 0,
      verification_passed: false,
      error: 'Requires confirmation',
    });

    await useRuleStore.getState().apply(false);

    const state = useRuleStore.getState();
    expect(state.rules).toHaveLength(0);
    expect(state.error).toBe('Requires confirmation');
  });
});