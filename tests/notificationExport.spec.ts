import { describe, expect, it } from 'vitest'
import type { NotificationDetail } from '../app/types/application'
import {
  buildNotificationCurlExport,
  buildNotificationEmlExport,
  buildNotificationJsonExport,
} from '../app/utils/notificationExport'

function createSampleDetail(overrides: Partial<NotificationDetail['notification']> = {}): NotificationDetail {
  return {
    notification: {
      id: '11111111-1111-4111-8111-111111111111',
      workspaceId: '22222222-2222-4222-8222-222222222222',
      projectId: '33333333-3333-4333-8333-333333333333',
      environmentId: '44444444-4444-4444-8444-444444444444',
      channelId: '55555555-5555-4555-8555-555555555555',
      channelType: 'email',
      status: 'captured',
      recipient: 'qa@notifytrap.local',
      sender: 'auth@notifytrap.local',
      subject: 'Your verification code',
      previewText: 'Code 482910',
      payloadJson: {
        textBody: 'Your code is 482910',
      },
      rawPayload: 'From: auth@notifytrap.local\nTo: qa@notifytrap.local\nSubject: Your verification code\n\nYour code is 482910',
      testRunIdentifier: 'ci-run-42',
      verificationArtifacts: [
        { kind: 'otpCode', value: '482910', label: 'OTP' },
      ],
      receivedAt: '2026-11-12T00:00:00Z',
      createdAt: '2026-11-12T00:00:00Z',
      ...overrides,
    },
    attachments: [],
  }
}

describe('notification export formatters', () => {
  it('exports raw RFC-5322 email payload with CRLF endings and synthesizes fallback headers when needed', () => {
    const rawEmail = buildNotificationEmlExport(createSampleDetail())
    expect(rawEmail).toContain('From: auth@notifytrap.local\r\n')
    expect(rawEmail).toContain('Your code is 482910\r\n')

    const smsAsEml = buildNotificationEmlExport(createSampleDetail({
      channelType: 'sms',
      rawPayload: '{"message":"Hello"}',
      payloadJson: { textBody: 'Hello SMS' },
    }))
    expect(smsAsEml).toContain('MIME-Version: 1.0\r\n')
    expect(smsAsEml).toContain('Hello SMS\r\n')
  })

  it('exports pretty JSON preserving testRunIdentifier and verificationArtifacts', () => {
    const parsed = JSON.parse(buildNotificationJsonExport(createSampleDetail()))
    expect(parsed.notification.testRunIdentifier).toBe('ci-run-42')
    expect(parsed.notification.verificationArtifacts[0].value).toBe('482910')
  })

  it('builds safe cURL commands for HTTP channels without leaking redacted secrets', () => {
    expect(buildNotificationCurlExport(createSampleDetail(), '127.0.0.1:8080')).toBeNull()

    const webhookCurl = buildNotificationCurlExport(
      createSampleDetail({
        channelType: 'webhook',
        rawPayload: '{"event":"user.verified","note":"it\'s ok"}',
        payloadJson: {
          method: 'POST',
          path: '/api/v1/webhooks/wh-demo/orders',
          headers: {
            'content-type': 'application/json',
            'authorization': '[REDACTED]',
            'x-api-key': '[REDACTED]',
            'x-trace-id': 'trace-99',
          },
        },
      }),
      '127.0.0.1:8080',
    )

    expect(webhookCurl).toContain("curl -X 'POST' 'http://127.0.0.1:8080/api/v1/webhooks/wh-demo/orders'")
    expect(webhookCurl).toContain("'x-channel-key: <CHANNEL_KEY>'")
    expect(webhookCurl).toContain("'x-trace-id: trace-99'")
    expect(webhookCurl).not.toContain('[REDACTED]')
    expect(webhookCurl).toContain("--data-raw '{\"event\":\"user.verified\",\"note\":\"it'\\''s ok\"}'")

    const smsCurl = buildNotificationCurlExport(
      createSampleDetail({
        channelType: 'sms',
        payloadJson: { to: '+989120001122', from: 'NotifyTrap', message: 'Code 1234' },
      }),
      'http://127.0.0.1:9090',
    )
    expect(smsCurl).toContain("'http://127.0.0.1:9090/api/v1/sms'")
    expect(smsCurl).toContain("'x-channel-key: <CHANNEL_KEY>'")
  })
})
