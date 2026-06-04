import { describe, it, expect, vi } from 'vitest';
import { render, screen, fireEvent } from '@testing-library/react';
import ImportSubscriptionDialog from '../ImportSubscriptionDialog';

describe('ImportSubscriptionDialog', () => {
  it('renders URL input field when isOpen=true', () => {
    render(
      <ImportSubscriptionDialog
        isOpen={true}
        onConfirm={() => {}}
        onCancel={() => {}}
      />
    );

    expect(screen.getByPlaceholderText('https://example.com/subscription')).toBeDefined();
  });

  it('does not render when isOpen=false', () => {
    render(
      <ImportSubscriptionDialog
        isOpen={false}
        onConfirm={() => {}}
        onCancel={() => {}}
      />
    );

    expect(screen.queryByPlaceholderText('https://example.com/subscription')).toBeNull();
  });

  it('disables confirm when URL is empty', () => {
    render(
      <ImportSubscriptionDialog
        isOpen={true}
        onConfirm={() => {}}
        onCancel={() => {}}
      />
    );

    const confirmBtn = screen.getAllByRole('button').find(b => b.textContent === '导入');
    expect(confirmBtn?.hasAttribute('disabled')).toBe(true);
  });

  it('calls onConfirm with URL value when confirm clicked', () => {
    const onConfirm = vi.fn();
    render(
      <ImportSubscriptionDialog
        isOpen={true}
        onConfirm={onConfirm}
        onCancel={() => {}}
      />
    );

    const input = screen.getByPlaceholderText('https://example.com/subscription');
    fireEvent.change(input, { target: { value: 'https://sub.example.com/link' } });

    const confirmBtn = screen.getAllByRole('button').find(b => b.textContent === '导入');
    if (confirmBtn) fireEvent.click(confirmBtn);

    expect(onConfirm).toHaveBeenCalledWith('https://sub.example.com/link');
  });

  it('calls onCancel when cancel clicked', () => {
    const onCancel = vi.fn();
    render(
      <ImportSubscriptionDialog
        isOpen={true}
        onConfirm={() => {}}
        onCancel={onCancel}
      />
    );

    fireEvent.click(screen.getByText('取消'));
    expect(onCancel).toHaveBeenCalledOnce();
  });

  it('submits on Enter key press', () => {
    const onConfirm = vi.fn();
    render(
      <ImportSubscriptionDialog
        isOpen={true}
        onConfirm={onConfirm}
        onCancel={() => {}}
      />
    );

    const input = screen.getByPlaceholderText('https://example.com/subscription');
    fireEvent.change(input, { target: { value: 'https://test.com/sub' } });
    fireEvent.keyDown(input, { key: 'Enter' });

    expect(onConfirm).toHaveBeenCalledWith('https://test.com/sub');
  });
});
