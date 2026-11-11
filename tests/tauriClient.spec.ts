import { afterEach, describe, expect, it } from 'vitest'
import { invokeTauriCommand, isTauriRuntime } from '../app/utils/tauriClient'

describe('tauriClient runtime guard', () => {
  afterEach(() => {
    delete (window as unknown as Record<string, unknown>).__TAURI_INTERNALS__
  })

  it('returns false outside Tauri desktop webview', () => {
    expect(isTauriRuntime()).toBe(false)
  })

  it('detects __TAURI_INTERNALS__ when running inside Tauri', () => {
    ;(window as unknown as Record<string, unknown>).__TAURI_INTERNALS__ = {}
    expect(isTauriRuntime()).toBe(true)
  })

  it('rejects command invocation cleanly when Tauri IPC is unavailable', async () => {
    await expect(invokeTauriCommand('get_application_health')).rejects.toThrow(
      'TAURI_RUNTIME_UNAVAILABLE',
    )
  })
})
