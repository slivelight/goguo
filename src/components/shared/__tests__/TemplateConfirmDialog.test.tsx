import { describe, it, expect, vi } from 'vitest';
import { render, screen, fireEvent } from '@testing-library/react';
import TemplateConfirmDialog from '../TemplateConfirmDialog';

describe('TemplateConfirmDialog', () => {
  it('renders developer template sites list when template=developer', () => {
    render(
      <TemplateConfirmDialog
        isOpen={true}
        template="developer"
        onConfirm={() => {}}
        onCancel={() => {}}
      />
    );

    expect(screen.getByText('GitHub')).toBeDefined();
    expect(screen.getByText('npm')).toBeDefined();
    expect(screen.getByText('Claude')).toBeDefined();
  });

  it('renders office template sites list when template=office', () => {
    render(
      <TemplateConfirmDialog
        isOpen={true}
        template="office"
        onConfirm={() => {}}
        onCancel={() => {}}
      />
    );

    expect(screen.getByText('Google')).toBeDefined();
    expect(screen.getByText('Wikipedia')).toBeDefined();
    expect(screen.getByText('WhatsApp')).toBeDefined();
  });

  it('renders 9 sites for developer template', () => {
    render(
      <TemplateConfirmDialog
        isOpen={true}
        template="developer"
        onConfirm={() => {}}
        onCancel={() => {}}
      />
    );

    const sites = screen.getAllByRole('listitem');
    expect(sites.length).toBe(9);
  });

  it('renders 6 sites for office template', () => {
    render(
      <TemplateConfirmDialog
        isOpen={true}
        template="office"
        onConfirm={() => {}}
        onCancel={() => {}}
      />
    );

    const sites = screen.getAllByRole('listitem');
    expect(sites.length).toBe(6);
  });

  it('calls onConfirm when confirm clicked', () => {
    const onConfirm = vi.fn();
    render(
      <TemplateConfirmDialog
        isOpen={true}
        template="developer"
        onConfirm={onConfirm}
        onCancel={() => {}}
      />
    );

    const confirmBtn = screen.getAllByRole('button').find(b => b.textContent === '应用');
    if (confirmBtn) fireEvent.click(confirmBtn);
    expect(onConfirm).toHaveBeenCalledOnce();
  });

  it('calls onCancel when cancel clicked', () => {
    const onCancel = vi.fn();
    render(
      <TemplateConfirmDialog
        isOpen={true}
        template="developer"
        onConfirm={() => {}}
        onCancel={onCancel}
      />
    );

    fireEvent.click(screen.getByText('取消'));
    expect(onCancel).toHaveBeenCalledOnce();
  });

  it('does not render when isOpen=false', () => {
    render(
      <TemplateConfirmDialog
        isOpen={false}
        template="developer"
        onConfirm={() => {}}
        onCancel={() => {}}
      />
    );

    expect(screen.queryByText('GitHub')).toBeNull();
  });

  it('shows domain count for each site', () => {
    render(
      <TemplateConfirmDialog
        isOpen={true}
        template="developer"
        onConfirm={() => {}}
        onCancel={() => {}}
      />
    );

    // GitHub should show domain count
    expect(screen.getByText(/47.*域名/)).toBeDefined();
  });

  it('shows domain list when site is expanded', () => {
    render(
      <TemplateConfirmDialog
        isOpen={true}
        template="developer"
        onConfirm={() => {}}
        onCancel={() => {}}
      />
    );

    // Click on the first expand button
    const expandButtons = screen.getAllByText('展开');
    fireEvent.click(expandButtons[0]);

    // Should now show domain details for that site
    expect(screen.getByText('github.com')).toBeDefined();
  });
});
