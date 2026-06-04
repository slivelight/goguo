import { describe, it, expect, beforeEach, vi } from 'vitest';
import { useSiteStore } from '../site-store';
import { addTargetSite, removeTargetSite, applyPresetTemplate, getSiteReachability, listSiteDefinitions } from '../../lib/tauri-ipc';

vi.mock('../../lib/tauri-ipc', () => ({
  addTargetSite: vi.fn(),
  removeTargetSite: vi.fn(),
  applyPresetTemplate: vi.fn(),
  getSiteReachability: vi.fn(),
  listSiteDefinitions: vi.fn(),
}));

describe('site-store', () => {
  beforeEach(() => {
    useSiteStore.getState().reset();
    vi.clearAllMocks();
  });

  it('initial state is correct', () => {
    const state = useSiteStore.getState();
    expect(state.sites).toEqual([]);
    expect(state.reachability).toEqual([]);
    expect(state.isLoading).toBe(false);
    expect(state.error).toBeNull();
  });

  it('fetchSites updates state from reachability', async () => {
    vi.mocked(getSiteReachability).mockResolvedValue({
      sites: [
        { site_id: 'github', reachable: true },
        { site_id: 'npm', reachable: true },
      ],
    });
    vi.mocked(listSiteDefinitions).mockResolvedValue([
      { id: 'github', name: 'GitHub', domain_count: 47, domains: { core: ['github.com'] } },
      { id: 'npm', name: 'npm', domain_count: 3, domains: { core: ['npmjs.com'] } },
    ]);

    await useSiteStore.getState().fetchSites();

    const state = useSiteStore.getState();
    expect(state.sites).toHaveLength(2);
    expect(state.sites[0].name).toBe('GitHub');
    expect(state.sites[0].domain_count).toBe(47);
    expect(state.reachability).toHaveLength(2);
  });

  it('addSite adds site on success', async () => {
    vi.mocked(addTargetSite).mockResolvedValue({
      success: true,
      site: { id: 'github', name: 'GitHub', domain_count: 5, domains: {} },
      rules_generated: 10,
      verification_passed: true,
    });

    const result = await useSiteStore.getState().addSite('github');

    expect(result.success).toBe(true);
    const state = useSiteStore.getState();
    expect(state.sites).toHaveLength(1);
    expect(state.sites[0].id).toBe('github');
  });

  it('addSite handles failure', async () => {
    vi.mocked(addTargetSite).mockResolvedValue({
      success: false,
      rules_generated: 0,
      verification_passed: false,
      error: 'Site not found',
    });

    const result = await useSiteStore.getState().addSite('unknown');

    expect(result.success).toBe(false);
    expect(useSiteStore.getState().error).toBe('Site not found');
  });

  it('removeSite removes site on success', async () => {
    vi.mocked(getSiteReachability).mockResolvedValue({
      sites: [{ site_id: 'github', reachable: true }],
    });
    vi.mocked(listSiteDefinitions).mockResolvedValue([
      { id: 'github', name: 'GitHub', domain_count: 47, domains: { core: ['github.com'] } },
    ]);
    vi.mocked(removeTargetSite).mockResolvedValue({
      success: true,
      remaining_sites: 0,
    });

    await useSiteStore.getState().fetchSites();
    await useSiteStore.getState().removeSite('github');

    const state = useSiteStore.getState();
    expect(state.sites).toHaveLength(0);
    expect(state.reachability).toHaveLength(0);
  });

  it('applyTemplate returns response', async () => {
    vi.mocked(applyPresetTemplate).mockResolvedValue({
      added_count: 3,
      failed_count: 0,
      sites: ['github', 'npm', 'docker'],
    });

    const result = await useSiteStore.getState().applyTemplate('developer');

    expect(result.added_count).toBe(3);
    expect(result.sites).toContain('github');
  });

  it('fetchReachability updates reachability', async () => {
    vi.mocked(getSiteReachability).mockResolvedValue({
      sites: [{ site_id: 'github', reachable: true, response_time_ms: 100 }],
    });

    await useSiteStore.getState().fetchReachability();

    const state = useSiteStore.getState();
    expect(state.reachability).toHaveLength(1);
    expect(state.reachability[0].response_time_ms).toBe(100);
  });
});