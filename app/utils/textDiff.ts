import type { Notification } from '~/types/application'

export type DiffLineKind = 'unchanged' | 'added' | 'removed'

export interface DiffLine {
  kind: DiffLineKind
  leftLineNumber: number | null
  rightLineNumber: number | null
  text: string
}

export type NotificationCompareMode = 'prettyPayload' | 'rawPayload' | 'metadata'

const MAXIMUM_LCS_LINES = 400

/**
 * Computes a deterministic line-by-line diff using LCS (falling back to linear diff on very large payloads).
 */
export function computeLineDiff(leftText: string, rightText: string): DiffLine[] {
  const leftLines = leftText.length === 0 ? [] : leftText.split(/\r?\n/)
  const rightLines = rightText.length === 0 ? [] : rightText.split(/\r?\n/)

  if (leftLines.length > MAXIMUM_LCS_LINES || rightLines.length > MAXIMUM_LCS_LINES) {
    return computeLinearFallbackDiff(leftLines, rightLines)
  }

  const rows = leftLines.length
  const cols = rightLines.length
  const table: Uint16Array[] = Array.from({ length: rows + 1 }, () => new Uint16Array(cols + 1))

  for (let i = rows - 1; i >= 0; i -= 1) {
    const currentRow = table[i]!
    const nextRow = table[i + 1]!
    for (let j = cols - 1; j >= 0; j -= 1) {
      if (leftLines[i] === rightLines[j]) {
        currentRow[j] = (nextRow[j + 1] ?? 0) + 1
      } else {
        currentRow[j] = Math.max(nextRow[j] ?? 0, currentRow[j + 1] ?? 0)
      }
    }
  }

  const diffLines: DiffLine[] = []
  let i = 0
  let j = 0
  while (i < rows && j < cols) {
    if (leftLines[i] === rightLines[j]) {
      diffLines.push({
        kind: 'unchanged',
        leftLineNumber: i + 1,
        rightLineNumber: j + 1,
        text: leftLines[i]!,
      })
      i += 1
      j += 1
    } else if ((table[i + 1]?.[j] ?? 0) >= (table[i]?.[j + 1] ?? 0)) {
      diffLines.push({
        kind: 'removed',
        leftLineNumber: i + 1,
        rightLineNumber: null,
        text: leftLines[i]!,
      })
      i += 1
    } else {
      diffLines.push({
        kind: 'added',
        leftLineNumber: null,
        rightLineNumber: j + 1,
        text: rightLines[j]!,
      })
      j += 1
    }
  }

  while (i < rows) {
    diffLines.push({
      kind: 'removed',
      leftLineNumber: i + 1,
      rightLineNumber: null,
      text: leftLines[i]!,
    })
    i += 1
  }

  while (j < cols) {
    diffLines.push({
      kind: 'added',
      leftLineNumber: null,
      rightLineNumber: j + 1,
      text: rightLines[j]!,
    })
    j += 1
  }

  return diffLines
}

/**
 * Formats a notification for side-by-side comparison across payload, raw source, or metadata.
 */
export function buildNotificationComparableText(
  notification: Notification,
  mode: NotificationCompareMode,
): string {
  if (mode === 'rawPayload') {
    return notification.rawPayload
  }
  if (mode === 'metadata') {
    return JSON.stringify(
      {
        channelType: notification.channelType,
        status: notification.status,
        sender: notification.sender,
        recipient: notification.recipient,
        subject: notification.subject,
        testRunIdentifier: notification.testRunIdentifier,
        verificationArtifacts: notification.verificationArtifacts,
      },
      null,
      2,
    )
  }
  return JSON.stringify(notification.payloadJson, null, 2)
}

function computeLinearFallbackDiff(leftLines: string[], rightLines: string[]): DiffLine[] {
  const result: DiffLine[] = []
  const maxLen = Math.max(leftLines.length, rightLines.length)
  for (let index = 0; index < maxLen; index += 1) {
    const left = leftLines[index]
    const right = rightLines[index]
    if (left !== undefined && right !== undefined && left === right) {
      result.push({
        kind: 'unchanged',
        leftLineNumber: index + 1,
        rightLineNumber: index + 1,
        text: left,
      })
    } else {
      if (left !== undefined) {
        result.push({
          kind: 'removed',
          leftLineNumber: index + 1,
          rightLineNumber: null,
          text: left,
        })
      }
      if (right !== undefined) {
        result.push({
          kind: 'added',
          leftLineNumber: null,
          rightLineNumber: index + 1,
          text: right,
        })
      }
    }
  }
  return result
}
