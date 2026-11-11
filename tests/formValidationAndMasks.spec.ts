import fs from 'node:fs'
import path from 'node:path'
import { describe, expect, it } from 'vitest'
import {
  createEnglishTextField,
  createIpv4AddressField,
  createOptionalSlugField,
  createSlugField,
  createVerificationCodeField,
  createWebhookUrlField,
  type ValidationTranslator,
} from '../app/schemas/commonValidators'
import {
  createDesignSystemShowcaseSchema,
  DEFAULT_SHOWCASE_FORM_VALUES,
} from '../app/schemas/designSystemShowcaseForm'
import {
  createServerSettingsFormSchema,
  DEFAULT_SERVER_SETTINGS_FORM_VALUES,
} from '../app/schemas/serverSettingsForm'
import {
  createChannelFormSchema,
  createInboxFilterFormSchema,
  createProjectFormSchema,
  createWorkspaceFormSchema,
  createWorkspaceSearchSchema,
} from '../app/schemas/workspaceForms'
import { applyInputMask, unmaskInputValue, type InputMaskKey } from '../app/utils/inputMasks'
import { isEnglishAsciiText, normalizeUnicodeDigits } from '../app/utils/numberNormalization'

function loadLocaleDictionary(localeCode: 'en' | 'fa'): Record<string, unknown> {
  const filePath = path.resolve(process.cwd(), `i18n/locales/${localeCode}.json`)
  return JSON.parse(fs.readFileSync(filePath, 'utf8')) as Record<string, unknown>
}

function createLocaleTranslator(localeCode: 'en' | 'fa'): ValidationTranslator {
  const rootDictionary = loadLocaleDictionary(localeCode)
  return (key: string, named?: Record<string, unknown>): string => {
    const resolved = key.split('.').reduce<unknown>((currentNode, segment) => {
      if (currentNode !== null && typeof currentNode === 'object' && segment in currentNode) {
        return (currentNode as Record<string, unknown>)[segment]
      }
      return undefined
    }, rootDictionary)

    if (typeof resolved !== 'string') {
      throw new Error(`Missing translation key "${key}" for locale "${localeCode}"`)
    }

    if (!named) {
      return resolved
    }

    return Object.entries(named).reduce(
      (formatted, [placeholder, replacement]) =>
        formatted.replaceAll(`{${placeholder}}`, String(replacement)),
      resolved,
    )
  }
}

describe('input masks', () => {
  it.each<{ maskKey: InputMaskKey; rawInput: string; expectedMasked: string; expectedUnmasked?: string }>([
    { maskKey: 'slugIdentifier', rawInput: 'SMTP-Prod_01!', expectedMasked: 'smtp-prod01' },
    { maskKey: 'tcpPort', rawInput: '4080abc99', expectedMasked: '40809' },
    { maskKey: 'ipv4Address', rawInput: '127.0.0.1', expectedMasked: '127.0.0.1' },
    { maskKey: 'ipv4Address', rawInput: '192.168.1a.25', expectedMasked: '192.168.1.25' },
    { maskKey: 'phoneE164', rawInput: '989120000000', expectedMasked: '+98 912 000 0000', expectedUnmasked: '989120000000' },
    { maskKey: 'verificationCode', rawInput: '482910', expectedMasked: '482-910', expectedUnmasked: '482910' },
    { maskKey: 'integerCount', rawInput: '30days', expectedMasked: '30' },
    { maskKey: 'tcpPort', rawInput: '۴۰۸۰', expectedMasked: '۴۰۸۰' },
    { maskKey: 'ipv4Address', rawInput: '۱۲۷.۰.۰.۱', expectedMasked: '۱۲۷.۰.۰.۱' },
    { maskKey: 'phoneE164', rawInput: '۹۸۹۱۲۰۰۰۰۰۰۰', expectedMasked: '+۹۸ ۹۱۲ ۰۰۰ ۰۰۰۰', expectedUnmasked: '۹۸۹۱۲۰۰۰۰۰۰۰' },
    { maskKey: 'verificationCode', rawInput: '۴۸۲۹۱۰', expectedMasked: '۴۸۲-۹۱۰', expectedUnmasked: '۴۸۲۹۱۰' },
    { maskKey: 'payloadMegabytes', rawInput: '۱۰', expectedMasked: '۱۰' },
    { maskKey: 'retentionDays', rawInput: '۳۰', expectedMasked: '۳۰' },
    { maskKey: 'tcpPort', rawInput: '٤٠٨٠', expectedMasked: '٤٠٨٠' },
    { maskKey: 'ipv4Address', rawInput: '١٢٧.٠.٠.١', expectedMasked: '١٢٧.٠.٠.١' },
    { maskKey: 'phoneE164', rawInput: '٩٨٩١٢٠٠٠٠٠٠٠', expectedMasked: '+٩٨ ٩١٢ ٠٠٠ ٠٠٠٠', expectedUnmasked: '٩٨٩١٢٠٠٠٠٠٠٠' },
    { maskKey: 'tcpPort', rawInput: '४०८०', expectedMasked: '४०८०' },
    { maskKey: 'ipv4Address', rawInput: '१२७.०.०.१', expectedMasked: '१२७.०.०.१' },
    { maskKey: 'verificationCode', rawInput: '४८२९१०', expectedMasked: '४८२-९१०', expectedUnmasked: '४८२९१०' },
  ])('formats $maskKey cleanly for "$rawInput"', ({ maskKey, rawInput, expectedMasked, expectedUnmasked }) => {
    expect(applyInputMask(maskKey, rawInput)).toBe(expectedMasked)
    if (expectedUnmasked !== undefined) {
      expect(unmaskInputValue(maskKey, expectedMasked)).toBe(expectedUnmasked)
    }
  })

  it('normalizes Persian, Arabic-Indic, and Hindi digits to English ASCII digits', () => {
    expect(normalizeUnicodeDigits('۴۰۸۰ / ١٠٢٥ / ९८')).toBe('4080 / 1025 / 98')
    expect(isEnglishAsciiText('acme-studio-01')).toBe(true)
    expect(isEnglishAsciiText('تست-01')).toBe(false)
  })
})

