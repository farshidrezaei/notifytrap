import type {
  ChannelChaosConfiguration,
  ChannelForwardingConfiguration,
  WebhookMockConfiguration,
} from '~/types/application'

export const DEFAULT_WEBHOOK_MOCK_CONFIGURATION: WebhookMockConfiguration = {
  isEnabled: false,
  statusCode: 200,
  headers: {},
  body: '{"ok":true,"mockedBy":"notifytrap"}',
  delayMilliseconds: 0,
}

export const DEFAULT_CHANNEL_CHAOS_CONFIGURATION: ChannelChaosConfiguration = {
  isEnabled: false,
  latencyMilliseconds: 0,
  failureRatePercent: 50,
  httpErrorStatusCode: 503,
  smtpErrorCode: 451,
  errorMessage: 'Simulated transient delivery failure',
}

export const DEFAULT_CHANNEL_FORWARDING_CONFIGURATION: ChannelForwardingConfiguration = {
  isEnabled: false,
  targetUrl: 'http://127.0.0.1:8000/webhook',
  customHeaders: {},
  includeOriginalHeaders: true,
  filterEventName: '',
  timeoutSeconds: 5,
}

/**
 * Parses a JSON object string into a flat header map for custom webhook mock responses.
 */
export function parseMockHeadersJson(rawJson: string): {
  headers: Record<string, string> | null
  isValid: boolean
} {
  const trimmed = rawJson.trim()
  if (trimmed.length === 0) {
    return { headers: {}, isValid: true }
  }

  try {
    const parsed: unknown = JSON.parse(trimmed)
    if (parsed === null || typeof parsed !== 'object' || Array.isArray(parsed)) {
      return { headers: null, isValid: false }
    }
    const headers: Record<string, string> = {}
    for (const [key, value] of Object.entries(parsed as Record<string, unknown>)) {
      const normalizedKey = key.trim()
      if (!normalizedKey || !/^[A-Za-z0-9-_]+$/.test(normalizedKey)) {
        return { headers: null, isValid: false }
      }
      headers[normalizedKey] = value === null || value === undefined ? '' : String(value)
    }
    return { headers, isValid: true }
  } catch {
    return { headers: null, isValid: false }
  }
}

/**
 * Validates that a webhook replay target URL strictly targets local loopback over HTTP.
 */
export function isValidLoopbackReplayUrl(rawUrl: string): boolean {
  const trimmed = rawUrl.trim()
  if (!trimmed.startsWith('http://') || trimmed.includes('#') || trimmed.includes('@')) {
    return false
  }
  try {
    const parsed = new URL(trimmed)
    if (parsed.protocol !== 'http:') {
      return false
    }
    const host = parsed.hostname.toLowerCase()
    return host === '127.0.0.1' || host === 'localhost' || host === '[::1]' || host === '::1'
  } catch {
    return false
  }
}

/**
 * Builds a convenient default loopback URL preserving the captured webhook subpath.
 */
export function buildDefaultLoopbackReplayUrl(subpath: string): string {
  const cleanSubpath = subpath.startsWith('/') ? subpath : `/${subpath}`
  return `http://127.0.0.1:3000${cleanSubpath}`
}
