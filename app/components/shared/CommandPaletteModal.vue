<script setup lang="ts">
import type { CommandPaletteGroup } from '@nuxt/ui'

const isOpen = defineModel<boolean>('open', { required: true })

const { t } = useI18n()
const router = useRouter()
const colorMode = useColorMode()
const { locale, changeLocale } = useLocaleController()
const { savedFilters } = useSavedFilters()
const {
  filterChannelType,
  filterStatus,
  filterTestRunIdentifier,
  filterSearchQuery,
  currentPage,
  refreshInbox,
} = useNotifications()

const searchTerm = ref('')

watch(isOpen, (opened) => {
  if (opened) {
    searchTerm.value = ''
  }
})

function closeAndRun(action: () => unknown): void {
  isOpen.value = false
  void action()
}

const commandGroups = computed<CommandPaletteGroup[]>(() => {
  const navigationItems = [
    { id: 'nav-overview', label: t('navigation.overview'), icon: 'i-lucide-layout-dashboard', onSelect: () => closeAndRun(() => router.push('/')) },
    { id: 'nav-inbox', label: t('navigation.inbox'), icon: 'i-lucide-inbox', onSelect: () => closeAndRun(() => router.push('/inbox')) },
    { id: 'nav-channels', label: t('navigation.channels'), icon: 'i-lucide-radio', onSelect: () => closeAndRun(() => router.push('/channels')) },
    { id: 'nav-projects', label: t('navigation.projects'), icon: 'i-lucide-folder-kanban', onSelect: () => closeAndRun(() => router.push('/projects')) },
    { id: 'nav-workspaces', label: t('navigation.workspaces'), icon: 'i-lucide-building-2', onSelect: () => closeAndRun(() => router.push('/workspaces')) },
    { id: 'nav-settings', label: t('navigation.settings'), icon: 'i-lucide-settings', onSelect: () => closeAndRun(() => router.push('/settings')) },
    { id: 'nav-design-system', label: t('navigation.designSystem'), icon: 'i-lucide-palette', onSelect: () => closeAndRun(() => router.push('/design-system')) },
  ]

  const actionItems = [
    {
      id: 'action-refresh-inbox',
      label: t('commandPalette.actions.refreshInbox'),
      icon: 'i-lucide-refresh-cw',
      onSelect: () => closeAndRun(() => refreshInbox()),
    },
    {
      id: 'action-toggle-theme',
      label: t('commandPalette.actions.toggleTheme'),
      icon: colorMode.value === 'dark' ? 'i-lucide-sun' : 'i-lucide-moon',
      onSelect: () => closeAndRun(() => {
        colorMode.preference = colorMode.value === 'dark' ? 'light' : 'dark'
      }),
    },
    {
      id: 'action-toggle-locale',
      label: t('commandPalette.actions.switchLanguage'),
      icon: 'i-lucide-languages',
      onSelect: () => closeAndRun(() => changeLocale(locale.value === 'en' ? 'fa' : 'en')),
    },
  ]

  const groups: CommandPaletteGroup[] = [
    {
      id: 'navigation',
      label: t('commandPalette.groups.navigation'),
      items: navigationItems,
    },
    {
      id: 'actions',
      label: t('commandPalette.groups.actions'),
      items: actionItems,
    },
  ]

  if (savedFilters.value.length > 0) {
    groups.push({
      id: 'saved-filters',
      label: t('commandPalette.groups.savedFilters'),
      items: savedFilters.value.map((savedFilter) => ({
        id: `filter-${savedFilter.id}`,
        label: savedFilter.name,
        suffix: savedFilter.searchQuery || savedFilter.channelType,
        icon: 'i-lucide-bookmark',
        onSelect: () => closeAndRun(async () => {
          filterChannelType.value = savedFilter.channelType
          filterStatus.value = savedFilter.status
          filterTestRunIdentifier.value = savedFilter.testRunIdentifier
          filterSearchQuery.value = savedFilter.searchQuery
          currentPage.value = 1
          await router.push('/inbox')
          await refreshInbox()
        }),
      })),
    })
  }

  return groups
})
</script>

<template>
  <UModal v-model:open="isOpen" :ui="{ content: 'sm:max-w-lg p-0 overflow-hidden' }">
    <template #content>
      <UCommandPalette
        v-model:search-term="searchTerm"
        :groups="commandGroups"
        :placeholder="t('commandPalette.placeholder')"
        :close="{ onClick: () => { isOpen = false } }"
        class="h-80"
      />
    </template>
  </UModal>
</template>
