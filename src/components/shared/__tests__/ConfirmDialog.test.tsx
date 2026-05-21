import { describe, it, expect, vi } from 'vitest';
import { render, screen, fireEvent } from '@testing-library/react';
import ConfirmDialog from '../ConfirmDialog';

describe('ConfirmDialog', () => {
  it('renders when isOpen is true', () => {
    render(
      <ConfirmDialog
        isOpen={true}
        title="确认操作"
        message="是否确认执行此操作？"
        onConfirm={() => {}}
        onCancel={() => {}}
      />
    );

    expect(screen.getByText('确认操作')).toBeDefined();
    expect(screen.getByText('是否确认执行此操作？')).toBeDefined();
  });

  it('does not render when isOpen is false', () => {
    render(
      <ConfirmDialog
        isOpen={false}
        title="确认操作"
        message="是否确认？"
        onConfirm={() => {}}
        onCancel={() => {}}
      />
    );

    expect(screen.queryByText('确认操作')).toBeNull();
  });

  it('calls onConfirm when confirm button clicked', () => {
    const onConfirm = vi.fn();
    render(
      <ConfirmDialog
        isOpen={true}
        title="测试标题"
        message="确认吗？"
        onConfirm={onConfirm}
        onCancel={() => {}}
      />
    );

    const buttons = screen.getAllByRole('button');
    const confirmBtn = buttons.find(b => b.textContent === '确认');
    if (confirmBtn) {
      fireEvent.click(confirmBtn);
    }
    expect(onConfirm).toHaveBeenCalledOnce();
  });

  it('calls onCancel when cancel button clicked', () => {
    const onCancel = vi.fn();
    render(
      <ConfirmDialog
        isOpen={true}
        title="确认"
        message="确认吗？"
        onConfirm={() => {}}
        onCancel={onCancel}
      />
    );

    fireEvent.click(screen.getByText('取消'));
    expect(onCancel).toHaveBeenCalledOnce();
  });

  it('shows value change visualization', () => {
    render(
      <ConfirmDialog
        isOpen={true}
        title="恢复 Baseline"
        message="恢复以下状态项？"
        currentValue="192.168.1.1"
        afterValue="192.168.1.100"
        onConfirm={() => {}}
        onCancel={() => {}}
      />
    );

    expect(screen.getByText('192.168.1.1')).toBeDefined();
    expect(screen.getByText('192.168.1.100')).toBeDefined();
  });

  it('renders danger button when danger prop is true', () => {
    render(
      <ConfirmDialog
        isOpen={true}
        title="危险操作"
        message="此操作不可撤销"
        danger={true}
        onConfirm={() => {}}
        onCancel={() => {}}
      />
    );

    const buttons = screen.getAllByRole('button');
    const confirmBtn = buttons.find(b => b.textContent === '确认');
    expect(confirmBtn?.className).toContain('btn-danger');
  });
});