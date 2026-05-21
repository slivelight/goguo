import { describe, it, expect, vi } from 'vitest';
import { render, screen } from '@testing-library/react';
import RecoveryOverlay from '../RecoveryOverlay';
import type { RecoveryProgressResponse } from '../../../lib/types';

describe('RecoveryOverlay', () => {
  it('does not render when has_task is false', () => {
    const progress: RecoveryProgressResponse = {
      has_task: false,
      total_items: 0,
      completed_count: 0,
      pending_count: 0,
      succeeded: 0,
      failed: 0,
    };

    render(<RecoveryOverlay progress={progress} />);
    expect(screen.queryByText('恢复进度')).toBeNull();
  });

  it('renders when has_task is true', () => {
    const progress: RecoveryProgressResponse = {
      has_task: true,
      status: 'in_progress',
      total_items: 10,
      completed_count: 5,
      pending_count: 5,
      succeeded: 5,
      failed: 0,
    };

    render(<RecoveryOverlay progress={progress} />);
    expect(screen.getByText('恢复进度')).toBeDefined();
  });

  it('shows progress stats', () => {
    const progress: RecoveryProgressResponse = {
      has_task: true,
      total_items: 10,
      completed_count: 7,
      pending_count: 3,
      succeeded: 6,
      failed: 1,
    };

    render(<RecoveryOverlay progress={progress} />);
    expect(screen.getByText('已完成: 7/10')).toBeDefined();
    expect(screen.getByText('成功: 6')).toBeDefined();
    expect(screen.getByText('失败: 1')).toBeDefined();
  });

  it('shows cancel button when in_progress', () => {
    const progress: RecoveryProgressResponse = {
      has_task: true,
      status: 'in_progress',
      total_items: 10,
      completed_count: 5,
      pending_count: 5,
      succeeded: 5,
      failed: 0,
    };

    const onCancel = vi.fn();
    render(<RecoveryOverlay progress={progress} onCancel={onCancel} />);
    expect(screen.getByText('取消恢复')).toBeDefined();
  });

  it('does not show cancel button when completed', () => {
    const progress: RecoveryProgressResponse = {
      has_task: true,
      status: 'completed',
      total_items: 10,
      completed_count: 10,
      pending_count: 0,
      succeeded: 10,
      failed: 0,
    };

    const onCancel = vi.fn();
    render(<RecoveryOverlay progress={progress} onCancel={onCancel} />);
    expect(screen.queryByText('取消恢复')).toBeNull();
  });
});