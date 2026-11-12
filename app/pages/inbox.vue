<script setup lang="ts">
const { t } = useI18n()
const {
  activeWorkspace,
  activeProject,
  activeEnvironment,
  hasActiveScope,
  isInitializing: isWorkspaceInitializing,
} = useWorkspaceContext()
const {
  notificationPage,
  selectedDetail,
  selectNotification,
} = useNotifications()

function handleInboxKeydown(event: KeyboardEvent): void {
  if (event.ctrlKey || event.metaKey || event.altKey) {
    return
  }
  const target = event.target as HTMLElement | null
  if (target && (target.tagName === 'INPUT' || target.tagName === 'TEXTAREA' || target.tagName === 'SELECT' || target.isContentEditable)) {
    return
  }

  if (event.key === '/') {
    event.preventDefault()
    const input = document.getElementById('inbox-search-input') as HTMLInputElement | null
    input?.focus()
    return
  }

  const items = notificationPage.value?.items ?? []
  if (items.length === 0) {
    return
  }

  if (event.key === 'j' || event.key === 'k') {
    event.preventDefault()
    const currentId = selectedDetail.value?.notification.id
    const currentIndex = currentId ? items.findIndex((item) => item.id === currentId) : -1
    const nextIndex = event.key === 'j'
      ? Math.min(items.length - 1, currentIndex + 1)
      : Math.max(0, currentIndex <= 0 ? 0 : currentIndex - 1)
    const nextItem = items[nextIndex]
    if (nextItem) {
      void selectNotification(nextItem.id)
    }
  }
}

onMounted(() => {
  window.addEventListener('keydown', handleInboxKeydown)
})

onBeforeUnmount(() => {
  window.removeEventListener('keydown', handleInboxKeydown)
})
</script>

<template>
  <div class="mx-auto flex h-full min-h-0 max-w-7xl flex-col gap-4">
    <SharedSectionHeader
      :title="t('inbox.title')"
      :description="
        hasActiveScope && activeWorkspace && activeProject && activeEnvironment
          ? t('inbox.scopedSubtitle', {
              workspace: activeWorkspace.name,
              project: activeProject.name,
              environment: t(`workspace.environments.${activeEnvironment.kind}`),
            })
          : t('inbox.subtitle')
      "
    />

    <SkeletonInboxView v-if="isWorkspaceInitializing" />

    <UEmpty
      v-else-if="!hasActiveScope"
      icon="i-lucide-folder-plus"
      :title="t('workspace.noActiveScopeTitle')"
      :description="t('workspace.noActiveScopeDescription')"
    >
      <template #actions>
        <UButton color="primary" to="/" :label="t('navigation.overview')" />
      </template>
    </UEmpty>

    <template v-else>
      <InboxFilterBar />

      <div class="grid min-h-0 flex-1 gap-4 lg:grid-cols-12">
        <div class="min-h-[320px] lg:col-span-5 lg:min-h-0">
          <InboxMessageList />
        </div>
        <div class="min-h-[380px] lg:col-span-7 lg:min-h-0">
          <InboxNotificationInspector />
        </div>
      </div>
    </template>
  </div>
</template>
