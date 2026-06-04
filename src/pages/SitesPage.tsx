import { useEffect, useState } from 'react';
import { useSiteStore } from '../stores/site-store';
import { useNotifStore } from '../stores/notif-store';
import StatusBadge from '../components/shared/StatusBadge';
import ConfirmDialog from '../components/shared/ConfirmDialog';
import AddSiteDialog from '../components/shared/AddSiteDialog';
import EditSiteDialog from '../components/shared/EditSiteDialog';
import TemplateConfirmDialog from '../components/shared/TemplateConfirmDialog';
import type { SiteInfo } from '../lib/types';

const BUILTIN_IDS = ['github', 'npmjs', 'claude', 'chatgpt', 'docker', 'google', 'stackoverflow', 'pypi', 'crates', 'oracle', 'wikipedia', 'whatsapp', 'instagram', 'canva', 'twitter-x'];

function SitesPage() {
  const [showAddDialog, setShowAddDialog] = useState(false);
  const [showDeleteDialog, setShowDeleteDialog] = useState(false);
  const [showTemplateDialog, setShowTemplateDialog] = useState(false);
  const [showEditDialog, setShowEditDialog] = useState(false);
  const [selectedTemplate, setSelectedTemplate] = useState('');
  const [siteToDelete, setSiteToDelete] = useState<string | null>(null);
  const [siteToEdit, setSiteToEdit] = useState<SiteInfo | null>(null);
  const [expandedSiteId, setExpandedSiteId] = useState<string | null>(null);

  const { sites, reachability, addSite, removeSite, applyTemplate, fetchSites } = useSiteStore();
  const { addNotification } = useNotifStore();

  useEffect(() => {
    fetchSites();
  }, []);

  const handleAddSite = async (siteId: string) => {
    const result = await addSite(siteId);
    if (result.success) {
      addNotification('success', '站点添加成功', `已添加 ${siteId}，覆盖 ${result.site?.domain_count || 0} 个域名`);
      setShowAddDialog(false);
    } else {
      addNotification('error', '添加失败', result.error || '未知错误');
    }
  };

  const handleSiteCreated = () => {
    setShowAddDialog(false);
    fetchSites();
    addNotification('success', '站点创建成功', '新站点已添加并生成代理规则');
  };

  const handleEditSite = (site: SiteInfo) => {
    setSiteToEdit(site);
    setShowEditDialog(true);
  };

  const handleEditSaved = () => {
    setShowEditDialog(false);
    setSiteToEdit(null);
    fetchSites();
    addNotification('success', '站点编辑成功', '域名已更新并重新生成代理规则');
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
              const isExpanded = expandedSiteId === site.id;
              const allDomains = Object.values(site.domains || {}).flat();
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
                        onClick={() => setExpandedSiteId(isExpanded ? null : site.id)}
                      >
                        {isExpanded ? '收起域名' : '展开域名'}
                      </button>
                      {!BUILTIN_IDS.includes(site.id) && (
                        <button
                          className="btn btn-secondary"
                          style={{ fontSize: '12px', padding: '4px 8px' }}
                          onClick={() => handleEditSite(site)}
                        >
                          编辑
                        </button>
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
                  {isExpanded && allDomains.length > 0 && (
                    <div style={{ marginTop: '8px', maxHeight: '120px', overflowY: 'auto' }}>
                      {allDomains.map((d) => (
                        <span
                          key={d}
                          style={{
                            display: 'inline-block',
                            padding: '2px 6px',
                            margin: '2px',
                            background: 'var(--color-bg-secondary, #f0f0f0)',
                            borderRadius: '3px',
                            fontSize: '11px',
                          }}
                        >
                          {d}
                        </span>
                      ))}
                    </div>
                  )}
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

      <AddSiteDialog
        isOpen={showAddDialog}
        onSiteCreated={handleSiteCreated}
        onCancel={() => setShowAddDialog(false)}
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

      {siteToEdit && (
        <EditSiteDialog
          isOpen={showEditDialog}
          site={siteToEdit}
          onSaved={handleEditSaved}
          onCancel={() => {
            setShowEditDialog(false);
            setSiteToEdit(null);
          }}
        />
      )}

      <TemplateConfirmDialog
        isOpen={showTemplateDialog}
        template={selectedTemplate}
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