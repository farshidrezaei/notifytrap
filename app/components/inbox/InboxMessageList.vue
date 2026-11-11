<script setup lang="ts">
import { formatShortTimestamp } from '~/utils/formatters'

const { t, locale } = useI18n()
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

async function handlePageUpdate(nextPage: number): Promise<void> {
  currentPage.value = nextPage
  await refreshInbox()
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

    <div v-else class="flex-1 divide-y divide-default overflow-y-auto">
      <button
        v-for="item in items"
        :key="item.id"
        type="button"
        :aria-pressed="selectedId === item.id"
        class="flex w-full flex-col gap-1.5 p-3 text-start transition-colors hover:bg-muted/60 focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-inset focus-visible:ring-primary"
        :class="selectedId === item.id ? 'bg-primary/10' : ''"
        @click="selectNotification(item.id)"
      >
        <div class="flex items-center justify-between gap-2">
          <div class="flex min-w-0 items-center gap-2">
            <span
              v-if="item.status === 'captured'"
              class="size-2 shrink-0 rounded-full bg-primary"
              :aria-label="t('inbox.status.captured')"
            />
            <SharedChannelBadge :channel-type="item.channelType" />
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

        <p
          class="truncate text-xs text-highlighted"
          :class="item.status === 'captured' ? 'font-semibold' : 'font-medium'"
        >
          {{ item.subject || item.recipient || t('inbox.noSubject') }}
        </p>

        <p class="truncate text-xs text-muted">
          {{ item.recipient }}
        </p>

        <p v-if="item.previewText" class="line-clamp-2 text-[11px] text-toned">
          {{ item.previewText }}
        </p>
      </button>
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
