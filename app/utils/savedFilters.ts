import type { ChannelType, NotificationStatus } from '~/types/application'

export interface SavedInboxFilter {
  id: string
  name: string
  channelType: ChannelType | 'all'
  status: NotificationStatus | 'all'
  testRunIdentifier: string
  searchQuery: string
}

const VALID_CHANNEL_TYPES = new Set<string>(['all', 'email', 'sms', 'push', 'webhook'])
const VALID_STATUSES = new Set<string>(['all', 'captured', 'read', 'archived'])

/**
 * Safely parses persisted JSON saved filters while discarding malformed entries.
 */
export function parseSavedInboxFilters(rawJson: string | null | undefined): SavedInboxFilter[] {
  if (!rawJson || !rawJson.trim()) {
    return []
  }
  try {
    const parsed: unknown = JSON.parse(rawJson)
    if (!Array.isArray(parsed)) {
      return []
    }
    return parsed
      .filter((entry): entry is Record<string, unknown> => Boolean(entry) && typeof entry === 'object')
      .map((entry) => ({
        id: typeof entry.id === 'string' ? entry.id.trim() : '',
        name: typeof entry.name === 'string' ? entry.name.trim() : '',
        channelType:
          typeof entry.channelType === 'string' && VALID_CHANNEL_TYPES.has(entry.channelType)
            ? (entry.channelType as ChannelType | 'all')
            : 'all',
        status:
          typeof entry.status === 'string' && VALID_STATUSES.has(entry.status)
            ? (entry.status as NotificationStatus | 'all')
            : 'all',
        testRunIdentifier: typeof entry.testRunIdentifier === 'string' ? entry.testRunIdentifier : '',
        searchQuery: typeof entry.searchQuery === 'string' ? entry.searchQuery : '',
      }))
      .filter((entry) => entry.id.length > 0 && entry.name.length > 0)
  } catch {
    return []
  }
}

/**
 * Builds the application setting key scoped to a specific project's saved inbox filters.
 */
export function buildSavedFiltersSettingKey(projectId: string): string {
  return `saved_filters.${projectId}`
}
