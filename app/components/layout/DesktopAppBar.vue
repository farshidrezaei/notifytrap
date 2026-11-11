<script setup lang="ts">
const isSidebarOpen = defineModel<boolean>('isSidebarOpen', { required: true })

const { t } = useI18n()
const route = useRoute()
const { locale, changeLocale } = useLocaleController()
const { isDesktopRuntime } = useApplicationHealth()
const { activeWorkspace, activeProject, hasActiveScope } = useWorkspaceContext()
const {
  isMaximized,
  isWindowControlsAvailable,
  minimizeWindow,
  toggleMaximizeWindow,
  closeWindow,
} = useWindowControls()

const ROUTE_TITLE_KEY_MAP: Record<string, string> = {
  '/': 'navigation.overview',
  '/inbox': 'navigation.inbox',
  '/workspaces': 'navigation.workspaces',
  '/projects': 'navigation.projects',
  '/channels': 'navigation.channels',
  '/settings': 'navigation.settings',
  '/design-system': 'navigation.designSystem',
}

const activePageTitle = computed(() => {
  const key = ROUTE_TITLE_KEY_MAP[route.path] ?? 'navigation.overview'
  return t(key)
})

const nextLocaleCode = computed(() => (locale.value === 'en' ? 'fa' : 'en'))
const nextLocaleShortLabel = computed(() => (locale.value === 'en' ? 'FA' : 'EN'))

function handleBarDoubleClick(event: MouseEvent): void {
  const target = event.target as HTMLElement | null
  if (target?.closest('button, a, input, select, [role="button"]')) {
    return
  }
  if (isWindowControlsAvailable.value) {
    void toggleMaximizeWindow()
  }
}
</script>

<template>
  <header
    data-tauri-drag-region
    class="flex h-11 shrink-0 select-none items-center justify-between gap-3 border-b border-default/60 bg-muted px-2.5"
    @dblclick="handleBarDoubleClick"
  >
    <!-- Start: Sidebar Toggle + Brand -->
    <div data-tauri-drag-region class="flex min-w-0 items-center gap-2">
      <UTooltip :text="t('window.toggleSidebar')">
        <UButton
          size="xs"
          color="neutral"
          variant="ghost"
          square
          :icon="isSidebarOpen ? 'i-lucide-panel-left-close' : 'i-lucide-panel-left-open'"
          :aria-label="t('window.toggleSidebar')"
          @click="isSidebarOpen = !isSidebarOpen"
        />
      </UTooltip>

      <div data-tauri-drag-region class="flex items-center gap-2">
        <SharedBrandLogo size="sm" is-decorative />
        <span data-tauri-drag-region class="text-xs font-semibold tracking-tight text-highlighted">
          {{ t('application.name') }}
        </span>
      </div>

      <span class="text-muted/60" aria-hidden="true">/</span>

      <template v-if="hasActiveScope && activeWorkspace && activeProject">
        <span data-tauri-drag-region class="hidden truncate text-xs text-muted md:inline">
          {{ activeWorkspace.name }}
        </span>
        <span class="hidden text-muted/60 md:inline" aria-hidden="true">/</span>
        <span data-tauri-drag-region class="hidden truncate text-xs font-medium text-toned sm:inline">
          {{ activeProject.name }}
        </span>
        <span class="hidden text-muted/60 sm:inline" aria-hidden="true">/</span>
      </template>

      <span data-tauri-drag-region class="truncate text-xs font-medium text-toned">
        {{ activePageTitle }}
      </span>
    </div>

    <!-- Center Drag Region -->
    <div data-tauri-drag-region class="hidden flex-1 sm:flex sm:items-center sm:justify-center">
      <UBadge
        :color="isDesktopRuntime ? 'success' : 'neutral'"
        variant="subtle"
        size="xs"
      >
        {{
          isDesktopRuntime
            ? t('application.status.desktopShellReady')
            : t('application.status.browserPreviewMode')
        }}
      </UBadge>
    </div>

    <!-- End: Quick Controls + Native Window Buttons -->
    <div class="flex shrink-0 items-center gap-1">
      <UTooltip :text="t('designSystem.controls.languageToggle')">
        <UButton
          size="xs"
          color="neutral"
          variant="ghost"
          icon="i-lucide-languages"
          :label="nextLocaleShortLabel"
          @click="changeLocale(nextLocaleCode)"
        />
      </UTooltip>

      <UColorModeButton size="xs" />

      <div class="mx-1 h-4 w-px bg-default" aria-hidden="true" />

      <div class="flex items-center gap-0.5" dir="ltr">
        <UTooltip :text="t('window.minimize')">
          <UButton
            size="xs"
            color="neutral"
            variant="ghost"
            square
            icon="i-lucide-minus"
            :disabled="!isWindowControlsAvailable"
            :aria-label="t('window.minimize')"
            @click="minimizeWindow"
          />
        </UTooltip>

        <UTooltip :text="isMaximized ? t('window.restore') : t('window.maximize')">
          <UButton
            size="xs"
            color="neutral"
            variant="ghost"
            square
            :icon="isMaximized ? 'i-lucide-copy' : 'i-lucide-square'"
            :disabled="!isWindowControlsAvailable"
            :aria-label="isMaximized ? t('window.restore') : t('window.maximize')"
            @click="toggleMaximizeWindow"
          />
        </UTooltip>

        <UTooltip :text="t('window.close')">
          <UButton
            size="xs"
            color="neutral"
            variant="ghost"
            square
            icon="i-lucide-x"
            class="hover:bg-error/15 hover:text-error"
            :disabled="!isWindowControlsAvailable"
            :aria-label="t('window.close')"
            @click="closeWindow"
          />
        </UTooltip>
      </div>
    </div>
  </header>
</template>
