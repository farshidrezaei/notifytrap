<script setup lang="ts">
const { navigationGroups } = useNavigationItems()
const { refreshHealth } = useApplicationHealth()
const {
  activeWorkspaceId,
  activeProjectId,
  activeEnvironmentId,
  initializeWorkspace,
} = useWorkspaceContext()
const { refreshChannels } = useChannels()
const { refreshInbox, ensureLiveSubscription } = useNotifications()
const { runBackgroundUpdateCheck } = useAppUpdater()
const { loadDesktopPreferences } = useDesktopPreferences()
const { loadSavedFilters } = useSavedFilters()

const isSidebarOpen = ref(true)
const isCommandPaletteOpen = ref(false)

function handleGlobalKeydown(event: KeyboardEvent): void {
  if ((event.ctrlKey || event.metaKey) && event.key.toLowerCase() === 'k') {
    event.preventDefault()
    isCommandPaletteOpen.value = !isCommandPaletteOpen.value
  }
}

watch(
  [activeWorkspaceId, activeProjectId, activeEnvironmentId],
  async () => {
    await Promise.all([refreshChannels(), refreshInbox(), loadSavedFilters()])
  },
)

onMounted(async () => {
  window.addEventListener('keydown', handleGlobalKeydown)
  await Promise.all([refreshHealth(), initializeWorkspace(), loadDesktopPreferences()])
  await Promise.all([refreshChannels(), refreshInbox(), loadSavedFilters(), ensureLiveSubscription()])
  void runBackgroundUpdateCheck()
})

onBeforeUnmount(() => {
  window.removeEventListener('keydown', handleGlobalKeydown)
})
</script>

<template>
  <div class="flex h-screen w-screen flex-col overflow-hidden bg-muted">
    <LayoutDesktopAppBar
      v-model:is-sidebar-open="isSidebarOpen"
      @open-command-palette="isCommandPaletteOpen = true"
    />

    <div class="relative flex min-h-0 flex-1 overflow-hidden">
      <USidebar
        v-model:open="isSidebarOpen"
        variant="inset"
        collapsible="icon"
        breakpoint="md"
        rail
        :ui="{
          root: 'peer [--sidebar-width:var(--nt-sidebar-width)] [--sidebar-width-icon:var(--nt-sidebar-collapsed-width)]',
          container: 'md:absolute inset-y-0 h-full py-2 border-transparent',
          inner: 'divide-transparent',
          header: 'flex-col items-stretch gap-2 px-2.5 pt-2.5 pb-1',
          body: 'px-2 py-1',
          footer: 'hidden',
          rail: 'inset-y-2',
        }"
      >
        <template #header="{ state }">
          <WorkspaceProjectSwitcher :is-collapsed="state === 'collapsed'" />
          <WorkspaceEnvironmentContextBar :is-collapsed="state === 'collapsed'" />
        </template>

        <template #default="{ state }">
          <UNavigationMenu
            :collapsed="state === 'collapsed'"
            :items="navigationGroups[0]"
            orientation="vertical"
            tooltip
          />

          <UNavigationMenu
            :collapsed="state === 'collapsed'"
            :items="navigationGroups[1]"
            orientation="vertical"
            tooltip
            class="mt-auto"
          />
        </template>
      </USidebar>

      <main
        class="m-2 flex min-w-0 flex-1 flex-col overflow-hidden rounded-[var(--nt-radius-lg)] bg-[var(--nt-surface)] ring ring-default md:ms-0"
        style="box-shadow: var(--nt-shadow-sm);"
      >
        <div class="flex-1 overflow-y-auto p-6">
          <slot />
        </div>
      </main>
    </div>

    <LayoutDesktopStatusFooter />
    <SharedCommandPaletteModal v-model:open="isCommandPaletteOpen" />
  </div>
</template>
