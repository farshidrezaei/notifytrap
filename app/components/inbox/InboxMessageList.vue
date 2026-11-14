<script setup lang="ts">
import type { VerificationArtifact } from '~/types/application'
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
} = useNotifications()

const items = computed(() => notificationPage.value?.items ?? [])
const totalCount = computed(() => notificationPage.value?.totalCount ?? 0)
const selectedId = computed(() => selectedDetail.value?.notification.id ?? null)

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

    <!-- Virtualized Scroll Container -->
    <div
      v-else
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
            class="flex w-full cursor-pointer flex-col gap-1.5 p-3 text-start transition-colors hover:bg-muted/60 focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-inset focus-visible:ring-primary"
            :class="selectedId === item.id ? 'bg-primary/10' : ''"
            @click="selectNotification(item.id)"
            @keydown.enter.prevent="selectNotification(item.id)"
            @keydown.space.prevent="selectNotification(item.id)"
          >
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
  </div>
</template>
