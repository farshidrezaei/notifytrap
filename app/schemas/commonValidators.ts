import { z } from 'zod'
import {
  hasOnlyAsciiNonWhitespace,
  isEnglishAsciiText,
  normalizeUnicodeDigits,
} from '../utils/numberNormalization'

export type ValidationTranslator = (
  key: string,
  named?: Record<string, unknown>,
) => string

export const MINIMUM_NAME_LENGTH = 2
export const MAXIMUM_NAME_LENGTH = 80
export const MINIMUM_SLUG_LENGTH = 2
export const MAXIMUM_SLUG_LENGTH = 64
export const MAXIMUM_DESCRIPTION_LENGTH = 280
export const MAXIMUM_SEARCH_QUERY_LENGTH = 64
export const MINIMUM_TCP_PORT = 1024
export const MAXIMUM_TCP_PORT = 65535
export const MINIMUM_VERIFICATION_CODE_DIGITS = 6
export const MINIMUM_PAYLOAD_MEGABYTES = 1
export const MAXIMUM_PAYLOAD_MEGABYTES = 100
export const MINIMUM_RETENTION_DAYS = 1
export const MAXIMUM_RETENTION_DAYS = 3650

const SLUG_PATTERN = /^[a-z0-9]+(?:-[a-z0-9]+)*$/
const IPV4_OCTETS_PATTERN = /^(\d{1,3})\.(\d{1,3})\.(\d{1,3})\.(\d{1,3})$/
const E164_PHONE_PATTERN = /^\+[1-9]\d{7,14}$/
const INTEGER_DIGITS_PATTERN = /^\d+$/
const VERIFICATION_CODE_CHARACTERS_PATTERN = /^[\d\s-]+$/
// eslint-disable-next-line no-control-regex
const SINGLE_LINE_CONTROL_PATTERN = /[\r\n\t\x00-\x1f\x7f]/
// eslint-disable-next-line no-control-regex
const MULTILINE_CONTROL_PATTERN = /[\x00-\x08\x0b\x0c\x0e-\x1f\x7f]/

export interface TextFieldConstraints {
  minLength?: number
  maxLength: number
}

export interface IntegerFieldConstraints {
  min: number
  max: number
}

function normalizePhoneCandidate(rawPhone: string): string {
  return normalizeUnicodeDigits(rawPhone).trim().replace(/[\s()-]/g, '')
}

function isValidIpv4Address(rawAddress: string): boolean {
  const match = IPV4_OCTETS_PATTERN.exec(rawAddress)
  if (!match) {
    return false
  }
  return match.slice(1, 5).every((octetText) => {
    if (octetText.length > 1 && octetText.startsWith('0')) {
      return false
    }
    const octetValue = Number(octetText)
    return octetValue >= 0 && octetValue <= 255
  })
}

/**
 * Validates a required trimmed text field with minimum and maximum bounds.
 */
export function createRequiredTextField(
  translate: ValidationTranslator,
  constraints: TextFieldConstraints,
) {
  const minimum = constraints.minLength ?? MINIMUM_NAME_LENGTH
  return z
    .string({ error: translate('validation.required') })
    .transform((inputValue) => inputValue.trim())
    .pipe(
      z
        .string()
        .min(1, { error: translate('validation.required') })
        .min(minimum, { error: translate('validation.minLength', { min: minimum }) })
        .max(constraints.maxLength, { error: translate('validation.maxLength', { max: constraints.maxLength }) })
        .refine((textValue) => !SINGLE_LINE_CONTROL_PATTERN.test(textValue), {
          error: translate('validation.invalidControlCharacters'),
        }),
    )
}

/**
 * Validates a required trimmed English-only printable ASCII field with bounded length.
 */
