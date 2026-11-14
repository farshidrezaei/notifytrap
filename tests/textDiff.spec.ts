import { describe, expect, it, vi } from 'vitest'
import {
  buildNotificationComparableText,
  buildSideBySideRows,
  computeLineDiff,
  type DiffLine,
} from '../app/utils/textDiff'
import { copyTextToClipboard } from '../app/utils/clipboard'
import type { Notification } from '../app/types/application'

describe('textDiff utility', () => {
  describe('computeLineDiff', () => {
    it('returns empty array when both inputs are empty strings', () => {
      const result = computeLineDiff('', '')
      expect(result).toEqual([])
    })

    it('identifies identical lines as unchanged with line numbers', () => {
      const left = 'line1\nline2\nline3'
      const right = 'line1\nline2\nline3'
      const result = computeLineDiff(left, right)

      expect(result).toEqual<DiffLine[]>([
        { kind: 'unchanged', leftLineNumber: 1, rightLineNumber: 1, text: 'line1' },
        { kind: 'unchanged', leftLineNumber: 2, rightLineNumber: 2, text: 'line2' },
        { kind: 'unchanged', leftLineNumber: 3, rightLineNumber: 3, text: 'line3' },
      ])
    })

    it('identifies added lines on the right', () => {
      const left = 'alpha\ngamma'
      const right = 'alpha\nbeta\ngamma'
      const result = computeLineDiff(left, right)

      expect(result).toEqual<DiffLine[]>([
        { kind: 'unchanged', leftLineNumber: 1, rightLineNumber: 1, text: 'alpha' },
        { kind: 'added', leftLineNumber: null, rightLineNumber: 2, text: 'beta' },
        { kind: 'unchanged', leftLineNumber: 2, rightLineNumber: 3, text: 'gamma' },
      ])
    })

    it('identifies removed lines from the left', () => {
      const left = 'first\nsecond\nthird'
      const right = 'first\nthird'
      const result = computeLineDiff(left, right)

      expect(result).toEqual<DiffLine[]>([
        { kind: 'unchanged', leftLineNumber: 1, rightLineNumber: 1, text: 'first' },
        { kind: 'removed', leftLineNumber: 2, rightLineNumber: null, text: 'second' },
        { kind: 'unchanged', leftLineNumber: 3, rightLineNumber: 2, text: 'third' },
      ])
    })

    it('handles complete replacement of content', () => {
      const left = 'old1\nold2'
      const right = 'new1\nnew2'
      const result = computeLineDiff(left, right)

      expect(result).toHaveLength(4)
      expect(result.filter((line) => line.kind === 'removed')).toHaveLength(2)
      expect(result.filter((line) => line.kind === 'added')).toHaveLength(2)
    })

    it('switches safely to linear fallback diff when line count exceeds threshold', () => {
      const largeLeft = Array.from({ length: 450 }, (_, i) => `line ${i}`).join('\n')
      const largeRight = Array.from({ length: 450 }, (_, i) => (i === 10 ? 'modified' : `line ${i}`)).join('\n')

      const result = computeLineDiff(largeLeft, largeRight)
      expect(result.length).toBeGreaterThan(400)
      const modifiedLine = result.find((line) => line.text === 'modified')
      expect(modifiedLine).toBeDefined()
      expect(modifiedLine?.kind).toBe('added')
    })
  })

  describe('buildSideBySideRows', () => {
    it('returns empty array for empty diff input', () => {
      const rows = buildSideBySideRows([])
      expect(rows).toEqual([])
    })

    it('aligns unchanged lines across both left and right columns', () => {
      const diffLines: DiffLine[] = [
        { kind: 'unchanged', leftLineNumber: 1, rightLineNumber: 1, text: 'common' },
      ]
      const rows = buildSideBySideRows(diffLines)

      expect(rows).toEqual([
        {
          left: { lineNumber: 1, text: 'common', kind: 'unchanged' },
          right: { lineNumber: 1, text: 'common', kind: 'unchanged' },
        },
      ])
    })

    it('aligns removed and added lines side by side in pairs', () => {
      const diffLines: DiffLine[] = [
        { kind: 'removed', leftLineNumber: 1, rightLineNumber: null, text: 'old line' },
        { kind: 'added', leftLineNumber: null, rightLineNumber: 1, text: 'new line' },
      ]
      const rows = buildSideBySideRows(diffLines)

      expect(rows).toEqual([
        {
          left: { lineNumber: 1, text: 'old line', kind: 'removed' },
          right: { lineNumber: 1, text: 'new line', kind: 'added' },
        },
      ])
    })

    it('handles unbalanced removed and added segments cleanly', () => {
      const diffLines: DiffLine[] = [
        { kind: 'removed', leftLineNumber: 1, rightLineNumber: null, text: 'old 1' },
        { kind: 'removed', leftLineNumber: 2, rightLineNumber: null, text: 'old 2' },
        { kind: 'added', leftLineNumber: null, rightLineNumber: 1, text: 'new 1' },
      ]
      const rows = buildSideBySideRows(diffLines)

      expect(rows).toEqual([
        {
          left: { lineNumber: 1, text: 'old 1', kind: 'removed' },
          right: { lineNumber: 1, text: 'new 1', kind: 'added' },
        },
        {
          left: { lineNumber: 2, text: 'old 2', kind: 'removed' },
          right: null,
        },
      ])
    })
  })

  describe('buildNotificationComparableText', () => {
    const mockNotification: Notification = {
      id: 'notif-1',
      workspaceId: 'ws-1',
      projectId: 'proj-1',
      environmentId: 'env-1',
      channelId: 'chan-1',
      channelType: 'email',
      status: 'unread',
      recipient: 'user@example.com',
      sender: 'app@example.com',
      subject: 'Welcome to App',
      previewText: 'Hello',
      payloadJson: {
        subject: 'Welcome to App',
        htmlBody: '<p>Hello HTML</p>',
        data: { active: true },
      },
      rawPayload: 'RAW PAYLOAD STRING',
      testRunIdentifier: 'ci-123',
      receivedAt: '2026-11-15T00:00:00Z',
      createdAt: '2026-11-15T00:00:00Z',
      updatedAt: '2026-11-15T00:00:00Z',
      attachments: [],
      verificationArtifacts: {
        otpCodes: ['482910'],
        magicLinks: [],
      },
    }

    it('returns raw payload in rawPayload mode', () => {
      expect(buildNotificationComparableText(mockNotification, 'rawPayload')).toBe('RAW PAYLOAD STRING')
    })

    it('extracts htmlBody from payloadJson in htmlBody mode', () => {
      expect(buildNotificationComparableText(mockNotification, 'htmlBody')).toBe('<p>Hello HTML</p>')
    })

    it('extracts html key from payloadJson when htmlBody is absent', () => {
      const notificationWithHtmlKey: Notification = {
        ...mockNotification,
        payloadJson: { html: '<div>Alternative HTML</div>' },
      }
      expect(buildNotificationComparableText(notificationWithHtmlKey, 'htmlBody')).toBe('<div>Alternative HTML</div>')
    })

    it('falls back to rawPayload in htmlBody mode when no html strings are found', () => {
      const nonHtmlNotification: Notification = {
        ...mockNotification,
        payloadJson: { message: 'plain SMS' },
      }
      expect(buildNotificationComparableText(nonHtmlNotification, 'htmlBody')).toBe('RAW PAYLOAD STRING')
    })

    it('formats metadata JSON in metadata mode', () => {
      const text = buildNotificationComparableText(mockNotification, 'metadata')
      const parsed = JSON.parse(text)
      expect(parsed.channelType).toBe('email')
      expect(parsed.recipient).toBe('user@example.com')
      expect(parsed.testRunIdentifier).toBe('ci-123')
      expect(parsed.verificationArtifacts.otpCodes).toEqual(['482910'])
    })

    it('formats pretty payload JSON in prettyPayload mode', () => {
      const text = buildNotificationComparableText(mockNotification, 'prettyPayload')
      const parsed = JSON.parse(text)
      expect(parsed.data.active).toBe(true)
    })
  })
})

