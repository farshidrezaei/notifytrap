import type { Notification, NotificationAttachmentSummary } from '~/types/application'

export type AuditSeverity = 'pass' | 'warning' | 'info'

export interface EmailAuditFinding {
  id: string
  severity: AuditSeverity
  translationKey: string
  params?: Record<string, string | number>
}

export interface EmailAuditReport {
  findings: EmailAuditFinding[]
  rawBytes: number
  imageCount: number
  linkCount: number
  missingAltCount: number
  remoteImageCount: number
  relativeLinkCount: number
}

/**
 * Performs deterministic offline quality, accessibility, and delivery diagnostics on a captured email.
 */
export function auditCapturedEmail(
  notification: Notification,
  attachments: readonly NotificationAttachmentSummary[],
): EmailAuditReport {
  const payload = notification.payloadJson
  const rawHtml = typeof payload.rawHtml === 'string' ? payload.rawHtml : ''
  const textBody = typeof payload.textBody === 'string' ? payload.textBody : ''
  const headers = normalizeHeaderNames(payload.headers)
  const findings: EmailAuditFinding[] = []

  const hasSubject = notification.subject.trim().length > 0 && notification.subject.trim() !== '(No Subject)'
  findings.push({
    id: 'subject',
    severity: hasSubject ? 'pass' : 'warning',
    translationKey: hasSubject
      ? 'inbox.email.audit.findings.subjectPresent'
      : 'inbox.email.audit.findings.subjectMissing',
  })

  if (rawHtml.trim().length > 0) {
    const hasPlainText = textBody.trim().length > 0
    findings.push({
      id: 'plainTextFallback',
      severity: hasPlainText ? 'pass' : 'warning',
      translationKey: hasPlainText
        ? 'inbox.email.audit.findings.plainTextPresent'
        : 'inbox.email.audit.findings.plainTextMissing',
    })
  }

  const imageTags = rawHtml.match(/<img\b[^>]*>/gi) ?? []
  let missingAltCount = 0
  let remoteImageCount = 0
  let unresolvedCidCount = 0
  const availableContentIds = new Set(
    attachments
      .map((item) => item.contentId?.replace(/^<|>$/g, '').trim().toLowerCase())
      .filter((value): value is string => Boolean(value)),
  )

  for (const tag of imageTags) {
    if (!/\balt\s*=\s*(['"]).*?\1/i.test(tag)) {
      missingAltCount += 1
    }
    const srcMatch = /\bsrc\s*=\s*(['"])(.*?)\1/i.exec(tag)
    const srcValue = srcMatch?.[2]?.trim() ?? ''
    const lowerSrc = srcValue.toLowerCase()
    if (lowerSrc.startsWith('http://') || lowerSrc.startsWith('https://')) {
      remoteImageCount += 1
    } else if (lowerSrc.startsWith('cid:')) {
      const cidValue = lowerSrc.slice(4).replace(/^<|>$/g, '').trim()
      if (!availableContentIds.has(cidValue)) {
        unresolvedCidCount += 1
      }
    }
  }

  if (imageTags.length > 0) {
    findings.push({
      id: 'imageAlt',
      severity: missingAltCount === 0 ? 'pass' : 'warning',
      translationKey: missingAltCount === 0
        ? 'inbox.email.audit.findings.allImagesHaveAlt'
        : 'inbox.email.audit.findings.imagesMissingAlt',
      params: { count: missingAltCount },
    })
  }

  if (remoteImageCount > 0) {
    findings.push({
      id: 'remoteImages',
      severity: 'info',
      translationKey: 'inbox.email.audit.findings.remoteImagesBlocked',
      params: { count: remoteImageCount },
    })
  }

  if (unresolvedCidCount > 0) {
    findings.push({
      id: 'unresolvedCid',
      severity: 'warning',
      translationKey: 'inbox.email.audit.findings.unresolvedCidImages',
      params: { count: unresolvedCidCount },
    })
  }

  const anchorTags = rawHtml.match(/<a\b[^>]*>/gi) ?? []
  let relativeLinkCount = 0
  for (const tag of anchorTags) {
    const hrefMatch = /\bhref\s*=\s*(['"])(.*?)\1/i.exec(tag)
    const hrefValue = hrefMatch?.[2]?.trim() ?? ''
    if (
      hrefValue.length === 0
      || hrefValue === '#'
      || (!/^(https?:|mailto:|tel:)/i.test(hrefValue))
    ) {
      relativeLinkCount += 1
    }
  }

  if (anchorTags.length > 0) {
    findings.push({
      id: 'links',
      severity: relativeLinkCount === 0 ? 'pass' : 'warning',
      translationKey: relativeLinkCount === 0
        ? 'inbox.email.audit.findings.linksValid'
        : 'inbox.email.audit.findings.linksRelativeOrEmpty',
      params: { count: relativeLinkCount },
    })
  }

  const hasListUnsubscribe = headers.has('list-unsubscribe')
  findings.push({
    id: 'listUnsubscribe',
    severity: hasListUnsubscribe ? 'pass' : 'info',
    translationKey: hasListUnsubscribe
      ? 'inbox.email.audit.findings.listUnsubscribePresent'
      : 'inbox.email.audit.findings.listUnsubscribeMissing',
  })

  const rawBytes = new TextEncoder().encode(notification.rawPayload).byteLength
  const isLargeEmail = rawBytes > 102_400
  findings.push({
    id: 'payloadSize',
    severity: isLargeEmail ? 'warning' : 'pass',
    translationKey: isLargeEmail
      ? 'inbox.email.audit.findings.largeHtmlSize'
      : 'inbox.email.audit.findings.compactSize',
    params: { kilobytes: Math.max(1, Math.round(rawBytes / 1024)) },
  })

  return {
    findings,
    rawBytes,
    imageCount: imageTags.length,
    linkCount: anchorTags.length,
    missingAltCount,
    remoteImageCount,
    relativeLinkCount,
  }
}

function normalizeHeaderNames(rawHeaders: unknown): Set<string> {
  const names = new Set<string>()
  if (Array.isArray(rawHeaders)) {
    for (const entry of rawHeaders) {
      if (entry && typeof entry === 'object' && 'name' in entry) {
        names.add(String((entry as { name?: unknown }).name ?? '').trim().toLowerCase())
      }
    }
  } else if (rawHeaders && typeof rawHeaders === 'object') {
    for (const key of Object.keys(rawHeaders)) {
      names.add(key.trim().toLowerCase())
    }
  }
  return names
}
