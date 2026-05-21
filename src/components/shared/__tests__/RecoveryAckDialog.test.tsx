import { describe, it, expect, vi } from 'vitest';
import { render, screen, fireEvent } from '@testing-library/react';
import RecoveryAckDialog from '../RecoveryAckDialog';

describe('RecoveryAckDialog', () => {
  it('renders when isOpen is true', () => {
    render(
      <RecoveryAckDialog
        isOpen={true}
        failedItems={['win-proxy', 'win-hosts']}
        onAcknowledge={() => {}}
      />
    );

    expect(screen.getByText('恢复任务失败项确认')).toBeDefined();
  });

  it('does not render when isOpen is false', () => {
    render(
      <RecoveryAckDialog
        isOpen={false}
        failedItems={['win-proxy']}
        onAcknowledge={() => {}}
      />
    );

    expect(screen.queryByText('恢复任务失败项确认')).toBeNull();
  });

  it('shows all failed items', () => {
    render(
      <RecoveryAckDialog
        isOpen={true}
        failedItems={['win-proxy', 'win-hosts', 'dns']}
        onAcknowledge={() => {}}
      />
    );

    expect(screen.getByText('win-proxy')).toBeDefined();
    expect(screen.getByText('win-hosts')).toBeDefined();
    expect(screen.getByText('dns')).toBeDefined();
  });

  it('calls onAcknowledge when button clicked', () => {
    const onAcknowledge = vi.fn();
    render(
      <RecoveryAckDialog
        isOpen={true}
        failedItems={['win-proxy']}
        onAcknowledge={onAcknowledge}
      />
    );

    fireEvent.click(screen.getByText('确认'));
    expect(onAcknowledge).toHaveBeenCalledOnce();
  });

  it('shows retry button when onRetry provided', () => {
    const onRetry = vi.fn();
    render(
      <RecoveryAckDialog
        isOpen={true}
        failedItems={['win-proxy']}
        onAcknowledge={() => {}}
        onRetry={onRetry}
      />
    );

    expect(screen.getByText('重试')).toBeDefined();
  });

  it('calls onRetry when retry button clicked', () => {
    const onRetry = vi.fn();
    render(
      <RecoveryAckDialog
        isOpen={true}
        failedItems={['win-proxy']}
        onAcknowledge={() => {}}
        onRetry={onRetry}
      />
    );

    fireEvent.click(screen.getByText('重试'));
    expect(onRetry).toHaveBeenCalledOnce();
  });
});