describe('clipboard utility', () => {
  it('returns false when navigator clipboard is unavailable', async () => {
    const originalClipboard = navigator.clipboard
    Object.defineProperty(navigator, 'clipboard', {
      value: undefined,
      configurable: true,
      writable: true,
    })

    const result = await copyTextToClipboard('sample')
    expect(result).toBe(false)

    Object.defineProperty(navigator, 'clipboard', {
      value: originalClipboard,
      configurable: true,
      writable: true,
    })
  })

  it('writes text to navigator clipboard and returns true when available', async () => {
    const writeTextMock = vi.fn().mockResolvedValue(undefined)
    Object.defineProperty(navigator, 'clipboard', {
      value: { writeText: writeTextMock },
      configurable: true,
      writable: true,
    })

    const result = await copyTextToClipboard('copy this value')
    expect(result).toBe(true)
    expect(writeTextMock).toHaveBeenCalledWith('copy this value')
  })

  it('returns false when clipboard writeText rejects', async () => {
    const writeTextMock = vi.fn().mockRejectedValue(new Error('Permission denied'))
    Object.defineProperty(navigator, 'clipboard', {
      value: { writeText: writeTextMock },
      configurable: true,
      writable: true,
    })

    const result = await copyTextToClipboard('failed value')
    expect(result).toBe(false)
  })
})
