import { useEffect, useState } from 'react';
import { useNavigate } from 'react-router-dom';
import { useDiagStore } from '../stores/diag-store';
import { useNotifStore } from '../stores/notif-store';
import StatusBadge from '../components/shared/StatusBadge';

function DiagnosticsPage() {
  const [auditOffset, setAuditOffset] = useState(0);

  const {
    reachability,
    nodePool,
    auditLog,
    fetchReachability,
    fetchNodePool,
    fetchAuditLog,
    diagnoseSite,
    isLoading
  } = useDiagStore();
  const navigate = useNavigate();
  const { addNotification } = useNotifStore();

  useEffect(() => {
    fetchReachability();
    fetchNodePool();
    fetchAuditLog(0, 20);
  }, []);

  const handleDiagnoseSite = async (siteId: string) => {
    const result = await diagnoseSite(siteId);
    if (result) {
      addNotification(
        result.reachable ? 'success' : 'error',
        `诊断完成: ${siteId}`,
        result.reachable ? `响应时间: ${result.response_time_ms}ms` : '站点不可达'
      );
    }
  };

  return (
    <div>
      <h1 style={{ fontSize: '24px', fontWeight: '700', marginBottom: '24px' }}>诊断</h1>
      
      <div className="card">
        <div className="card-header">节点池状态</div>
        <div style={{ display: 'flex', gap: '16px', marginBottom: '12px' }}>
          <div>
            <span style={{ color: 'var(--color-text-secondary)' }}>总节点: </span>
            <span style={{ fontWeight: '600' }}>{nodePool.total_nodes}</span>
          </div>
          <div>
            <span style={{ color: 'var(--color-text-secondary)' }}>可用: </span>
            <span style={{ fontWeight: '600' }}>{nodePool.available_nodes}</span>
          </div>
          {nodePool.current_node && (
            <div>
              <span style={{ color: 'var(--color-text-secondary)' }}>当前: </span>
              <span style={{ fontWeight: '600' }}>{nodePool.current_node}</span>
            </div>
          )}
        </div>
        <button className="btn btn-secondary" onClick={fetchNodePool}>
          刷新节点池
        </button>
        {nodePool.nodes.length > 0 && (
          <div style={{ marginTop: '12px', overflowX: 'auto' }}>
            <table style={{ width: '100%', borderCollapse: 'collapse', fontSize: '14px' }}>
              <thead>
                <tr style={{ borderBottom: '2px solid var(--color-border)' }}>
                  <th style={{ textAlign: 'left', padding: '8px', color: 'var(--color-text-secondary)' }}>名称</th>
                  <th style={{ textAlign: 'left', padding: '8px', color: 'var(--color-text-secondary)' }}>协议</th>
                  <th style={{ textAlign: 'left', padding: '8px', color: 'var(--color-text-secondary)' }}>状态</th>
                  <th style={{ textAlign: 'left', padding: '8px', color: 'var(--color-text-secondary)' }}>延迟</th>
                  <th style={{ textAlign: 'left', padding: '8px', color: 'var(--color-text-secondary)' }}>地址</th>
                </tr>
              </thead>
              <tbody>
                {nodePool.nodes.map((node) => {
                  const isCurrent = nodePool.current_node === node.name;
                  const statusMap: Record<string, 'success' | 'error' | 'stopped'> = {
                    available: 'success',
                    unhealthy: 'error',
                    removed: 'stopped',
                  };
                  return (
                    <tr
                      key={node.name}
                      style={{
                        backgroundColor: isCurrent ? 'var(--color-primary-bg, rgba(59,130,246,0.08))' : 'transparent',
                        borderBottom: '1px solid var(--color-border)',
                      }}
                    >
                      <td style={{ padding: '8px', fontWeight: isCurrent ? '600' : '400' }}>
                        {node.name}
                        {isCurrent && (
                          <span style={{ marginLeft: '6px', fontSize: '11px', color: 'var(--color-primary, #3b82f6)' }}>
                            (当前)
                          </span>
                        )}
                      </td>
                      <td style={{ padding: '8px' }}>
                        <span className="status-badge running" style={{ fontSize: '12px' }}>
                          {node.protocol}
                        </span>
                      </td>
                      <td style={{ padding: '8px' }}>
                        <StatusBadge status={statusMap[node.status] ?? 'stopped'} />
                      </td>
                      <td style={{ padding: '8px' }}>
                        {node.latency_ms != null ? `${node.latency_ms}ms` : '—'}
                      </td>
                      <td style={{ padding: '8px', color: 'var(--color-text-secondary)', fontSize: '13px' }}>
                        {node.address}
                      </td>
                    </tr>
                  );
                })}
              </tbody>
            </table>
          </div>
        )}
      </div>

      <div className="card">
        <div className="card-header">站点可达性</div>
        <div style={{ marginBottom: '12px' }}>
          <button className="btn btn-secondary" onClick={fetchReachability}>
            刷新可达性
          </button>
        </div>
        {reachability.length === 0 ? (
          <p style={{ color: 'var(--color-text-secondary)' }}>暂无站点数据</p>
        ) : (
          <div style={{ display: 'grid', gridTemplateColumns: 'repeat(3, 1fr)', gap: '12px' }}>
            {reachability.map((r) => (
              <div key={r.site_id} className="card" style={{ padding: '12px' }}>
                <div style={{ display: 'flex', justifyContent: 'space-between', alignItems: 'center' }}>
                  <h4 style={{ fontSize: '14px', fontWeight: '600' }}>{r.site_id}</h4>
                  <StatusBadge 
                    status={r.reachable ? 'success' : 'error'}
                    label={r.reachable ? `${r.response_time_ms || 0}ms` : '不可达'}
                  />
                </div>
                <button
                  className="btn btn-secondary"
                  style={{ fontSize: '12px', padding: '4px 8px', marginTop: '8px' }}
                  onClick={() => handleDiagnoseSite(r.site_id)}
                  disabled={isLoading}
                >
                  重新诊断
                </button>
                {(!r.reachable || (r.response_time_ms ?? 0) > 2000) && (
                  <div style={{
                    marginTop: '8px',
                    padding: '8px',
                    background: 'var(--color-bg-secondary, #f5f5f5)',
                    borderRadius: '6px',
                    fontSize: '12px',
                    color: 'var(--color-text-secondary)'
                  }}>
                    {r.reachable === false && (
                      <>
                        <div style={{ fontWeight: '600', marginBottom: '4px', color: 'var(--color-error, #e74c3c)' }}>站点不可达</div>
                        <div>可能原因：代理节点故障、DNS 解析失败、网络中断</div>
                        <div style={{ marginTop: '4px' }}>建议操作：</div>
                        <ul style={{ margin: '2px 0', paddingLeft: '16px' }}>
                          <li>点击"重新诊断"再次检测</li>
                          <li>检查节点池中当前节点是否正常</li>
                          <li>前往设置切换代理节点</li>
                        </ul>
                        <button
                          className="btn btn-secondary"
                          style={{ fontSize: '11px', padding: '2px 6px', marginTop: '4px' }}
                          onClick={() => navigate('/settings')}
                        >
                          前往设置
                        </button>
                      </>
                    )}
                    {r.reachable && (r.response_time_ms ?? 0) > 2000 && (
                      <>
                        <div>响应较慢（{r.response_time_ms}ms）</div>
                        <div>建议：检查当前节点延迟，或切换到延迟更低的节点</div>
                      </>
                    )}
                  </div>
                )}
              </div>
            ))}
          </div>
        )}
      </div>

      <div className="card">
        <div className="card-header">审计日志</div>
        <p style={{ color: 'var(--color-text-secondary)', marginBottom: '12px' }}>
          总记录: {auditLog.total_count}
        </p>
        {auditLog.records.length === 0 ? (
          <p style={{ color: 'var(--color-text-secondary)' }}>暂无审计记录</p>
        ) : (
          <div style={{ maxHeight: '300px', overflowY: 'auto' }}>
            {auditLog.records.map((record, idx) => (
              <div 
                key={idx} 
                style={{ 
                  padding: '8px', 
                  borderBottom: '1px solid var(--color-border)',
                  fontSize: '14px'
                }}
              >
                <div style={{ display: 'flex', justifyContent: 'space-between' }}>
                  <span style={{ fontWeight: '600' }}>{record.action}</span>
                  <StatusBadge 
                    status={record.result === 'success' ? 'success' : 'error'}
                    label={record.result}
                  />
                </div>
                <div style={{ color: 'var(--color-text-secondary)', fontSize: '12px' }}>
                  {new Date(record.timestamp).toLocaleString()} - {record.target}
                </div>
              </div>
            ))}
          </div>
        )}
        {auditLog.records.length < auditLog.total_count && (
          <button
            className="btn btn-secondary"
            style={{ marginTop: '12px' }}
            onClick={() => {
              const next = auditOffset + 20;
              setAuditOffset(next);
              fetchAuditLog(next, 20);
            }}
          >
            加载更多
          </button>
        )}
      </div>
    </div>
  );
}

export default DiagnosticsPage;