import { describe, it, expect, vi, beforeEach } from 'vitest';
import { render, screen, fireEvent, waitFor } from '@testing-library/react';
import EditSiteDialog from '../EditSiteDialog';
import type { SiteInfo } from '../../../lib/types';

vi.mock('../../../lib/tauri-ipc', () => ({
  updateSiteDomains: vi.fn(),
}));

import { updateSiteDomains } from '../../../lib/tauri-ipc';

const mockCustomSite: SiteInfo = {
  id: 'my-site',
  name: 'My Site',
  domain_count: 3,
  domains: {
    core: ['a.example.com', 'b.example.com', 'c.example.com'],
  },
};

const mockTemplateAppliedSite: SiteInfo = {
  id: 'github',
  name: 'GitHub',
  domain_count: 47,
  domains: {
    core: ['github.com', 'github.io'],
    api: ['api.github.com'],
  },
};

describe('EditSiteDialog', () => {
  const mockOnSaved = vi.fn();
  const mockOnCancel = vi.fn();

  beforeEach(() => {
    vi.clearAllMocks();
  });

  it('does not render when isOpen=false', () => {
    render(<EditSiteDialog isOpen={false} site={mockCustomSite} onSaved={mockOnSaved} onCancel={mockOnCancel} />);
    expect(screen.queryByText(/编辑站点/)).toBeNull();
  });

  it('renders site name and domain list when open', () => {
    render(<EditSiteDialog isOpen={true} site={mockCustomSite} onSaved={mockOnSaved} onCancel={mockOnCancel} />);
    expect(screen.getByText(/编辑站点.*My Site/)).toBeDefined();
    expect(screen.getByText('a.example.com')).toBeDefined();
    expect(screen.getByText('b.example.com')).toBeDefined();
  });

  it('shows delete button for each domain', () => {
    render(<EditSiteDialog isOpen={true} site={mockCustomSite} onSaved={mockOnSaved} onCancel={mockOnCancel} />);
    const deleteButtons = screen.getAllByText('删除');
    expect(deleteButtons.length).toBe(3);
  });

  it('shows URL input for adding domains', () => {
    render(<EditSiteDialog isOpen={true} site={mockCustomSite} onSaved={mockOnSaved} onCancel={mockOnCancel} />);
    expect(screen.getByPlaceholderText(/输入网址添加域名/)).toBeDefined();
  });

  it('toggles domain delete/restore on click', () => {
    render(<EditSiteDialog isOpen={true} site={mockCustomSite} onSaved={mockOnSaved} onCancel={mockOnCancel} />);
    const deleteButtons = screen.getAllByText('删除');
    // Click delete on a.example.com → should show "恢复"
    fireEvent.click(deleteButtons[0]);
    expect(screen.getByText('恢复')).toBeDefined();
    // Domain text should still be visible (not removed from list)
    expect(screen.getByText('a.example.com')).toBeDefined();
    // Click restore → back to "删除"
    fireEvent.click(screen.getByText('恢复'));
    expect(screen.getAllByText('删除').length).toBe(3);
  });

  it('adds domain from URL input with underline style', () => {
    render(<EditSiteDialog isOpen={true} site={mockCustomSite} onSaved={mockOnSaved} onCancel={mockOnCancel} />);
    const input = screen.getByPlaceholderText(/输入网址添加域名/);
    fireEvent.change(input, { target: { value: 'https://d.example.com/path' } });
    fireEvent.click(screen.getByText('添加域名'));
    expect(screen.getByText(/d.example.com/)).toBeDefined();
    expect(screen.getByText('(新增)')).toBeDefined();
  });

  it('calls updateSiteDomains on confirm with correct add/remove lists', async () => {
    vi.mocked(updateSiteDomains).mockResolvedValue({
      success: true,
      site: { id: 'my-site', name: 'My Site', domain_count: 3, domains: {} },
      rules_generated: 3,
    });

    render(<EditSiteDialog isOpen={true} site={mockCustomSite} onSaved={mockOnSaved} onCancel={mockOnCancel} />);

    // Mark b.example.com for deletion
    const deleteButtons = screen.getAllByText('删除');
    fireEvent.click(deleteButtons[1]); // b.example.com

    // Add a new domain
    const input = screen.getByPlaceholderText(/输入网址添加域名/);
    fireEvent.change(input, { target: { value: 'https://d.example.com' } });
    fireEvent.click(screen.getByText('添加域名'));

    fireEvent.click(screen.getByText('确定'));

    await waitFor(() => {
      expect(updateSiteDomains).toHaveBeenCalledWith(
        'my-site',
        ['d.example.com'],
        ['b.example.com'],
      );
    });
  });

  it('calls onSaved after successful update', async () => {
    vi.mocked(updateSiteDomains).mockResolvedValue({
      success: true,
      site: { id: 'my-site', name: 'My Site', domain_count: 4, domains: {} },
      rules_generated: 4,
    });

    render(<EditSiteDialog isOpen={true} site={mockCustomSite} onSaved={mockOnSaved} onCancel={mockOnCancel} />);

    // Add a domain
    const input = screen.getByPlaceholderText(/输入网址添加域名/);
    fireEvent.change(input, { target: { value: 'https://new.example.com' } });
    fireEvent.click(screen.getByText('添加域名'));

    fireEvent.click(screen.getByText('确定'));

    await waitFor(() => {
      expect(mockOnSaved).toHaveBeenCalled();
    });
  });

  it('calls onCancel when cancel clicked', () => {
    render(<EditSiteDialog isOpen={true} site={mockCustomSite} onSaved={mockOnSaved} onCancel={mockOnCancel} />);
    fireEvent.click(screen.getByText('取消'));
    expect(mockOnCancel).toHaveBeenCalledOnce();
  });

  it('allows editing template-applied sites (e.g. github)', () => {
    render(<EditSiteDialog isOpen={true} site={mockTemplateAppliedSite} onSaved={mockOnSaved} onCancel={mockOnCancel} />);
    // Title should say "编辑站点"
    expect(screen.getByText(/编辑站点.*GitHub/)).toBeDefined();
    // Domains should be listed
    expect(screen.getByText('github.com')).toBeDefined();
    expect(screen.getByText('api.github.com')).toBeDefined();
    // Delete buttons visible for all domains
    expect(screen.getAllByText('删除').length).toBe(3);
    // Input and confirm button visible
    expect(screen.getByPlaceholderText(/输入网址添加域名/)).toBeDefined();
    expect(screen.getByText('确定')).toBeDefined();
  });

  it('disables confirm when no changes made', () => {
    render(<EditSiteDialog isOpen={true} site={mockCustomSite} onSaved={mockOnSaved} onCancel={mockOnCancel} />);
    const confirmButton = screen.getByText('确定') as HTMLButtonElement;
    expect(confirmButton.disabled).toBe(true);
  });
});
