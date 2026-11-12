/**
 * High-performance, offline-first MJML detection, structural validation,
 * and semantic HTML transpilation for developer email inspection.
 */

export interface MjmlTagSummary {
  name: string
  count: number
}

export interface MjmlInspectionResult {
  isMjml: boolean
  hasRootMjmlTag: boolean
  hasBodyTag: boolean
  tagSummaries: MjmlTagSummary[]
  totalTagCount: number
  compiledHtml: string
}

const MJML_TAG_REGEX = /<\/?(mj-[a-z0-9-]+|mjml)\b[^>]*>/gi
const MJML_CONTENT_SNIFFER = /<mjml\b|<mj-body\b|<mj-section\b/i

/**
 * Checks whether content contains MJML markup.
 */
export function isMjmlMarkup(content: string): boolean {
  return MJML_CONTENT_SNIFFER.test(content)
}

/**
 * Parses attributes from an MJML XML-like element tag.
 */
function parseTagAttributes(tagText: string): Record<string, string> {
  const attributes: Record<string, string> = {}
  const attributeRegex = /([a-z0-9-]+)\s*=\s*(?:"([^"]*)"|'([^']*)'|([^\s>]+))/gi
  let match: RegExpExecArray | null

  while ((match = attributeRegex.exec(tagText)) !== null) {
    const key = match[1]?.toLowerCase()
    const value = match[2] ?? match[3] ?? match[4] ?? ''
    if (key) {
      attributes[key] = value
    }
  }

  return attributes
}

/**
 * Transpiles common MJML markup into responsive email HTML with table layout.
 */
