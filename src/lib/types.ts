export type DeploymentMode = 'windows_only' | 'wsl_only' | 'linux_only' | 'coordinated';
export type RecoveryStatus = 'pending' | 'in_progress' | 'completed' | 'failed' | 'user_acknowledged';
export type ComparisonResult = 'match' | 'deviated' | 'missing_in_baseline';

export interface AssessmentResponse {
  version: number;
  timestamp: string;
  item_count: number;
}

export interface StateSummaryResponse {
  total: number;
  restorable_count: number;
  detectable_count: number;
  excluded_count: number;
}

export interface SnapshotItem {
  id: string;
  platform: string;
  category: string;
  value: Record<string, unknown>;
  collected_at: string;
  classification_reason: string;
}

export interface ComparisonItem {
  state_item_id: string;
  result: ComparisonResult;
  baseline_value?: unknown;
  current_value?: unknown;
}

export interface BaselineStatusResponse {
  has_baseline: boolean;
  items: ComparisonItem[];
}

export interface ServiceStatusResponse {
  mihomo_running: boolean;
  proxy_guard_restart_count: number;
}

export interface RecoveryProgressResponse {
  has_task: boolean;
  status?: RecoveryStatus;
  total_items: number;
  completed_count: number;
  pending_count: number;
  succeeded: number;
  failed: number;
}

export interface AuditRecord {
  timestamp: string;
  action: string;
  target: string;
  result: string;
  reason?: string;
  details?: unknown;
}

export interface AuditLogResponse {
  total_count: number;
  records: AuditRecord[];
}

export interface AuditLogParams {
  offset?: number;
  limit?: number;
  action_type?: string;
  from?: string;
  to?: string;
}

export interface DeploymentModeResponse {
  mode: DeploymentMode;
  detected: DeploymentMode;
  is_auto: boolean;
}

export interface WslStatusResponse {
  is_wsl: boolean;
  distro_name?: string;
  distro_version?: string;
  network_mode: string;
  reachable: boolean;
}

export interface NetworkModeResponse {
  mode: string;
  proxy_strategy: string;
}

export interface SiteInfo {
  id: string;
  name: string;
  domain_count: number;
  domains: Record<string, string[]>;
}

export interface FiveElementPrompt {
  reason: string;
  attempted_actions: string[];
  attempt_count: number;
  suggested_action: string;
  needs_manual_handling: boolean;
}

export interface AddSiteResponse {
  success: boolean;
  site?: SiteInfo;
  rules_generated: number;
  verification_passed: boolean;
  error?: string;
  five_element_prompt?: FiveElementPrompt;
}

export interface RemoveSiteResponse {
  success: boolean;
  remaining_sites: number;
  error?: string;
}

export interface TemplateResponse {
  added_count: number;
  failed_count: number;
  sites: string[];
}

export interface SiteReachability {
  site_id: string;
  reachable: boolean;
  response_time_ms?: number;
}

export interface ReachabilityResponse {
  sites: SiteReachability[];
}

export interface NodeInfo {
  name: string;
  protocol: string;
  status: string;
  latency_ms?: number;
  address: string;
}

export interface NodePoolStatus {
  total_nodes: number;
  available_nodes: number;
  current_node?: string;
  nodes: NodeInfo[];
}

export interface SubscriptionResponse {
  imported: number;
  unsupported: number;
  source_url: string;
}

export interface SubscriptionSource {
  name: string;
  url: string;
  enabled: boolean;
}

export interface RecoveryStartedPayload {
  task_id: string;
  total_items: number;
}

export interface RecoveryItemCompletedPayload {
  state_item_id: string;
  success: boolean;
  failure_reason?: string;
}

export interface RecoveryCompletedPayload {
  task_id: string;
  succeeded: number;
  failed: number;
}

export interface RecoveryFailedPayload {
  task_id: string;
  failed_items: string[];
}

export interface BaselineConfirmedPayload {
  version: number;
  item_count: number;
}

export interface BaselineDeviationPayload {
  deviated_items: string[];
}

export interface SiteProbeDetail {
  url: string;
  reachable: boolean;
  latency_ms: number | null;
  error: string | null;
}

export interface NonTargetVerification {
  sites_probed: number;
  sites_reachable: number;
  details: SiteProbeDetail[];
}

export interface ServiceStoppedPayload {
  reason: string;
  recovery_triggered: boolean;
  non_target_verification: NonTargetVerification | null;
}

export interface ServiceStartedPayload {
  mihomo_running: boolean;
}

export interface AutoRecoveryTriggeredPayload {
  restart_attempts: number;
  max_attempts: number;
}

export interface SiteDefinitionInfo {
  id: string;
  name: string;
  domain_count: number;
  domains: Record<string, string[]>;
}

export interface CreateSiteResponse {
  success: boolean;
  site?: SiteDefinitionInfo;
  rules_generated: number;
  error?: string;
}

export interface UpdateSiteDomainsResponse {
  success: boolean;
  site?: SiteDefinitionInfo;
  rules_generated: number;
  error?: string;
}