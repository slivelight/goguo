import { useState } from 'react';
import { updateSiteDomains } from '../../lib/tauri-ipc';
import type { SiteInfo } from '../../lib/types';

interface EditSiteDialogProps {
  isOpen: boolean;
  site: SiteInfo;
  onSaved: () => void;
  onCancel: () => void;
}

function extractDomainFromUrl(input: string): string | null {
  const trimmed = input.trim();
  if (!trimmed) return null;

  const afterScheme = trimmed.includes('://')
    ? trimmed.slice(trimmed.indexOf('://') + 3)
    : trimmed;

  const hostPart = afterScheme.split(/[/?#]/)[0] || afterScheme;
  const domain = hostPart.split(':')[0];
  return domain || null;
}

function EditSiteDialog({ isOpen, site, onSaved, onCancel }: EditSiteDialogProps) {
  const allOriginal = Object.values(site.domains).flat();
  const [removedSet, setRemovedSet] = useState<Set<string>>(new Set());
  const [addedList, setAddedList] = useState<string[]>([]);
  const [urlInput, setUrlInput] = useState('');
  const [isSaving, setIsSaving] = useState(false);
  const [error, setError] = useState<string | null>(null);

  if (!isOpen) return null;

  const toggleRemove = (domain: string) => {
    setRemovedSet(prev => {
      const next = new Set(prev);
      if (next.has(domain)) {
        next.delete(domain);
      } else {
        next.add(domain);
      }
      return next;
    });
  };

  const handleAddDomain = () => {
    const domain = extractDomainFromUrl(urlInput);
    if (!domain) return;
    if (allOriginal.includes(domain) || addedList.includes(domain)) return;
    if (removedSet.has(domain)) {
      setRemovedSet(prev => {
        const next = new Set(prev);
        next.delete(domain);
        return next;
      });
    } else {
      setAddedList(prev => [...prev, domain]);
    }
    setUrlInput('');
  };

  const handleRemoveAdded = (domain: string) => {
    setAddedList(prev => prev.filter(d => d !== domain));
  };

  const handleConfirm = async () => {
    setIsSaving(true);
    setError(null);
    try {
      const result = await updateSiteDomains(
        site.id,
        addedList,
        Array.from(removedSet),
      );
      if (result.success) {
        onSaved();
      } else {
        setError(result.error || '保存失败');
      }
    } catch {
      setError('保存失败');
    } finally {
      setIsSaving(false);
    }
  };

  const hasChanges = addedList.length > 0 || removedSet.size > 0;

  return (
    <div className="confirm-dialog-overlay">
      <div className="confirm-dialog" style={{ maxWidth: '520px' }}>
        <h3 className="confirm-dialog-title">编辑站点 — {site.name}</h3>

        {/* Existing domains list */}
        <div style={{ marginBottom: '12px', maxHeight: '240px', overflowY: 'auto' }}>
          {allOriginal.map((d) => {
            const isRemoved = removedSet.has(d);
            return (
              <div
                key={d}
                style={{
                  display: 'flex',
                  justifyContent: 'space-between',
                  alignItems: 'center',
                  padding: '4px 0',
                  borderBottom: '1px solid var(--color-border, #eee)',
                  opacity: isRemoved ? 0.5 : 1,
                }}
              >
                <span style={{
                  fontSize: '13px',
                  color: isRemoved ? 'var(--color-error, #e74c3c)' : 'inherit',
                  textDecoration: isRemoved ? 'line-through' : 'none',
                }}>
                  {d}
                </span>
                <button
                  type="button"
                  className="btn btn-secondary"
                  style={{ fontSize: '11px', padding: '2px 6px' }}
                  onClick={() => toggleRemove(d)}
                >
                  {isRemoved ? '恢复' : '删除'}
                </button>
              </div>
            );
          })}
          {/* Newly added domains */}
          {addedList.map((d) => (
            <div
              key={d}
              style={{
                display: 'flex',
                justifyContent: 'space-between',
                alignItems: 'center',
                padding: '4px 0',
                borderBottom: '1px solid var(--color-border, #eee)',
              }}
            >
              <span style={{
                fontSize: '13px',
                color: 'var(--color-success, #22c55e)',
                textDecoration: 'underline',
                fontWeight: '500',
              }}>
                {d}
                <span style={{ fontSize: '11px', marginLeft: '4px', fontWeight: '400' }}>(新增)</span>
              </span>
              <button
                type="button"
                className="btn btn-secondary"
                style={{ fontSize: '11px', padding: '2px 6px' }}
                onClick={() => handleRemoveAdded(d)}
              >
                删除
              </button>
            </div>
          ))}
          {allOriginal.length === 0 && addedList.length === 0 && (
            <p style={{ color: 'var(--color-text-secondary)', fontSize: '13px', textAlign: 'center' }}>
              暂无域名
            </p>
          )}
        </div>

        {/* Add domain input */}
        <div style={{ marginBottom: '12px' }}>
          <input
            type="text"
            placeholder="输入网址添加域名，如 https://new.example.com"
            value={urlInput}
            onChange={(e) => setUrlInput(e.target.value)}
            onKeyDown={(e) => { if (e.key === 'Enter') handleAddDomain(); }}
            style={{
              width: '100%',
              padding: '6px 10px',
              fontSize: '14px',
              border: '1px solid var(--color-border, #ddd)',
              borderRadius: '4px',
              boxSizing: 'border-box',
              marginBottom: '6px',
            }}
          />
          <button
            type="button"
            className="btn btn-secondary"
            onClick={handleAddDomain}
            disabled={!urlInput.trim()}
          >
            添加域名
          </button>
        </div>

        {error && (
          <p style={{ color: 'var(--color-error, #e74c3c)', fontSize: '13px', marginBottom: '8px' }}>{error}</p>
        )}

        <div className="confirm-dialog-actions">
          <button
            type="button"
            className="btn btn-primary"
            onClick={handleConfirm}
            disabled={!hasChanges || isSaving}
          >
            {isSaving ? '保存中...' : '确定'}
          </button>
          <button
            type="button"
            className="btn btn-secondary"
            onClick={onCancel}
          >
            取消
          </button>
        </div>
      </div>
    </div>
  );
}

export default EditSiteDialog;
