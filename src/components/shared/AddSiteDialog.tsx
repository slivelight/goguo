import { useState } from 'react';
import { lookupSite, createSite, addTargetSite } from '../../lib/tauri-ipc';
import type { SiteDefinitionInfo } from '../../lib/types';

interface AddSiteDialogProps {
  isOpen: boolean;
  onSiteCreated: () => void;
  onCancel: () => void;
}

function AddSiteDialog({ isOpen, onSiteCreated, onCancel }: AddSiteDialogProps) {
  const [urlInput, setUrlInput] = useState('');
  const [confirmed, setConfirmed] = useState(false);
  const [submitting, setSubmitting] = useState(false);
  const [matchedSite, setMatchedSite] = useState<SiteDefinitionInfo | null>(null);
  const [siteName, setSiteName] = useState('');
  const [isDegraded, setIsDegraded] = useState(false);
  const [error, setError] = useState<string | null>(null);

  if (!isOpen) return null;

  const reset = () => {
    setUrlInput('');
    setConfirmed(false);
    setSubmitting(false);
    setMatchedSite(null);
    setSiteName('');
    setIsDegraded(false);
    setError(null);
  };

  const handleLookup = async () => {
    const trimmed = urlInput.trim();
    if (!trimmed) return;
    setError(null);
    setConfirmed(false);

    try {
      const result = await lookupSite(trimmed);
      if (result) {
        setMatchedSite(result);
        setSiteName(result.name);
        setIsDegraded(false);
        setConfirmed(true);
      } else {
        setMatchedSite(null);
        setSiteName('');
        setIsDegraded(true);
        setConfirmed(true);
      }
    } catch {
      setError('查找失败');
    }
  };

  const handleKeyDown = (e: React.KeyboardEvent) => {
    if (e.key === 'Enter' && urlInput.trim()) {
      handleLookup();
    }
  };

  const handleConfirm = async () => {
    if (!siteName.trim()) return;

    // If lookup matched an existing site (builtin or custom), just activate it
    if (matchedSite && !isDegraded) {
      setSubmitting(true);
      try {
        const result = await addTargetSite(matchedSite.id);
        if (result.success) {
          reset();
          onSiteCreated();
        } else {
          setError(result.error || '添加失败');
          setSubmitting(false);
        }
      } catch {
        setError('添加失败');
        setSubmitting(false);
      }
      return;
    }

    // Degraded mode: create a new custom site
    const domains = matchedSite
      ? Object.values(matchedSite.domains).flat()
      : [];

    if (domains.length === 0) {
      setError('降级模式下请至少添加一个域名');
      return;
    }

    setSubmitting(true);
    try {
      const nameKey = siteName.trim().toLowerCase().replace(/\s+/g, '-');
      const result = await createSite(nameKey, siteName.trim(), domains);
      if (result.success) {
        reset();
        onSiteCreated();
      } else {
        setError(result.error || '创建失败');
        setSubmitting(false);
      }
    } catch {
      setError('创建失败');
      setSubmitting(false);
    }
  };

  const allDomains = matchedSite
    ? Object.values(matchedSite.domains).flat()
    : [];

  return (
    <div className="confirm-dialog-overlay">
      <div className="confirm-dialog" style={{ maxWidth: '520px' }}>
        <h3 className="confirm-dialog-title">添加站点</h3>

        {/* Phase 1: URL Input */}
        <div style={{ marginBottom: '12px' }}>
          <input
            type="text"
            placeholder="输入网址或域名，如 https://github.com"
            value={urlInput}
            onChange={(e) => setUrlInput(e.target.value)}
            onKeyDown={handleKeyDown}
            disabled={confirmed}
            style={{
              width: '100%',
              padding: '8px 12px',
              fontSize: '14px',
              border: '1px solid var(--color-border, #ddd)',
              borderRadius: '6px',
              boxSizing: 'border-box',
            }}
          />
          <button
            type="button"
            className="btn btn-secondary"
            style={{ marginTop: '8px' }}
            onClick={handleLookup}
            disabled={!urlInput.trim() || submitting}
          >
            查找
          </button>
        </div>

        {/* Phase 2: Confirm */}
        {confirmed && matchedSite && (
          <div style={{ marginBottom: '12px' }}>
            <p style={{ fontWeight: '600', marginBottom: '8px' }}>
              已匹配: {matchedSite.name} ({matchedSite.domain_count} 个域名)
            </p>
            <label style={{ fontSize: '12px', color: 'var(--color-text-secondary)', display: 'block', marginBottom: '4px' }}>
              站点名称（可编辑）
            </label>
            <input
              type="text"
              value={siteName}
              onChange={(e) => setSiteName(e.target.value)}
              style={{
                width: '100%',
                padding: '6px 10px',
                fontSize: '14px',
                border: '1px solid var(--color-border, #ddd)',
                borderRadius: '4px',
                boxSizing: 'border-box',
                marginBottom: '8px',
              }}
            />
            <div style={{ maxHeight: '200px', overflowY: 'auto', fontSize: '13px' }}>
              {allDomains.map((d) => (
                <span
                  key={d}
                  style={{
                    display: 'inline-block',
                    padding: '2px 8px',
                    margin: '2px',
                    background: 'var(--color-bg-secondary, #f0f0f0)',
                    borderRadius: '4px',
                    fontSize: '12px',
                  }}
                >
                  {d}
                </span>
              ))}
            </div>
          </div>
        )}

        {confirmed && isDegraded && (
          <div style={{ marginBottom: '12px' }}>
            <p style={{ color: 'var(--color-warning, #f59e0b)', marginBottom: '8px' }}>
              未找到匹配的服务提供商，将以降级模式创建自定义站点
            </p>
            <label style={{ fontSize: '12px', color: 'var(--color-text-secondary)', display: 'block', marginBottom: '4px' }}>
              站点名称
            </label>
            <input
              type="text"
              value={siteName}
              onChange={(e) => setSiteName(e.target.value)}
              placeholder="输入站点名称"
              style={{
                width: '100%',
                padding: '6px 10px',
                fontSize: '14px',
                border: '1px solid var(--color-border, #ddd)',
                borderRadius: '4px',
                boxSizing: 'border-box',
              }}
            />
          </div>
        )}

        {error && (
          <p style={{ color: 'var(--color-error, #e74c3c)', fontSize: '13px', marginBottom: '8px' }}>
            {error}
          </p>
        )}

        {/* Actions */}
        <div className="confirm-dialog-actions">
          {confirmed && (
            <button
              type="button"
              className="btn btn-primary"
              onClick={handleConfirm}
              disabled={!siteName.trim() || submitting}
            >
              确认添加
            </button>
          )}
          <button
            type="button"
            className="btn btn-secondary"
            onClick={() => {
              reset();
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

export default AddSiteDialog;
