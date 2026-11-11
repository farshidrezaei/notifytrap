import fs from 'node:fs'
import path from 'node:path'
import { describe, expect, it } from 'vitest'
import type { ValidationTranslator } from '../app/schemas/commonValidators'
import {
  createScopedChannelModalSchema,
  createWorkspaceOnboardingSchema,
} from '../app/schemas/workspaceForms'
import type { ChannelCredentialReveal, ChannelSummary } from '../app/types/application'
import { buildSandboxedEmailDocument } from '../app/utils/emailSandbox'
import {
  buildMonogramLabel,
  formatByteSize,
  formatSlugIdentifier,
} from '../app/utils/formatters'
import { buildChannelIntegrationSnippet } from '../app/utils/integrationSnippets'

function createLocaleTranslator(localeCode: 'en' | 'fa'): ValidationTranslator {
  const filePath = path.resolve(process.cwd(), `i18n/locales/${localeCode}.json`)
  const dictionary = JSON.parse(fs.readFileSync(filePath, 'utf8')) as Record<string, unknown>
  return (key: string, named?: Record<string, unknown>): string => {
    const resolved = key.split('.').reduce<unknown>((currentNode, segment) => {
      if (currentNode !== null && typeof currentNode === 'object' && segment in currentNode) {
        return (currentNode as Record<string, unknown>)[segment]
      }
      return undefined
    }, dictionary)

    if (typeof resolved !== 'string') {
      throw new Error(`Missing translation key "${key}" for locale "${localeCode}"`)
    }

    if (!named) {
      return resolved
    }

    return Object.entries(named).reduce(
      (formatted, [tokenKey, tokenValue]) =>
        formatted.replaceAll(`{${tokenKey}}`, String(tokenValue)),
      resolved,
    )
  }
}

describe('Phase 5 email HTML sandbox builder', () => {
  it('enforces offline zero-script CSP policies in both light and dark themes', () => {
    const darkDocument = buildSandboxedEmailDocument('<p>Hello <strong>World</strong></p>', true)
    const lightDocument = buildSandboxedEmailDocument('<p>Hello</p>', false)

    for (const documentHtml of [darkDocument, lightDocument]) {
      expect(documentHtml).toContain("default-src 'none'")
      expect(documentHtml).toContain("script-src 'none'")
      expect(documentHtml).toContain("connect-src 'none'")
      expect(documentHtml).toContain("frame-src 'none'")
      expect(documentHtml).toContain("form-action 'none'")
      expect(documentHtml).toContain('img-src data:')
    }

    expect(darkDocument).toContain('background: #111827;')
    expect(darkDocument).toContain('color: #f8fafc;')
    expect(lightDocument).toContain('background: #ffffff;')
    expect(lightDocument).toContain('color: #172033;')
  })
})

describe('Phase 5 integration snippet generator', () => {
  const baseChannel: Omit<ChannelSummary, 'channelType' | 'credentialIdentifier'> = {
    id: '00000000-0000-4000-8000-000000000010',
    workspaceId: '00000000-0000-4000-8000-000000000001',
    projectId: '00000000-0000-4000-8000-000000000002',
    environmentId: '00000000-0000-4000-8000-000000000003',
    name: 'Test Channel',
    isEnabled: true,
    createdAt: '2026-11-11T00:00:00Z',
    updatedAt: '2026-11-11T00:00:00Z',
  }

  const credential: ChannelCredentialReveal = {
    channelId: baseChannel.id,
    credentialIdentifier: 'billing-mailer',
    secretToken: 'secret-token-value',
    compositeKey: 'billing-mailer.secret-token-value',
  }

  it.each([
    {
      channelType: 'email' as const,
      expectedFragments: ['MAIL_MAILER=smtp', 'MAIL_HOST=127.0.0.1', 'MAIL_PORT=2025', 'MAIL_PASSWORD=secret-token-value'],
    },
    {
      channelType: 'sms' as const,
      expectedFragments: ['http://127.0.0.1:5080/api/v1/sms', 'X-Channel-Key: billing-mailer.secret-token-value'],
    },
    {
      channelType: 'push' as const,
      expectedFragments: ['http://127.0.0.1:5080/api/v1/push', 'device_token'],
    },
    {
      channelType: 'webhook' as const,
      expectedFragments: [
        'http://127.0.0.1:5080/api/v1/webhooks/billing-mailer/events',
        'X-Channel-Key: secret-token-value',
      ],
    },
  ])('builds accurate local snippet for $channelType channels', ({ channelType, expectedFragments }) => {
    const output = buildChannelIntegrationSnippet({
      channel: {
        ...baseChannel,
        channelType,
        credentialIdentifier: 'billing-mailer',
      },
      credential,
      httpEndpoint: '127.0.0.1:5080',
      smtpEndpoint: '127.0.0.1:2025',
    })

    for (const fragment of expectedFragments) {
      expect(output).toContain(fragment)
    }
  })

  it('uses redacted secret placeholder when credential has not been revealed', () => {
    const output = buildChannelIntegrationSnippet({
      channel: {
        ...baseChannel,
        channelType: 'sms',
        credentialIdentifier: 'otp-sms',
      },
      credential: null,
      httpEndpoint: '127.0.0.1:4080',
      smtpEndpoint: '127.0.0.1:1025',
    })

    expect(output).toContain('otp-sms.<SECRET_TOKEN>')
    expect(output).not.toContain('secret-token-value')
  })
})

