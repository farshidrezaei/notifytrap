import type {
  ChannelType,
  Notification,
  NotificationCapturedEvent,
  NotificationDetail,
  NotificationPage,
  NotificationStatus,
} from '~/types/application'
import {
  invokeTauriCommand,
  isTauriRuntime,
  subscribeTauriEvent,
} from '~/utils/tauriClient'

const DEFAULT_INBOX_PAGE_SIZE = 25

/**
 * Manages scoped inbox notification querying, inspector selection, status transitions, and live capture events.
 */
export function useNotifications() {
  const { activeWorkspaceId, activeProjectId, activeEnvironmentId } = useWorkspaceContext()

  const notificationPage = useState<NotificationPage | null>('scoped-notifications-page', () => null)
  const selectedDetail = useState<NotificationDetail | null>('selected-notification-detail', () => null)
  const isLoadingList = useState<boolean>('notifications-list-loading', () => false)
  const isLoadingDetail = useState<boolean>('notification-detail-loading', () => false)
  const isListenerRegistered = useState<boolean>('notification-captured-listener', () => false)

  const filterChannelType = useState<ChannelType | 'all'>('inbox-filter-channel-type', () => 'all')
  const filterStatus = useState<NotificationStatus | 'all'>('inbox-filter-status', () => 'all')
  const filterSearchQuery = useState<string>('inbox-filter-search', () => '')
  const currentPage = useState<number>('inbox-current-page', () => 1)

  const unreadCount = computed<number>(() => {
    if (!notificationPage.value) {
      return 0
    }
    return notificationPage.value.items.filter((item) => item.status === 'captured').length
  })

  async function refreshInbox(): Promise<void> {
    if (
      !isTauriRuntime()
      || !activeWorkspaceId.value
      || !activeProjectId.value
      || !activeEnvironmentId.value
    ) {
      notificationPage.value = null
      selectedDetail.value = null
      return
    }

    isLoadingList.value = true
    try {
      const offset = (currentPage.value - 1) * DEFAULT_INBOX_PAGE_SIZE
      notificationPage.value = await invokeTauriCommand<NotificationPage>('query_notifications', {
        query: {
          workspaceId: activeWorkspaceId.value,
          projectId: activeProjectId.value,
          environmentId: activeEnvironmentId.value,
          channelType: filterChannelType.value === 'all' ? undefined : filterChannelType.value,
          status: filterStatus.value === 'all' ? undefined : filterStatus.value,
          searchQuery: filterSearchQuery.value.trim() || undefined,
          limit: DEFAULT_INBOX_PAGE_SIZE,
          offset,
        },
      })
    } finally {
      isLoadingList.value = false
    }
  }

  async function ensureLiveSubscription(): Promise<void> {
    if (isListenerRegistered.value || !isTauriRuntime()) {
      return
    }
    isListenerRegistered.value = true

    await subscribeTauriEvent<NotificationCapturedEvent>('notification:captured', async (payload) => {
      const item = payload.notification
      if (
        item.workspaceId === activeWorkspaceId.value
        && item.projectId === activeProjectId.value
        && item.environmentId === activeEnvironmentId.value
      ) {
        await refreshInbox()
      }
    })
  }

  async function selectNotification(notificationId: string): Promise<void> {
    if (!isTauriRuntime()) {
      return
    }

    isLoadingDetail.value = true
    try {
      const detail = await invokeTauriCommand<NotificationDetail>('get_notification_detail', {
        notificationId,
      })

      if (detail.notification.status === 'captured') {
        const updated = await invokeTauriCommand<Notification>('update_notification_status', {
          notificationId,
          status: 'read',
        })
        selectedDetail.value = {
          ...detail,
          notification: updated,
        }
        patchSummaryStatus(notificationId, 'read')
      } else {
        selectedDetail.value = detail
      }
    } finally {
      isLoadingDetail.value = false
    }
  }

  async function updateNotificationStatus(
    notificationId: string,
    status: NotificationStatus,
  ): Promise<void> {
    const updated = await invokeTauriCommand<Notification>('update_notification_status', {
      notificationId,
      status,
    })

    if (selectedDetail.value?.notification.id === notificationId) {
      selectedDetail.value = {
        ...selectedDetail.value,
        notification: updated,
      }
    }
    patchSummaryStatus(notificationId, status)
  }

  async function deleteNotification(notificationId: string): Promise<void> {
    await invokeTauriCommand('delete_notification', { notificationId })
    if (selectedDetail.value?.notification.id === notificationId) {
      selectedDetail.value = null
    }
    await refreshInbox()
  }

  async function clearScopeNotifications(): Promise<number> {
    if (!activeWorkspaceId.value || !activeProjectId.value || !activeEnvironmentId.value) {
      return 0
    }
    const deletedCount = await invokeTauriCommand<number>('clear_scope_notifications', {
      workspaceId: activeWorkspaceId.value,
      projectId: activeProjectId.value,
      environmentId: activeEnvironmentId.value,
    })
    selectedDetail.value = null
    currentPage.value = 1
    await refreshInbox()
    return deletedCount
  }

  function patchSummaryStatus(notificationId: string, status: NotificationStatus): void {
    if (!notificationPage.value) {
      return
    }
    notificationPage.value = {
      ...notificationPage.value,
      items: notificationPage.value.items.map((item) =>
        item.id === notificationId ? { ...item, status } : item,
      ),
    }
  }

  return {
    notificationPage: readonly(notificationPage),
    selectedDetail: readonly(selectedDetail),
    isLoadingList: readonly(isLoadingList),
    isLoadingDetail: readonly(isLoadingDetail),
    filterChannelType,
    filterStatus,
    filterSearchQuery,
    currentPage,
    pageSize: DEFAULT_INBOX_PAGE_SIZE,
    unreadCount,
    refreshInbox,
    ensureLiveSubscription,
    selectNotification,
    updateNotificationStatus,
    deleteNotification,
    clearScopeNotifications,
  }
}
