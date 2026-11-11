import fs from 'node:fs'
import path from 'node:path'
import { describe, expect, it } from 'vitest'
import { buildSandboxedEmailDocument, escapeEmailHtml } from '../app/utils/emailSandbox'

function collectVueFiles(directory: string): string[] {
  const collected: string[] = []
  for (const entry of fs.readdirSync(directory, { withFileTypes: true })) {
    const entryPath = path.join(directory, entry.name)
    if (entry.isDirectory()) {
      collected.push(...collectVueFiles(entryPath))
    } else if (entry.name.endsWith('.vue')) {
      collected.push(entryPath)
    }
  }
  return collected
}

describe('Phase 6 security, theme, RTL, and accessibility hardening', () => {
  it('escapes untrusted plain-text email markup before sandboxed srcdoc rendering', () => {
    const escaped = escapeEmailHtml('<script>alert("xss")</script> & <user@example.com>')
    expect(escaped).toBe('&lt;script&gt;alert(&quot;xss&quot;)&lt;/script&gt; &amp; &lt;user@example.com&gt;')
    expect(escaped).not.toContain('<script>')
  })

  it.each([
    { isDarkTheme: false, expectedBackground: '#ffffff', expectedText: '#172033' },
    { isDarkTheme: true, expectedBackground: '#111827', expectedText: '#f8fafc' },
  ])(
    'builds strict offline CSP iframe document across light/dark themes (dark=$isDarkTheme)',
    ({ isDarkTheme, expectedBackground, expectedText }) => {
      const documentHtml = buildSandboxedEmailDocument('<p>Hello NotifyTrap</p>', isDarkTheme)
      expect(documentHtml).toContain("default-src 'none'")
      expect(documentHtml).toContain("script-src 'none'")
      expect(documentHtml).toContain("connect-src 'none'")
      expect(documentHtml).toContain("img-src data:")
      expect(documentHtml).toContain(`background: ${expectedBackground};`)
      expect(documentHtml).toContain(`color: ${expectedText};`)
      expect(documentHtml).toContain('<p>Hello NotifyTrap</p>')
    },
  )

  it('enforces RTL/LTR logical CSS classes and aria-label on icon-only buttons across Vue views', () => {
    const appDirectory = path.resolve(process.cwd(), 'app')
    const vueFiles = collectVueFiles(appDirectory)
    expect(vueFiles.length).toBeGreaterThan(0)

    const nonLogicalUtilityPattern =
      /(?<![a-z-])(?:ml|mr|pl|pr|left|right|text-left|text-right|border-l|border-r)-[A-Za-z0-9[\]./%-]+/g

    for (const vueFilePath of vueFiles) {
      const source = fs.readFileSync(vueFilePath, 'utf8')
      const matchedClasses = source.match(nonLogicalUtilityPattern) ?? []
      expect(
        matchedClasses,
        `Non-logical directional utility found in ${path.relative(process.cwd(), vueFilePath)}`,
      ).toEqual([])

      const buttonTagMatches = [...source.matchAll(/<(UButton|button)\b([\s\S]*?)>/g)]
      for (const buttonMatch of buttonTagMatches) {
        const attributes = buttonMatch[2] ?? ''
        const isIconOnly = /\bsquare\b/.test(attributes) && !/\b:?label=/.test(attributes)
        if (isIconOnly) {
          expect(
            /\b:?aria-label=/.test(attributes),
            `Icon-only button missing aria-label in ${path.relative(process.cwd(), vueFilePath)}`,
          ).toBe(true)
        }
      }
    }
  })

  it('guarantees both Light/Dark themes and LTR/RTL locales are configured in nuxt config and app CSS', () => {
    const nuxtConfig = fs.readFileSync(path.resolve(process.cwd(), 'nuxt.config.ts'), 'utf8')
    expect(nuxtConfig).toContain("code: 'en'")
    expect(nuxtConfig).toContain("dir: 'ltr'")
    expect(nuxtConfig).toContain("code: 'fa'")
    expect(nuxtConfig).toContain("dir: 'rtl'")

    const mainCss = fs.readFileSync(path.resolve(process.cwd(), 'app/assets/css/main.css'), 'utf8')
    expect(mainCss).toContain(':root')
    expect(mainCss).toContain('.dark')
  })
})