export function transpileMjmlToResponsiveHtml(mjmlSource: string): string {
  if (!isMjmlMarkup(mjmlSource)) {
    return mjmlSource
  }

  let html = mjmlSource

  // 1. Strip <mj-head> and internal configuration tags while preserving <mj-title> and <mj-style>
  html = html.replace(/<mj-preview\b[^>]*>([\s\S]*?)<\/mj-preview>/gi, (_match, previewText) => {
    return `<div style="display:none;font-size:1px;color:#ffffff;line-height:1px;max-height:0px;max-width:0px;opacity:0;overflow:hidden;">${previewText}</div>`
  })

  html = html.replace(/<mj-style\b[^>]*>([\s\S]*?)<\/mj-style>/gi, (_match, cssStyles) => {
    return `<style type="text/css">${cssStyles}</style>`
  })

  // Remove other mj-head content
  html = html.replace(/<mj-head\b[^>]*>[\s\S]*?<\/mj-head>/gi, '')

  // 2. Transpile <mj-button>
  html = html.replace(/<mj-button\b([^>]*)>([\s\S]*?)<\/mj-button>/gi, (_match, rawAttrs, buttonText) => {
    const attributes = parseTagAttributes(rawAttrs)
    const href = attributes.href || '#'
    const backgroundColor = attributes['background-color'] || '#3b82f6'
    const color = attributes.color || '#ffffff'
    const borderRadius = attributes['border-radius'] || '6px'
    const padding = attributes.padding || '10px 20px'
    const fontSize = attributes['font-size'] || '14px'
    const fontWeight = attributes['font-weight'] || '600'
    const align = attributes.align || 'center'

    return `
      <div style="text-align:${align};margin:12px 0;">
        <!--[if mso]>
        <v:roundrect xmlns:v="urn:schemas-microsoft-com:vml" href="${href}" style="padding:${padding};" arcsize="10%" fillcolor="${backgroundColor}" stroke="f">
        <v:textbox inset="0,0,0,0">
        <center style="color:${color};font-size:${fontSize};font-weight:${fontWeight};">
        <![endif]-->
        <a href="${href}" style="background-color:${backgroundColor};color:${color};display:inline-block;padding:${padding};border-radius:${borderRadius};font-size:${fontSize};font-weight:${fontWeight};text-decoration:none;font-family:sans-serif;">
          ${buttonText}
        </a>
        <!--[if mso]></center></v:textbox></v:roundrect><![endif]-->
      </div>`
  })

  // 3. Transpile <mj-image>
  html = html.replace(/<mj-image\b([^>]*)\/?>/gi, (_match, rawAttrs) => {
    const attributes = parseTagAttributes(rawAttrs)
    const src = attributes.src || ''
    const alt = attributes.alt || ''
    const width = attributes.width || '100%'
    const align = attributes.align || 'center'
    const padding = attributes.padding || '10px 0'

    return `
      <div style="text-align:${align};padding:${padding};">
        <img src="${src}" alt="${alt}" style="max-width:${width};height:auto;display:inline-block;border:0;outline:none;text-decoration:none;" />
      </div>`
  })

  // 4. Transpile <mj-divider>
  html = html.replace(/<mj-divider\b([^>]*)\/?>/gi, (_match, rawAttrs) => {
    const attributes = parseTagAttributes(rawAttrs)
    const borderColor = attributes['border-color'] || '#e5e7eb'
    const borderWidth = attributes['border-width'] || '1px'
    const width = attributes.width || '100%'
    return `<div style="padding:10px 0;"><hr style="border:0;border-top:${borderWidth} solid ${borderColor};width:${width};margin:0 auto;" /></div>`
  })

  // 5. Transpile <mj-spacer>
  html = html.replace(/<mj-spacer\b([^>]*)\/?>/gi, (_match, rawAttrs) => {
    const attributes = parseTagAttributes(rawAttrs)
    const height = attributes.height || '20px'
    return `<div style="height:${height};line-height:${height};font-size:1px;">&nbsp;</div>`
  })

  // 6. Transpile <mj-text>
  html = html.replace(/<mj-text\b([^>]*)>([\s\S]*?)<\/mj-text>/gi, (_match, rawAttrs, innerContent) => {
    const attributes = parseTagAttributes(rawAttrs)
    const color = attributes.color || '#374151'
    const fontSize = attributes['font-size'] || '14px'
    const lineHeight = attributes['line-height'] || '1.6'
    const align = attributes.align || 'inherit'
    const padding = attributes.padding || '6px 0'
    const fontFamily = attributes['font-family'] || 'sans-serif'

    return `<div style="color:${color};font-size:${fontSize};line-height:${lineHeight};text-align:${align};padding:${padding};font-family:${fontFamily};">${innerContent}</div>`
  })

  // 7. Transpile <mj-column>
  html = html.replace(/<mj-column\b([^>]*)>([\s\S]*?)<\/mj-column>/gi, (_match, rawAttrs, innerContent) => {
    const attributes = parseTagAttributes(rawAttrs)
    const width = attributes.width || '100%'
    const verticalAlign = attributes['vertical-align'] || 'top'
    return `
      <td style="width:${width};vertical-align:${verticalAlign};padding:0 8px;" class="mj-column">
        ${innerContent}
      </td>`
  })

  // 8. Transpile <mj-section>
  html = html.replace(/<mj-section\b([^>]*)>([\s\S]*?)<\/mj-section>/gi, (_match, rawAttrs, innerContent) => {
    const attributes = parseTagAttributes(rawAttrs)
    const backgroundColor = attributes['background-color'] || 'transparent'
    const padding = attributes.padding || '16px 0'
    return `
      <table role="presentation" border="0" cellpadding="0" cellspacing="0" width="100%" style="background-color:${backgroundColor};padding:${padding};margin:0 auto;">
        <tr>
          <td>
            <table role="presentation" border="0" cellpadding="0" cellspacing="0" width="100%" style="max-width:600px;margin:0 auto;">
              <tr>
                ${innerContent}
              </tr>
            </table>
          </td>
        </tr>
      </table>`
  })

  // 9. Transpile <mj-body>
  html = html.replace(/<mj-body\b([^>]*)>([\s\S]*?)<\/mj-body>/gi, (_match, rawAttrs, innerContent) => {
    const attributes = parseTagAttributes(rawAttrs)
    const backgroundColor = attributes['background-color'] || '#f9fafb'
    const width = attributes.width || '600px'
    return `
      <div style="background-color:${backgroundColor};margin:0;padding:20px 0;width:100%;font-family:sans-serif;">
        <table role="presentation" border="0" cellpadding="0" cellspacing="0" width="100%">
          <tr>
            <td align="center">
              <div style="max-width:${width};margin:0 auto;text-align:start;">
                ${innerContent}
              </div>
            </td>
          </tr>
        </table>
      </div>`
  })

  // 10. Strip root <mjml> tags
  html = html.replace(/<\/?mjml\b[^>]*>/gi, '')

  return `<!DOCTYPE html>
<html>
<head>
  <meta charset="utf-8">
  <meta name="viewport" content="width=device-width, initial-scale=1.0">
  <style>
    body { margin: 0; padding: 0; word-break: break-word; }
    table { border-collapse: collapse; }
    @media only screen and (max-width: 480px) {
      .mj-column { display: block !important; width: 100% !important; }
    }
  </style>
</head>
<body>
${html.trim()}
</body>
</html>`
}

/**
 * Inspects MJML markup, calculates structural statistics, and provides transpiled HTML.
 */
export function inspectMjmlContent(content: string): MjmlInspectionResult {
  const isMjml = isMjmlMarkup(content)
  if (!isMjml) {
    return {
      isMjml: false,
      hasRootMjmlTag: false,
      hasBodyTag: false,
      tagSummaries: [],
      totalTagCount: 0,
      compiledHtml: '',
    }
  }

  const tagCounts = new Map<string, number>()
  let match: RegExpExecArray | null

  while ((match = MJML_TAG_REGEX.exec(content)) !== null) {
    const rawTag = match[1]?.toLowerCase()
    if (rawTag && !rawTag.startsWith('/')) {
      tagCounts.set(rawTag, (tagCounts.get(rawTag) ?? 0) + 1)
    }
  }

  const tagSummaries: MjmlTagSummary[] = Array.from(tagCounts.entries())
    .map(([name, count]) => ({ name, count }))
    .sort((firstItem, secondItem) => secondItem.count - firstItem.count)

  const totalTagCount = tagSummaries.reduce((sum, item) => sum + item.count, 0)
  const hasRootMjmlTag = /<mjml\b/i.test(content)
  const hasBodyTag = /<mj-body\b/i.test(content)
  const compiledHtml = transpileMjmlToResponsiveHtml(content)

  return {
    isMjml: true,
    hasRootMjmlTag,
    hasBodyTag,
    tagSummaries,
    totalTagCount,
    compiledHtml,
  }
}
