import { useEffect, useState } from 'react';
import { useRuleStore } from '../stores/rule-store';
import { useNotifStore } from '../stores/notif-store';
import CodeBlock from '../components/shared/CodeBlock';
import ConfirmDialog from '../components/shared/ConfirmDialog';

function RulesPage() {
  const [showApplyDialog, setShowApplyDialog] = useState(false);

  const { rules, previewData, preview, apply, isLoading } = useRuleStore();
  const { addNotification } = useNotifStore();

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
          <CodeBlock 
            code={previewData.join('\n')} 
            language="mihomo-rules"
            maxHeight="400px"
          />
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