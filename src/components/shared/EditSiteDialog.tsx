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
  const [domains, setDomains] = useState<string[]>(allOriginal);
  const [urlInput, setUrlInput] = useState('');
  const [isSaving, setIsSaving] = useState(false);
  const [error, setError] = useState<string | null>(null);

  if (!isOpen) return null;

  const addedDomains = domains.filter(d => !allOriginal.includes(d));
  const removedDomains = allOriginal.filter(d => !domains.includes(d));

  const handleAddDomain = () => {
    const domain = extractDomainFromUrl(urlInput);
    if (!domain) return;
    if (domains.includes(domain)) return;
    setDomains([...domains, domain]);
    setUrlInput('');
  };

  const handleRemoveDomain = (domain: string) => {
    setDomains(domains.filter(d => d !== domain));
  };

  const handleSave = async () => {
    setIsSaving(true);
    setError(null);
    try {
      const result = await updateSiteDomains(site.id, addedDomains, removedDomains);
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

  const hasChanges = addedDomains.length > 0 || removedDomains.length > 0;

  return (
    <div className="confirm-dialog-overlay">
      <div className="confirm-dialog" style={{ maxWidth: '520px' }}>
        <h3 className="confirm-dialog-title">编辑站点 — {site.name}</h3>

        {/* Domain list */}
        <div style={{ marginBottom: '12px', maxHeight: '200px', overflowY: 'auto' }}>
          {domains.map((d) => {
            const isAdded = addedDomains.includes(d);
            const isOriginal = allOriginal.includes(d);
            return (
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
                  color: isAdded ? 'var(--color-success, #22c55e)' : 'inherit',
                  fontWeight: isAdded ? '600' : '400',
                }}>
                  {d}
                  {isAdded && <span style={{ fontSize: '11px', marginLeft: '4px' }}>(新增)</span>}
                </span>
                {isOriginal && (
                  <button
                    type="button"
                    className="btn btn-secondary"
                    style={{ fontSize: '11px', padding: '2px 6px' }}
                    onClick={() => handleRemoveDomain(d)}
                  >
                    移除
                  </button>
                )}
              </div>
            );
          })}
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
            onClick={handleSave}
            disabled={!hasChanges || isSaving}
          >
            {isSaving ? '保存中...' : '保存'}
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
