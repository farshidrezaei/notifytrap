import type {
  TunnelStartConfiguration,
  TunnelStatusReport,
} from '~/types/application'
import {
  extractCommandErrorMessage,
  invokeTauriCommand,
  isTauriRuntime,
  subscribeTauriEvent,
} from '~/utils/tauriClient'

const DEFAULT_REPORT: TunnelStatusReport = {
  isActive: false,
  status: 'disconnected',
  provider: 'cloudflared',
  localTarget: 'http://127.0.0.1:4080',
}

export function useTunnel() {
  const tunnelStatus = useState<TunnelStatusReport>('tunnel-status-report', () => DEFAULT_REPORT)
  const isOperating = useState<boolean>('tunnel-is-operating', () => false)
  const tunnelErrorMessage = useState<string | null>('tunnel-error-message', () => null)

  const isTunnelActive = computed(() => tunnelStatus.value.isActive)
  const isTunnelConnected = computed(() => tunnelStatus.value.status === 'connected')
  const publicTunnelUrl = computed(() => tunnelStatus.value.publicUrl ?? null)

  async function fetchTunnelStatus(): Promise<void> {
    if (!isTauriRuntime()) {
      return
    }
    try {
      const report = await invokeTauriCommand<TunnelStatusReport>('get_tunnel_status')
      tunnelStatus.value = report
    } catch (error) {
      // Keep existing state
    }
  }

  async function startTunnel(configuration: TunnelStartConfiguration): Promise<void> {
    if (!isTauriRuntime()) {
      return
    }
    isOperating.value = true
    tunnelErrorMessage.value = null
    try {
      const report = await invokeTauriCommand<TunnelStatusReport>('start_tunnel', { configuration })
      tunnelStatus.value = report
    } catch (error) {
      const message = extractCommandErrorMessage(error, 'Failed to start tunnel')
      tunnelErrorMessage.value = message
      throw error
    } finally {
      isOperating.value = false
    }
  }

  async function stopTunnel(): Promise<void> {
    if (!isTauriRuntime()) {
      return
    }
    isOperating.value = true
    tunnelErrorMessage.value = null
    try {
      const report = await invokeTauriCommand<TunnelStatusReport>('stop_tunnel')
      tunnelStatus.value = report
    } catch (error) {
      const message = extractCommandErrorMessage(error, 'Failed to stop tunnel')
      tunnelErrorMessage.value = message
      throw error
    } finally {
      isOperating.value = false
    }
  }

  onMounted(() => {
    fetchTunnelStatus()
    subscribeTauriEvent<TunnelStatusReport>('tunnel:status_changed', (updatedReport) => {
      tunnelStatus.value = updatedReport
    })
  })

  return {
    tunnelStatus: readonly(tunnelStatus),
    isOperating: readonly(isOperating),
    tunnelErrorMessage: readonly(tunnelErrorMessage),
    isTunnelActive,
    isTunnelConnected,
    publicTunnelUrl,
    fetchTunnelStatus,
    startTunnel,
    stopTunnel,
  }
}
