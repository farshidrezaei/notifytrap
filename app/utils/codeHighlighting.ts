import hljs from 'highlight.js/lib/core'
import bashLanguage from 'highlight.js/lib/languages/bash'
import httpLanguage from 'highlight.js/lib/languages/http'
import iniLanguage from 'highlight.js/lib/languages/ini'
import jsonLanguage from 'highlight.js/lib/languages/json'
import plaintextLanguage from 'highlight.js/lib/languages/plaintext'
import xmlLanguage from 'highlight.js/lib/languages/xml'

export type SupportedCodeLanguage = 'json' | 'html' | 'bash' | 'dotenv' | 'http' | 'plaintext'
export type CodeLanguageOption = SupportedCodeLanguage | 'auto'

export interface HighlightedCodeResult {
  language: SupportedCodeLanguage
  label: string
  html: string
}

hljs.registerLanguage('json', jsonLanguage)
hljs.registerLanguage('xml', xmlLanguage)
hljs.registerLanguage('bash', bashLanguage)
hljs.registerLanguage('ini', iniLanguage)
hljs.registerLanguage('http', httpLanguage)
hljs.registerLanguage('plaintext', plaintextLanguage)

const LANGUAGE_METADATA: Record<SupportedCodeLanguage, { label: string; highlightKey: string }> = {
  json: { label: 'JSON', highlightKey: 'json' },
  html: { label: 'HTML', highlightKey: 'xml' },
  bash: { label: 'BASH', highlightKey: 'bash' },
  dotenv: { label: '.ENV', highlightKey: 'ini' },
  http: { label: 'MIME / HTTP', highlightKey: 'http' },
  plaintext: { label: 'TEXT', highlightKey: 'plaintext' },
}

function escapeHtml(raw: string): string {
  return raw
    .replaceAll('&', '&amp;')
    .replaceAll('<', '&lt;')
    .replaceAll('>', '&gt;')
    .replaceAll('"', '&quot;')
    .replaceAll("'", '&#39;')
}

/**
 * Resolves a human-readable uppercase badge label for any supported code language.
 */
export function resolveCodeLanguageLabel(language: SupportedCodeLanguage): string {
  return LANGUAGE_METADATA[language].label
}

/**
 * Infers code language from an optional MIME content type and lightweight content heuristics.
 */
export function detectCodeLanguage(
  sourceCode: string,
  hintContentType?: string,
): SupportedCodeLanguage {
  const normalizedHint = (hintContentType ?? '').trim().toLowerCase()
  if (normalizedHint.includes('json')) {
    return 'json'
  }
  if (normalizedHint.includes('html') || normalizedHint.includes('xml')) {
    return 'html'
  }
  if (normalizedHint.includes('rfc822') || normalizedHint.includes('http')) {
    return 'http'
  }

  const trimmed = sourceCode.trim()
  if (!trimmed) {
    return 'plaintext'
  }

  if (trimmed.startsWith('{') || trimmed.startsWith('[')) {
    try {
      JSON.parse(trimmed)
      return 'json'
    } catch {
      // Not valid JSON; continue through other heuristics.
    }
  }

  if (/^<!doctype\s+html/i.test(trimmed) || /^<[a-zA-Z!/][\s\S]*>/.test(trimmed)) {
    return 'html'
  }

  if (/^(curl|wget|http)\s+/i.test(trimmed) || trimmed.startsWith('#!/')) {
    return 'bash'
  }

  if (/^[A-Z][A-Z0-9_]{1,}=/m.test(trimmed)) {
    return 'dotenv'
  }

  if (/^(MIME-Version|Content-Type|From|To|Subject|HTTP\/\d)/im.test(trimmed)) {
    return 'http'
  }

  return 'plaintext'
}

/**
 * Highlights a code snippet offline with highlight.js core and returns safe HTML + resolved language metadata.
 */
export function renderHighlightedCode(
  sourceCode: string,
  language: CodeLanguageOption = 'auto',
  hintContentType?: string,
): HighlightedCodeResult {
  const resolvedLanguage: SupportedCodeLanguage =
    language === 'auto' ? detectCodeLanguage(sourceCode, hintContentType) : language
  const metadata = LANGUAGE_METADATA[resolvedLanguage]

  try {
    const highlighted = hljs.highlight(sourceCode, {
      language: metadata.highlightKey,
      ignoreIllegals: true,
    })
    return {
      language: resolvedLanguage,
      label: metadata.label,
      html: highlighted.value,
    }
  } catch {
    return {
      language: 'plaintext',
      label: LANGUAGE_METADATA.plaintext.label,
      html: escapeHtml(sourceCode),
    }
  }
}
