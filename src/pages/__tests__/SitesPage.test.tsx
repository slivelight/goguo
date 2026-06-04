import { describe, it, expect, vi, beforeEach } from 'vitest';
import { render, screen, fireEvent } from '@testing-library/react';
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
        { id: 'github', name: 'GitHub', domain_count: 5, domains: {} },
        { id: 'npm', name: 'npm', domain_count: 2, domains: {} },
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

  it('shows AddSiteDialog with input field when add button clicked', () => {
    render(<SitesPage />);
    fireEvent.click(screen.getByText('添加站点'));
    expect(screen.getByPlaceholderText(/输入网址或域名/)).toBeDefined();
  });

  it('shows TemplateConfirmDialog with site list when developer template clicked', () => {
    render(<SitesPage />);
    fireEvent.click(screen.getByText('开发者模板'));
    expect(screen.getByText('GitHub')).toBeDefined();
    expect(screen.getByText('npm')).toBeDefined();
  });

  it('shows TemplateConfirmDialog with site list when office template clicked', () => {
    render(<SitesPage />);
    fireEvent.click(screen.getByText('办公模板'));
    expect(screen.getByText('Google')).toBeDefined();
    expect(screen.getByText('Wikipedia')).toBeDefined();
  });

  it('shows domain list when site card is expanded', () => {
    vi.mocked(siteStore.useSiteStore).mockReturnValue({
      sites: [
        { id: 'github', name: 'GitHub', domain_count: 3, domains: { core: ['github.com', 'github.io', 'githubusercontent.com'] } },
      ],
      reachability: [
        { site_id: 'github', reachable: true },
      ],
      fetchSites: vi.fn(),
      addSite: vi.fn(),
      removeSite: vi.fn(),
      applyTemplate: vi.fn(),
    } as unknown as ReturnType<typeof siteStore.useSiteStore>);

    render(<SitesPage />);

    // Click expand button for github
    const expandBtn = screen.getByText('展开域名');
    fireEvent.click(expandBtn);

    expect(screen.getByText('github.com')).toBeDefined();
    expect(screen.getByText('github.io')).toBeDefined();
  });
});