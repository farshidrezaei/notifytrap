import type { ApplicationSettingEntry, NotificationSummary } from '~/types/application'
import { invokeTauriCommand, isTauriRuntime } from '~/utils/tauriClient'

const SETTING_MINIMIZE_TO_TRAY = 'desktop.minimizeToTray'
const SETTING_NATIVE_NOTIFICATIONS = 'desktop.nativeNotifications'

/**
 * Manages desktop tray close behavior and optional OS-level capture notifications.
 */
export function useDesktopPreferences() {
  const isMinimizeToTrayEnabled = useState<boolean>('desktop-pref-minimize-tray', () => false)
  const isNativeNotificationsEnabled = useState<boolean>('desktop-pref-native-notifications', () => false)
  const isLoaded = useState<boolean>('desktop-pref-loaded', () => false)

  async function loadDesktopPreferences(): Promise<void> {
    if (!isTauriRuntime()) {
      isLoaded.value = true
      return
    }
    const [trayEntry, nativeEntry] = await Promise.all([
      invokeTauriCommand<ApplicationSettingEntry | null>('get_application_setting', {
        key: SETTING_MINIMIZE_TO_TRAY,
      }),
      invokeTauriCommand<ApplicationSettingEntry | null>('get_application_setting', {
        key: SETTING_NATIVE_NOTIFICATIONS,
      }),
    ])
    isMinimizeToTrayEnabled.value = trayEntry?.value === 'true'
    isNativeNotificationsEnabled.value = nativeEntry?.value === 'true'
    isLoaded.value = true
  }

  async function setMinimizeToTray(enabled: boolean): Promise<void> {
    isMinimizeToTrayEnabled.value = enabled
    if (!isTauriRuntime()) {
      return
    }
    await invokeTauriCommand<ApplicationSettingEntry>('set_application_setting', {
      key: SETTING_MINIMIZE_TO_TRAY,
      value: enabled ? 'true' : 'false',
    })
  }

  async function setNativeNotifications(enabled: boolean): Promise<void> {
    if (enabled && typeof window !== 'undefined' && 'Notification' in window) {
      if (Notification.permission === 'default') {
        await Notification.requestPermission()
      }
    }
    isNativeNotificationsEnabled.value = enabled
    if (!isTauriRuntime()) {
      return
    }
    await invokeTauriCommand<ApplicationSettingEntry>('set_application_setting', {
      key: SETTING_NATIVE_NOTIFICATIONS,
      value: enabled ? 'true' : 'false',
    })
  }

  function dispatchNativeNotificationIfEnabled(
    notification: Pick<NotificationSummary, 'channelType' | 'recipient' | 'subject' | 'previewText'>,
  ): void {
    if (!isNativeNotificationsEnabled.value || !isTauriRuntime()) {
      return
    }
    const title = notification.subject || `${notification.channelType.toUpperCase()} · ${notification.recipient}`
    const body = notification.previewText || notification.recipient
    void invokeTauriCommand('send_native_notification', { title, body }).catch(() => {
      // Ignore desktop notification daemon errors
    })
  }

  return {
    isMinimizeToTrayEnabled: readonly(isMinimizeToTrayEnabled),
    isNativeNotificationsEnabled: readonly(isNativeNotificationsEnabled),
    isLoaded: readonly(isLoaded),
    loadDesktopPreferences,
    setMinimizeToTray,
    setNativeNotifications,
    dispatchNativeNotificationIfEnabled,
  }
}
