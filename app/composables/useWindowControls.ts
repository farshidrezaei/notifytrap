import { getCurrentWindow } from '@tauri-apps/api/window'
import { isTauriRuntime } from '~/utils/tauriClient'

/**
 * Provides safe native window control actions for frameless Tauri 2 windows.
 */
export function useWindowControls() {
  const isMaximized = ref(false)
  const isWindowControlsAvailable = computed(() => isTauriRuntime())
  let unlistenResize: (() => void) | null = null

  async function syncMaximizedState(): Promise<void> {
    if (!isTauriRuntime()) {
      return
    }
    try {
      isMaximized.value = await getCurrentWindow().isMaximized()
    } catch {
      isMaximized.value = false
    }
  }

  async function minimizeWindow(): Promise<void> {
    if (!isTauriRuntime()) {
      return
    }
    await getCurrentWindow().minimize()
  }

  async function toggleMaximizeWindow(): Promise<void> {
    if (!isTauriRuntime()) {
      return
    }
    const appWindow = getCurrentWindow()
    await appWindow.toggleMaximize()
    await syncMaximizedState()
  }

  async function closeWindow(): Promise<void> {
    if (!isTauriRuntime()) {
      return
    }
    await getCurrentWindow().close()
  }

  onMounted(async () => {
    if (!isTauriRuntime()) {
      return
    }
    await syncMaximizedState()
    try {
      unlistenResize = await getCurrentWindow().onResized(() => {
        void syncMaximizedState()
      })
    } catch {
      unlistenResize = null
    }
  })

  onUnmounted(() => {
    if (unlistenResize) {
      unlistenResize()
      unlistenResize = null
    }
  })

  return {
    isMaximized: readonly(isMaximized),
    isWindowControlsAvailable,
    minimizeWindow,
    toggleMaximizeWindow,
    closeWindow,
  }
}