describe.each(['en', 'fa'] as const)('zod form schemas (%s)', (localeCode) => {
  const translate = createLocaleTranslator(localeCode)

  it('validates default server settings and coerces numeric fields', () => {
    const schema = createServerSettingsFormSchema(translate)
    const result = schema.safeParse(DEFAULT_SERVER_SETTINGS_FORM_VALUES)
    expect(result.success).toBe(true)
    if (result.success) {
      expect(result.data.httpPort).toBe(4080)
      expect(result.data.smtpPort).toBe(1025)
      expect(result.data.retentionDays).toBe(30)
    }
  })

  it.each([
    {
      caseName: 'matching HTTP and SMTP ports',
      patch: { httpPort: '4080', smtpPort: '4080' },
      expectedPath: 'smtpPort',
    },
    {
      caseName: 'privileged port below 1024',
      patch: { httpPort: '80' },
      expectedPath: 'httpPort',
    },
    {
      caseName: 'port above 65535',
      patch: { smtpPort: '70000' },
      expectedPath: 'smtpPort',
    },
    {
      caseName: 'out-of-range IPv4 octet',
      patch: { bindAddress: '256.0.0.1' },
      expectedPath: 'bindAddress',
    },
    {
      caseName: 'incomplete IPv4 address',
      patch: { bindAddress: '127.0.1' },
      expectedPath: 'bindAddress',
    },
    {
      caseName: 'leading-zero IPv4 octet',
      patch: { bindAddress: '127.00.0.1' },
      expectedPath: 'bindAddress',
    },
    {
      caseName: 'retention above maximum',
      patch: { retentionDays: '5000' },
      expectedPath: 'retentionDays',
    },
  ])('rejects invalid server settings: $caseName', ({ patch, expectedPath }) => {
    const schema = createServerSettingsFormSchema(translate)
    const result = schema.safeParse({
      ...DEFAULT_SERVER_SETTINGS_FORM_VALUES,
      ...patch,
    })

    expect(result.success).toBe(false)
    if (!result.success) {
      const matchingIssue = result.error.issues.find(
        (issue) => issue.path[0] === expectedPath,
      )
      expect(matchingIssue).toBeDefined()
      expect(matchingIssue?.message.length).toBeGreaterThan(0)
      expect(matchingIssue?.message).not.toContain('validation.')
    }
  })

  it('validates workspace, project, and channel forms', () => {
    const validUuid = '11111111-1111-4111-8111-111111111111'

    const workspaceResult = createWorkspaceFormSchema(translate).safeParse({
      name: '  Arsam Studio  ',
      slug: 'arsam-studio',
      description: 'Local workspace',
    })
    expect(workspaceResult.success).toBe(true)

    const projectResult = createProjectFormSchema(translate).safeParse({
      workspaceId: validUuid,
      name: 'Notify App',
      slug: 'notify-app',
      description: '',
    })
    expect(projectResult.success).toBe(true)

    const channelResult = createChannelFormSchema(translate).safeParse({
      projectId: validUuid,
      environmentKind: 'local',
      name: 'Webhook Debugger',
      channelType: 'webhook',
      credentialIdentifier: 'webhook-local-1',
      targetEndpointUrl: 'http://127.0.0.1:4080/api/v1/webhooks/local',
      isEnabled: true,
    })
    expect(channelResult.success).toBe(true)

    const badWebhookChannel = createChannelFormSchema(translate).safeParse({
      projectId: validUuid,
      environmentKind: 'local',
      name: 'Webhook Debugger',
      channelType: 'webhook',
      credentialIdentifier: '-invalid--slug-',
      targetEndpointUrl: 'ftp://example.com/hook',
      isEnabled: true,
    })
    expect(badWebhookChannel.success).toBe(false)
  })

  it('validates design system showcase form and rejects malformed masked inputs', () => {
    const schema = createDesignSystemShowcaseSchema(translate)
    expect(schema.safeParse(DEFAULT_SHOWCASE_FORM_VALUES).success).toBe(true)

    const invalidResult = schema.safeParse({
      ...DEFAULT_SHOWCASE_FORM_VALUES,
      channelIdentifier: '-bad',
      recipientPhone: '+0012',
      verificationCode: '12-3',
      payloadJson: '{invalid-json',
      isCaptureEnabled: false,
    })

    expect(invalidResult.success).toBe(false)
    if (!invalidResult.success) {
      const errorPaths = invalidResult.error.issues.map((issue) => String(issue.path[0]))
      expect(errorPaths).toEqual(
        expect.arrayContaining([
          'channelIdentifier',
          'recipientPhone',
          'verificationCode',
          'payloadJson',
          'isCaptureEnabled',
        ]),
      )
    }
  })

  it('applies displayName, descriptionText, searchKeyword, payloadMegabytes, and retentionDays masks', () => {
    expect(applyInputMask('displayName', 'Acme\n\tStudio \x07۹۹')).toBe('AcmeStudio ۹۹')
    expect(applyInputMask('displayName', 'استودیو توسعه')).toBe('استودیو توسعه')
    expect(applyInputMask('descriptionText', 'Line 1\nLine 2\x07')).toBe('Line 1\nLine 2')
    expect(applyInputMask('searchKeyword', 'invoice\r\n42')).toBe('invoice42')
    expect(applyInputMask('payloadMegabytes', '100abc9')).toBe('100')
    expect(applyInputMask('retentionDays', '36509')).toBe('3650')
  })

  it('validates inbox filter, switcher search, and control character boundaries via Zod', () => {
    const inboxSchema = createInboxFilterFormSchema(translate)
    expect(
      inboxSchema.safeParse({
        channelType: 'email',
        status: 'captured',
        searchQuery: '  invoice  ',
      }).success,
    ).toBe(true)

    expect(
      inboxSchema.safeParse({
        channelType: 'all',
        status: 'all',
        searchQuery: 'a'.repeat(65),
      }).success,
    ).toBe(false)

    expect(
      createWorkspaceSearchSchema(translate).safeParse({
        searchQuery: 'bad\u0007query',
      }).success,
    ).toBe(false)

    expect(
      createWorkspaceFormSchema(translate).safeParse({
        name: 'Acme\nStudio',
        slug: 'acme-studio',
        description: 'Valid description',
      }).success,
    ).toBe(false)
  })

  it.each([
    {
      script: 'Persian',
      bindAddress: '۱۲۷.۰.۰.۱',
      httpPort: '۴۰۸۰',
      smtpPort: '۱۰۲۵',
      maximumHttpPayloadMegabytes: '۵',
      maximumSmtpMessageMegabytes: '۱۰',
      retentionDays: '۳۰',
      phoneNumber: '+۹۸ ۹۱۲ ۰۰۰ ۰۰۰۰',
      verificationCode: '۴۸۲-۹۱۰',
    },
    {
      script: 'Arabic-Indic',
      bindAddress: '١٢٧.٠.٠.١',
      httpPort: '٤٠٨٠',
      smtpPort: '١٠٢٥',
      maximumHttpPayloadMegabytes: '٥',
      maximumSmtpMessageMegabytes: '١٠',
      retentionDays: '٣٠',
      phoneNumber: '+٩٨ ٩١٢ ٠٠٠ ٠٠٠٠',
      verificationCode: '٤٨٢-٩١٠',
    },
    {
      script: 'Hindi',
      bindAddress: '१२७.०.०.१',
      httpPort: '४०८०',
      smtpPort: '१०२५',
      maximumHttpPayloadMegabytes: '५',
      maximumSmtpMessageMegabytes: '१०',
      retentionDays: '३०',
      phoneNumber: '+९८ ९१२ ००० ००००',
      verificationCode: '४८२-९१०',
    },
  ])(
    'normalizes $script numeric inputs to English values before backend submission',
    ({
      bindAddress,
      httpPort,
      smtpPort,
      maximumHttpPayloadMegabytes,
      maximumSmtpMessageMegabytes,
      retentionDays,
      phoneNumber,
      verificationCode,
    }) => {
      const serverResult = createServerSettingsFormSchema(translate).safeParse({
        ...DEFAULT_SERVER_SETTINGS_FORM_VALUES,
        bindAddress,
        httpPort,
        smtpPort,
        maximumHttpPayloadMegabytes,
        maximumSmtpMessageMegabytes,
        retentionDays,
      })

      expect(serverResult.success).toBe(true)
      if (serverResult.success) {
        expect(serverResult.data.bindAddress).toBe('127.0.0.1')
        expect(serverResult.data.httpPort).toBe(4080)
        expect(serverResult.data.smtpPort).toBe(1025)
        expect(serverResult.data.maximumHttpPayloadMegabytes).toBe(5)
        expect(serverResult.data.maximumSmtpMessageMegabytes).toBe(10)
        expect(serverResult.data.retentionDays).toBe(30)
      }

      const showcaseResult = createDesignSystemShowcaseSchema(translate).safeParse({
        ...DEFAULT_SHOWCASE_FORM_VALUES,
        recipientPhone: phoneNumber,
        localCapturePort: smtpPort,
        verificationCode,
      })

      expect(showcaseResult.success).toBe(true)
      if (showcaseResult.success) {
        expect(showcaseResult.data.recipientPhone).toBe('+989120000000')
        expect(showcaseResult.data.localCapturePort).toBe(1025)
        expect(showcaseResult.data.verificationCode).toBe('482910')
      }
    },
  )

  it('strictly validates English-only fields, slugs, URLs, IPv4 addresses, and verification codes', () => {
    const englishField = createEnglishTextField(translate, { minLength: 2, maxLength: 64 })
    expect(englishField.safeParse('acme_prod-01').success).toBe(true)
    expect(englishField.safeParse('سلام').success).toBe(false)
    expect(englishField.safeParse('app-۱۲').success).toBe(false)

    const slugField = createSlugField(translate)
    expect(slugField.safeParse('acme-studio-01').success).toBe(true)
    for (const invalidSlug of ['Acme-Studio', 'acme_studio', 'acme studio', 'acme--studio', '-acme', 'acme-', 'تست-اسلاگ', 'a']) {
      expect(slugField.safeParse(invalidSlug).success).toBe(false)
    }

    const optionalSlugField = createOptionalSlugField(translate)
    expect(optionalSlugField.safeParse('').success).toBe(true)
    expect(optionalSlugField.safeParse('INVALID_CAPS').success).toBe(false)
    expect(optionalSlugField.safeParse('تست').success).toBe(false)

    const webhookField = createWebhookUrlField(translate)
    expect(webhookField.safeParse('https://hooks.example.com/notify').success).toBe(true)
    expect(webhookField.safeParse('https://تست.com').success).toBe(false)
    expect(webhookField.safeParse('https://example.com/a b').success).toBe(false)

    const ipv4Field = createIpv4AddressField(translate)
    expect(ipv4Field.safeParse('۱۲۷.الف.۰.۱').success).toBe(false)

    const verificationField = createVerificationCodeField(translate)
    expect(verificationField.safeParse('سلام123456').success).toBe(false)
    expect(verificationField.safeParse('12ab34-56').success).toBe(false)
  })
})
