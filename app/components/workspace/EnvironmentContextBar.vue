<script setup lang="ts">
import type { DropdownMenuItem } from '@nuxt/ui'

const props = withDefaults(
  defineProps<{
    isCollapsed?: boolean
  }>(),
  {
    isCollapsed: false,
  },
)

const { t } = useI18n()
const toast = useToast()
const {
  activeProject,
  environments,
  activeEnvironment,
  isInitializing: isWorkspaceInitializing,
  selectEnvironment,
  setDefaultEnvironment,
} = useWorkspaceContext()

const environmentMenuItems = computed<DropdownMenuItem[][]>(() => {
  const selectionItems: DropdownMenuItem[] = environments.value.map((environment) => {
    const isActive = environment.id === activeEnvironment.value?.id
    const defaultSuffix = environment.isDefault ? ` (${t('workspace.environment.defaultBadge')})` : ''
    return {
      label: `${t(`workspace.environments.${environment.kind}`)}${defaultSuffix}`,
      icon: isActive ? 'i-lucide-check-circle-2' : 'i-lucide-layers',
      onSelect: async () => {
        await selectEnvironment(environment.id)
      },
    }
  })

  if (!activeEnvironment.value || activeEnvironment.value.isDefault) {
    return [selectionItems]
  }

  return [
    selectionItems,
    [
      {
        label: t('workspace.environment.setAsDefault'),
        icon: 'i-lucide-star',
        onSelect: async () => {
          if (!activeEnvironment.value) {
            return
          }
          await setDefaultEnvironment(activeEnvironment.value.id)
          toast.add({
            title: t('workspace.environment.defaultUpdatedTitle'),
            description: t(`workspace.environments.${activeEnvironment.value.kind}`),
            color: 'success',
            icon: 'i-lucide-star',
          })
        },
      },
    ],
  ]
})

const activeEnvironmentLabel = computed(() => {
  if (!activeEnvironment.value) {
    return t('workspace.environments.local')
  }
  return t(`workspace.environments.${activeEnvironment.value.kind}`)
})
</script>

<template>
  <div v-if="isWorkspaceInitializing" class="w-full">
    <USkeleton
      v-if="props.isCollapsed"
      class="mx-auto size-8 rounded-md"
    />
    <div
      v-else
      class="flex w-full items-center justify-between rounded-md border border-default/60 bg-default/50 px-2.5 py-1.5"
    >
      <span class="flex items-center gap-1.5">
        <USkeleton class="size-3.5 rounded" />
        <USkeleton class="h-3.5 w-16 rounded" />
      </span>
      <span class="flex items-center gap-1">
        <USkeleton class="h-4 w-12 rounded-full" />
        <USkeleton class="size-3.5 rounded" />
      </span>
    </div>
  </div>

  <div v-else-if="activeProject" class="w-full">
    <UDropdownMenu :items="environmentMenuItems" :content="{ side: 'bottom', align: 'start' }">
      <UTooltip
        v-if="props.isCollapsed"
        :text="`${t('workspace.activeEnvironment')}: ${activeEnvironmentLabel}`"
      >
        <UButton
          color="neutral"
          variant="ghost"
          square
          icon="i-lucide-layers"
          class="mx-auto flex size-8 items-center justify-center rounded-md text-primary"
          :aria-label="t('workspace.activeEnvironment')"
        />
      </UTooltip>

      <UButton
        v-else
        color="neutral"
        variant="ghost"
        size="xs"
        block
        class="justify-between rounded-md border border-default/60 bg-default/50 px-2.5 py-1.5 text-toned hover:bg-elevated"
      >
        <span class="flex min-w-0 items-center gap-1.5">
          <UIcon name="i-lucide-layers" class="size-3.5 shrink-0 text-primary" />
          <span class="truncate text-xs font-medium">{{ activeEnvironmentLabel }}</span>
        </span>
        <span class="flex items-center gap-1">
          <UBadge v-if="activeEnvironment?.isDefault" color="primary" variant="subtle" size="xs">
            {{ t('workspace.environment.defaultBadge') }}
          </UBadge>
          <UIcon name="i-lucide-chevron-down" class="size-3.5 text-muted" />
        </span>
      </UButton>
    </UDropdownMenu>
  </div>
</template>
