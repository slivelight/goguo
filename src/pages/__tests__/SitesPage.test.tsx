import { describe, it, expect, vi, beforeEach } from 'vitest';
import { render, screen } from '@testing-library/react';
import SitesPage from '../SitesPage';
import * as siteStore from '../../stores/site-store';
import * as notifStore from '../../stores/notif-store';

vi.mock('../../stores/site-store', () => ({
  useSiteStore: vi.fn(),
}));

vi.mock('../../stores/notif-store', () => ({
  useNotifStore: vi.fn(),
}));

describe('SitesPage', () => {
  beforeEach(() => {
    vi.clearAllMocks();
    vi.mocked(siteStore.useSiteStore).mockReturnValue({
      sites: [],
      reachability: [],
      fetchSites: vi.fn(),
      addSite: vi.fn(),
      removeSite: vi.fn(),
      applyTemplate: vi.fn(),
    } as unknown as ReturnType<typeof siteStore.useSiteStore>);
    
    vi.mocked(notifStore.useNotifStore).mockReturnValue({
      addNotification: vi.fn(),
    } as unknown as ReturnType<typeof notifStore.useNotifStore>);
  });

  it('renders sites title', () => {
    render(<SitesPage />);
    expect(screen.getByText('站点管理')).toBeDefined();
  });

  it('shows add site button', () => {
    render(<SitesPage />);
    expect(screen.getByText('添加站点')).toBeDefined();
  });

  it('shows empty state when no sites', () => {
    render(<SitesPage />);
    expect(screen.getByText('暂无已添加站点')).toBeDefined();
  });

  it('shows preset templates', () => {
    render(<SitesPage />);
    expect(screen.getByText('预设模板')).toBeDefined();
    expect(screen.getByText('开发者模板')).toBeDefined();
    expect(screen.getByText('办公模板')).toBeDefined();
  });

  it('shows site list when sites exist', () => {
    vi.mocked(siteStore.useSiteStore).mockReturnValue({
      sites: [
        { id: 'github', name: 'GitHub', domain_count: 5 },
        { id: 'npm', name: 'npm', domain_count: 2 },
      ],
      reachability: [
        { site_id: 'github', reachable: true },
        { site_id: 'npm', reachable: true },
      ],
      fetchSites: vi.fn(),
      addSite: vi.fn(),
      removeSite: vi.fn(),
      applyTemplate: vi.fn(),
    } as unknown as ReturnType<typeof siteStore.useSiteStore>);

    render(<SitesPage />);
    expect(screen.getByText('GitHub')).toBeDefined();
    expect(screen.getByText('npm')).toBeDefined();
  });
});