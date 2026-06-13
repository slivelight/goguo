import { useEffect, useMemo, useState } from 'react';
import { useRuleStore } from '../stores/rule-store';
import { useNotifStore } from '../stores/notif-store';
import CodeBlock from '../components/shared/CodeBlock';

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

/** Extract strategy from a rule line like "DOMAIN-SUFFIX,github.com,proxy" → "proxy" */
function extractStrategy(ruleLine: string): string {
  const segments = ruleLine.split(',');
  return segments.length >= 3 ? segments[segments.length - 1].trim().toLowerCase() : '';
}

interface RuleGroup {
  siteKey: string;
  rules: string[];
  strategy: string;
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
    const strategy = extractStrategy(rules[0]);
    groups.push({ siteKey, rules, strategy });
  }

  // Sort by rule count descending
  groups.sort((a, b) => b.rules.length - a.rules.length);
  return groups;
}

function RulesPage() {
  const [collapsedGroups, setCollapsedGroups] = useState<Set<string>>(new Set());

  const { rules, previewData, preview, isLoading } = useRuleStore();
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
              const strategyLabel = group.strategy === 'proxy' ? '代理' : group.strategy === 'direct' ? '直连' : group.strategy;
              const strategyColor = group.strategy === 'proxy' ? 'var(--color-success, #22c55e)' : group.strategy === 'direct' ? 'var(--color-text-secondary, #888)' : 'var(--color-warning, #f59e0b)';
              return (
                <div key={group.siteKey} style={{ marginBottom: '12px' }}>
                  <button
                    type="button"
                    className="btn btn-secondary"
                    onClick={() => toggleGroup(group.siteKey)}
                    style={{
                      display: 'flex',
                      alignItems: 'center',
                      gap: '6px',
                      padding: '8px',
                      width: '100%',
                      textAlign: 'left',
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
                    <span style={{
                      fontSize: '11px',
                      padding: '1px 6px',
                      borderRadius: '4px',
                      background: strategyColor,
                      color: '#fff',
                      fontWeight: '600',
                    }}>
                      {strategyLabel}
                    </span>
                  </button>
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
        <div className="card-header">当前生效规则</div>
        <p style={{ color: 'var(--color-text-secondary)', marginBottom: '8px', fontSize: '13px' }}>
          规则在添加/删除站点时自动生效，无需手动应用
        </p>
        <p style={{ color: 'var(--color-text-secondary)', marginBottom: '12px' }}>
          已生效: {rules.length} 条
        </p>
        {rules.length > 0 ? (
          <CodeBlock
            code={rules.join('\n')}
            language="mihomo-rules"
            maxHeight="200px"
          />
        ) : (
          <p style={{ color: 'var(--color-text-secondary)' }}>尚未生效任何规则</p>
        )}
      </div>

      <div style={{ display: 'flex', gap: '8px' }}>
        <button
          className="btn btn-secondary"
          onClick={handlePreviewClick}
          disabled={isLoading}
        >
          {isLoading ? '预览中...' : '刷新预览'}
        </button>
      </div>
    </div>
  );
}

export default RulesPage;
