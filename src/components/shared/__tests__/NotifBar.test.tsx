import { describe, it, expect, vi, beforeEach } from 'vitest';
import { render, screen, fireEvent } from '@testing-library/react';
import NotifBar from '../NotifBar';
import * as notifStore from '../../../stores/notif-store';

vi.mock('../../../stores/notif-store', () => ({
  useNotifStore: vi.fn(),
}));

describe('NotifBar', () => {
  beforeEach(() => {
    vi.clearAllMocks();
  });

  it('renders empty state when no notifications', () => {
    vi.mocked(notifStore.useNotifStore).mockReturnValue({
      notifications: [],
      unreadCount: 0,
      markAllAsRead: vi.fn(),
      clearAll: vi.fn(),
    } as unknown as ReturnType<typeof notifStore.useNotifStore>);

    render(<NotifBar />);
    expect(screen.getByText('暂无通知')).toBeDefined();
  });

  it('renders notifications list', () => {
    vi.mocked(notifStore.useNotifStore).mockReturnValue({
      notifications: [
        { id: '1', type: 'info', title: '通知1', message: '消息1', timestamp: '2026-05-21T08:00:00Z', read: false },
        { id: '2', type: 'success', title: '通知2', message: '消息2', timestamp: '2026-05-21T08:05:00Z', read: true },
      ],
      unreadCount: 1,
      markAllAsRead: vi.fn(),
      clearAll: vi.fn(),
    } as unknown as ReturnType<typeof notifStore.useNotifStore>);

    render(<NotifBar />);
    expect(screen.getByText('通知1')).toBeDefined();
    expect(screen.getByText('通知2')).toBeDefined();
  });

  it('shows unread count', () => {
    vi.mocked(notifStore.useNotifStore).mockReturnValue({
      notifications: [
        { id: '1', type: 'info', title: '通知', message: '消息', timestamp: '2026-05-21T08:00:00Z', read: false },
      ],
      unreadCount: 1,
      markAllAsRead: vi.fn(),
      clearAll: vi.fn(),
    } as unknown as ReturnType<typeof notifStore.useNotifStore>);

    render(<NotifBar />);
    expect(screen.getByText('1 条未读')).toBeDefined();
  });

  it('calls markAllAsRead when button clicked', () => {
    const markAllAsRead = vi.fn();
    vi.mocked(notifStore.useNotifStore).mockReturnValue({
      notifications: [
        { id: '1', type: 'info', title: '通知', message: '消息', timestamp: '2026-05-21T08:00:00Z', read: false },
      ],
      unreadCount: 1,
      markAllAsRead,
      clearAll: vi.fn(),
    } as unknown as ReturnType<typeof notifStore.useNotifStore>);

    render(<NotifBar />);
    fireEvent.click(screen.getByText('全部标记已读'));
    expect(markAllAsRead).toHaveBeenCalledOnce();
  });

  it('calls clearAll when button clicked', () => {
    const clearAll = vi.fn();
    vi.mocked(notifStore.useNotifStore).mockReturnValue({
      notifications: [
        { id: '1', type: 'info', title: '通知', message: '消息', timestamp: '2026-05-21T08:00:00Z', read: true },
      ],
      unreadCount: 0,
      markAllAsRead: vi.fn(),
      clearAll,
    } as unknown as ReturnType<typeof notifStore.useNotifStore>);

    render(<NotifBar />);
    fireEvent.click(screen.getByText('清空'));
    expect(clearAll).toHaveBeenCalledOnce();
  });
});