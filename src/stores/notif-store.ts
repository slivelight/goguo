import { create } from 'zustand';
import type { RecoveryStartedPayload, RecoveryCompletedPayload, RecoveryFailedPayload, RecoveryItemCompletedPayload } from '../lib/types';
import { subscribeRecoveryStarted, subscribeRecoveryCompleted, subscribeRecoveryFailed, subscribeRecoveryItemCompleted } from '../lib/events';

type NotificationType = 'info' | 'success' | 'warning' | 'error';

interface Notification {
  id: string;
  type: NotificationType;
  title: string;
  message: string;
  timestamp: string;
  read: boolean;
}

interface NotifState {
  notifications: Notification[];
  unreadCount: number;
  maxNotifications: number;
}

interface NotifActions {
  addNotification: (type: NotificationType, title: string, message: string) => void;
  markAsRead: (id: string) => void;
  markAllAsRead: () => void;
  clearNotification: (id: string) => void;
  clearAll: () => void;
  handleRecoveryStarted: (payload: RecoveryStartedPayload) => void;
  handleRecoveryCompleted: (payload: RecoveryCompletedPayload) => void;
  handleRecoveryFailed: (payload: RecoveryFailedPayload) => void;
  handleRecoveryItemCompleted: (payload: RecoveryItemCompletedPayload) => void;
  reset: () => void;
}

const initialState: NotifState = {
  notifications: [],
  unreadCount: 0,
  maxNotifications: 50,
};

function generateId(): string {
  return `${Date.now()}-${Math.random().toString(36).slice(2, 9)}`;
}

export const useNotifStore = create<NotifState & NotifActions>((set, get) => ({
  ...initialState,

  addNotification: (type: NotificationType, title: string, message: string) => {
    const notification: Notification = {
      id: generateId(),
      type,
      title,
      message,
      timestamp: new Date().toISOString(),
      read: false,
    };
    set((state) => {
      const notifications = [notification, ...state.notifications].slice(0, state.maxNotifications);
      const unreadCount = notifications.filter((n) => !n.read).length;
      return { notifications, unreadCount };
    });

    // System notification for background app (error/warning only)
    if (document.hidden && (type === 'error' || type === 'warning')) {
      try {
        if (typeof Notification !== 'undefined' && Notification.permission === 'granted') {
          new Notification(title, { body: message });
        }
      } catch {
        // Notification API unavailable — skip silently
      }
    }
  },

  markAsRead: (id: string) => {
    set((state) => {
      const notifications = state.notifications.map((n) =>
        n.id === id ? { ...n, read: true } : n
      );
      const unreadCount = notifications.filter((n) => !n.read).length;
      return { notifications, unreadCount };
    });
  },

  markAllAsRead: () => {
    set((state) => {
      const notifications = state.notifications.map((n) => ({ ...n, read: true }));
      return { notifications, unreadCount: 0 };
    });
  },

  clearNotification: (id: string) => {
    set((state) => {
      const notifications = state.notifications.filter((n) => n.id !== id);
      const unreadCount = notifications.filter((n) => !n.read).length;
      return { notifications, unreadCount };
    });
  },

  clearAll: () => set(initialState),

  handleRecoveryStarted: (payload: RecoveryStartedPayload) => {
    get().addNotification(
      'info',
      '恢复任务开始',
      `开始恢复 ${payload.total_items} 个状态项`
    );
  },

  handleRecoveryCompleted: (payload: RecoveryCompletedPayload) => {
    get().addNotification(
      payload.failed === 0 ? 'success' : 'warning',
      '恢复任务完成',
      `成功 ${payload.succeeded} 项，失败 ${payload.failed} 项`
    );
  },

  handleRecoveryFailed: (payload: RecoveryFailedPayload) => {
    get().addNotification(
      'error',
      '恢复任务失败',
      `失败项: ${payload.failed_items.join(', ')}`
    );
  },

  handleRecoveryItemCompleted: (payload: RecoveryItemCompletedPayload) => {
    if (!payload.success) {
      get().addNotification(
        'warning',
        '状态项恢复失败',
        `${payload.state_item_id}: ${payload.failure_reason || '未知原因'}`
      );
    }
  },

  reset: () => set(initialState),
}));

export function initializeNotifStore(): void {
  subscribeRecoveryStarted((payload) => {
    useNotifStore.getState().handleRecoveryStarted(payload);
  });
  subscribeRecoveryCompleted((payload) => {
    useNotifStore.getState().handleRecoveryCompleted(payload);
  });
  subscribeRecoveryFailed((payload) => {
    useNotifStore.getState().handleRecoveryFailed(payload);
  });
  subscribeRecoveryItemCompleted((payload) => {
    useNotifStore.getState().handleRecoveryItemCompleted(payload);
  });
}