<script setup lang="ts">
import {
  createInboxFilterFormSchema,
  NOTIFICATION_CHANNEL_KINDS,
} from '~/schemas/workspaceForms'
import type { ChannelType, NotificationStatus } from '~/types/application'
import { INPUT_MASKS } from '~/utils/inputMasks'
import { extractCommandErrorMessage } from '~/utils/tauriClient'

const { t } = useI18n()
const toast = useToast()
const { hasActiveScope } = useWorkspaceContext()
const {
  filterChannelType,
  filterStatus,
  filterSearchQuery,
  currentPage,
  isLoadingList,
  refreshInbox,
  clearScopeNotifications,
} = useNotifications()

const filterSchema = computed(() => createInboxFilterFormSchema(t))
const filterFormState = reactive<{
  channelType: ChannelType | 'all'
  status: NotificationStatus | 'all'
  searchQuery: string
}>({
  channelType: filterChannelType.value,
  status: filterStatus.value,
  searchQuery: filterSearchQuery.value,
})

const isClearConfirmOpen = ref(false)
const isClearing = ref(false)

const channelFilters = computed<Array<{ label: string; value: ChannelType | 'all' }>>(() => [
  { label: t('inbox.filters.allChannels'), value: 'all' },
  ...NOTIFICATION_CHANNEL_KINDS.map((kind) => ({
    label: t(`channels.types.${kind}`),
    value: kind,
  })),
])

const statusSelectOptions = computed<Array<{ label: string; value: NotificationStatus | 'all' }>>(() => [
  { label: t('inbox.filters.allStatuses'), value: 'all' },
  { label: t('inbox.status.captured'), value: 'captured' },
  { label: t('inbox.status.read'), value: 'read' },
  { label: t('inbox.status.archived'), value: 'archived' },
])

async function handleSelectChannel(channelType: ChannelType | 'all'): Promise<void> {
  filterFormState.channelType = channelType
  filterChannelType.value = channelType
  currentPage.value = 1
  await refreshInbox()
}

async function handleStatusChange(): Promise<void> {
  const parsed = filterSchema.value.safeParse(filterFormState)
  if (!parsed.success) {
    return
  }
  filterStatus.value = parsed.data.status
  currentPage.value = 1
  await refreshInbox()
}

let searchDebounceTimer: ReturnType<typeof setTimeout> | undefined
watch(
  () => filterFormState.searchQuery,
  () => {
    const parsed = filterSchema.value.safeParse(filterFormState)
    if (!parsed.success) {
      return
    }
    if (searchDebounceTimer) {
      clearTimeout(searchDebounceTimer)
    }
    searchDebounceTimer = setTimeout(() => {
      filterSearchQuery.value = parsed.data.searchQuery
      currentPage.value = 1
      void refreshInbox()
    }, 220)
  },
)

onBeforeUnmount(() => {
  if (searchDebounceTimer) {
    clearTimeout(searchDebounceTimer)
  }
})

async function handleConfirmClear(): Promise<void> {
  isClearing.value = true
  try {
    const deletedCount = await clearScopeNotifications()
    isClearConfirmOpen.value = false
    toast.add({
      title: t('inbox.toast.clearedTitle'),
      description: t('inbox.toast.clearedDescription', { count: deletedCount }),
      color: 'success',
      icon: 'i-lucide-trash-2',
    })
  } catch (error) {
    toast.add({
      title: t('common.errorTitle'),
      description: extractCommandErrorMessage(error, t('common.unexpectedError')),
      color: 'error',
      icon: 'i-lucide-alert-circle',
    })
  } finally {
    isClearing.value = false
  }
}
</script>

<template>
  <div class="flex flex-col gap-3 rounded-lg border border-default bg-muted/30 p-3">
    <UForm
      :schema="filterSchema"
      :state="filterFormState"
      :validate-on="['blur', 'change', 'input']"
      class="flex flex-wrap items-start justify-between gap-2"
    >
      <div class="flex flex-wrap items-center gap-1 pt-0.5">
        <UButton
          v-for="option in channelFilters"
          :key="option.value"
          size="xs"
          :color="filterChannelType === option.value ? 'primary' : 'neutral'"
          :variant="filterChannelType === option.value ? 'soft' : 'ghost'"
          :label="option.label"
          @click="handleSelectChannel(option.value)"
        />
      </div>

      <div class="flex flex-wrap items-start gap-2">
        <UFormField name="status">
          <USelect
            v-model="filterFormState.status"
            :items="statusSelectOptions"
            value-key="value"
            size="xs"
            class="w-36"
            @update:model-value="handleStatusChange"
          />
        </UFormField>

        <UFormField name="searchQuery">
          <UInput
            v-model="filterFormState.searchQuery"
            v-maska="INPUT_MASKS.searchKeyword"
            :maxlength="64"
            size="xs"
            icon="i-lucide-search"
            class="w-56"
            :placeholder="t('inbox.filters.searchPlaceholder')"
          />
        </UFormField>

        <UTooltip :text="t('common.refresh')">
          <UButton
            size="xs"
            color="neutral"
            variant="outline"
            square
            icon="i-lucide-refresh-cw"
            :loading="isLoadingList"
            :disabled="!hasActiveScope"
            :aria-label="t('common.refresh')"
            @click="refreshInbox()"
          />
        </UTooltip>

        <UTooltip :text="t('inbox.actions.clearInbox')">
          <UButton
            size="xs"
            color="error"
            variant="ghost"
            square
            icon="i-lucide-trash-2"
            :disabled="!hasActiveScope"
            :aria-label="t('inbox.actions.clearInbox')"
            @click="isClearConfirmOpen = true"
          />
        </UTooltip>
      </div>
    </UForm>

    <UModal
      v-model:open="isClearConfirmOpen"
      :title="t('inbox.clearModal.title')"
      :description="t('inbox.clearModal.description')"
    >
      <template #body>
        <div class="flex justify-end gap-2">
          <UButton
            color="neutral"
            variant="ghost"
            :label="t('common.cancel')"
            @click="isClearConfirmOpen = false"
          />
          <UButton
            color="error"
            :loading="isClearing"
            :label="t('inbox.actions.clearInbox')"
            @click="handleConfirmClear"
          />
        </div>
      </template>
    </UModal>
  </div>
</template>
