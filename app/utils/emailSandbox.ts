/**
 * Escapes untrusted plain text before wrapping inside a preformatted HTML email fallback.
 */
export function escapeEmailHtml(rawText: string): string {
  return rawText
    .replaceAll('&', '&amp;')
    .replaceAll('<', '&lt;')
    .replaceAll('>', '&gt;')
    .replaceAll('"', '&quot;')
    .replaceAll("'", '&#39;')
}

export interface EmailSandboxOptions {
  isDarkTheme: boolean
  simulateEmailDarkMode?: boolean
}

/**
 * Wraps backend-sanitized email HTML inside a strict offline CSP document for iframe srcdoc rendering.
 */
export function buildSandboxedEmailDocument(
  sanitizedHtml: string,
  optionsOrIsDark: boolean | EmailSandboxOptions,
): string {
  const options: EmailSandboxOptions = typeof optionsOrIsDark === 'boolean'
    ? { isDarkTheme: optionsOrIsDark, simulateEmailDarkMode: false }
    : optionsOrIsDark

  const useDarkCanvas = Boolean(options.simulateEmailDarkMode || options.isDarkTheme)
  const backgroundColor = useDarkCanvas ? '#111827' : '#ffffff'
  const textColor = useDarkCanvas ? '#f8fafc' : '#172033'
  const linkColor = useDarkCanvas ? '#818cf8' : '#4f46e5'
  const darkFilterCss = options.simulateEmailDarkMode
    ? 'color-scheme: dark; filter: invert(0.92) hue-rotate(180deg);'
    : ''
  const darkImageFixCss = options.simulateEmailDarkMode
    ? 'img { filter: invert(1) hue-rotate(180deg); }'
    : ''

  return `<!DOCTYPE html>
<html>
<head>
<meta charset="utf-8">
<meta http-equiv="Content-Security-Policy" content="default-src 'none'; style-src 'unsafe-inline'; img-src data:; font-src 'none'; script-src 'none'; connect-src 'none'; frame-src 'none'; form-action 'none';">
<style>
  html {
    background: ${backgroundColor};
    ${darkFilterCss}
  }
  html, body {
    margin: 0;
    padding: 16px;
    background: ${backgroundColor};
    color: ${textColor};
    font-family: system-ui, -apple-system, sans-serif;
    font-size: 14px;
    line-height: 1.6;
    word-break: break-word;
  }
  a { color: ${linkColor}; }
  img { max-width: 100%; height: auto; }
  ${darkImageFixCss}
  table { border-collapse: collapse; max-width: 100%; }
  pre, code { font-family: ui-monospace, monospace; font-size: 12px; }
</style>
</head>
<body>${sanitizedHtml}</body>
</html>`
}
