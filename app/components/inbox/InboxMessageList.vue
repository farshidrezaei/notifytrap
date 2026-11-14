<script setup lang="ts">
import type { NotificationStatus, VerificationArtifact } from '~/types/application'
import { copyTextToClipboard } from '~/utils/clipboard'
import { formatShortTimestamp } from '~/utils/formatters'

const ITEM_ESTIMATED_HEIGHT_PIXELS = 84

const { t, locale } = useI18n()
const toast = useToast()
const {
  notificationPage,
  selectedDetail,
  isLoadingList,
  currentPage,
  pageSize,
  refreshInbox,
  selectNotification,
  selectedNotificationIds,
  toggleSelectNotification,
  toggleSelectAll,
  clearSelection,
  bulkDeleteSelected,
  bulkUpdateStatusSelected,
  bulkTagSelected,
} = useNotifications()

const items = computed(() => notificationPage.value?.items ?? [])
const totalCount = computed(() => notificationPage.value?.totalCount ?? 0)
const selectedId = computed(() => selectedDetail.value?.notification.id ?? null)

const isAllSelected = computed({
  get: () => items.value.length > 0 && items.value.every((item) => selectedNotificationIds.value.includes(item.id)),
  set: (val: boolean) => toggleSelectAll(val),
})

const selectedCount = computed(() => selectedNotificationIds.value.length)

const isTagModalOpen = ref(false)
const isDeleteConfirmModalOpen = ref(false)
const tagNameInput = ref('')
const isActionLoading = ref(false)

const {
  containerRef,
  visibleItems,
  totalHeight,
  offsetY,
  scrollToIndex,
} = useVirtualList(items, {
  itemHeight: ITEM_ESTIMATED_HEIGHT_PIXELS,
  overscan: 6,
})

// Scroll selected item into view if it changes
watch(selectedId, (newId) => {
  if (!newId) return
  const foundIndex = items.value.findIndex((item) => item.id === newId)
  if (foundIndex >= 0) {
    scrollToIndex(foundIndex)
  }
})

function firstOtpArtifact(artifacts: readonly VerificationArtifact[]): VerificationArtifact | undefined {
  return artifacts.find((artifact) => artifact.kind === 'otpCode')
}

async function handlePageUpdate(nextPage: number): Promise<void> {
  currentPage.value = nextPage
  clearSelection()
  await refreshInbox()
}

async function handleCopyOtp(code: string): Promise<void> {
  const didCopy = await copyTextToClipboard(code)
  toast.add({
    title: didCopy ? t('inbox.verification.copiedOtpTitle', { code }) : t('common.copyFailedTitle'),
    color: didCopy ? 'success' : 'error',
    icon: didCopy ? 'i-lucide-check' : 'i-lucide-alert-circle',
  })
}

async function handleBulkStatus(status: NotificationStatus): Promise<void> {
  isActionLoading.value = true
  try {
    const count = await bulkUpdateStatusSelected(status)
    toast.add({
      title: t('inbox.bulk.toast.statusUpdated', { count }),
      color: 'success',
      icon: 'i-lucide-check',
    })
  } finally {
    isActionLoading.value = false
  }
}

function openTagModal(): void {
  tagNameInput.value = ''
  isTagModalOpen.value = true
}

async function handleApplyTag(): Promise<void> {
  const cleanTag = tagNameInput.value.trim()
  if (!cleanTag) return

  isActionLoading.value = true
  try {
    const count = await bulkTagSelected(cleanTag)
    isTagModalOpen.value = false
    toast.add({
      title: t('inbox.bulk.toast.tagged', { count }),
      color: 'success',
      icon: 'i-lucide-tag',
    })
  } finally {
    isActionLoading.value = false
  }
}

async function handleConfirmBulkDelete(): Promise<void> {
  isActionLoading.value = true
  try {
    const count = await bulkDeleteSelected()
    isDeleteConfirmModalOpen.value = false
    toast.add({
      title: t('inbox.bulk.toast.deleted', { count }),
      color: 'success',
      icon: 'i-lucide-trash-2',
    })
  } finally {
    isActionLoading.value = false
  }
}
</script>

