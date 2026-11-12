<script setup lang="ts">
import {
  createInboxFilterFormSchema,
  NOTIFICATION_CHANNEL_KINDS,
} from '~/schemas/workspaceForms'
import type { ChannelType, NotificationStatus } from '~/types/application'
import type { SavedInboxFilter } from '~/utils/savedFilters'
import { extractCommandErrorMessage } from '~/utils/tauriClient'

const { t } = useI18n()
const toast = useToast()
const { hasActiveScope } = useWorkspaceContext()
const {
  filterChannelType,
  filterStatus,
  filterTestRunIdentifier,
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
  testRunIdentifier: string
  searchQuery: string
}>({
  channelType: filterChannelType.value,
  status: filterStatus.value,
  testRunIdentifier: filterTestRunIdentifier.value,
  searchQuery: filterSearchQuery.value,
})

const isClearConfirmOpen = ref(false)
const isClearing = ref(false)
const isSavePopoverOpen = ref(false)
const newFilterName = ref('')
const { savedFilters, saveFilter, deleteSavedFilter } = useSavedFilters()

async function handleSaveCurrentFilter(): Promise<void> {
  if (!newFilterName.value.trim()) {
    return
  }
  await saveFilter({
    name: newFilterName.value,
    channelType: filterFormState.channelType,
    status: filterFormState.status,
    testRunIdentifier: filterFormState.testRunIdentifier,
    searchQuery: filterFormState.searchQuery,
  })
  newFilterName.value = ''
  isSavePopoverOpen.value = false
}

async function handleApplySavedFilter(saved: SavedInboxFilter): Promise<void> {
  filterFormState.channelType = saved.channelType
  filterFormState.status = saved.status
  filterFormState.testRunIdentifier = saved.testRunIdentifier
  filterFormState.searchQuery = saved.searchQuery
  filterChannelType.value = saved.channelType
  filterStatus.value = saved.status
  filterTestRunIdentifier.value = saved.testRunIdentifier
  filterSearchQuery.value = saved.searchQuery
  currentPage.value = 1
  await refreshInbox()
}

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
  () => [filterFormState.searchQuery, filterFormState.testRunIdentifier],
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
      filterTestRunIdentifier.value = parsed.data.testRunIdentifier
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

        <UFormField name="testRunIdentifier">
          <UInput
            v-model="filterFormState.testRunIdentifier"
            :maxlength="128"
            size="xs"
            icon="i-lucide-tag"
            class="w-40"
            :placeholder="t('inbox.filters.runTagPlaceholder')"
          />
        </UFormField>

        <UFormField name="searchQuery">
          <UInput
            id="inbox-search-input"
            v-model="filterFormState.searchQuery"
            :maxlength="160"
            size="xs"
            icon="i-lucide-search"
            class="w-60"
            :placeholder="t('inbox.filters.searchPlaceholder')"
          />
        </UFormField>

        <UPopover>
          <UTooltip :text="t('inbox.filters.syntaxHintTitle')">
            <UButton
              size="xs"
              color="neutral"
              variant="ghost"
              square
              icon="i-lucide-help-circle"
              :aria-label="t('inbox.filters.syntaxHintTitle')"
            />
          </UTooltip>
          <template #content>
            <div class="w-72 space-y-2 p-3 text-xs">
              <p class="font-semibold text-highlighted">
                {{ t('inbox.filters.syntaxHintTitle') }}
              </p>
              <p class="text-muted">
                {{ t('inbox.filters.syntaxHintDescription') }}
              </p>
              <code dir="ltr" class="block rounded bg-muted px-2 py-1.5 font-mono text-[11px] text-default">
                channel:email status:captured otp:true -spam /order-[0-9]+/
              </code>
            </div>
          </template>
        </UPopover>

        <UPopover v-model:open="isSavePopoverOpen">
          <UTooltip :text="t('inbox.filters.saveFilterButton')">
            <UButton
              size="xs"
              color="neutral"
              variant="outline"
              square
              icon="i-lucide-bookmark-plus"
              :disabled="!hasActiveScope"
              :aria-label="t('inbox.filters.saveFilterButton')"
            />
          </UTooltip>
          <template #content>
            <div class="flex w-64 flex-col gap-2 p-3">
              <p class="text-xs font-semibold text-highlighted">
                {{ t('inbox.filters.saveFilterTitle') }}
              </p>
              <UInput
                v-model="newFilterName"
                size="xs"
                :maxlength="40"
                :placeholder="t('inbox.filters.saveFilterPlaceholder')"
                @keydown.enter.prevent="handleSaveCurrentFilter"
              />
              <div class="flex justify-end gap-1.5">
                <UButton
                  size="xs"
                  color="neutral"
                  variant="ghost"
                  :label="t('common.cancel')"
                  @click="isSavePopoverOpen = false"
                />
                <UButton
                  size="xs"
                  color="primary"
                  :label="t('common.save')"
                  @click="handleSaveCurrentFilter"
                />
              </div>
            </div>
          </template>
        </UPopover>

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

    <div
      v-if="savedFilters.length > 0"
      class="flex flex-wrap items-center gap-1.5 border-t border-default/60 pt-2"
    >
      <span class="text-[11px] font-medium text-muted">
        {{ t('inbox.filters.savedFiltersLabel') }}
      </span>
      <div
        v-for="saved in savedFilters"
        :key="saved.id"
        class="inline-flex items-center gap-1 rounded-md border border-default bg-default/70 px-2 py-0.5 text-xs"
      >
        <button
          type="button"
          class="font-medium text-default hover:text-primary"
          @click="handleApplySavedFilter(saved)"
        >
          {{ saved.name }}
        </button>
        <button
          type="button"
          class="text-muted hover:text-error"
          :aria-label="t('inbox.filters.deleteSavedFilter', { name: saved.name })"
          @click="deleteSavedFilter(saved.id)"
        >
          <UIcon name="i-lucide-x" class="size-3" />
        </button>
      </div>
    </div>

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
