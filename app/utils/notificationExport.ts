import type { NotificationDetail } from '~/types/application'

const PLACEHOLDER_CHANNEL_KEY = '<CHANNEL_KEY>'

/**
 * Serializes a captured notification detail into pretty-printed redacted JSON.
 */
export function buildNotificationJsonExport(detail: NotificationDetail): string {
  return JSON.stringify(detail, null, 2)
}

/**
 * Produces an RFC-5322 `.eml` document from a captured email or synthesizes one from metadata.
 */
export function buildNotificationEmlExport(detail: NotificationDetail): string {
  const { notification } = detail
  const trimmedRaw = notification.rawPayload.trim()
  if (
    notification.channelType === 'email'
    && /^(From|To|Subject|Date|MIME-Version|Received):/im.test(trimmedRaw)
  ) {
    return `${trimmedRaw.replace(/\r?\n/g, '\r\n')}\r\n`
  }

  const textBody = typeof notification.payloadJson.textBody === 'string'
    ? notification.payloadJson.textBody
    : notification.previewText || notification.rawPayload
  const safeSubject = notification.subject.replace(/[\r\n]+/g, ' ').trim()
  const safeSender = notification.sender.replace(/[\r\n]+/g, ' ').trim() || 'noreply@notifytrap.local'
  const safeRecipient = notification.recipient.replace(/[\r\n]+/g, ' ').trim() || 'inbox@notifytrap.local'
  const rfcDate = new Date(notification.receivedAt).toUTCString()

  return [
    `From: ${safeSender}`,
    `To: ${safeRecipient}`,
    `Subject: ${safeSubject}`,
    `Date: ${rfcDate}`,
    'MIME-Version: 1.0',
    'Content-Type: text/plain; charset=utf-8',
    '',
    textBody,
    '',
  ].join('\r\n')
}

/**
 * Builds a reproducible local `curl` command for HTTP-captured channels (`sms`, `push`, `webhook`).
 */
export function buildNotificationCurlExport(
  detail: NotificationDetail,
  httpEndpoint: string,
): string | null {
  const { notification } = detail
  if (notification.channelType === 'email') {
    return null
  }

  const baseUrl = normalizeHttpBaseUrl(httpEndpoint)
  if (notification.channelType === 'sms') {
    const body = JSON.stringify({
      to: notification.payloadJson.to ?? notification.recipient,
      from: notification.payloadJson.from ?? notification.sender,
      message: notification.payloadJson.message ?? notification.previewText,
      metadata: notification.payloadJson.metadata ?? {},
    })
    return formatCurlCommand('POST', `${baseUrl}/api/v1/sms`, {
      'content-type': 'application/json',
      'x-channel-key': PLACEHOLDER_CHANNEL_KEY,
    }, body)
  }

  if (notification.channelType === 'push') {
    const body = JSON.stringify({
      device_token: notification.payloadJson.deviceToken ?? notification.recipient,
      platform: notification.payloadJson.platform ?? notification.sender,
      title: notification.payloadJson.title ?? notification.subject,
      body: notification.payloadJson.body ?? notification.previewText,
      data: notification.payloadJson.data ?? {},
    })
    return formatCurlCommand('POST', `${baseUrl}/api/v1/push`, {
      'content-type': 'application/json',
      'x-channel-key': PLACEHOLDER_CHANNEL_KEY,
    }, body)
  }

  const method = typeof notification.payloadJson.method === 'string'
    ? notification.payloadJson.method.toUpperCase()
    : 'POST'
  const path = typeof notification.payloadJson.path === 'string'
    ? notification.payloadJson.path
    : '/api/v1/webhooks/<CREDENTIAL_IDENTIFIER>'
  const headers = normalizeExportHeaders(notification.payloadJson.headers)
  headers['x-channel-key'] = PLACEHOLDER_CHANNEL_KEY

  return formatCurlCommand(method, `${baseUrl}${path}`, headers, notification.rawPayload)
}

/**
 * Triggers a browser/WebView file download for exported notification artifacts.
 */
export function triggerTextFileDownload(
  filename: string,
  content: string,
  mimeType: string,
): void {
  if (typeof document === 'undefined' || typeof URL === 'undefined') {
    return
  }
  const blob = new Blob([content], { type: mimeType })
  const objectUrl = URL.createObjectURL(blob)
  const anchor = document.createElement('a')
  anchor.href = objectUrl
  anchor.download = filename
  anchor.style.display = 'none'
  document.body.appendChild(anchor)
  anchor.click()
  document.body.removeChild(anchor)
  URL.revokeObjectURL(objectUrl)
}

function normalizeHttpBaseUrl(httpEndpoint: string): string {
  const trimmed = httpEndpoint.trim()
  if (!trimmed) {
    return 'http://127.0.0.1:8080'
  }
  if (trimmed.startsWith('http://') || trimmed.startsWith('https://')) {
    return trimmed.replace(/\/+$/, '')
  }
  return `http://${trimmed.replace(/\/+$/, '')}`
}

function normalizeExportHeaders(rawHeaders: unknown): Record<string, string> {
  if (!rawHeaders || typeof rawHeaders !== 'object' || Array.isArray(rawHeaders)) {
    return {}
  }
  const result: Record<string, string> = {}
  for (const [key, value] of Object.entries(rawHeaders)) {
    const lowerKey = key.toLowerCase()
    if (lowerKey === 'host' || lowerKey === 'content-length') {
      continue
    }
    if (typeof value === 'string' && value !== '[REDACTED]') {
      result[lowerKey] = value
    }
  }
  return result
}

function formatCurlCommand(
  method: string,
  url: string,
  headers: Record<string, string>,
  body: string,
): string {
  const parts: string[] = [`curl -X ${escapeShellArgument(method)} ${escapeShellArgument(url)}`]
  for (const [key, value] of Object.entries(headers)) {
    parts.push(`  -H ${escapeShellArgument(`${key}: ${value}`)}`)
  }
  if (body.length > 0 && method !== 'GET' && method !== 'HEAD') {
    parts.push(`  --data-raw ${escapeShellArgument(body)}`)
  }
  return parts.join(' \\\n')
}

function escapeShellArgument(value: string): string {
  return `'${value.replace(/'/g, `'\\''`)}'`
}
