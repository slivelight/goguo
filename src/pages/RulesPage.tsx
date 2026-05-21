import { useEffect, useMemo, useState } from 'react';
import { useRuleStore } from '../stores/rule-store';
import { useNotifStore } from '../stores/notif-store';
import CodeBlock from '../components/shared/CodeBlock';
import ConfirmDialog from '../components/shared/ConfirmDialog';

/** Extract site key from a domain like "github.com" → "github", "npmjs.org" → "npm" */
function extractSiteKey(domain: string): string {
  const parts = domain.split('.');
  // Take the meaningful part before the TLD
  // e.g. "api.npmjs.org" → skip subdomain, take "npmjs"
  //      "github.com" → "github"
  const tldIndex = parts.length >= 2 ? parts.length - 2 : 0;
  let key = parts[tldIndex];
  // Strip common suffixes like "js" from "npmjs"
  key = key.replace(/js$/, '');
  return key || domain;
}

interface RuleGroup {
  siteKey: string;
  rules: string[];
}

function groupRulesBySite(previewData: string[]): RuleGroup[] {
  const groupMap = new Map<string, string[]>();

  for (const line of previewData) {
    const segments = line.split(',');
    const domain = segments.length >= 2 ? segments[1].trim() : 'other';
    const siteKey = extractSiteKey(domain);

    if (!groupMap.has(siteKey)) {
      groupMap.set(siteKey, []);
    }
    groupMap.get(siteKey)!.push(line);
  }

  const groups: RuleGroup[] = [];
  for (const [siteKey, rules] of groupMap) {
    groups.push({ siteKey, rules });
  }

  // Sort by rule count descending
  groups.sort((a, b) => b.rules.length - a.rules.length);
  return groups;
}

function RulesPage() {
  const [showApplyDialog, setShowApplyDialog] = useState(false);
  const [collapsedGroups, setCollapsedGroups] = useState<Set<string>>(new Set());

  const { rules, previewData, preview, apply, isLoading, failurePrompt } = useRuleStore();
  const { addNotification } = useNotifStore();

  const ruleGroups = useMemo(() => groupRulesBySite(previewData), [previewData]);

  const toggleGroup = (siteKey: string) => {
    setCollapsedGroups(prev => {
      const next = new Set(prev);
      if (next.has(siteKey)) {
        next.delete(siteKey);
      } else {
        next.add(siteKey);
      }
      return next;
    });
  };

  useEffect(() => {
    preview();
  }, []);

  const handlePreviewClick = async () => {
    await preview();
    addNotification('info', '规则预览', `已预览 ${previewData.length} 条规则`);
  };

  const handleApplyClick = async () => {
    await apply(true);
    if (rules.length > 0) {
      addNotification('success', '规则应用成功', `已应用 ${rules.length} 条规则`);
    }
    setShowApplyDialog(false);
  };

  return (
    <div>
      <h1 style={{ fontSize: '24px', fontWeight: '700', marginBottom: '24px' }}>规则预览</h1>
      
      <div className="card">
        <div className="card-header">预览规则</div>
        <p style={{ color: 'var(--color-text-secondary)', marginBottom: '12px' }}>
          规则总数: {previewData.length}
        </p>
        {previewData.length > 0 ? (
          <div>
            {ruleGroups.map((group) => {
              const isCollapsed = collapsedGroups.has(group.siteKey);
              return (
                <div key={group.siteKey} style={{ marginBottom: '12px' }}>
                  <div
                    onClick={() => toggleGroup(group.siteKey)}
                    style={{
                      cursor: 'pointer',
                      display: 'flex',
                      alignItems: 'center',
                      gap: '6px',
                      padding: '8px',
                      background: 'var(--color-bg-secondary, #f5f5f5)',
                      borderRadius: '6px',
                      marginBottom: isCollapsed ? '0' : '8px',
                      userSelect: 'none',
                    }}
                  >
                    <span style={{ fontSize: '12px', transition: 'transform 0.15s' }}>
                      {isCollapsed ? '▸' : '▾'}
                    </span>
                    <span style={{ fontWeight: '600', fontSize: '14px' }}>
                      {group.siteKey}
                    </span>
                    <span style={{ color: 'var(--color-text-secondary)', fontSize: '12px' }}>
                      ({group.rules.length} 条规则)
                    </span>
                  </div>
                  {!isCollapsed && (
                    <CodeBlock
                      code={group.rules.join('\n')}
                      language="mihomo-rules"
                      maxHeight="200px"
                    />
                  )}
                </div>
              );
            })}
          </div>
        ) : (
          <p style={{ color: 'var(--color-text-secondary)' }}>暂无规则，请先添加目标站点</p>
        )}
      </div>

      <div className="card">
        <div className="card-header">已应用规则</div>
        <p style={{ color: 'var(--color-text-secondary)', marginBottom: '12px' }}>
          已应用: {rules.length} 条
        </p>
        {rules.length > 0 ? (
          <CodeBlock
            code={rules.join('\n')}
            language="mihomo-rules"
            maxHeight="200px"
          />
        ) : (
          <p style={{ color: 'var(--color-text-secondary)' }}>尚未应用任何规则</p>
        )}
      </div>

      {failurePrompt && (
        <div className="card" style={{ borderColor: 'var(--color-error, #e74c3c)' }}>
          <div className="card-header" style={{ color: 'var(--color-error, #e74c3c)' }}>
            规则应用失败详情
          </div>
          <p style={{ marginBottom: '8px' }}>
            <strong>原因：</strong>{failurePrompt.reason}
          </p>
          {failurePrompt.attempted_actions.length > 0 && (
            <div style={{ marginBottom: '8px' }}>
              <strong>已尝试的操作：</strong>
              <ul style={{ margin: '4px 0', paddingLeft: '20px' }}>
                {failurePrompt.attempted_actions.map((action, i) => (
                  <li key={i}>{action}</li>
                ))}
              </ul>
            </div>
          )}
          <p style={{ marginBottom: '8px', padding: '8px', background: 'var(--color-bg-secondary, #f5f5f5)', borderRadius: '6px' }}>
            <strong>建议操作：</strong>{failurePrompt.suggested_action}
          </p>
          {failurePrompt.needs_manual_handling && (
            <span style={{
              display: 'inline-block',
              padding: '4px 10px',
              background: 'var(--color-error, #e74c3c)',
              color: '#fff',
              borderRadius: '4px',
              fontSize: '12px',
              fontWeight: '600',
            }}>
              需要人工处理
            </span>
          )}
        </div>
      )}

      <div style={{ display: 'flex', gap: '8px' }}>
        <button 
          className="btn btn-secondary" 
          onClick={handlePreviewClick}
          disabled={isLoading}
        >
          {isLoading ? '预览中...' : '刷新预览'}
        </button>
        {previewData.length > 0 && (
          <button 
            className="btn btn-primary" 
            onClick={() => setShowApplyDialog(true)}
            disabled={isLoading}
          >
            {isLoading ? '应用中...' : '应用规则'}
          </button>
        )}
      </div>

      <ConfirmDialog
        isOpen={showApplyDialog}
        title="应用规则"
        message={`将应用 ${previewData.length} 条代理规则，确认执行？`}
        confirmText="确认应用"
        onConfirm={handleApplyClick}
        onCancel={() => setShowApplyDialog(false)}
      />
    </div>
  );
}

export default RulesPage;