export function createEnglishTextField(
  translate: ValidationTranslator,
  constraints: TextFieldConstraints,
) {
  const minimum = constraints.minLength ?? MINIMUM_NAME_LENGTH
  return z
    .string({ error: translate('validation.required') })
    .transform((inputValue) => inputValue.trim())
    .pipe(
      z
        .string()
        .min(1, { error: translate('validation.required') })
        .refine(isEnglishAsciiText, { error: translate('validation.englishOnly') })
        .refine((textValue) => !SINGLE_LINE_CONTROL_PATTERN.test(textValue), {
          error: translate('validation.invalidControlCharacters'),
        })
        .min(minimum, { error: translate('validation.minLength', { min: minimum }) })
        .max(constraints.maxLength, { error: translate('validation.maxLength', { max: constraints.maxLength }) }),
    )
}

/**
 * Validates an optional trimmed text field enforcing only a maximum character limit.
 */
export function createOptionalTextField(
  translate: ValidationTranslator,
  maxLength: number,
) {
  return z
    .string()
    .transform((inputValue) => inputValue.replace(/\r\n/g, '\n').trim())
    .pipe(
      z
        .string()
        .max(maxLength, { error: translate('validation.maxLength', { max: maxLength }) })
        .refine((textValue) => !MULTILINE_CONTROL_PATTERN.test(textValue), {
          error: translate('validation.invalidControlCharacters'),
        }),
    )
}

/**
 * Validates an optional bounded search keyword matching backend search validation.
 */
export function createSearchQueryField(translate: ValidationTranslator) {
  return z
    .string()
    .optional()
    .default('')
    .transform((inputValue) => inputValue.trim())
    .pipe(
      z
        .string()
        .max(MAXIMUM_SEARCH_QUERY_LENGTH, {
          error: translate('validation.maxLength', { max: MAXIMUM_SEARCH_QUERY_LENGTH }),
        })
        .refine((queryValue) => !SINGLE_LINE_CONTROL_PATTERN.test(queryValue), {
          error: translate('validation.invalidControlCharacters'),
        }),
    )
}

/**
 * Strictly validates an English lowercase hyphenated slug for Workspace, Project, and Channel identifiers.
 */
export function createSlugField(translate: ValidationTranslator) {
  return z
    .string({ error: translate('validation.required') })
    .transform((inputValue) => inputValue.trim())
    .pipe(
      z
        .string()
        .min(1, { error: translate('validation.required') })
        .refine(isEnglishAsciiText, { error: translate('validation.englishOnly') })
        .min(MINIMUM_SLUG_LENGTH, { error: translate('validation.minLength', { min: MINIMUM_SLUG_LENGTH }) })
        .max(MAXIMUM_SLUG_LENGTH, { error: translate('validation.maxLength', { max: MAXIMUM_SLUG_LENGTH }) })
        .regex(SLUG_PATTERN, { error: translate('validation.invalidSlug') }),
    )
}

/**
 * Validates an optional slug field that allows blank input while propagating strict slug errors when non-empty.
 */
export function createOptionalSlugField(translate: ValidationTranslator) {
  const requiredSlugSchema = createSlugField(translate)
  return z
    .string()
    .optional()
    .default('')
    .transform((inputValue) => inputValue.trim())
    .superRefine((trimmedValue, refinementContext) => {
      if (trimmedValue.length === 0) {
        return
      }
      const parsed = requiredSlugSchema.safeParse(trimmedValue)
      if (!parsed.success) {
        for (const issue of parsed.error.issues) {
          refinementContext.addIssue({
            code: 'custom',
            message: issue.message,
          })
        }
      }
    })
}

/**
 * Validates a bounded integer accepting English, Persian, Arabic, and Hindi digits and outputs a JS number.
 */
export function createBoundedIntegerField(
  translate: ValidationTranslator,
  constraints: IntegerFieldConstraints,
  rangeErrorKey = 'validation.integerRange',
) {
  return z
    .union([z.string(), z.number()], { error: translate('validation.required') })
    .transform((rawValue) => normalizeUnicodeDigits(String(rawValue).trim()))
    .pipe(
      z
        .string()
        .min(1, { error: translate('validation.required') })
        .regex(INTEGER_DIGITS_PATTERN, { error: translate('validation.invalidInteger') }),
    )
    .transform((digitsText) => Number.parseInt(digitsText, 10))
    .refine(
      (numericValue) => numericValue >= constraints.min && numericValue <= constraints.max,
      { error: translate(rangeErrorKey, { min: constraints.min, max: constraints.max }) },
    )
}

