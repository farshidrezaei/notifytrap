import { describe, expect, it } from 'vitest'
import type { Notification, NotificationAttachmentSummary } from '../app/types/application'
import { auditCapturedEmail } from '../app/utils/emailAudit'
import { inspectMjmlContent, isMjmlMarkup, transpileMjmlToResponsiveHtml } from '../app/utils/mjmlInspector'
import { scanForUnrenderedPlaceholders } from '../app/utils/templateLinter'

describe('Phase 1: Template Linter & Unrendered Placeholders', () => {
  it('detects diverse template placeholders and runtime sentinels', () => {
    const testCases = [
      {
        subject: 'Welcome {{ user.firstName }}!',
        text: 'Your code is ${verification_code}',
        html: '<p>Hello world</p>',
        expectedPlaceholders: ['{{ user.firstName }}', '${verification_code}'],
        expectedHasAny: true,
      },
      {
        subject: 'Password reset',
        text: 'Dear user, your email is undefined. Please verify.',
        html: '<div>Status: null</div>',
        expectedPlaceholders: ['undefined', 'null'],
        expectedHasAny: true,
      },
      {
        subject: 'Clean Subject',
        text: 'Clean plain text message.',
        html: '<p class="null-item">Clean HTML content with regular words.</p>',
        expectedPlaceholders: [],
        expectedHasAny: false,
      },
      {
        subject: 'Python or ERB format',
        text: 'Your balance is {user_balance}. Link: <%= user.activationUrl %>',
        html: '',
        expectedPlaceholders: ['{user_balance}', '<%= user.activationUrl %>'],
        expectedHasAny: true,
      },
    ]

    for (const testCase of testCases) {
      const result = scanForUnrenderedPlaceholders(testCase.subject, testCase.text, testCase.html)
      expect(result.hasUnrenderedPlaceholders).toBe(testCase.expectedHasAny)
      for (const placeholder of testCase.expectedPlaceholders) {
        expect(result.uniquePlaceholders).toContain(placeholder)
      }
    }
  })
})

describe('Phase 1: MJML Inspector & Semantic Transpiler', () => {
  it('identifies MJML markup accurately', () => {
    expect(isMjmlMarkup('<mjml><mj-body></mj-body></mjml>')).toBe(true)
    expect(isMjmlMarkup('<mj-section><mj-column></mj-column></mj-section>')).toBe(true)
    expect(isMjmlMarkup('<html><body>Regular HTML</body></html>')).toBe(false)
  })

  it('inspects MJML tags and calculates structural summary', () => {
    const sampleMjml = `
      <mjml>
        <mj-body>
          <mj-section background-color="#ffffff">
            <mj-column width="100%">
              <mj-text font-size="16px">Welcome to NotifyTrap</mj-text>
              <mj-button href="https://example.com">Confirm Email</mj-button>
            </mj-column>
          </mj-section>
        </mj-body>
      </mjml>
    `

    const inspection = inspectMjmlContent(sampleMjml)
    expect(inspection.isMjml).toBe(true)
    expect(inspection.hasRootMjmlTag).toBe(true)
    expect(inspection.hasBodyTag).toBe(true)
    expect(inspection.totalTagCount).toBeGreaterThanOrEqual(5)

    const tagNames = inspection.tagSummaries.map((item) => item.name)
    expect(tagNames).toContain('mjml')
    expect(tagNames).toContain('mj-body')
    expect(tagNames).toContain('mj-section')
    expect(tagNames).toContain('mj-column')
    expect(tagNames).toContain('mj-text')
    expect(tagNames).toContain('mj-button')
  })

  it('transpiles MJML elements into responsive email HTML with table layout', () => {
    const sampleMjml = `
      <mjml>
        <mj-body background-color="#f4f4f4">
          <mj-section background-color="#ffffff">
            <mj-column>
              <mj-text color="#111827">Hello World</mj-text>
              <mj-button href="https://notifytrap.dev" background-color="#2563eb">Activate</mj-button>
              <mj-divider border-color="#e5e7eb" />
            </mj-column>
          </mj-section>
        </mj-body>
      </mjml>
    `

    const html = transpileMjmlToResponsiveHtml(sampleMjml)
    expect(html).toContain('<!DOCTYPE html>')
    expect(html).toContain('table')
    expect(html).toContain('Hello World')
    expect(html).toContain('https://notifytrap.dev')
    expect(html).toContain('Activate')
    expect(html).not.toContain('<mj-section')
  })
})

describe('Phase 1: Email Audit Integration with Template Linter', () => {
  it('flags unrendered placeholders in email audit report', () => {
    const mockNotification: Notification = {
      id: 'notification-1',
      workspaceId: 'workspace-1',
      projectId: 'project-1',
      environmentId: 'env-1',
      channelId: 'channel-1',
      channelType: 'email',
      direction: 'inbound',
      sender: 'system@example.com',
      recipient: 'user@example.com',
      subject: 'Hello {{ user.name }}',
      previewText: 'Your code is undefined',
      payloadJson: {
        rawHtml: '<p>Welcome {{ user.name }}, your token is null</p>',
        textBody: 'Welcome {{ user.name }}',
        headers: [],
      },
      rawPayload: 'Subject: Hello {{ user.name }}\r\n\r\nBody with {{ user.name }}',
      status: 'captured',
      receivedAt: '2026-11-12T12:00:00Z',
      createdAt: '2026-11-12T12:00:00Z',
      updatedAt: '2026-11-12T12:00:00Z',
    }

    const emptyAttachments: NotificationAttachmentSummary[] = []
    const report = auditCapturedEmail(mockNotification, emptyAttachments)

    expect(report.unrenderedPlaceholderCount).toBeGreaterThan(0)
    const placeholderFinding = report.findings.find((item) => item.id === 'unrenderedPlaceholders')
    expect(placeholderFinding).toBeDefined()
    expect(placeholderFinding?.severity).toBe('warning')
    expect(placeholderFinding?.translationKey).toBe('inbox.email.audit.findings.unrenderedPlaceholdersFound')
  })
})
