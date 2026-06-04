import { useState } from 'react';
import type { SnapshotItem, StateSummaryResponse } from '../../lib/types';

interface BaselineReviewDialogProps {
  isOpen: boolean;
  summary: StateSummaryResponse;
  items: SnapshotItem[];
  onConfirm: () => void;
  onCancel: () => void;
}

const ITEM_LABELS: Record<string, string> = {
  'linux-proxy-env': '代理环境变量',
  'linux-git-proxy': 'Git 代理设置',
  'linux-resolv-conf': 'DNS 解析配置',
  'linux-etc-environment': '系统环境变量',
  'linux-shell-proxy': 'Shell 代理配置',
  'linux-reachability': '网络连通性',
  'wsl-proxy-env': '代理环境变量',
  'wsl-git-proxy': 'Git 代理设置',
  'wsl-resolv-conf': 'DNS 解析配置',
  'wsl-etc-environment': '系统环境变量',
  'wsl-shell-proxy': 'Shell 代理配置',
  'wsl-reachability': '网络连通性',
  'wsl-wsl2-network-mode': 'WSL2 网络模式',
  'win-hosts': 'Hosts 文件',
  'win-system-proxy': '系统代理',
  'win-pac': 'PAC 自动配置',
  'win-http-proxy': 'HTTP 代理',
  'win-dns-cache': 'DNS 缓存',
  'win-dns-servers': 'DNS 服务器',
  'win-proxy-processes': '代理进程',
  'win-tun-status': 'TUN 状态',
  'win-wsl2-network-mode': 'WSL2 网络模式',
};

const CATEGORY_INFO = [
  { key: 'restorable', label: '可恢复配置', desc: '代理设置、DNS 等，出问题时可一键恢复' },
  { key: 'detectable', label: '仅监测项', desc: '网络连通性等，仅检测不修改' },
  { key: 'excluded', label: '服务管理项', desc: '由代理服务自动管理的配置' },
];

function fmtVal(v: unknown): string {
  if (v == null) return '(空)';
  if (typeof v === 'string') return v || '(空)';
  if (typeof v === 'boolean') return v ? '是' : '否';
  if (typeof v === 'number') return String(v);
  if (Array.isArray(v)) return v.length === 0 ? '(空)' : v.join(', ');
  return String(v);
}

function BaselineReviewDialog({ isOpen, summary, items, onConfirm, onCancel }: BaselineReviewDialogProps) {
  const [expandedKey, setExpandedKey] = useState<string | null>(null);

  if (!isOpen) return null;

  return (
    <div className="confirm-dialog-overlay">
      <div className="confirm-dialog" style={{ maxWidth: '560px', maxHeight: '80vh', overflowY: 'auto' }}>
        <h3 className="confirm-dialog-title">确认系统配置快照</h3>
        <p className="confirm-dialog-message">
          已采集 <strong>{summary.total}</strong> 项系统配置。确认后，这些配置将作为恢复基准点。
        </p>

        {CATEGORY_INFO.map(({ key, label, desc }) => {
          const count = summary[`${key}_count` as keyof StateSummaryResponse] as number;
          if (count === 0) return null;
          const expanded = expandedKey === key;
          const categoryItems = items.filter(item => item.category === key);

          return (
            <div key={key} style={{ marginBottom: '12px' }}>
              <div style={{ display: 'flex', alignItems: 'center', gap: '8px', marginBottom: '4px' }}>
                <strong>{label}</strong>
                <button
                  type="button"
                  className="btn btn-secondary"
                  onClick={() => setExpandedKey(prev => prev === key ? null : key)}
                >
                  {count} 项 {expanded ? '\u25B2' : '\u25BC'}
                </button>
              </div>
              {!expanded && (
                <p className="confirm-dialog-message" style={{ margin: '0 0 4px', fontSize: '13px' }}>{desc}</p>
              )}
              {expanded && categoryItems.length > 0 && (
                <div style={{ marginTop: '8px' }}>
                  {categoryItems.map(item => (
                    <div key={item.id} style={{ marginBottom: '8px', padding: '8px', background: 'var(--color-bg-primary, #fff)', borderRadius: '6px', border: '1px solid var(--color-border, #eee)' }}>
                      <div style={{ fontSize: '13px', fontWeight: 600, marginBottom: '4px' }}>
                        {ITEM_LABELS[item.id] ?? item.id}
                        <span style={{ fontWeight: 400, color: 'var(--color-text-secondary)', fontSize: '11px', marginLeft: '8px' }}>({item.platform})</span>
                      </div>
                      {Object.entries(item.value).filter(([k]) => k !== 'error').map(([k, v]) => (
                        <div key={k} style={{ display: 'flex', padding: '2px 0', fontSize: '12px', fontFamily: 'monospace' }}>
                          <span style={{ color: 'var(--color-text-secondary)', minWidth: '110px', flexShrink: 0 }}>{k}</span>
                          <span style={{ wordBreak: 'break-all' }}>{fmtVal(v)}</span>
                        </div>
                      ))}
                    </div>
                  ))}
                </div>
              )}
            </div>
          );
        })}

        <div style={{ padding: '10px', borderRadius: '6px', background: 'var(--color-info-bg, #eff6ff)', border: '1px solid var(--color-info-border, #bfdbfe)', margin: '12px 0', fontSize: '13px', color: 'var(--color-text-secondary)' }}>
          提示：后续如果配置被意外修改，可以通过"恢复"功能一键还原到当前状态。
        </div>

        <div className="confirm-dialog-actions">
          <button type="button" className="btn btn-primary" onClick={onConfirm}>确认并保存</button>
          <button type="button" className="btn btn-secondary" onClick={onCancel}>取消</button>
        </div>
      </div>
    </div>
  );
}

export default BaselineReviewDialog;
