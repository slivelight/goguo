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

describe('EditSiteDialog', () => {
  const mockOnSaved = vi.fn();
  const mockOnCancel = vi.fn();

  beforeEach(() => {
    vi.clearAllMocks();
  });

  it('does not render when isOpen=false', () => {
    render(<EditSiteDialog isOpen={false} site={mockCustomSite} onSaved={mockOnSaved} onCancel={mockOnCancel} />);
    expect(screen.queryByText('编辑站点')).toBeNull();
  });

  it('renders site name and domain list when open', () => {
    render(<EditSiteDialog isOpen={true} site={mockCustomSite} onSaved={mockOnSaved} onCancel={mockOnCancel} />);
    expect(screen.getByText(/My Site/)).toBeDefined();
    expect(screen.getByText('a.example.com')).toBeDefined();
    expect(screen.getByText('b.example.com')).toBeDefined();
  });

  it('shows remove button for each domain', () => {
    render(<EditSiteDialog isOpen={true} site={mockCustomSite} onSaved={mockOnSaved} onCancel={mockOnCancel} />);
    const removeButtons = screen.getAllByText('移除');
    expect(removeButtons.length).toBe(3);
  });

  it('shows URL input for adding domains', () => {
    render(<EditSiteDialog isOpen={true} site={mockCustomSite} onSaved={mockOnSaved} onCancel={mockOnCancel} />);
    expect(screen.getByPlaceholderText(/输入网址添加域名/)).toBeDefined();
  });

  it('adds domain from URL input', () => {
    render(<EditSiteDialog isOpen={true} site={mockCustomSite} onSaved={mockOnSaved} onCancel={mockOnCancel} />);
    const input = screen.getByPlaceholderText(/输入网址添加域名/);
    fireEvent.change(input, { target: { value: 'https://d.example.com/path' } });
    fireEvent.click(screen.getByText('添加域名'));
    expect(screen.getByText('d.example.com')).toBeDefined();
  });

  it('removes domain when remove clicked', () => {
    render(<EditSiteDialog isOpen={true} site={mockCustomSite} onSaved={mockOnSaved} onCancel={mockOnCancel} />);
    const removeButtons = screen.getAllByText('移除');
    fireEvent.click(removeButtons[0]); // Remove a.example.com
    expect(screen.queryByText('a.example.com')).toBeNull();
    expect(screen.getByText('b.example.com')).toBeDefined();
  });

  it('calls updateSiteDomains on save', async () => {
    vi.mocked(updateSiteDomains).mockResolvedValue({
      success: true,
      site: { id: 'my-site', name: 'My Site', domain_count: 2, domains: {} },
      rules_generated: 2,
    });

    render(<EditSiteDialog isOpen={true} site={mockCustomSite} onSaved={mockOnSaved} onCancel={mockOnCancel} />);

    // Remove one domain
    const removeButtons = screen.getAllByText('移除');
    fireEvent.click(removeButtons[1]); // Remove b.example.com

    fireEvent.click(screen.getByText('保存'));

    await waitFor(() => {
      expect(updateSiteDomains).toHaveBeenCalledWith(
        'my-site',
        [],
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

    fireEvent.click(screen.getByText('保存'));

    await waitFor(() => {
      expect(mockOnSaved).toHaveBeenCalled();
    });
  });

  it('calls onCancel when cancel clicked', () => {
    render(<EditSiteDialog isOpen={true} site={mockCustomSite} onSaved={mockOnSaved} onCancel={mockOnCancel} />);
    fireEvent.click(screen.getByText('取消'));
    expect(mockOnCancel).toHaveBeenCalledOnce();
  });
});
