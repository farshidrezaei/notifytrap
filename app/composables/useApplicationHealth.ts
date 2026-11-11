import type { ApplicationHealthSnapshot, ServerStatusChangedEvent } from '~/types/application'
import {
  invokeTauriCommand,
  isTauriRuntime,
  subscribeTauriEvent,
} from '~/utils/tauriClient'

/**
 * Manages application health snapshot and live local capture server status updates.
 */
export function useApplicationHealth() {
  const health = useState<ApplicationHealthSnapshot | null>('application-health', () => null)
  const isLoading = useState<boolean>('application-health-loading', () => false)
  const hasInitialized = useState<boolean>('application-health-initialized', () => false)
  const errorKey = useState<string | null>('application-health-error', () => null)
  const isListenerRegistered = useState<boolean>('application-health-listener-ready', () => false)
  const isDesktopRuntime = computed(() => isTauriRuntime())
  const isInitialLoading = computed(() => isDesktopRuntime.value && !hasInitialized.value)

  async function refreshHealth(): Promise<void> {
    if (!isTauriRuntime()) {
      health.value = null
      errorKey.value = null
      hasInitialized.value = true
      return
    }

    isLoading.value = true
    errorKey.value = null

    try {
      health.value = await invokeTauriCommand<ApplicationHealthSnapshot>('get_application_health')
      await ensureStatusSubscription()
    } catch {
      errorKey.value = 'errors.healthLoadFailed'
    } finally {
      hasInitialized.value = true
      isLoading.value = false
    }
  }

  async function ensureStatusSubscription(): Promise<void> {
    if (isListenerRegistered.value || !isTauriRuntime()) {
      return
    }
    isListenerRegistered.value = true

    await subscribeTauriEvent<ServerStatusChangedEvent>('server:status_changed', (payload) => {
      if (!health.value) {
        return
      }
      health.value = {
        ...health.value,
        isHttpServerRunning: payload.isHttpServerRunning,
        isSmtpServerRunning: payload.isSmtpServerRunning,
        httpEndpoint: payload.httpEndpoint,
        smtpEndpoint: payload.smtpEndpoint,
      }
    })
  }

  return {
    health: readonly(health),
    isLoading: readonly(isLoading),
    hasInitialized: readonly(hasInitialized),
    isInitialLoading,
    errorKey: readonly(errorKey),
    isDesktopRuntime,
    refreshHealth,
  }
}
