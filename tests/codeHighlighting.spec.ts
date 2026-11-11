import fs from 'node:fs'
import path from 'node:path'
import { describe, expect, it } from 'vitest'
import {
  detectCodeLanguage,
  renderHighlightedCode,
  resolveCodeLanguageLabel,
  type SupportedCodeLanguage,
} from '../app/utils/codeHighlighting'
import { resolveChannelIntegrationSnippetLanguage } from '../app/utils/integrationSnippets'

describe('offline code syntax highlighting & language detection', () => {
  it.each<{
    label: string
    code: string
    contentType?: string
    expectedLanguage: SupportedCodeLanguage
    expectedBadge: string
  }>([
    {
      label: 'valid JSON payload',
      code: '{"event":"invoice.paid","amount":4900}',
      expectedLanguage: 'json',
      expectedBadge: 'JSON',
    },
    {
      label: 'JSON MIME type hint',
      code: 'payload_preview',
      contentType: 'application/vnd.api+json',
      expectedLanguage: 'json',
      expectedBadge: 'JSON',
    },
    {
      label: 'HTML document',
      code: '<!DOCTYPE html><html><body><h1>Hello</h1></body></html>',
      expectedLanguage: 'html',
      expectedBadge: 'HTML',
    },
    {
      label: 'cURL shell snippet',
      code: 'curl -X POST http://127.0.0.1:4080/api/v1/sms \\\n  -H "X-Channel-Key: test"',
      expectedLanguage: 'bash',
      expectedBadge: 'BASH',
    },
    {
      label: 'SMTP dotenv snippet',
      code: 'MAIL_MAILER=smtp\nMAIL_HOST=127.0.0.1\nMAIL_PORT=1025',
      expectedLanguage: 'dotenv',
      expectedBadge: '.ENV',
    },
    {
      label: 'raw RFC 2822 MIME email source',
      code: 'From: app@example.test\nTo: dev@example.test\nSubject: Hi\nMIME-Version: 1.0\n\nBody',
      expectedLanguage: 'http',
      expectedBadge: 'MIME / HTTP',
    },
    {
      label: 'unstructured plain text fallback',
      code: 'Simple query string token_value_123',
      expectedLanguage: 'plaintext',
      expectedBadge: 'TEXT',
    },
  ])('detects $label as $expectedLanguage ($expectedBadge)', ({ code, contentType, expectedLanguage, expectedBadge }) => {
    const detected = detectCodeLanguage(code, contentType)
    expect(detected).toBe(expectedLanguage)
    expect(resolveCodeLanguageLabel(detected)).toBe(expectedBadge)

    const result = renderHighlightedCode(code, 'auto', contentType)
    expect(result.language).toBe(expectedLanguage)
    expect(result.label).toBe(expectedBadge)
    expect(result.html.length).toBeGreaterThan(0)
  })

  it('produces highlight.js token spans for syntax-rich languages', () => {
    const jsonResult = renderHighlightedCode('{"active": true, "count": 42}', 'json')
    expect(jsonResult.html).toContain('hljs-attr')
    expect(jsonResult.html).toContain('hljs-number')

    const htmlResult = renderHighlightedCode('<div class="card">Hi</div>', 'html')
    expect(htmlResult.html).toContain('hljs-tag')
    expect(htmlResult.html).toContain('hljs-attr')

    const envResult = renderHighlightedCode('MAIL_HOST=127.0.0.1\nMAIL_PORT=1025', 'dotenv')
    expect(envResult.html).toContain('hljs-attr')
  })

  it('escapes dangerous HTML script tags safely before returning markup', () => {
    const xssSample = '<script>alert("xss")</script>'
    for (const language of ['html', 'json', 'plaintext'] as const) {
      const rendered = renderHighlightedCode(xssSample, language)
      expect(rendered.html).not.toContain('<script>')
      expect(rendered.html).toContain('&lt;')
    }
  })

  it.each([
    { channelType: 'email' as const, expected: 'dotenv' as const },
    { channelType: 'sms' as const, expected: 'bash' as const },
    { channelType: 'push' as const, expected: 'bash' as const },
    { channelType: 'webhook' as const, expected: 'bash' as const },
  ])('maps integration snippet for $channelType to $expected', ({ channelType, expected }) => {
    expect(resolveChannelIntegrationSnippetLanguage(channelType)).toBe(expected)
  })

  it('routes all code viewer components through SharedCodeBlock', () => {
    const codeComponents = [
      'app/pages/index.vue',
      'app/components/channels/ChannelCredentialCard.vue',
      'app/components/inbox/WebhookInspectorView.vue',
      'app/components/inbox/PushInspectorView.vue',
      'app/components/inbox/SmsInspectorView.vue',
      'app/components/design-system/ComponentShowcaseSection.vue',
    ]

    for (const relativePath of codeComponents) {
      const content = fs.readFileSync(path.resolve(process.cwd(), relativePath), 'utf8')
      expect(content).toContain('<SharedCodeBlock')
      expect(content).not.toMatch(/<pre[\s>]/)
    }

    const emailInspector = fs.readFileSync(
      path.resolve(process.cwd(), 'app/components/inbox/EmailInspectorView.vue'),
      'utf8',
    )
    expect(emailInspector).toContain('<SharedCodeBlock')
  })
})
