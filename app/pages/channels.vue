<script setup lang="ts">
const { t } = useI18n()
const {
  activeWorkspace,
  activeProject,
  activeEnvironment,
  hasActiveScope,
  isInitializing: isWorkspaceInitializing,
} = useWorkspaceContext()
const { channels, isLoading, refreshChannels } = useChannels()

const isCreateModalOpen = ref(false)
</script>

<template>
  <div class="mx-auto flex max-w-5xl flex-col gap-6">
    <SharedSectionHeader
      :title="t('channels.title')"
      :description="
        hasActiveScope && activeWorkspace && activeProject && activeEnvironment
          ? t('channels.scopedSubtitle', {
              workspace: activeWorkspace.name,
              project: activeProject.name,
              environment: t(`workspace.environments.${activeEnvironment.kind}`),
            })
          : t('channels.subtitle')
      "
    >
      <template #actions>
        <div class="flex items-center gap-2">
          <UButton
            size="sm"
            color="neutral"
            variant="outline"
            icon="i-lucide-refresh-cw"
            :loading="isLoading"
            :disabled="!hasActiveScope"
            :label="t('common.refresh')"
            @click="refreshChannels()"
          />
          <UButton
            size="sm"
            color="primary"
            icon="i-lucide-plus"
            :disabled="!hasActiveScope"
            :label="t('channels.actions.addChannel')"
            @click="isCreateModalOpen = true"
          />
        </div>
      </template>
    </SharedSectionHeader>

    <SkeletonChannelCardList v-if="isWorkspaceInitializing || isLoading" />

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

    <UEmpty
      v-else-if="channels.length === 0"
      icon="i-lucide-radio"
      :title="t('channels.emptyTitle')"
      :description="t('channels.emptyDescription')"
    >
      <template #actions>
        <UButton
          color="primary"
          icon="i-lucide-plus"
          :label="t('channels.actions.addChannel')"
          @click="isCreateModalOpen = true"
        />
      </template>
    </UEmpty>

    <div v-else class="space-y-4">
      <ChannelsChannelCredentialCard
        v-for="channel in channels"
        :key="channel.id"
        :channel="channel"
      />
    </div>

    <ChannelsCreateChannelModal v-model:open="isCreateModalOpen" />
  </div>
</template>
