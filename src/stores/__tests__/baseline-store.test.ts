import { describe, it, expect, beforeEach, vi } from 'vitest';
import { useBaselineStore, initializeBaselineStore } from '../baseline-store';
import { getBaselineStatus, confirmBaseline, startInitialAssessment } from '../../lib/tauri-ipc';
import { subscribeBaselineConfirmed, subscribeBaselineDeviation } from '../../lib/events';

vi.mock('../../lib/tauri-ipc', () => ({
  getBaselineStatus: vi.fn(),
  confirmBaseline: vi.fn(),
  startInitialAssessment: vi.fn(),
  triggerReadjustment: vi.fn(),
}));

vi.mock('../../lib/events', () => ({
  subscribeBaselineConfirmed: vi.fn((_cb) => Promise.resolve(() => {})),
  subscribeBaselineDeviation: vi.fn((_cb) => Promise.resolve(() => {})),
}));

describe('baseline-store', () => {
  beforeEach(() => {
    useBaselineStore.getState().reset();
    vi.clearAllMocks();
  });

  it('initial state is correct', () => {
    const state = useBaselineStore.getState();
    expect(state.hasBaseline).toBe(false);
    expect(state.items).toEqual([]);
    expect(state.version).toBeNull();
    expect(state.itemCount).toBeNull();
    expect(state.deviatedItems).toEqual([]);
    expect(state.isLoading).toBe(false);
    expect(state.error).toBeNull();
  });

  it('fetchBaselineStatus updates state on success', async () => {
    vi.mocked(getBaselineStatus).mockResolvedValue({
      has_baseline: true,
      items: [
        { state_item_id: 'win-hosts', result: 'match' },
        { state_item_id: 'win-proxy', result: 'deviated' },
      ],
    });

    await useBaselineStore.getState().fetchBaselineStatus();

    const state = useBaselineStore.getState();
    expect(state.hasBaseline).toBe(true);
    expect(state.items).toHaveLength(2);
    expect(state.isLoading).toBe(false);
  });

  it('fetchBaselineStatus handles error', async () => {
    vi.mocked(getBaselineStatus).mockRejectedValue(new Error('Failed to load'));

    await useBaselineStore.getState().fetchBaselineStatus();

    const state = useBaselineStore.getState();
    expect(state.isLoading).toBe(false);
    expect(state.error).toBe('Failed to load');
  });

  it('confirmBaseline updates version and itemCount', async () => {
    vi.mocked(confirmBaseline).mockResolvedValue({
      version: 2,
      timestamp: '2026-05-21T08:00:00Z',
      item_count: 9,
    });

    await useBaselineStore.getState().confirmBaseline();

    const state = useBaselineStore.getState();
    expect(state.hasBaseline).toBe(true);
    expect(state.version).toBe(2);
    expect(state.itemCount).toBe(9);
  });

  it('startAssessment sets hasBaseline to false', async () => {
    vi.mocked(startInitialAssessment).mockResolvedValue({
      version: 1,
      timestamp: '2026-05-21T08:00:00Z',
      item_count: 9,
    });

    await useBaselineStore.getState().startAssessment();

    const state = useBaselineStore.getState();
    expect(state.hasBaseline).toBe(false);
    expect(state.version).toBe(1);
  });

  it('handleBaselineConfirmed updates state', () => {
    useBaselineStore.getState().handleBaselineConfirmed({
      version: 3,
      item_count: 10,
    });

    const state = useBaselineStore.getState();
    expect(state.hasBaseline).toBe(true);
    expect(state.version).toBe(3);
    expect(state.itemCount).toBe(10);
  });

  it('handleBaselineDeviation updates deviatedItems', () => {
    useBaselineStore.getState().handleBaselineDeviation({
      deviated_items: ['win-proxy', 'win-hosts'],
    });

    const state = useBaselineStore.getState();
    expect(state.deviatedItems).toEqual(['win-proxy', 'win-hosts']);
  });

  it('getDeviatedCount returns correct count', () => {
    useBaselineStore.setState({
      items: [
        { state_item_id: 'a', result: 'match' },
        { state_item_id: 'b', result: 'deviated' },
        { state_item_id: 'c', result: 'deviated' },
      ],
    });

    expect(useBaselineStore.getState().getDeviatedCount()).toBe(2);
  });

  it('getMatchCount returns correct count', () => {
    useBaselineStore.setState({
      items: [
        { state_item_id: 'a', result: 'match' },
        { state_item_id: 'b', result: 'deviated' },
        { state_item_id: 'c', result: 'match' },
      ],
    });

    expect(useBaselineStore.getState().getMatchCount()).toBe(2);
  });

  it('initializeBaselineStore subscribes to events', async () => {
    initializeBaselineStore();

    expect(subscribeBaselineConfirmed).toHaveBeenCalled();
    expect(subscribeBaselineDeviation).toHaveBeenCalled();
  });
});