import fs from 'node:fs'
import path from 'node:path'

const LOCALES_DIRECTORY = path.resolve(process.cwd(), 'i18n/locales')
const BASE_LOCALE = 'en'

/**
 * Flattens a nested locale dictionary into dotted leaf key -> string pairs
 * while flagging empty values and structural mismatches.
 */
export function flattenLocaleDictionary(node, prefix = '', errors = []) {
  const entries = {}

  if (typeof node !== 'object' || node === null || Array.isArray(node)) {
    errors.push(`Invalid locale node at "${prefix || '<root>'}": expected object.`)
    return entries
  }

  for (const [key, value] of Object.entries(node)) {
    const fullKey = prefix ? `${prefix}.${key}` : key

    if (typeof value === 'string') {
      if (value.trim().length === 0) {
        errors.push(`Empty translation string at "${fullKey}".`)
      }
      entries[fullKey] = value
    } else if (typeof value === 'object' && value !== null && !Array.isArray(value)) {
      Object.assign(entries, flattenLocaleDictionary(value, fullKey, errors))
    } else {
      errors.push(`Unsupported value type at "${fullKey}": expected string or object.`)
    }
  }

  return entries
}

/**
 * Extracts interpolation variable names like `{version}` from a translation string.
 */
export function extractPlaceholders(text) {
  const matches = text.match(/\{([a-zA-Z0-9_]+)\}/g) ?? []
  return [...new Set(matches)].sort()
}

/**
 * Validates all locale JSON files in `i18n/locales` against the base locale.
 */
export function validateLocaleDirectory(directory = LOCALES_DIRECTORY) {
  const errors = []
  const files = fs
    .readdirSync(directory)
    .filter((file) => file.endsWith('.json'))
    .sort()

  if (!files.includes(`${BASE_LOCALE}.json`)) {
    return [`Missing base locale file "${BASE_LOCALE}.json" in ${directory}`]
  }

  const parsedLocales = {}
  for (const file of files) {
    const localeCode = file.replace(/\.json$/, '')
    const raw = fs.readFileSync(path.join(directory, file), 'utf8')
    let parsed
    try {
      parsed = JSON.parse(raw)
    } catch (error) {
      errors.push(`Invalid JSON in "${file}": ${error.message}`)
      continue
    }
    const localeErrors = []
    parsedLocales[localeCode] = flattenLocaleDictionary(parsed, '', localeErrors)
    for (const message of localeErrors) {
      errors.push(`[${localeCode}] ${message}`)
    }
  }

  const baseEntries = parsedLocales[BASE_LOCALE] ?? {}
  const baseKeys = Object.keys(baseEntries).sort()

  for (const [localeCode, entries] of Object.entries(parsedLocales)) {
    if (localeCode === BASE_LOCALE) continue

    for (const key of baseKeys) {
      if (!(key in entries)) {
        errors.push(`[${localeCode}] Missing translation key "${key}".`)
        continue
      }

      const expectedPlaceholders = extractPlaceholders(baseEntries[key])
      const actualPlaceholders = extractPlaceholders(entries[key])
      if (JSON.stringify(expectedPlaceholders) !== JSON.stringify(actualPlaceholders)) {
        errors.push(
          `[${localeCode}] Placeholder mismatch at "${key}": expected [${expectedPlaceholders.join(', ')}], got [${actualPlaceholders.join(', ')}].`,
        )
      }
    }

    for (const key of Object.keys(entries)) {
      if (!(key in baseEntries)) {
        errors.push(`[${localeCode}] Extra translation key "${key}" not present in ${BASE_LOCALE}.`)
      }
    }
  }

  return errors
}

if (import.meta.url === `file://${process.argv[1]}`) {
  const validationErrors = validateLocaleDirectory()
  if (validationErrors.length > 0) {
    console.error('i18n validation failed:')
    for (const err of validationErrors) {
      console.error(` - ${err}`)
    }
    process.exit(1)
  }
  console.log('i18n validation passed for all locales.')
}
