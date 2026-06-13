import { listen, type UnlistenFn } from '@tauri-apps/api/event';
import type {
  RecoveryStartedPayload,
  RecoveryItemCompletedPayload,
  RecoveryCompletedPayload,
  RecoveryFailedPayload,
  BaselineConfirmedPayload,
  BaselineDeviationPayload,
  ServiceStoppedPayload,
  ServiceStartedPayload,
  AutoRecoveryTriggeredPayload,
  ProxyRecoveringPayload,
  ProxyRecoveredPayload,
} from './types';

export type TauriEvent =
  | 'recovery:started'
  | 'recovery:item-completed'
  | 'recovery:completed'
  | 'recovery:failed'
  | 'baseline:confirmed'
  | 'baseline:deviation-detected'
  | 'service:started'
  | 'service:stopped'
  | 'proxy-guard:recovery-triggered'
  | 'proxy:recovering'
  | 'proxy:recovered';

export function subscribeRecoveryStarted(
  callback: (payload: RecoveryStartedPayload) => void
): Promise<UnlistenFn> {
  return listen('recovery:started', (event) => callback(event.payload as RecoveryStartedPayload));
}

export function subscribeRecoveryItemCompleted(
  callback: (payload: RecoveryItemCompletedPayload) => void
): Promise<UnlistenFn> {
  return listen('recovery:item-completed', (event) => callback(event.payload as RecoveryItemCompletedPayload));
}

export function subscribeRecoveryCompleted(
  callback: (payload: RecoveryCompletedPayload) => void
): Promise<UnlistenFn> {
  return listen('recovery:completed', (event) => callback(event.payload as RecoveryCompletedPayload));
}

export function subscribeRecoveryFailed(
  callback: (payload: RecoveryFailedPayload) => void
): Promise<UnlistenFn> {
  return listen('recovery:failed', (event) => callback(event.payload as RecoveryFailedPayload));
}

export function subscribeBaselineConfirmed(
  callback: (payload: BaselineConfirmedPayload) => void
): Promise<UnlistenFn> {
  return listen('baseline:confirmed', (event) => callback(event.payload as BaselineConfirmedPayload));
}

export function subscribeBaselineDeviation(
  callback: (payload: BaselineDeviationPayload) => void
): Promise<UnlistenFn> {
  return listen('baseline:deviation-detected', (event) => callback(event.payload as BaselineDeviationPayload));
}

export function subscribeServiceStarted(
  callback: (payload: ServiceStartedPayload) => void
): Promise<UnlistenFn> {
  return listen('service:started', (event) => callback(event.payload as ServiceStartedPayload));
}

export function subscribeServiceStopped(
  callback: (payload: ServiceStoppedPayload) => void
): Promise<UnlistenFn> {
  return listen('service:stopped', (event) => callback(event.payload as ServiceStoppedPayload));
}

export function subscribeAutoRecoveryTriggered(
  callback: (payload: AutoRecoveryTriggeredPayload) => void
): Promise<UnlistenFn> {
  return listen('proxy-guard:recovery-triggered', (event) => callback(event.payload as AutoRecoveryTriggeredPayload));
}

export function subscribeProxyRecovering(
  callback: (payload: ProxyRecoveringPayload) => void
): Promise<UnlistenFn> {
  return listen('proxy:recovering', (event) => callback(event.payload as ProxyRecoveringPayload));
}

export function subscribeProxyRecovered(
  callback: (payload: ProxyRecoveredPayload) => void
): Promise<UnlistenFn> {
  return listen('proxy:recovered', (event) => callback(event.payload as ProxyRecoveredPayload));
}