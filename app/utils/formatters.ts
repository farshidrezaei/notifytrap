import { normalizeUnicodeDigits } from './numberNormalization'

/**
 * Converts a display name into a lowercase hyphenated slug identifier safe for form prefill.
 */
export function formatSlugIdentifier(displayName: string): string {
  return normalizeUnicodeDigits(displayName)
    .trim()
    .toLowerCase()
    .replace(/[^a-z0-9]+/g, '-')
    .replace(/^-+|-+$/g, '')
    .replace(/-{2,}/g, '-')
    .slice(0, 64)
    .replace(/^-+|-+$/g, '')
}

/**
 * Formats byte counts into compact human-readable size strings without external dependencies.
 */
export function formatByteSize(byteCount: number): string {
  if (!Number.isFinite(byteCount) || byteCount <= 0) {
    return '0 B'
  }
  const units = ['B', 'KB', 'MB', 'GB'] as const
  const unitIndex = Math.min(Math.floor(Math.log(byteCount) / Math.log(1024)), units.length - 1)
  const scaled = byteCount / 1024 ** unitIndex
  const decimals = unitIndex === 0 ? 0 : 1
  return `${scaled.toFixed(decimals)} ${units[unitIndex]}`
}

/**
 * Formats an ISO-8601 timestamp into a localized short date-time label.
 */
export function formatShortTimestamp(isoTimestamp: string, localeCode: string): string {
  const parsed = new Date(isoTimestamp)
  if (Number.isNaN(parsed.getTime())) {
    return isoTimestamp
  }
  return new Intl.DateTimeFormat(localeCode === 'fa' ? 'fa-IR' : 'en-US', {
    dateStyle: 'short',
    timeStyle: 'short',
  }).format(parsed)
}

/**
 * Computes up to two uppercase monogram initials for avatar badges.
 */
export function buildMonogramLabel(displayName: string | undefined): string {
  if (!displayName) {
    return 'NT'
  }
  const words = displayName.trim().split(/\s+/).filter(Boolean)
  if (words.length === 0) {
    return 'NT'
  }
  if (words.length === 1) {
    return words[0]!.slice(0, 2).toUpperCase()
  }
  return `${words[0]![0] ?? ''}${words[1]![0] ?? ''}`.toUpperCase()
}
