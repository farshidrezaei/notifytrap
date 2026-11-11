/**
 * Matches ASCII (0-9), Persian (۰-۹), Arabic-Indic (٠-٩), or Hindi Devanagari (०-९) decimal digits.
 */
export const UNICODE_DIGIT_CHARACTER_PATTERN = /[0-9\u06F0-\u06F9\u0660-\u0669\u0966-\u096F]/

const PERSIAN_DIGIT_ZERO_CODE = 0x06f0
const ARABIC_DIGIT_ZERO_CODE = 0x0660
const HINDI_DIGIT_ZERO_CODE = 0x0966

// eslint-disable-next-line no-control-regex
const ENGLISH_ASCII_PRINTABLE_PATTERN = /^[\x20-\x7E]+$/
// eslint-disable-next-line no-control-regex
const ENGLISH_ASCII_NON_WHITESPACE_PATTERN = /^[\x21-\x7E]+$/

/**
 * Converts Persian, Arabic-Indic, and Hindi decimal digits into standard ASCII English digits (0-9).
 */
export function normalizeUnicodeDigits(inputValue: string): string {
  let normalized = ''

  for (const character of inputValue) {
    const codePoint = character.codePointAt(0)
    if (codePoint === undefined) {
      continue
    }

    if (codePoint >= PERSIAN_DIGIT_ZERO_CODE && codePoint <= PERSIAN_DIGIT_ZERO_CODE + 9) {
      normalized += String(codePoint - PERSIAN_DIGIT_ZERO_CODE)
      continue
    }
    if (codePoint >= ARABIC_DIGIT_ZERO_CODE && codePoint <= ARABIC_DIGIT_ZERO_CODE + 9) {
      normalized += String(codePoint - ARABIC_DIGIT_ZERO_CODE)
      continue
    }
    if (codePoint >= HINDI_DIGIT_ZERO_CODE && codePoint <= HINDI_DIGIT_ZERO_CODE + 9) {
      normalized += String(codePoint - HINDI_DIGIT_ZERO_CODE)
      continue
    }

    normalized += character
  }

  return normalized
}

/**
 * Returns true when the non-empty string consists solely of printable ASCII English characters.
 */
export function isEnglishAsciiText(inputValue: string): boolean {
  return ENGLISH_ASCII_PRINTABLE_PATTERN.test(inputValue)
}

/**
 * Returns true when the non-empty string consists solely of printable non-whitespace ASCII characters.
 */
export function hasOnlyAsciiNonWhitespace(inputValue: string): boolean {
  return ENGLISH_ASCII_NON_WHITESPACE_PATTERN.test(inputValue)
}
