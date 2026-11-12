import type { ApplicationSettingEntry } from '~/types/application'
import {
  buildSavedFiltersSettingKey,
  parseSavedInboxFilters,
  type SavedInboxFilter,
} from '~/utils/savedFilters'
import { invokeTauriCommand, isTauriRuntime } from '~/utils/tauriClient'

/**
 * Persists and manages named saved search filters scoped to the active project.
 */
export function useSavedFilters() {
  const { activeProjectId } = useWorkspaceContext()
  const savedFilters = useState<SavedInboxFilter[]>('inbox-saved-filters', () => [])

  async function loadSavedFilters(): Promise<void> {
    if (!activeProjectId.value || !isTauriRuntime()) {
      savedFilters.value = []
      return
    }
    const entry = await invokeTauriCommand<ApplicationSettingEntry | null>('get_application_setting', {
      key: buildSavedFiltersSettingKey(activeProjectId.value),
    })
    savedFilters.value = parseSavedInboxFilters(typeof entry?.value === 'string' ? entry.value : null)
  }

  async function persistSavedFilters(nextFilters: SavedInboxFilter[]): Promise<void> {
    savedFilters.value = nextFilters
    if (!activeProjectId.value || !isTauriRuntime()) {
      return
    }
    await invokeTauriCommand<ApplicationSettingEntry>('set_application_setting', {
      key: buildSavedFiltersSettingKey(activeProjectId.value),
      value: JSON.stringify(nextFilters),
    })
  }

  async function saveFilter(filterInput: Omit<SavedInboxFilter, 'id'>): Promise<void> {
    const trimmedName = filterInput.name.trim()
    if (!trimmedName) {
      return
    }
    const nextEntry: SavedInboxFilter = {
      ...filterInput,
      id: `filter-${Date.now().toString(36)}`,
      name: trimmedName,
    }
    await persistSavedFilters([nextEntry, ...savedFilters.value.filter((item) => item.name !== trimmedName)].slice(0, 12))
  }

  async function deleteSavedFilter(filterId: string): Promise<void> {
    await persistSavedFilters(savedFilters.value.filter((item) => item.id !== filterId))
  }

  return {
    savedFilters: readonly(savedFilters),
    loadSavedFilters,
    saveFilter,
    deleteSavedFilter,
  }
}
