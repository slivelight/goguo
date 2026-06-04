import { describe, it, expect, vi, beforeEach } from 'vitest';
import { render, screen, fireEvent, waitFor } from '@testing-library/react';
import AddSiteDialog from '../AddSiteDialog';

vi.mock('../../../lib/tauri-ipc', () => ({
  lookupSite: vi.fn(),
  createSite: vi.fn(),
}));

import { lookupSite, createSite } from '../../../lib/tauri-ipc';

describe('AddSiteDialog (v2)', () => {
  const mockOnSiteCreated = vi.fn();
  const mockOnCancel = vi.fn();

  beforeEach(() => {
    vi.clearAllMocks();
  });

  it('does not render when isOpen=false', () => {
    render(<AddSiteDialog isOpen={false} onSiteCreated={mockOnSiteCreated} onCancel={mockOnCancel} />);
    expect(screen.queryByText('添加站点')).toBeNull();
  });

  it('renders URL input when open', () => {
    render(<AddSiteDialog isOpen={true} onSiteCreated={mockOnSiteCreated} onCancel={mockOnCancel} />);
    expect(screen.getByPlaceholderText(/输入网址或域名/)).toBeDefined();
  });

  it('shows lookup button with URL input', () => {
    render(<AddSiteDialog isOpen={true} onSiteCreated={mockOnSiteCreated} onCancel={mockOnCancel} />);
    const input = screen.getByPlaceholderText(/输入网址或域名/);
    fireEvent.change(input, { target: { value: 'https://github.com' } });
    expect(screen.getByText('查找')).toBeDefined();
  });

  it('calls lookupSite on lookup click', async () => {
    vi.mocked(lookupSite).mockResolvedValue(null);
    render(<AddSiteDialog isOpen={true} onSiteCreated={mockOnSiteCreated} onCancel={mockOnCancel} />);

    const input = screen.getByPlaceholderText(/输入网址或域名/);
    fireEvent.change(input, { target: { value: 'https://github.com' } });
    fireEvent.click(screen.getByText('查找'));

    await waitFor(() => {
      expect(lookupSite).toHaveBeenCalledWith('https://github.com');
    });
  });

  it('shows matched site domains after lookup', async () => {
    vi.mocked(lookupSite).mockResolvedValue({
      id: 'github',
      name: 'GitHub',
      domain_count: 47,
      domains: { core: ['github.com', 'github.io'], api: ['api.github.com'] },
    });

    render(<AddSiteDialog isOpen={true} onSiteCreated={mockOnSiteCreated} onCancel={mockOnCancel} />);

    const input = screen.getByPlaceholderText(/输入网址或域名/);
    fireEvent.change(input, { target: { value: 'https://github.com' } });
    fireEvent.click(screen.getByText('查找'));

    await waitFor(() => {
      expect(screen.getByText(/GitHub/)).toBeDefined();
      expect(screen.getByText('github.com')).toBeDefined();
    });
  });

  it('shows editable site name after match', async () => {
    vi.mocked(lookupSite).mockResolvedValue({
      id: 'github',
      name: 'GitHub',
      domain_count: 47,
      domains: { core: ['github.com'] },
    });

    render(<AddSiteDialog isOpen={true} onSiteCreated={mockOnSiteCreated} onCancel={mockOnCancel} />);

    const input = screen.getByPlaceholderText(/输入网址或域名/);
    fireEvent.change(input, { target: { value: 'https://github.com' } });
    fireEvent.click(screen.getByText('查找'));

    await waitFor(() => {
      const nameInput = screen.getByDisplayValue('GitHub');
      expect(nameInput).toBeDefined();
    });
  });

  it('shows degraded mode when no match', async () => {
    vi.mocked(lookupSite).mockResolvedValue(null);

    render(<AddSiteDialog isOpen={true} onSiteCreated={mockOnSiteCreated} onCancel={mockOnCancel} />);

    const input = screen.getByPlaceholderText(/输入网址或域名/);
    fireEvent.change(input, { target: { value: 'https://unknown-site.com' } });
    fireEvent.click(screen.getByText('查找'));

    await waitFor(() => {
      expect(screen.getByText(/未找到匹配/)).toBeDefined();
    });
  });

  it('calls createSite on confirm with matched domains', async () => {
    vi.mocked(lookupSite).mockResolvedValue({
      id: 'npmjs',
      name: 'npm',
      domain_count: 3,
      domains: { core: ['npmjs.com', 'registry.npmjs.org', 'static.npmjs.com'] },
    });
    vi.mocked(createSite).mockResolvedValue({
      success: true,
      site: { id: 'npm', name: 'npm', domain_count: 3, domains: {} },
      rules_generated: 3,
    });

    render(<AddSiteDialog isOpen={true} onSiteCreated={mockOnSiteCreated} onCancel={mockOnCancel} />);

    const input = screen.getByPlaceholderText(/输入网址或域名/);
    fireEvent.change(input, { target: { value: 'npmjs.com' } });
    fireEvent.click(screen.getByText('查找'));

    await waitFor(() => {
      expect(screen.getByText('确认添加')).toBeDefined();
    });

    fireEvent.click(screen.getByText('确认添加'));

    await waitFor(() => {
      expect(createSite).toHaveBeenCalledWith('npm', 'npm', ['npmjs.com', 'registry.npmjs.org', 'static.npmjs.com']);
    });
  });

  it('calls onSiteCreated after successful create', async () => {
    vi.mocked(lookupSite).mockResolvedValue({
      id: 'chatgpt',
      name: 'ChatGPT',
      domain_count: 23,
      domains: { core: ['chatgpt.com'] },
    });
    vi.mocked(createSite).mockResolvedValue({
      success: true,
      site: { id: 'chatgpt', name: 'ChatGPT', domain_count: 23, domains: {} },
      rules_generated: 23,
    });

    render(<AddSiteDialog isOpen={true} onSiteCreated={mockOnSiteCreated} onCancel={mockOnCancel} />);

    const input = screen.getByPlaceholderText(/输入网址或域名/);
    fireEvent.change(input, { target: { value: 'chatgpt.com' } });
    fireEvent.click(screen.getByText('查找'));

    await waitFor(() => {
      expect(screen.getByText('确认添加')).toBeDefined();
    });

    fireEvent.click(screen.getByText('确认添加'));

    await waitFor(() => {
      expect(mockOnSiteCreated).toHaveBeenCalled();
    });
  });

  it('calls onCancel when cancel clicked', () => {
    render(<AddSiteDialog isOpen={true} onSiteCreated={mockOnSiteCreated} onCancel={mockOnCancel} />);
    fireEvent.click(screen.getByText('取消'));
    expect(mockOnCancel).toHaveBeenCalledOnce();
  });
});
