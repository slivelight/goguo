import { useEffect } from 'react';
import { useWizardStore } from '../stores/wizard-store';
import type { WizardStep } from '../stores/wizard-store';
import { useNavigate } from 'react-router-dom';
import StatusBadge from '../components/shared/StatusBadge';
import { detectDeploymentMode, startInitialAssessment, confirmBaseline, applyPresetTemplate, addTargetSite } from '../lib/tauri-ipc';
import { useNotifStore } from '../stores/notif-store';

function WizardPage() {
  const navigate = useNavigate();
  const {
    currentStep,
    deploymentMode,
    hasBaseline,
    selectedSites,
    completedSteps,
    nextStep,
    prevStep,
    setDeploymentMode,
    setHasBaseline,
    selectSites,
    markComplete,
  } = useWizardStore();
  const { addNotification } = useNotifStore();

  useEffect(() => {
    detectDeploymentMode().then((mode) => {
      setDeploymentMode(mode.mode);
    });
  }, []);

  const deploymentModes = [
    { id: 'windows_only', label: 'Windows Only', desc: '仅在 Windows 上运行', impact: '仅监控和管理 Windows 系统的代理配置（hosts、系统代理、PAC）。适用于纯 Windows 环境或 WSL 网络由 Windows 代理覆盖的场景。' },
    { id: 'wsl_only', label: 'WSL Only', desc: '仅在 WSL 上运行', impact: '仅监控和管理 WSL Linux 环境的代理配置（环境变量、Git 代理、DNS）。适用于所有工作都在 WSL 内完成的场景。' },
    { id: 'linux_only', label: 'Linux Only', desc: '仅在 Linux 上运行', impact: '仅监控和管理原生 Linux 系统的代理配置。适用于纯 Linux 桌面环境。' },
    { id: 'coordinated', label: 'Coordinated', desc: 'Windows + WSL 协调模式', impact: '同时管理 Windows 和 WSL 两侧的代理配置，确保双环境网络一致性。适用于 Windows + WSL 混合开发场景。' },
  ];

  const siteOptions = [
    { id: 'github', label: 'GitHub', desc: '开发者必备', domainCount: 8 },
    { id: 'npm', label: 'npm', desc: 'Node.js 包管理', domainCount: 3 },
    { id: 'docker', label: 'Docker', desc: '容器平台', domainCount: 4 },
    { id: 'claude', label: 'Claude', desc: 'AI 助手', domainCount: 2 },
  ];

  const handleStepAction = async () => {
    switch (currentStep) {
      case 'deployment-mode':
        if (deploymentMode) {
          markComplete('deployment-mode');
          nextStep();
        }
        break;
      case 'initial-assessment':
        try {
          await startInitialAssessment();
          addNotification('success', '评估完成', '已采集初始网络状态');
          markComplete('initial-assessment');
          nextStep();
        } catch (err) {
          addNotification('error', '评估失败', err instanceof Error ? err.message : '未知错误');
        }
        break;
      case 'baseline-confirm':
        try {
          await confirmBaseline();
          setHasBaseline(true);
          addNotification('success', 'Baseline 已确认', '当前网络状态已保存');
          markComplete('baseline-confirm');
          nextStep();
        } catch (err) {
          addNotification('error', '确认失败', err instanceof Error ? err.message : '未知错误');
        }
        break;
      case 'site-selection':
        if (selectedSites.length > 0) {
          try {
            const developerIds = ['github', 'npm', 'docker', 'claude'];
            const isDeveloperSubset = selectedSites.every(s => developerIds.includes(s));
            if (isDeveloperSubset && selectedSites.length >= 2) {
              await applyPresetTemplate('developer');
            } else {
              for (const siteId of selectedSites) {
                await addTargetSite(siteId);
              }
            }
            addNotification('success', '站点已添加', `已添加 ${selectedSites.length} 个站点`);
            markComplete('site-selection');
            nextStep();
          } catch (err) {
            addNotification('error', '添加失败', err instanceof Error ? err.message : '未知错误');
          }
        }
        break;
      case 'rule-preview':
        markComplete('rule-preview');
        nextStep();
        break;
      case 'finish':
        markComplete('finish');
        navigate('/dashboard');
        break;
      default:
        nextStep();
    }
  };

  const stepTitles: Record<string, string> = {
    welcome: '欢迎使用 GoGuo',
    'deployment-mode': '选择部署模式',
    'initial-assessment': '初始网络评估',
    'baseline-confirm': '确认 Baseline',
    'site-selection': '选择目标站点',
    'rule-preview': '预览规则',
    finish: '完成设置',
  };

  const canProceed = () => {
    switch (currentStep) {
      case 'deployment-mode':
        return deploymentMode !== null;
      case 'site-selection':
        return selectedSites.length > 0;
      default:
        return true;
    }
  };

  return (
    <div style={{ maxWidth: '600px', margin: '0 auto', padding: '24px' }}>
      <div style={{ marginBottom: '24px' }}>
        <div style={{ display: 'flex', justifyContent: 'space-between', marginBottom: '16px' }}>
          {['welcome', 'deployment-mode', 'initial-assessment', 'baseline-confirm', 'site-selection', 'rule-preview', 'finish'].map((step, idx) => (
            <div
              key={step}
              style={{
                width: '40px',
                height: '40px',
                borderRadius: '20px',
                backgroundColor: completedSteps.includes(step as WizardStep) 
                  ? 'var(--color-success)' 
                  : currentStep === (step as WizardStep) 
                    ? 'var(--color-primary)' 
                    : 'var(--color-border)',
                display: 'flex',
                alignItems: 'center',
                justifyContent: 'center',
                color: completedSteps.includes(step as WizardStep) || currentStep === (step as WizardStep) ? 'white' : 'var(--color-text-secondary)',
                fontWeight: '600',
              }}
            >
              {idx + 1}
            </div>
          ))}
        </div>
        <h2 style={{ fontSize: '24px', fontWeight: '700', textAlign: 'center' }}>
          {stepTitles[currentStep]}
        </h2>
      </div>

      <div className="card" style={{ minHeight: '300px' }}>
        {currentStep === 'welcome' && (
          <div style={{ textAlign: 'center', padding: '24px' }}>
            <h3 style={{ marginBottom: '16px' }}>网络可达性诊断工具</h3>
            <p style={{ color: 'var(--color-text-secondary)', marginBottom: '24px' }}>
              GoGuo 帮助您在国内网络环境下保持目标站点可达，同时不影响本地直连访问。
            </p>
            <p style={{ color: 'var(--color-text-secondary)', fontSize: '14px' }}>
              本向导将引导您完成初始配置，预计需要 2-3 分钟。
            </p>
          </div>
        )}

        {currentStep === 'deployment-mode' && (
          <div>
            <p style={{ color: 'var(--color-text-secondary)', marginBottom: '16px' }}>
              选择您的运行环境：
            </p>
            <div style={{ display: 'grid', gridTemplateColumns: 'repeat(2, 1fr)', gap: '12px' }}>
              {deploymentModes.map((mode) => (
                <div
                  key={mode.id}
                  style={{
                    padding: '16px',
                    border: deploymentMode === mode.id 
                      ? '2px solid var(--color-primary)' 
                      : '1px solid var(--color-border)',
                    borderRadius: '8px',
                    cursor: 'pointer',
                    background: deploymentMode === mode.id 
                      ? 'var(--color-bg-tertiary)' 
                      : 'transparent',
                  }}
                  onClick={() => setDeploymentMode(mode.id)}
                >
                  <div style={{ fontWeight: '600' }}>{mode.label}</div>
                  <div style={{ fontSize: '12px', color: 'var(--color-text-secondary)' }}>
                    {mode.desc}
                  </div>
                  <div style={{ fontSize: '11px', color: 'var(--color-text-secondary)', marginTop: '4px', lineHeight: '1.4' }}>
                    {mode.impact}
                  </div>
                </div>
              ))}
            </div>
          </div>
        )}

        {currentStep === 'initial-assessment' && (
          <div style={{ textAlign: 'center', padding: '24px' }}>
            <p style={{ color: 'var(--color-text-secondary)', marginBottom: '24px' }}>
              点击"开始评估"采集当前网络状态，包括 hosts 文件、系统代理、环境变量等。
            </p>
            <StatusBadge status="warning" label="尚未评估" />
          </div>
        )}

        {currentStep === 'baseline-confirm' && (
          <div style={{ textAlign: 'center', padding: '24px' }}>
            <p style={{ color: 'var(--color-text-secondary)', marginBottom: '24px' }}>
              确认当前网络状态为可用 Baseline，后续恢复将以此为参照。
            </p>
            <StatusBadge status={hasBaseline ? 'success' : 'warning'} 
              label={hasBaseline ? '已确认' : '待确认'} />
          </div>
        )}

        {currentStep === 'site-selection' && (
          <div>
            <p style={{ color: 'var(--color-text-secondary)', marginBottom: '16px' }}>
              选择需要保持可达的目标站点：
            </p>
            <div style={{ display: 'grid', gridTemplateColumns: 'repeat(2, 1fr)', gap: '12px' }}>
              {siteOptions.map((site) => (
                <div
                  key={site.id}
                  style={{
                    padding: '16px',
                    border: selectedSites.includes(site.id) 
                      ? '2px solid var(--color-primary)' 
                      : '1px solid var(--color-border)',
                    borderRadius: '8px',
                    cursor: 'pointer',
                    background: selectedSites.includes(site.id) 
                      ? 'var(--color-bg-tertiary)' 
                      : 'transparent',
                  }}
                  onClick={() => {
                    if (selectedSites.includes(site.id)) {
                      selectSites(selectedSites.filter((s) => s !== site.id));
                    } else {
                      selectSites([...selectedSites, site.id]);
                    }
                  }}
                >
                  <div style={{ fontWeight: '600' }}>{site.label}</div>
                  <div style={{ fontSize: '12px', color: 'var(--color-text-secondary)' }}>
                    {site.desc}
                  </div>
                  <div style={{ fontSize: '12px', color: 'var(--color-text-tertiary)', marginTop: '4px' }}>
                    覆盖 {site.domainCount} 个域名
                  </div>
                </div>
              ))}
            </div>
            {selectedSites.length > 0 && (
              <p style={{ color: 'var(--color-text-secondary)', fontSize: '12px', marginTop: '12px' }}>
                已选择 {selectedSites.length} 个站点，共覆盖 {siteOptions.filter((s) => selectedSites.includes(s.id)).reduce((sum, s) => sum + s.domainCount, 0)} 个域名
              </p>
            )}
          </div>
        )}

        {currentStep === 'rule-preview' && (
          <div style={{ textAlign: 'center', padding: '24px' }}>
            <p style={{ color: 'var(--color-text-secondary)', marginBottom: '24px' }}>
              已为选定的站点生成代理规则，可在"规则预览"页面查看详情。
            </p>
            <StatusBadge status="success" label="规则已生成" />
          </div>
        )}

        {currentStep === 'finish' && (
          <div style={{ textAlign: 'center', padding: '24px' }}>
            <h3 style={{ marginBottom: '16px' }}>🎉 配置完成！</h3>
            <p style={{ color: 'var(--color-text-secondary)', marginBottom: '24px' }}>
              您已完成初始设置，现在可以开始使用 GoGuo。
            </p>
            <div style={{ display: 'flex', gap: '8px', justifyContent: 'center' }}>
              <StatusBadge status="success" label={`部署模式: ${deploymentMode || '自动'}`} />
              <StatusBadge status="success" label={`站点: ${selectedSites.length}`} />
            </div>
          </div>
        )}
      </div>

      <div style={{ display: 'flex', gap: '12px', marginTop: '24px', justifyContent: 'space-between' }}>
        {currentStep !== 'welcome' && (
          <button className="btn btn-secondary" onClick={prevStep}>
            上一步
          </button>
        )}
        <button 
          className="btn btn-primary" 
          onClick={handleStepAction}
          disabled={!canProceed()}
          style={{ marginLeft: 'auto' }}
        >
          {currentStep === 'finish' ? '开始使用' : '下一步'}
        </button>
      </div>
    </div>
  );
}

export default WizardPage;