<template>
  <div class="flex h-full min-h-0 flex-col overflow-hidden rounded-lg border border-default bg-default/50">
    <SkeletonInboxMessageList v-if="isLoadingList" />

    <div v-else-if="items.length === 0" class="flex flex-1 items-center justify-center p-6">
      <UEmpty
        icon="i-lucide-inbox"
        :title="t('inbox.emptyTitle')"
        :description="t('inbox.emptyDescription')"
      />
    </div>

    <!-- Bulk Action Toolbar -->
    <div
      v-if="items.length > 0"
      class="flex flex-wrap items-center justify-between gap-2 border-b border-default bg-muted/40 px-3 py-2 text-xs"
    >
      <div class="flex items-center gap-2">
        <UCheckbox
          v-model="isAllSelected"
          :aria-label="t('inbox.bulk.selectAll')"
        />
        <span v-if="selectedCount > 0" class="font-medium text-primary">
          {{ t('inbox.bulk.selectedCount', { count: selectedCount }) }}
        </span>
        <span v-else class="text-muted">
          {{ t('inbox.bulk.selectAll') }}
        </span>
      </div>

      <!-- Quick Actions for selected batch -->
      <div v-if="selectedCount > 0" class="flex flex-wrap items-center gap-1">
        <UButton
          size="xs"
          color="neutral"
          variant="soft"
          icon="i-lucide-check-check"
          :loading="isActionLoading"
          :label="t('inbox.bulk.markRead')"
          @click="handleBulkStatus('read')"
        />
        <UButton
          size="xs"
          color="neutral"
          variant="ghost"
          icon="i-lucide-archive"
          :loading="isActionLoading"
          :label="t('inbox.bulk.archive')"
          @click="handleBulkStatus('archived')"
        />
        <UButton
          size="xs"
          color="primary"
          variant="soft"
          icon="i-lucide-tag"
          :loading="isActionLoading"
          :label="t('inbox.bulk.tag')"
          @click="openTagModal"
        />
        <UButton
          size="xs"
          color="error"
          variant="soft"
          icon="i-lucide-trash-2"
          :loading="isActionLoading"
          :label="t('inbox.bulk.delete')"
          @click="isDeleteConfirmModalOpen = true"
        />
        <UButton
          size="xs"
          color="neutral"
          variant="ghost"
          icon="i-lucide-x"
          square
          :aria-label="t('inbox.bulk.deselectAll')"
          @click="clearSelection"
        />
      </div>
    </div>

    <!-- Virtualized Scroll Container -->
    <div
      v-if="items.length > 0"
      ref="containerRef"
      class="flex-1 overflow-y-auto"
      tabindex="0"
      :aria-label="t('inbox.listLabel')"
    >
      <div
        class="relative w-full"
        :style="{ height: `${totalHeight}px` }"
      >
        <div
          class="absolute inset-x-0 top-0 divide-y divide-default"
          :style="{ transform: `translateY(${offsetY}px)` }"
        >
          <div
            v-for="{ data: item } in visibleItems"
            :key="item.id"
            role="button"
            tabindex="0"
            :aria-pressed="selectedId === item.id"
            class="flex w-full cursor-pointer items-start gap-2.5 p-3 text-start transition-colors hover:bg-muted/60 focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-inset focus-visible:ring-primary"
            :class="[
              selectedId === item.id ? 'bg-primary/10' : '',
              selectedNotificationIds.includes(item.id) ? 'bg-primary/5' : '',
            ]"
            @click="selectNotification(item.id)"
            @keydown.enter.prevent="selectNotification(item.id)"
            @keydown.space.prevent="selectNotification(item.id)"
          >
            <!-- Checkbox for multi-select -->
            <div class="pt-0.5" @click.stop>
              <UCheckbox
                :model-value="selectedNotificationIds.includes(item.id)"
                :aria-label="item.subject || item.recipient"
                @update:model-value="toggleSelectNotification(item.id)"
              />
            </div>

            <!-- Card Content -->
            <div class="flex flex-1 min-w-0 flex-col gap-1.5">
              <div class="flex items-center justify-between gap-2">
                <div class="flex min-w-0 flex-wrap items-center gap-1.5">
                  <span
                    v-if="item.status === 'captured'"
                    class="size-2 shrink-0 rounded-full bg-primary"
                    :aria-label="t('inbox.status.captured')"
                  />
                  <SharedChannelBadge :channel-type="item.channelType" />
                  <UBadge
                    v-if="item.testRunIdentifier"
                    color="secondary"
                    variant="subtle"
                    size="xs"
                    icon="i-lucide-tag"
                  >
                    {{ item.testRunIdentifier }}
                  </UBadge>
                  <UBadge
                    v-if="item.status === 'archived'"
                    color="neutral"
                    variant="subtle"
                    size="xs"
                  >
                    {{ t('inbox.status.archived') }}
                  </UBadge>
                </div>

                <span class="shrink-0 text-[11px] text-muted">
                  {{ formatShortTimestamp(item.receivedAt, locale) }}
                </span>
              </div>

              <div class="flex items-center justify-between gap-2">
                <p
                  class="min-w-0 truncate text-xs text-highlighted"
                  :class="item.status === 'captured' ? 'font-semibold' : 'font-medium'"
                >
                  {{ item.subject || item.recipient || t('inbox.noSubject') }}
                </p>

                <UButton
                  v-if="firstOtpArtifact(item.verificationArtifacts)"
                  size="xs"
                  color="primary"
                  variant="soft"
                  icon="i-lucide-key-round"
                  :label="firstOtpArtifact(item.verificationArtifacts)?.value"
                  @click.stop="handleCopyOtp(firstOtpArtifact(item.verificationArtifacts)!.value)"
                />
              </div>

              <p class="truncate text-xs text-muted">
                {{ item.recipient }}
              </p>

              <p v-if="item.previewText" class="line-clamp-2 text-[11px] text-toned">
                {{ item.previewText }}
              </p>
            </div>
          </div>
        </div>
      </div>
    </div>

    <!-- Pagination Footer -->
    <div
      v-if="totalCount > pageSize"
      class="flex items-center justify-between border-t border-default bg-muted/30 px-3 py-2"
    >
      <span class="text-xs text-muted">
        {{ t('inbox.paginationTotal', { count: totalCount }) }}
      </span>
      <UPagination
        :page="currentPage"
        :items-per-page="pageSize"
        :total="totalCount"
        size="xs"
        @update:page="handlePageUpdate"
      />
    </div>

    <!-- Tag Assignment Modal -->
    <UModal v-model:open="isTagModalOpen">
      <template #content>
        <div class="p-4 space-y-4">
          <div class="space-y-1">
            <h3 class="text-sm font-semibold text-highlighted">
              {{ t('inbox.bulk.tagTitle') }}
            </h3>
            <p class="text-xs text-muted">
              {{ t('inbox.bulk.selectedCount', { count: selectedCount }) }}
            </p>
          </div>

          <UInput
            v-model="tagNameInput"
            dir="ltr"
            class="w-full font-mono text-xs"
            :placeholder="t('inbox.bulk.tagPlaceholder')"
            @keydown.enter.prevent="handleApplyTag"
          />

          <div class="flex items-center justify-end gap-2 pt-2 border-t border-default">
            <UButton
              size="xs"
              color="neutral"
              variant="ghost"
              :label="t('common.cancel')"
              @click="isTagModalOpen = false"
            />
            <UButton
              size="xs"
              color="primary"
              :loading="isActionLoading"
              :disabled="!tagNameInput.trim()"
              :label="t('inbox.bulk.tagSubmit')"
              @click="handleApplyTag"
            />
          </div>
        </div>
      </template>
    </UModal>

    <!-- Delete Confirmation Modal -->
    <UModal v-model:open="isDeleteConfirmModalOpen">
      <template #content>
        <div class="p-4 space-y-4">
          <div class="flex items-center gap-3">
            <div class="flex size-9 shrink-0 items-center justify-center rounded-full bg-error/10 text-error">
              <UIcon name="i-lucide-alert-triangle" class="size-5" />
            </div>
            <div class="space-y-1">
              <h3 class="text-sm font-semibold text-highlighted">
                {{ t('inbox.bulk.deleteConfirmTitle') }}
              </h3>
              <p class="text-xs text-muted">
                {{ t('inbox.bulk.deleteConfirmDescription', { count: selectedCount }) }}
              </p>
            </div>
          </div>

          <div class="flex items-center justify-end gap-2 pt-2 border-t border-default">
            <UButton
              size="xs"
              color="neutral"
              variant="ghost"
              :label="t('common.cancel')"
              @click="isDeleteConfirmModalOpen = false"
            />
            <UButton
              size="xs"
              color="error"
              :loading="isActionLoading"
              :label="t('inbox.bulk.delete')"
              @click="handleConfirmBulkDelete"
            />
          </div>
        </div>
      </template>
    </UModal>
  </div>
</template>
