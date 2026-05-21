import { invoke } from '@tauri-apps/api/core';
import type {
  AssessmentResponse,
  StateSummaryResponse,
  BaselineStatusResponse,
  ServiceStatusResponse,
  RecoveryProgressResponse,
  AuditLogResponse,
  AuditLogParams,
  DeploymentModeResponse,
  WslStatusResponse,
  NetworkModeResponse,
  AddSiteResponse,
  RemoveSiteResponse,
  TemplateResponse,
  ReachabilityResponse,
  SiteReachability,
  NodePoolStatus,
  SubscriptionResponse,
  SubscriptionSource,
} from './types';

export async function startInitialAssessment(): Promise<AssessmentResponse> {
  return invoke('start_initial_assessment');
}

export async function getStateSummary(): Promise<StateSummaryResponse> {
  return invoke('get_state_summary');
}

export async function triggerReadjustment(): Promise<AssessmentResponse> {
  return invoke('trigger_readjustment');
}

export async function confirmBaseline(): Promise<AssessmentResponse> {
  return invoke('confirm_baseline');
}

export async function getBaselineStatus(): Promise<BaselineStatusResponse> {
  return invoke('get_baseline_status');
}

export async function stopService(): Promise<void> {
  return invoke('stop_service');
}

export async function getServiceStatus(): Promise<ServiceStatusResponse> {
  return invoke('get_service_status');
}

export async function getRecoveryProgress(): Promise<RecoveryProgressResponse> {
  return invoke('get_recovery_progress');
}

export async function getAuditLog(params?: AuditLogParams): Promise<AuditLogResponse> {
  return invoke('get_audit_log', { params });
}

export async function detectDeploymentMode(): Promise<DeploymentModeResponse> {
  return invoke('detect_deployment_mode');
}

export async function getDeploymentMode(): Promise<DeploymentModeResponse> {
  return invoke('get_deployment_mode');
}

export async function setDeploymentMode(mode: string): Promise<DeploymentModeResponse> {
  return invoke('set_deployment_mode', { mode });
}

export async function getWslStatus(): Promise<WslStatusResponse> {
  return invoke('get_wsl_status');
}

export async function getNetworkMode(): Promise<NetworkModeResponse> {
  return invoke('get_network_mode');
}

export async function addTargetSite(siteId: string): Promise<AddSiteResponse> {
  return invoke('add_target_site', { siteId });
}

export async function removeTargetSite(siteId: string): Promise<RemoveSiteResponse> {
  return invoke('remove_target_site', { siteId });
}

export async function applyPresetTemplate(template: string): Promise<TemplateResponse> {
  return invoke('apply_preset_template', { template });
}

export async function previewRules(): Promise<string[]> {
  return invoke('preview_rules');
}

export async function applyRules(confirm: boolean): Promise<AddSiteResponse> {
  return invoke('apply_rules', { confirm });
}

export async function getSiteReachability(): Promise<ReachabilityResponse> {
  return invoke('get_site_reachability');
}

export async function getDiagnosis(siteId: string): Promise<SiteReachability | null> {
  return invoke('get_diagnosis', { siteId });
}

export async function getNodePoolStatus(): Promise<NodePoolStatus> {
  return invoke('get_node_pool_status');
}

export async function overrideRule(ruleType: string, domain: string): Promise<boolean> {
  return invoke('override_rule', { ruleType, domain });
}

export async function importSubscription(url: string): Promise<SubscriptionResponse> {
  return invoke('import_subscription', { url });
}

export async function getSubscriptionSources(): Promise<SubscriptionSource[]> {
  return invoke('get_subscription_sources');
}