/**
 * Validates an unprivileged local TCP port between 1024 and 65535.
 */
export function createTcpPortField(translate: ValidationTranslator) {
  return createBoundedIntegerField(
    translate,
    { min: MINIMUM_TCP_PORT, max: MAXIMUM_TCP_PORT },
    'validation.invalidPortRange',
  )
}

/**
 * Validates an IPv4 address accepting multi-script digits and normalizing output to English ASCII.
 */
export function createIpv4AddressField(translate: ValidationTranslator) {
  return z
    .string({ error: translate('validation.required') })
    .transform((inputValue) => normalizeUnicodeDigits(inputValue).trim())
    .pipe(
      z
        .string()
        .min(1, { error: translate('validation.required') })
        .refine(isEnglishAsciiText, { error: translate('validation.englishOnly') })
        .refine(isValidIpv4Address, { error: translate('validation.invalidIpv4Address') }),
    )
}

/**
 * Validates an E.164 international phone number accepting multi-script digits and normalizing to ASCII.
 */
export function createPhoneNumberField(translate: ValidationTranslator) {
  return z
    .string({ error: translate('validation.required') })
    .transform(normalizePhoneCandidate)
    .pipe(
      z
        .string()
        .min(1, { error: translate('validation.required') })
        .refine(isEnglishAsciiText, { error: translate('validation.englishOnly') })
        .regex(E164_PHONE_PATTERN, { error: translate('validation.invalidPhoneE164') }),
    )
}

/**
 * Validates a 6-digit verification code accepting multi-script digits and outputting ASCII digits.
 */
export function createVerificationCodeField(translate: ValidationTranslator) {
  const exactLengthError = translate('validation.invalidVerificationCode', {
    digits: MINIMUM_VERIFICATION_CODE_DIGITS,
  })

  return z
    .string({ error: translate('validation.required') })
    .transform((inputValue) => normalizeUnicodeDigits(inputValue).trim())
    .pipe(
      z
        .string()
        .min(1, { error: translate('validation.required') })
        .refine(isEnglishAsciiText, { error: translate('validation.englishOnly') })
        .regex(VERIFICATION_CODE_CHARACTERS_PATTERN, { error: exactLengthError }),
    )
    .transform((normalizedCode) => normalizedCode.replace(/[\s-]+/g, ''))
    .pipe(
      z
        .string()
        .length(MINIMUM_VERIFICATION_CODE_DIGITS, { error: exactLengthError }),
    )
}

/**
 * Validates that a non-empty textarea value contains a valid JSON object or array.
 */
export function createJsonPayloadField(translate: ValidationTranslator) {
  return z
    .string({ error: translate('validation.required') })
    .transform((inputValue) => inputValue.trim())
    .pipe(
      z
        .string()
        .min(1, { error: translate('validation.required') })
        .refine(
          (jsonCandidate) => {
            try {
              const parsed: unknown = JSON.parse(jsonCandidate)
              return typeof parsed === 'object' && parsed !== null
            }
            catch {
              return false
            }
          },
          { error: translate('validation.invalidJsonObject') },
        ),
    )
}

/**
 * Validates an English ASCII HTTP/HTTPS webhook target URL without whitespace or non-ASCII characters.
 */
export function createWebhookUrlField(translate: ValidationTranslator) {
  return z
    .string({ error: translate('validation.required') })
    .transform((inputValue) => inputValue.trim())
    .pipe(
      z
        .string()
        .min(1, { error: translate('validation.required') })
        .refine(isEnglishAsciiText, { error: translate('validation.englishOnly') })
        .refine(hasOnlyAsciiNonWhitespace, { error: translate('validation.invalidUrl') })
        .url({ error: translate('validation.invalidUrl') })
        .refine(
          (urlValue) => urlValue.startsWith('http://') || urlValue.startsWith('https://'),
          { error: translate('validation.invalidHttpScheme') },
        ),
    )
}
