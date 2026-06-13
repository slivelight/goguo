import { create } from 'zustand';
import type { ServiceStartedPayload, ServiceStoppedPayload, AutoRecoveryTriggeredPayload, ProxyRecoveringPayload, ProxyRecoveredPayload } from '../lib/types';
import { getServiceStatus } from '../lib/tauri-ipc';
import { subscribeServiceStarted, subscribeServiceStopped, subscribeAutoRecoveryTriggered, subscribeProxyRecovering, subscribeProxyRecovered } from '../lib/events';

interface ServiceState {
  mihomoRunning: boolean;
  proxyGuardRestartCount: number;
  isRecovering: boolean;
  isLoading: boolean;
  error: string | null;
}

interface ServiceActions {
  fetchServiceStatus: () => Promise<void>;
  handleServiceStarted: (payload: ServiceStartedPayload) => void;
  handleServiceStopped: (_payload: ServiceStoppedPayload) => void;
  handleAutoRecoveryTriggered: (payload: AutoRecoveryTriggeredPayload) => void;
  handleProxyRecovering: (payload: ProxyRecoveringPayload) => void;
  handleProxyRecovered: (payload: ProxyRecoveredPayload) => void;
  reset: () => void;
}

const initialState: ServiceState = {
  mihomoRunning: false,
  proxyGuardRestartCount: 0,
  isRecovering: false,
  isLoading: false,
  error: null,
};

export const useServiceStore = create<ServiceState & ServiceActions>((set) => ({
  ...initialState,

  fetchServiceStatus: async () => {
    set({ isLoading: true, error: null });
    try {
      const status = await getServiceStatus();
      set({
        mihomoRunning: status.mihomo_running,
        proxyGuardRestartCount: status.proxy_guard_restart_count,
        isLoading: false,
      });
    } catch (err) {
      set({
        isLoading: false,
        error: err instanceof Error ? err.message : 'Failed to fetch service status',
      });
    }
  },

  handleServiceStarted: (payload: ServiceStartedPayload) => {
    set({
      mihomoRunning: payload.mihomo_running,
    });
  },

  handleServiceStopped: (_payload: ServiceStoppedPayload) => {
    set({
      mihomoRunning: false,
    });
  },

  handleAutoRecoveryTriggered: (payload: AutoRecoveryTriggeredPayload) => {
    set({
      proxyGuardRestartCount: payload.restart_attempts,
    });
  },

  handleProxyRecovering: (_payload: ProxyRecoveringPayload) => {
    set({ isRecovering: true });
  },

  handleProxyRecovered: (_payload: ProxyRecoveredPayload) => {
    set({ isRecovering: false });
  },

  reset: () => set(initialState),
}));

export function initializeServiceStore(): void {
  subscribeServiceStarted((payload) => {
    useServiceStore.getState().handleServiceStarted(payload);
  });
  subscribeServiceStopped((payload) => {
    useServiceStore.getState().handleServiceStopped(payload);
  });
  subscribeAutoRecoveryTriggered((payload) => {
    useServiceStore.getState().handleAutoRecoveryTriggered(payload);
  });
  subscribeProxyRecovering((payload) => {
    useServiceStore.getState().handleProxyRecovering(payload);
  });
  subscribeProxyRecovered((payload) => {
    useServiceStore.getState().handleProxyRecovered(payload);
  });
}