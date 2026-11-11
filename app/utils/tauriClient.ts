import { invoke } from '@tauri-apps/api/core'
import { listen } from '@tauri-apps/api/event'
import type { CommandErrorPayload } from '~/types/application'

/**
 * Returns true when running inside a Tauri 2 native desktop WebView.
 */
export function isTauriRuntime(): boolean {
  if (typeof window === 'undefined') {
    return false
  }
  return '__TAURI_INTERNALS__' in window
}

/**
 * Invokes a typed Tauri backend command with consistent runtime checking.
 */
export async function invokeTauriCommand<TResult>(
  commandName: string,
  payload?: Record<string, unknown>,
): Promise<TResult> {
  if (!isTauriRuntime()) {
    throw new Error('TAURI_RUNTIME_UNAVAILABLE')
  }
  return invoke<TResult>(commandName, payload)
}

/**
 * Subscribes to a typed backend Tauri event and returns a safe cleanup callback.
 */
export async function subscribeTauriEvent<TPayload>(
  eventName: string,
  handler: (payload: TPayload) => void,
): Promise<() => void> {
  if (!isTauriRuntime()) {
    return () => {}
  }

  const unlisten = await listen<TPayload>(eventName, (event) => {
    handler(event.payload)
  })
  return unlisten
}

/**
 * Extracts a safe display message from an IPC CommandErrorPayload or standard Error.
 */
export function extractCommandErrorMessage(error: unknown, fallbackMessage: string): string {
  if (
    error !== null
    && typeof error === 'object'
    && 'message' in error
    && typeof (error as CommandErrorPayload).message === 'string'
  ) {
    return (error as CommandErrorPayload).message
  }
  if (error instanceof Error && error.message.length > 0) {
    return error.message
  }
  return fallbackMessage
}
