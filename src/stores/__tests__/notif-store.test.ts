import { describe, it, expect, beforeEach, vi } from 'vitest';
import { useNotifStore, initializeNotifStore } from '../notif-store';
import { subscribeRecoveryStarted, subscribeRecoveryCompleted, subscribeRecoveryFailed } from '../../lib/events';

vi.mock('../../lib/events', () => ({
  subscribeRecoveryStarted: vi.fn((_cb) => Promise.resolve(() => {})),
  subscribeRecoveryCompleted: vi.fn((_cb) => Promise.resolve(() => {})),
  subscribeRecoveryFailed: vi.fn((_cb) => Promise.resolve(() => {})),
  subscribeRecoveryItemCompleted: vi.fn((_cb) => Promise.resolve(() => {})),
  subscribeProxyRecovering: vi.fn((_cb) => Promise.resolve(() => {})),
  subscribeProxyRecovered: vi.fn((_cb) => Promise.resolve(() => {})),
}));

describe('notif-store', () => {
  beforeEach(() => {
    useNotifStore.getState().reset();
    vi.clearAllMocks();
  });

  it('initial state is correct', () => {
    const state = useNotifStore.getState();
    expect(state.notifications).toEqual([]);
    expect(state.unreadCount).toBe(0);
    expect(state.maxNotifications).toBe(50);
  });

  it('addNotification adds notification and updates unreadCount', () => {
    useNotifStore.getState().addNotification('info', 'Test', 'Test message');

    const state = useNotifStore.getState();
    expect(state.notifications).toHaveLength(1);
    expect(state.unreadCount).toBe(1);
    expect(state.notifications[0].type).toBe('info');
    expect(state.notifications[0].title).toBe('Test');
    expect(state.notifications[0].read).toBe(false);
  });

  it('markAsRead reduces unreadCount', () => {
    useNotifStore.getState().addNotification('info', 'Test', 'Message');
    const id = useNotifStore.getState().notifications[0].id;
    useNotifStore.getState().markAsRead(id);

    const state = useNotifStore.getState();
    expect(state.unreadCount).toBe(0);
    expect(state.notifications[0].read).toBe(true);
  });

  it('markAllAsRead marks all as read', () => {
    useNotifStore.getState().addNotification('info', 'Test1', 'Msg1');
    useNotifStore.getState().addNotification('error', 'Test2', 'Msg2');
    useNotifStore.getState().markAllAsRead();

    const state = useNotifStore.getState();
    expect(state.unreadCount).toBe(0);
    expect(state.notifications.every((n) => n.read)).toBe(true);
  });

  it('clearNotification removes notification', () => {
    useNotifStore.getState().addNotification('info', 'Test', 'Message');
    const id = useNotifStore.getState().notifications[0].id;
    useNotifStore.getState().clearNotification(id);

    const state = useNotifStore.getState();
    expect(state.notifications).toHaveLength(0);
    expect(state.unreadCount).toBe(0);
  });

  it('clearAll removes all notifications', () => {
    useNotifStore.getState().addNotification('info', 'Test1', 'Msg1');
    useNotifStore.getState().addNotification('info', 'Test2', 'Msg2');
    useNotifStore.getState().clearAll();

    const state = useNotifStore.getState();
    expect(state.notifications).toHaveLength(0);
    expect(state.unreadCount).toBe(0);
  });

  it('handleRecoveryStarted adds info notification', () => {
    useNotifStore.getState().handleRecoveryStarted({
      task_id: 'task-1',
      total_items: 5,
    });

    const state = useNotifStore.getState();
    expect(state.notifications).toHaveLength(1);
    expect(state.notifications[0].type).toBe('info');
    expect(state.notifications[0].title).toBe('恢复任务开始');
  });

  it('handleRecoveryCompleted adds success notification when no failures', () => {
    useNotifStore.getState().handleRecoveryCompleted({
      task_id: 'task-1',
      succeeded: 5,
      failed: 0,
    });

    const state = useNotifStore.getState();
    expect(state.notifications[0].type).toBe('success');
  });

  it('handleRecoveryCompleted adds warning notification when has failures', () => {
    useNotifStore.getState().handleRecoveryCompleted({
      task_id: 'task-1',
      succeeded: 3,
      failed: 2,
    });

    const state = useNotifStore.getState();
    expect(state.notifications[0].type).toBe('warning');
  });

  it('handleRecoveryFailed adds error notification', () => {
    useNotifStore.getState().handleRecoveryFailed({
      task_id: 'task-1',
      failed_items: ['win-proxy', 'win-hosts'],
    });

    const state = useNotifStore.getState();
    expect(state.notifications[0].type).toBe('error');
    expect(state.notifications[0].message).toContain('win-proxy');
  });

  it('notifications are limited to maxNotifications', () => {
    for (let i = 0; i < 60; i++) {
      useNotifStore.getState().addNotification('info', `Test ${i}`, `Msg ${i}`);
    }

    const state = useNotifStore.getState();
    expect(state.notifications.length).toBe(50);
  });

  it('initializeNotifStore subscribes to recovery events', async () => {
    initializeNotifStore();

    expect(subscribeRecoveryStarted).toHaveBeenCalled();
    expect(subscribeRecoveryCompleted).toHaveBeenCalled();
    expect(subscribeRecoveryFailed).toHaveBeenCalled();
  });
});