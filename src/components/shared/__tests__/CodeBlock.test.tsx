import { describe, it, expect, vi } from 'vitest';
import { render, screen, fireEvent } from '@testing-library/react';
import CodeBlock from '../CodeBlock';

describe('CodeBlock', () => {
  it('renders code content', () => {
    render(<CodeBlock code="console.log('test');" />);
    expect(screen.getByText("console.log('test');")).toBeDefined();
  });

  it('shows language label', () => {
    render(<CodeBlock code="test" language="javascript" />);
    expect(screen.getByText('javascript')).toBeDefined();
  });

  it('has copy button', () => {
    render(<CodeBlock code="test code" />);
    expect(screen.getByText('复制')).toBeDefined();
  });

  it('copies code to clipboard', async () => {
    const writeText = vi.fn();
    Object.assign(navigator, { clipboard: { writeText } });

    render(<CodeBlock code="test code" />);
    fireEvent.click(screen.getByText('复制'));

    expect(writeText).toHaveBeenCalledWith('test code');
  });
});