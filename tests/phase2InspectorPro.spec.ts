import { describe, expect, it } from 'vitest'
import type { Notification, NotificationAttachmentSummary } from '../app/types/application'
import {
  decodeBase64ToUtf8Text,
  isPreviewableTextMime,
  isSafePreviewableImageMime,
  resolveInlineCidImages,
} from '../app/utils/attachmentPreview'
import { auditCapturedEmail } from '../app/utils/emailAudit'
import { buildSandboxedEmailDocument } from '../app/utils/emailSandbox'
import { analyzeSmsMessage } from '../app/utils/smsAnalyzer'
import { buildNotificationComparableText, computeLineDiff } from '../app/utils/textDiff'

function createSampleEmail(overrides: Partial<Notification> = {}): Notification {
  return {
    id: 'email-1',
    workspaceId: 'w-1',
    projectId: 'p-1',
    environmentId: 'e-1',
    channelId: 'c-1',
    channelType: 'email',
    status: 'captured',
    recipient: 'user@example.test',
    sender: 'noreply@example.test',
    subject: 'Welcome',
    previewText: 'Hello',
    payloadJson: {
      textBody: 'Hello plain',
      rawHtml: '<p>Hello</p><img src="cid:logo@test" alt="Logo"><a href="https://example.test">Open</a>',
      headers: [{ name: 'List-Unsubscribe', value: '<mailto:unsub@example.test>' }],
    },
    rawPayload: 'Subject: Welcome\r\n\r\nHello',
    testRunIdentifier: '',
    verificationArtifacts: [],
    receivedAt: '2026-11-12T00:00:00Z',
    createdAt: '2026-11-12T00:00:00Z',
    ...overrides,
  }
}

describe('Phase 2 Rich Inspector utilities', () => {
  it('calculates GSM-7 and UCS-2 SMS encoding and segment boundaries accurately', () => {
    expect(analyzeSmsMessage('')).toMatchObject({
      encoding: 'GSM-7',
      segmentCount: 0,
      charactersRemainingInSegment: 160,
    })

    const gsmSingle = analyzeSmsMessage('A'.repeat(160))
    expect(gsmSingle).toMatchObject({
      encoding: 'GSM-7',
      unitCount: 160,
      segmentCount: 1,
      charactersRemainingInSegment: 0,
    })

    const gsmWithExtension = analyzeSmsMessage(`${'A'.repeat(159)}€`)
    expect(gsmWithExtension).toMatchObject({
      encoding: 'GSM-7',
      characterCount: 160,
      unitCount: 161,
      segmentCount: 2,
    })

    const persianSingle = analyzeSmsMessage('کد تایید شما: ۱۲۳۴۵۶')
    expect(persianSingle.encoding).toBe('UCS-2')
    expect(persianSingle.segmentCount).toBe(1)

    const persianMulti = analyzeSmsMessage('ا'.repeat(71))
    expect(persianMulti).toMatchObject({
      encoding: 'UCS-2',
      unitCount: 71,
      segmentCount: 2,
      charactersRemainingInSegment: 63,
    })
  })

  it('audits captured email HTML, headers, alt text, links, and inline cid references', () => {
    const attachments: NotificationAttachmentSummary[] = [
      {
        id: 'att-1',
        notificationId: 'email-1',
        filename: 'logo.png',
        contentType: 'image/png',
        sizeBytes: 1200,
        contentId: '<logo@test>',
        isInline: true,
        createdAt: '2026-11-12T00:00:00Z',
      },
    ]

    const cleanReport = auditCapturedEmail(createSampleEmail(), attachments)
    expect(cleanReport.missingAltCount).toBe(0)
    expect(cleanReport.remoteImageCount).toBe(0)
    expect(cleanReport.relativeLinkCount).toBe(0)
    expect(cleanReport.findings.every((f) => f.severity === 'pass')).toBe(true)

    const messyReport = auditCapturedEmail(
      createSampleEmail({
        subject: '(No Subject)',
        payloadJson: {
          textBody: '',
          rawHtml: '<img src="https://cdn.test/a.png"><img src="cid:missing"><a href="#">Click</a>',
          headers: [],
        },
      }),
      [],
    )
    expect(messyReport.missingAltCount).toBe(2)
    expect(messyReport.remoteImageCount).toBe(1)
    expect(messyReport.relativeLinkCount).toBe(1)
    expect(messyReport.findings.some((f) => f.id === 'unresolvedCid' && f.severity === 'warning')).toBe(true)
    expect(messyReport.findings.some((f) => f.id === 'plainTextFallback' && f.severity === 'warning')).toBe(true)
  })

  it('computes line diffs and notification comparison views deterministically', () => {
    const diff = computeLineDiff('alpha\nbeta\ngamma', 'alpha\ndelta\ngamma\nepsilon')
    expect(diff.map((d) => `${d.kind}:${d.text}`)).toEqual([
      'unchanged:alpha',
      'removed:beta',
      'added:delta',
      'unchanged:gamma',
      'added:epsilon',
    ])

    const email = createSampleEmail()
    expect(buildNotificationComparableText(email, 'rawPayload')).toBe(email.rawPayload)
    expect(buildNotificationComparableText(email, 'metadata')).toContain('"subject": "Welcome"')
    expect(buildNotificationComparableText(email, 'prettyPayload')).toContain('"textBody": "Hello plain"')
  })

  it('validates safe attachment preview MIMEs, resolves inline cid images, and builds dark simulator HTML', () => {
    expect(isSafePreviewableImageMime('image/png')).toBe(true)
    expect(isSafePreviewableImageMime('image/jpeg; charset=binary')).toBe(true)
    expect(isSafePreviewableImageMime('image/svg+xml')).toBe(false)
    expect(isPreviewableTextMime('application/json; charset=utf-8')).toBe(true)
    expect(decodeBase64ToUtf8Text(btoa('hello'))).toBe('hello')

    const cidMap = new Map([['logo@test', 'data:image/png;base64,AAAA']])
    const resolved = resolveInlineCidImages('<img src="cid:<logo@test>" alt="L">', cidMap)
    expect(resolved).toContain('src="data:image/png;base64,AAAA"')

    const simulatedDark = buildSandboxedEmailDocument('<p>Hi</p>', {
      isDarkTheme: false,
      simulateEmailDarkMode: true,
    })
    expect(simulatedDark).toContain('color-scheme: dark;')
    expect(simulatedDark).toContain("img-src data:;")
  })
})
