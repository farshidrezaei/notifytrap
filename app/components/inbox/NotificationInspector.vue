<script setup lang="ts">
import { formatShortTimestamp } from '~/utils/formatters'
import { extractCommandErrorMessage } from '~/utils/tauriClient'

const { t, locale } = useI18n()
const toast = useToast()
const {
  selectedDetail,
  isLoadingDetail,
  updateNotificationStatus,
  deleteNotification,
} = useNotifications()

const isUpdatingStatus = ref(false)
const isDeleteConfirmOpen = ref(false)
const isDeleting = ref(false)

const currentNotification = computed(() => selectedDetail.value?.notification ?? null)

async function handleStatusTransition(status: 'captured' | 'read' | 'archived'): Promise<void> {
  if (!currentNotification.value) {
    return
  }
  isUpdatingStatus.value = true
  try {
    await updateNotificationStatus(currentNotification.value.id, status)
  } catch (error) {
    toast.add({
      title: t('common.errorTitle'),
      description: extractCommandErrorMessage(error, t('common.unexpectedError')),
      color: 'error',
      icon: 'i-lucide-alert-circle',
    })
  } finally {
    isUpdatingStatus.value = false
  }
}

async function handleConfirmDelete(): Promise<void> {
  if (!currentNotification.value) {
    return
  }
  isDeleting.value = true
  try {
    await deleteNotification(currentNotification.value.id)
    isDeleteConfirmOpen.value = false
    toast.add({
      title: t('inbox.toast.deletedTitle'),
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
    isDeleting.value = false
  }
}
</script>

<template>
  <div class="flex h-full min-h-0 flex-col overflow-hidden rounded-lg border border-default bg-default/50">
    <SkeletonInboxInspector v-if="isLoadingDetail" />

    <div
      v-else-if="!currentNotification"
      class="flex flex-1 items-center justify-center p-6"
    >
      <UEmpty
        icon="i-lucide-panel-right"
        :title="t('inbox.inspectorEmptyTitle')"
        :description="t('inbox.inspectorEmptyDescription')"
      />
    </div>

    <div v-else class="flex min-h-0 flex-1 flex-col overflow-y-auto p-4">
      <div class="mb-4 flex flex-wrap items-start justify-between gap-3 border-b border-default pb-3">
        <div class="min-w-0 space-y-1">
          <div class="flex flex-wrap items-center gap-2">
            <SharedChannelBadge :channel-type="currentNotification.channelType" />
            <UBadge color="neutral" variant="subtle" size="xs">
              {{ t(`inbox.status.${currentNotification.status}`) }}
            </UBadge>
            <span class="text-xs text-muted">
              {{ formatShortTimestamp(currentNotification.receivedAt, locale) }}
            </span>
          </div>
          <h2 class="text-base font-semibold text-highlighted">
            {{ currentNotification.subject || currentNotification.recipient || t('inbox.noSubject') }}
          </h2>
        </div>

        <div class="flex items-center gap-1.5">
          <UButton
            v-if="currentNotification.status !== 'captured'"
            size="xs"
            color="neutral"
            variant="outline"
            icon="i-lucide-mail-question"
            :loading="isUpdatingStatus"
            :label="t('inbox.actions.markUnread')"
            @click="handleStatusTransition('captured')"
          />
          <UButton
            v-else
            size="xs"
            color="neutral"
            variant="outline"
            icon="i-lucide-mail-open"
            :loading="isUpdatingStatus"
            :label="t('inbox.actions.markRead')"
            @click="handleStatusTransition('read')"
          />

          <UButton
            v-if="currentNotification.status !== 'archived'"
            size="xs"
            color="neutral"
            variant="outline"
            icon="i-lucide-archive"
            :loading="isUpdatingStatus"
            :label="t('inbox.actions.archive')"
            @click="handleStatusTransition('archived')"
          />

          <UTooltip :text="t('inbox.actions.deleteNotification')">
            <UButton
              size="xs"
              color="error"
              variant="ghost"
              square
              icon="i-lucide-trash-2"
              :aria-label="t('inbox.actions.deleteNotification')"
              @click="isDeleteConfirmOpen = true"
            />
          </UTooltip>
        </div>
      </div>

      <InboxEmailInspectorView
        v-if="currentNotification.channelType === 'email'"
        :notification="currentNotification"
        :attachments="selectedDetail?.attachments ?? []"
      />
      <InboxSmsInspectorView
        v-else-if="currentNotification.channelType === 'sms'"
        :notification="currentNotification"
      />
      <InboxPushInspectorView
        v-else-if="currentNotification.channelType === 'push'"
        :notification="currentNotification"
      />
      <InboxWebhookInspectorView
        v-else
        :notification="currentNotification"
      />
    </div>

    <UModal
      v-model:open="isDeleteConfirmOpen"
      :title="t('inbox.deleteModal.title')"
      :description="t('inbox.deleteModal.description')"
    >
      <template #body>
        <div class="flex justify-end gap-2">
          <UButton
            color="neutral"
            variant="ghost"
            :label="t('common.cancel')"
            @click="isDeleteConfirmOpen = false"
          />
          <UButton
            color="error"
            :loading="isDeleting"
            :label="t('inbox.actions.deleteNotification')"
            @click="handleConfirmDelete"
          />
        </div>
      </template>
    </UModal>
  </div>
</template>
