export const AUTO_UPDATE_SETTING_KEY = 'app.auto_update_check_enabled'
export const UPDATE_CHECK_TIMEOUT_MILLISECONDS = 8000

export interface AvailableUpdateSummary {
  currentVersion: string
  version: string
  publishedAt?: string
  releaseNotes: string
}

/**
 * Computes a bounded 0-100 download progress percentage from streamed chunk totals.
 */
export function calculateDownloadProgressPercentage(
  downloadedBytes: number,
  totalContentLength: number | null | undefined,
): number | null {
  if (!totalContentLength || totalContentLength <= 0) {
    return null
  }
  const safeDownloaded = Math.max(0, downloadedBytes)
  const rawPercentage = Math.round((safeDownloaded / totalContentLength) * 100)
  return Math.min(100, Math.max(0, rawPercentage))
}

/**
 * Normalizes optional release notes text with a clean fallback when upstream metadata omits a body.
 */
export function normalizeUpdateReleaseNotes(rawBody: string | undefined): string {
  const trimmed = (rawBody ?? '').trim()
  return trimmed
}

/**
 * Formats an available update payload snapshot for UI display and toast notifications.
 */
export function buildAvailableUpdateSummary(metadata: {
  currentVersion: string
  version: string
  date?: string
  body?: string
}): AvailableUpdateSummary {
  return {
    currentVersion: metadata.currentVersion,
    version: metadata.version,
    publishedAt: metadata.date,
    releaseNotes: normalizeUpdateReleaseNotes(metadata.body),
  }
}
