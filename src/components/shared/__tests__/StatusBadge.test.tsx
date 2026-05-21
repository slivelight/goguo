import { describe, it, expect } from 'vitest';
import { render, screen } from '@testing-library/react';
import StatusBadge from '../StatusBadge';

describe('StatusBadge', () => {
  it('renders running status', () => {
    render(<StatusBadge status="running" />);
    expect(screen.getByText('运行中')).toBeDefined();
  });

  it('renders stopped status', () => {
    render(<StatusBadge status="stopped" />);
    expect(screen.getByText('已停止')).toBeDefined();
  });

  it('renders error status', () => {
    render(<StatusBadge status="error" />);
    expect(screen.getByText('异常')).toBeDefined();
  });

  it('renders warning status', () => {
    render(<StatusBadge status="warning" />);
    expect(screen.getByText('警告')).toBeDefined();
  });

  it('renders success status', () => {
    render(<StatusBadge status="success" />);
    expect(screen.getByText('成功')).toBeDefined();
  });

  it('renders custom label', () => {
    render(<StatusBadge status="running" label="在线" />);
    expect(screen.getByText('在线')).toBeDefined();
  });

  it('applies correct className', () => {
    render(<StatusBadge status="error" />);
    const badge = screen.getByText('异常');
    expect(badge.className).toContain('error');
  });
});