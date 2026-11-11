import { relaunch } from '@tauri-apps/plugin-process'
import { check, type Update } from '@tauri-apps/plugin-updater'
import type { ApplicationSettingEntry } from '~/types/application'
import {
  extractCommandErrorMessage,
  invokeTauriCommand,
  isTauriRuntime,
} from '~/utils/tauriClient'
import {
  AUTO_UPDATE_SETTING_KEY,
  buildAvailableUpdateSummary,
  calculateDownloadProgressPercentage,
  UPDATE_CHECK_TIMEOUT_MILLISECONDS,
  type AvailableUpdateSummary,
} from '~/utils/updaterHelpers'

let activePendingUpdate: Update | null = null

async function replacePendingUpdate(nextUpdate: Update | null): Promise<void> {
  if (activePendingUpdate && activePendingUpdate !== nextUpdate) {
    try {
      await activePendingUpdate.close()
    } catch {
      // Ignore cleanup errors for already closed update resources.
    }
  }
  activePendingUpdate = nextUpdate
}

/**
 * Coordinates manual and silent background desktop updates via Tauri updater with toast action triggers.
 */
export function useAppUpdater() {
  const { t } = useI18n()
  const router = useRouter()
  const toast = useToast()

  const isAutoUpdateCheckEnabled = useState<boolean>('updater-auto-check-enabled', () => true)
  const isCheckingForUpdates = useState<boolean>('updater-is-checking', () => false)
  const isDownloadingUpdate = useState<boolean>('updater-is-downloading', () => false)
  const isRestartPending = useState<boolean>('updater-is-restart-pending', () => false)
  const hasRunBackgroundCheck = useState<boolean>('updater-background-checked', () => false)
  const hasLoadedPreference = useState<boolean>('updater-preference-loaded', () => false)
  const notifiedVersion = useState<string | null>('updater-notified-version', () => null)
  const availableUpdate = useState<AvailableUpdateSummary | null>(
    'updater-available-summary',
    () => null,
  )
  const downloadProgressPercentage = useState<number | null>(
    'updater-download-progress',
    () => null,
  )
  const lastCheckedAt = useState<string | null>('updater-last-checked-at', () => null)
  const updateErrorMessage = useState<string | null>('updater-error-message', () => null)

  async function loadAutoUpdatePreference(): Promise<boolean> {
    if (hasLoadedPreference.value || !isTauriRuntime()) {
      hasLoadedPreference.value = true
      return isAutoUpdateCheckEnabled.value
    }

    try {
      const entry = await invokeTauriCommand<ApplicationSettingEntry<boolean> | null>(
        'get_application_setting',
        { key: AUTO_UPDATE_SETTING_KEY },
      )
      if (entry && typeof entry.value === 'boolean') {
        isAutoUpdateCheckEnabled.value = entry.value
      }
    } catch {
      // Keep default auto-check enabled if preference cannot be loaded yet.
    } finally {
      hasLoadedPreference.value = true
    }

    return isAutoUpdateCheckEnabled.value
  }

  async function setAutoUpdateCheckEnabled(shouldEnable: boolean): Promise<void> {
    const previousValue = isAutoUpdateCheckEnabled.value
    isAutoUpdateCheckEnabled.value = shouldEnable

    if (!isTauriRuntime()) {
      return
    }

    try {
      await invokeTauriCommand('set_application_setting', {
        key: AUTO_UPDATE_SETTING_KEY,
        value: shouldEnable,
      })
    } catch (error) {
      isAutoUpdateCheckEnabled.value = previousValue
      toast.add({
        title: t('common.errorTitle'),
        description: extractCommandErrorMessage(error, t('common.unexpectedError')),
        icon: 'i-lucide-alert-circle',
        color: 'error',
      })
    }
  }

  function showUpdateAvailableToast(summary: AvailableUpdateSummary): void {
    toast.add({
      title: t('settings.updates.toastAvailableTitle', { version: summary.version }),
      description:
        summary.releaseNotes ||
        t('settings.updates.toastAvailableDescription', {
          currentVersion: summary.currentVersion,
          version: summary.version,
        }),
      icon: 'i-lucide-arrow-up-circle',
      color: 'primary',
      duration: 0,
      actions: [
        {
          label: t('settings.updates.actions.installNow'),
          color: 'primary',
          variant: 'solid',
          onClick: () => {
            void downloadAndInstallUpdate()
          },
        },
        {
          label: t('settings.updates.actions.openSettings'),
          color: 'neutral',
          variant: 'outline',
          onClick: () => {
            void router.push('/settings#application-updates')
          },
        },
      ],
    })
  }

  async function checkForUpdates(options: { isBackground?: boolean } = {}): Promise<void> {
    const isBackground = options.isBackground ?? false

    if (isCheckingForUpdates.value || isDownloadingUpdate.value) {
      return
    }

    if (!isTauriRuntime()) {
      if (!isBackground) {
        toast.add({
          title: t('settings.updates.desktopOnlyTitle'),
          description: t('settings.updates.desktopOnlyDescription'),
          icon: 'i-lucide-info',
          color: 'neutral',
        })
      }
      return
    }

    isCheckingForUpdates.value = true
    updateErrorMessage.value = null

    try {
      const candidateUpdate = await check({
        timeout: UPDATE_CHECK_TIMEOUT_MILLISECONDS,
      })
      lastCheckedAt.value = new Date().toISOString()

      if (!candidateUpdate) {
        availableUpdate.value = null
        await replacePendingUpdate(null)
        if (!isBackground) {
          toast.add({
            title: t('settings.updates.upToDateTitle'),
            description: t('settings.updates.upToDateDescription'),
            icon: 'i-lucide-check-circle-2',
            color: 'success',
          })
        }
        return
      }

      const summary = buildAvailableUpdateSummary(candidateUpdate)
      availableUpdate.value = summary
      await replacePendingUpdate(candidateUpdate)

      if (!isBackground || notifiedVersion.value !== summary.version) {
        notifiedVersion.value = summary.version
        showUpdateAvailableToast(summary)
      }
    } catch (error) {
      if (!isBackground) {
        const message = extractCommandErrorMessage(error, t('errors.updateCheckFailed'))
        updateErrorMessage.value = message
        toast.add({
          title: t('common.errorTitle'),
          description: message,
          icon: 'i-lucide-alert-circle',
          color: 'error',
        })
      }
    } finally {
      isCheckingForUpdates.value = false
    }
  }

  async function runBackgroundUpdateCheck(): Promise<void> {
    if (hasRunBackgroundCheck.value || !isTauriRuntime()) {
      return
    }
    hasRunBackgroundCheck.value = true

    const isEnabled = await loadAutoUpdatePreference()
    if (!isEnabled) {
      return
    }

    await checkForUpdates({ isBackground: true })
  }

  async function downloadAndInstallUpdate(): Promise<void> {
    if (!activePendingUpdate || isDownloadingUpdate.value) {
      return
    }

    isDownloadingUpdate.value = true
    updateErrorMessage.value = null
    downloadProgressPercentage.value = 0

    let totalContentLength: number | null = null
    let downloadedBytes = 0

    try {
      await activePendingUpdate.downloadAndInstall((progressEvent) => {
        if (progressEvent.event === 'Started') {
          totalContentLength = progressEvent.data.contentLength ?? null
          downloadedBytes = 0
          downloadProgressPercentage.value = 0
        } else if (progressEvent.event === 'Progress') {
          downloadedBytes += progressEvent.data.chunkLength
          downloadProgressPercentage.value = calculateDownloadProgressPercentage(
            downloadedBytes,
            totalContentLength,
          )
        } else if (progressEvent.event === 'Finished') {
          downloadProgressPercentage.value = 100
        }
      })

      isRestartPending.value = true
      await replacePendingUpdate(null)

      toast.add({
        title: t('settings.updates.installedTitle'),
        description: t('settings.updates.installedDescription'),
        icon: 'i-lucide-rotate-cw',
        color: 'success',
        duration: 0,
        actions: [
          {
            label: t('settings.updates.actions.restartNow'),
            color: 'success',
            variant: 'solid',
            onClick: () => {
              void restartApplication()
            },
          },
        ],
      })
    } catch (error) {
      const message = extractCommandErrorMessage(error, t('errors.updateInstallFailed'))
      updateErrorMessage.value = message
      toast.add({
        title: t('common.errorTitle'),
        description: message,
        icon: 'i-lucide-alert-circle',
        color: 'error',
      })
    } finally {
      isDownloadingUpdate.value = false
    }
  }

  async function restartApplication(): Promise<void> {
    if (!isTauriRuntime()) {
      return
    }
    await relaunch()
  }

  return {
    isAutoUpdateCheckEnabled: readonly(isAutoUpdateCheckEnabled),
    isCheckingForUpdates: readonly(isCheckingForUpdates),
    isDownloadingUpdate: readonly(isDownloadingUpdate),
    isRestartPending: readonly(isRestartPending),
    availableUpdate: readonly(availableUpdate),
    downloadProgressPercentage: readonly(downloadProgressPercentage),
    lastCheckedAt: readonly(lastCheckedAt),
    updateErrorMessage: readonly(updateErrorMessage),
    loadAutoUpdatePreference,
    setAutoUpdateCheckEnabled,
    checkForUpdates,
    runBackgroundUpdateCheck,
    downloadAndInstallUpdate,
    restartApplication,
  }
}
