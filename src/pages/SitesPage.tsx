import { useEffect, useState } from 'react';
import { useSiteStore } from '../stores/site-store';
import { useNotifStore } from '../stores/notif-store';
import StatusBadge from '../components/shared/StatusBadge';
import ConfirmDialog from '../components/shared/ConfirmDialog';

function SitesPage() {
  const [showAddDialog, setShowAddDialog] = useState(false);
  const [showDeleteDialog, setShowDeleteDialog] = useState(false);
  const [showTemplateDialog, setShowTemplateDialog] = useState(false);
  const [siteIdInput, setSiteIdInput] = useState('');
  const [selectedTemplate, setSelectedTemplate] = useState('');
  const [siteToDelete, setSiteToDelete] = useState<string | null>(null);

  const { sites, reachability, addSite, removeSite, applyTemplate, fetchSites } = useSiteStore();
  const { addNotification } = useNotifStore();

  useEffect(() => {
    fetchSites();
  }, []);

  const handleAddSite = async () => {
    if (!siteIdInput.trim()) return;
    const result = await addSite(siteIdInput.trim());
    if (result.success) {
      addNotification('success', '站点添加成功', `已添加 ${siteIdInput}，覆盖 ${result.site?.domain_count || 0} 个域名`);
      setSiteIdInput('');
      setShowAddDialog(false);
    } else {
      addNotification('error', '添加失败', result.error || '未知错误');
    }
  };

  const handleDeleteSite = async () => {
    if (!siteToDelete) return;
    const result = await removeSite(siteToDelete);
    if (result.success) {
      addNotification('success', '站点删除成功', `已删除 ${siteToDelete}`);
    } else {
      addNotification('error', '删除失败', result.error || '未知错误');
    }
    setSiteToDelete(null);
    setShowDeleteDialog(false);
  };

  const handleApplyTemplate = async () => {
    const result = await applyTemplate(selectedTemplate);
    if (result.added_count > 0) {
      addNotification('success', '模板应用成功', `已添加 ${result.added_count} 个站点`);
      fetchSites();
    }
    setSelectedTemplate('');
    setShowTemplateDialog(false);
  };

  const getSiteReachability = (siteId: string) => {
    return reachability.find(r => r.site_id === siteId);
  };

  return (
    <div>
      <h1 style={{ fontSize: '24px', fontWeight: '700', marginBottom: '24px' }}>站点管理</h1>
      
      <div className="card">
        <div className="card-header">站点列表</div>
        <div style={{ marginBottom: '12px' }}>
          <button className="btn btn-primary" onClick={() => setShowAddDialog(true)}>
            添加站点
          </button>
        </div>
        {sites.length === 0 ? (
          <p style={{ color: 'var(--color-text-secondary)' }}>暂无已添加站点</p>
        ) : (
          <div style={{ display: 'grid', gridTemplateColumns: 'repeat(2, 1fr)', gap: '12px' }}>
            {sites.map((site) => {
              const reachable = getSiteReachability(site.id);
              return (
                <div key={site.id} className="card" style={{ padding: '12px' }}>
                  <div style={{ display: 'flex', justifyContent: 'space-between', alignItems: 'center' }}>
                    <div>
                      <h4 style={{ fontSize: '16px', fontWeight: '600' }}>{site.name}</h4>
                      <p style={{ color: 'var(--color-text-secondary)', fontSize: '12px' }}>
                        域名数: {site.domain_count}
                      </p>
                    </div>
                    <div style={{ display: 'flex', alignItems: 'center', gap: '8px' }}>
                      {reachable && (
                        <StatusBadge 
                          status={reachable.reachable ? 'success' : 'error'}
                          label={reachable.reachable ? '可达' : '不可达'}
                        />
                      )}
                      <button 
                        className="btn btn-secondary" 
                        style={{ fontSize: '12px', padding: '4px 8px' }}
                        onClick={() => {
                          setSiteToDelete(site.id);
                          setShowDeleteDialog(true);
                        }}
                      >
                        删除
                      </button>
                    </div>
                  </div>
                </div>
              );
            })}
          </div>
        )}
      </div>

      <div className="card">
        <div className="card-header">预设模板</div>
        <div style={{ display: 'flex', gap: '8px' }}>
          <button className="btn btn-secondary" onClick={() => {
            setSelectedTemplate('developer');
            setShowTemplateDialog(true);
          }}>
            开发者模板
          </button>
          <button className="btn btn-secondary" onClick={() => {
            setSelectedTemplate('office');
            setShowTemplateDialog(true);
          }}>
            办公模板
          </button>
        </div>
      </div>

      <ConfirmDialog
        isOpen={showAddDialog}
        title="添加目标站点"
        message="输入站点标识（如 github、npm）"
        confirmText="添加"
        onConfirm={handleAddSite}
        onCancel={() => {
          setShowAddDialog(false);
          setSiteIdInput('');
        }}
      />

      <ConfirmDialog
        isOpen={showDeleteDialog}
        title="删除站点"
        message={`确认删除站点 "${siteToDelete}"？删除后将重新生成规则。`}
        confirmText="删除"
        danger={true}
        onConfirm={handleDeleteSite}
        onCancel={() => {
          setShowDeleteDialog(false);
          setSiteToDelete(null);
        }}
      />

      <ConfirmDialog
        isOpen={showTemplateDialog}
        title="应用预设模板"
        message={`确认应用 "${selectedTemplate}" 模板？将自动添加多个站点。`}
        confirmText="应用"
        onConfirm={handleApplyTemplate}
        onCancel={() => {
          setShowTemplateDialog(false);
          setSelectedTemplate('');
        }}
      />
    </div>
  );
}

export default SitesPage;