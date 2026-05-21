import { useEffect } from 'react';
import type { UnlistenFn } from '@tauri-apps/api/event';
import { subscribeRecoveryStarted, subscribeRecoveryItemCompleted, subscribeRecoveryCompleted, subscribeRecoveryFailed, subscribeBaselineConfirmed, subscribeBaselineDeviation, subscribeServiceStarted, subscribeServiceStopped, subscribeAutoRecoveryTriggered } from '../lib/events';
import type { RecoveryStartedPayload, RecoveryItemCompletedPayload, RecoveryCompletedPayload, RecoveryFailedPayload, BaselineConfirmedPayload, BaselineDeviationPayload, ServiceStartedPayload, ServiceStoppedPayload, AutoRecoveryTriggeredPayload } from '../lib/types';

export function useRecoveryStarted(callback: (payload: RecoveryStartedPayload) => void): void {
  useEffect(() => {
    let unlisten: UnlistenFn | undefined;
    subscribeRecoveryStarted(callback).then((fn) => {
      unlisten = fn;
    });
    return () => {
      if (unlisten) unlisten();
    };
  }, [callback]);
}

export function useRecoveryItemCompleted(callback: (payload: RecoveryItemCompletedPayload) => void): void {
  useEffect(() => {
    let unlisten: UnlistenFn | undefined;
    subscribeRecoveryItemCompleted(callback).then((fn) => {
      unlisten = fn;
    });
    return () => {
      if (unlisten) unlisten();
    };
  }, [callback]);
}

export function useRecoveryCompleted(callback: (payload: RecoveryCompletedPayload) => void): void {
  useEffect(() => {
    let unlisten: UnlistenFn | undefined;
    subscribeRecoveryCompleted(callback).then((fn) => {
      unlisten = fn;
    });
    return () => {
      if (unlisten) unlisten();
    };
  }, [callback]);
}

export function useRecoveryFailed(callback: (payload: RecoveryFailedPayload) => void): void {
  useEffect(() => {
    let unlisten: UnlistenFn | undefined;
    subscribeRecoveryFailed(callback).then((fn) => {
      unlisten = fn;
    });
    return () => {
      if (unlisten) unlisten();
    };
  }, [callback]);
}

export function useBaselineConfirmed(callback: (payload: BaselineConfirmedPayload) => void): void {
  useEffect(() => {
    let unlisten: UnlistenFn | undefined;
    subscribeBaselineConfirmed(callback).then((fn) => {
      unlisten = fn;
    });
    return () => {
      if (unlisten) unlisten();
    };
  }, [callback]);
}

export function useBaselineDeviation(callback: (payload: BaselineDeviationPayload) => void): void {
  useEffect(() => {
    let unlisten: UnlistenFn | undefined;
    subscribeBaselineDeviation(callback).then((fn) => {
      unlisten = fn;
    });
    return () => {
      if (unlisten) unlisten();
    };
  }, [callback]);
}

export function useServiceStarted(callback: (payload: ServiceStartedPayload) => void): void {
  useEffect(() => {
    let unlisten: UnlistenFn | undefined;
    subscribeServiceStarted(callback).then((fn) => {
      unlisten = fn;
    });
    return () => {
      if (unlisten) unlisten();
    };
  }, [callback]);
}

export function useServiceStopped(callback: (payload: ServiceStoppedPayload) => void): void {
  useEffect(() => {
    let unlisten: UnlistenFn | undefined;
    subscribeServiceStopped(callback).then((fn) => {
      unlisten = fn;
    });
    return () => {
      if (unlisten) unlisten();
    };
  }, [callback]);
}

export function useAutoRecoveryTriggered(callback: (payload: AutoRecoveryTriggeredPayload) => void): void {
  useEffect(() => {
    let unlisten: UnlistenFn | undefined;
    subscribeAutoRecoveryTriggered(callback).then((fn) => {
      unlisten = fn;
    });
    return () => {
      if (unlisten) unlisten();
    };
  }, [callback]);
}