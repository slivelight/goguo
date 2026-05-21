import { useEffect } from 'react';
import { useDiagStore } from '../stores/diag-store';
import { useNotifStore } from '../stores/notif-store';
import StatusBadge from '../components/shared/StatusBadge';

function DiagnosticsPage() {
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
  const { addNotification } = useNotifStore();

  useEffect(() => {
    fetchReachability();
    fetchNodePool();
    fetchAuditLog();
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
            {auditLog.records.slice(0, 10).map((record, idx) => (
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
        <button 
          className="btn btn-secondary" 
          style={{ marginTop: '12px' }}
          onClick={() => fetchAuditLog(0, 20)}
        >
          加载更多
        </button>
      </div>
    </div>
  );
}

export default DiagnosticsPage;