describe('Phase 5 formatting helpers & onboarding/channel schemas', () => {
  it('formats slug identifiers, byte sizes, and monograms deterministically', () => {
    expect(formatSlugIdentifier('  Acme Studio!! v2 ')).toBe('acme-studio-v2')
    expect(formatSlugIdentifier('پروژه ۱۲ جدید 2026')).toBe('12-2026')
    const truncatedSlug = formatSlugIdentifier(`${'a'.repeat(63)}-trailing-segment`)
    expect(truncatedSlug.length).toBeLessThanOrEqual(64)
    expect(truncatedSlug.endsWith('-')).toBe(false)
    expect(formatByteSize(0)).toBe('0 B')
    expect(formatByteSize(512)).toBe('512 B')
    expect(formatByteSize(2048)).toBe('2.0 KB')
    expect(buildMonogramLabel('Billing API')).toBe('BA')
    expect(buildMonogramLabel('NotifyTrap')).toBe('NO')
    expect(buildMonogramLabel(undefined)).toBe('NT')
  })

  it.each(['en', 'fa'] as const)(
    'validates onboarding and scoped channel modal forms in %s',
    (localeCode) => {
      const translate = createLocaleTranslator(localeCode)
      const onboardingSchema = createWorkspaceOnboardingSchema(translate)
      const scopedChannelSchema = createScopedChannelModalSchema(translate)

      const validOnboarding = onboardingSchema.safeParse({
        workspaceName: 'Acme Studio',
        workspaceSlug: 'acme-studio',
        projectName: 'Billing API',
        projectSlug: 'billing-api',
      })
      expect(validOnboarding.success).toBe(true)

      const invalidOnboarding = onboardingSchema.safeParse({
        workspaceName: 'A',
        workspaceSlug: '-bad-',
        projectName: '',
        projectSlug: 'ok-slug',
      })
      expect(invalidOnboarding.success).toBe(false)

      const blankIdentifierChannel = scopedChannelSchema.safeParse({
        name: 'Transactional Mailer',
        channelType: 'email',
        credentialIdentifier: '',
        isEnabled: true,
      })
      expect(blankIdentifierChannel.success).toBe(true)

      const validIdentifierChannel = scopedChannelSchema.safeParse({
        name: 'Transactional Mailer',
        channelType: 'email',
        credentialIdentifier: 'mailer-01',
        isEnabled: true,
      })
      expect(validIdentifierChannel.success).toBe(true)

      for (const badIdentifier of ['INVALID_CAPS', 'a', 'تست', 'Acme']) {
        const invalidIdentifierChannel = scopedChannelSchema.safeParse({
          name: 'Transactional Mailer',
          channelType: 'webhook',
          credentialIdentifier: badIdentifier,
          isEnabled: true,
        })
        expect(invalidIdentifierChannel.success).toBe(false)
      }
    },
  )
})
