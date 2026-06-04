import { useState } from 'react';

interface ImportSubscriptionDialogProps {
  isOpen: boolean;
  onConfirm: (url: string) => void;
  onCancel: () => void;
  isLoading?: boolean;
}

function ImportSubscriptionDialog({ isOpen, onConfirm, onCancel, isLoading }: ImportSubscriptionDialogProps) {
  const [input, setInput] = useState('');

  if (!isOpen) return null;

  const handleSubmit = () => {
    const trimmed = input.trim();
    if (!trimmed) return;
    onConfirm(trimmed);
    setInput('');
  };

  const handleKeyDown = (e: React.KeyboardEvent) => {
    if (e.key === 'Enter' && input.trim()) {
      handleSubmit();
    }
  };

  return (
    <div className="confirm-dialog-overlay">
      <div className="confirm-dialog">
        <h3 className="confirm-dialog-title">导入订阅源</h3>
        <p className="confirm-dialog-message">
          输入订阅链接 URL，系统将自动解析并导入节点信息
        </p>
        <input
          type="url"
          placeholder="https://example.com/subscription"
          value={input}
          onChange={(e) => setInput(e.target.value)}
          onKeyDown={handleKeyDown}
          style={{
            width: '100%',
            padding: '8px 12px',
            fontSize: '14px',
            border: '1px solid var(--color-border, #ddd)',
            borderRadius: '6px',
            marginBottom: '12px',
            boxSizing: 'border-box',
          }}
        />
        <div className="confirm-dialog-actions">
          <button
            type="button"
            className="btn btn-primary"
            onClick={handleSubmit}
            disabled={!input.trim() || isLoading}
          >
            {isLoading ? '导入中...' : '导入'}
          </button>
          <button
            type="button"
            className="btn btn-secondary"
            onClick={() => {
              setInput('');
              onCancel();
            }}
          >
            取消
          </button>
        </div>
      </div>
    </div>
  );
}

export default ImportSubscriptionDialog;
