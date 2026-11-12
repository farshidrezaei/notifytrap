/**
 * Deterministic scanner for detecting unrendered template variables and runtime
 * sentinel values leaked into captured notifications.
 */

const MUSTACHE_REGEX = /\{\{\s*[\w$.[\]-]+\s*\}\}/g
const RAW_MUSTACHE_REGEX = /\{!!\s*[\w$.[\]-]+\s*!!\}/g
const TEMPLATE_LITERAL_REGEX = /\$\{\s*[\w$.[\]-]+\s*\}/g
const ERB_OR_DJANGO_REGEX = /<[%#]=\s*[\w$.[\]-]+\s*%>|\{%\s*[\w$.[\]-]+\s*%\}/g
const PYTHON_FORMAT_REGEX = /(?<=[^a-zA-Z0-9_{])\{[a-zA-Z_][a-zA-Z0-9_]*\}(?=[^a-zA-Z0-9_}])/g

/**
 * Strips HTML tags and style/script blocks so sentinels inside markup attributes
 * (like class="null-state") are not falsely flagged.
 */
function extractVisibleText(htmlOrText: string): string {
  return htmlOrText
    .replace(/<style\b[^>]*>[\s\S]*?<\/style>/gi, '')
    .replace(/<script\b[^>]*>[\s\S]*?<\/script>/gi, '')
    .replace(/<[^>]+>/g, ' ')
    .replace(/&[a-zA-Z0-9#]+;/g, ' ')
}

export interface TemplateLinterResult {
  hasUnrenderedPlaceholders: boolean
  totalCount: number
  uniquePlaceholders: string[]
  sampleDisplay: string
}

/**
 * Scans text, subject, and HTML for unrendered template placeholders.
 */
export function scanForUnrenderedPlaceholders(
  subject: string,
  textBody: string,
  rawHtml: string,
): TemplateLinterResult {
  const combinedRaw = `${subject}\n${textBody}\n${rawHtml}`
  const visibleText = `${subject}\n${textBody}\n${extractVisibleText(rawHtml)}`

  const matches = new Set<string>()

  for (const regex of [
    MUSTACHE_REGEX,
    RAW_MUSTACHE_REGEX,
    TEMPLATE_LITERAL_REGEX,
    ERB_OR_DJANGO_REGEX,
    PYTHON_FORMAT_REGEX,
  ]) {
    // Reset regex state
    regex.lastIndex = 0
    const found = combinedRaw.match(regex) ?? []
    for (const item of found) {
      matches.add(item.trim())
    }
  }

  // Find standalone keywords without module-level stateful regex issues
  const sentinelRegex = /\b(undefined|null|None|NaN)\b|\[object Object\]/g
  let sentinelMatch: RegExpExecArray | null
  while ((sentinelMatch = sentinelRegex.exec(visibleText)) !== null) {
    const captured = (sentinelMatch[1] ?? sentinelMatch[0])?.trim()
    if (captured) {
      matches.add(captured)
    }
  }

  const uniquePlaceholders = Array.from(matches)
  const totalCount = uniquePlaceholders.length
  const sampleDisplay = uniquePlaceholders.slice(0, 3).join(', ')

  return {
    hasUnrenderedPlaceholders: totalCount > 0,
    totalCount,
    uniquePlaceholders,
    sampleDisplay,
  }
}
