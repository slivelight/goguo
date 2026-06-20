import { invoke } from '@tauri-apps/api/core';
import type {
  AssessmentResponse,
  StateSummaryResponse,
  SnapshotItem,
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
  SiteDefinitionInfo,
  CreateSiteResponse,
  UpdateSiteDomainsResponse,
} from './types';

export async function startInitialAssessment(): Promise<AssessmentResponse> {
  return invoke('tauri_start_initial_assessment');
}

export async function getStateSummary(): Promise<StateSummaryResponse> {
  return invoke('tauri_get_state_summary');
}

export async function getSnapshotDetails(): Promise<SnapshotItem[]> {
  return invoke('tauri_get_snapshot_details');
}

export async function triggerReadjustment(): Promise<AssessmentResponse> {
  return invoke('tauri_trigger_readjustment');
}

export async function confirmBaseline(): Promise<AssessmentResponse> {
  return invoke('tauri_confirm_baseline');
}

export async function getBaselineStatus(): Promise<BaselineStatusResponse> {
  return invoke('tauri_get_baseline_status');
}

export async function stopService(): Promise<void> {
  return invoke('tauri_stop_service');
}

export async function getServiceStatus(): Promise<ServiceStatusResponse> {
  return invoke('tauri_get_service_status');
}

export async function getRecoveryProgress(): Promise<RecoveryProgressResponse> {
  return invoke('tauri_get_recovery_progress');
}

export async function getIsRestoring(): Promise<boolean> {
  return invoke('tauri_get_is_restoring');
}

export async function getAuditLog(params?: AuditLogParams): Promise<AuditLogResponse> {
  return invoke('tauri_get_audit_log', { params });
}

export async function detectDeploymentMode(): Promise<DeploymentModeResponse> {
  return invoke('tauri_detect_deployment_mode');
}

export async function getDeploymentMode(): Promise<DeploymentModeResponse> {
  return invoke('tauri_get_deployment_mode');
}

export async function setDeploymentMode(mode: string): Promise<DeploymentModeResponse> {
  return invoke('tauri_set_deployment_mode', { mode });
}

export async function getWslStatus(): Promise<WslStatusResponse> {
  return invoke('tauri_get_wsl_status');
}

export async function getNetworkMode(): Promise<NetworkModeResponse> {
  return invoke('tauri_get_network_mode');
}

export async function addTargetSite(siteId: string): Promise<AddSiteResponse> {
  return invoke('add_target_site', { siteId });
}

export async function removeTargetSite(siteId: string): Promise<RemoveSiteResponse> {
  return invoke('remove_target_site', { siteId });
}

export async function listTargetSites(): Promise<string[]> {
  return invoke('list_target_sites');
}

export async function applyPresetTemplate(template: string): Promise<TemplateResponse> {
  return invoke('apply_preset_template', { template });
}

export async function previewRules(): Promise<string[]> {
  return invoke('preview_rules');
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

export async function listSiteDefinitions(): Promise<SiteDefinitionInfo[]> {
  return invoke('list_site_definitions');
}

export async function lookupSite(input: string): Promise<SiteDefinitionInfo | null> {
  return invoke('lookup_site', { input });
}

export async function createSite(name: string, displayName: string, domains: string[]): Promise<CreateSiteResponse> {
  return invoke('tauri_create_site', { name, displayName, domains });
}

export async function updateSiteDomains(siteId: string, addDomains: string[], removeDomains: string[]): Promise<UpdateSiteDomainsResponse> {
  return invoke('tauri_update_site_domains', { siteId, addDomains, removeDomains });
}