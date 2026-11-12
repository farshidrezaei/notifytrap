<script setup lang="ts">
import type { DropdownMenuItem } from '@nuxt/ui'
import type { VerificationArtifact } from '~/types/application'
import { copyTextToClipboard } from '~/utils/clipboard'
import { formatShortTimestamp } from '~/utils/formatters'
import {
  buildNotificationCurlExport,
  buildNotificationEmlExport,
  buildNotificationJsonExport,
  triggerTextFileDownload,
} from '~/utils/notificationExport'
import { extractCommandErrorMessage } from '~/utils/tauriClient'

const { t, locale } = useI18n()
const toast = useToast()
const { health } = useApplicationHealth()
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
const verificationArtifacts = computed<readonly VerificationArtifact[]>(
  () => currentNotification.value?.verificationArtifacts ?? [],
)

const exportMenuItems = computed<DropdownMenuItem[][]>(() => {
  const detail = selectedDetail.value
  if (!detail) {
    return []
  }

  const httpEndpoint = health.value?.httpEndpoint ?? '127.0.0.1:8080'
  const curlCommand = buildNotificationCurlExport(detail, httpEndpoint)
  const fileBaseName = `notifytrap-${detail.notification.channelType}-${detail.notification.id.slice(0, 8)}`

  const primaryActions: DropdownMenuItem[] = [
    {
      label: t('inbox.export.downloadEml'),
      icon: 'i-lucide-mail',
      onSelect: () => {
        triggerTextFileDownload(
          `${fileBaseName}.eml`,
          buildNotificationEmlExport(detail),
          'message/rfc822;charset=utf-8',
        )
        toast.add({
          title: t('inbox.export.downloadedTitle', { format: '.eml' }),
          color: 'success',
          icon: 'i-lucide-download',
        })
      },
    },
    {
      label: t('inbox.export.downloadJson'),
      icon: 'i-lucide-file-json',
      onSelect: () => {
        triggerTextFileDownload(
          `${fileBaseName}.json`,
          buildNotificationJsonExport(detail),
          'application/json;charset=utf-8',
        )
        toast.add({
          title: t('inbox.export.downloadedTitle', { format: 'JSON' }),
          color: 'success',
          icon: 'i-lucide-download',
        })
      },
    },
    {
      label: t('inbox.export.copyJson'),
      icon: 'i-lucide-copy',
      onSelect: async () => {
        await copyValueWithFeedback(
          buildNotificationJsonExport(detail),
          t('inbox.export.copiedTitle', { format: 'JSON' }),
        )
      },
    },
  ]

  if (curlCommand) {
    primaryActions.push({
      label: t('inbox.export.copyCurl'),
      icon: 'i-lucide-terminal',
      onSelect: async () => {
        await copyValueWithFeedback(
          curlCommand,
          t('inbox.export.copiedTitle', { format: 'cURL' }),
        )
      },
    })
  }

  return [primaryActions]
})

async function copyValueWithFeedback(value: string, successTitle: string): Promise<void> {
  const didCopy = await copyTextToClipboard(value)
  toast.add({
    title: didCopy ? successTitle : t('common.copyFailedTitle'),
    color: didCopy ? 'success' : 'error',
    icon: didCopy ? 'i-lucide-check' : 'i-lucide-alert-circle',
  })
}

async function handleCopyArtifact(artifact: VerificationArtifact): Promise<void> {
  const title = artifact.kind === 'otpCode'
    ? t('inbox.verification.copiedOtpTitle', { code: artifact.value })
    : t('inbox.verification.copiedLinkTitle')
  await copyValueWithFeedback(artifact.value, title)
}

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
            <UBadge
              v-if="currentNotification.testRunIdentifier"
              color="secondary"
              variant="subtle"
              size="xs"
              icon="i-lucide-tag"
            >
              {{ currentNotification.testRunIdentifier }}
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

          <UDropdownMenu :items="exportMenuItems">
            <UButton
              size="xs"
              color="neutral"
              variant="outline"
              icon="i-lucide-download"
              trailing-icon="i-lucide-chevron-down"
              :label="t('inbox.export.menuLabel')"
            />
          </UDropdownMenu>

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

      <div
        v-if="verificationArtifacts.length > 0"
        class="mb-4 flex flex-wrap items-center gap-2 rounded-lg border border-primary/25 bg-primary/5 px-3 py-2"
      >
        <span class="text-xs font-medium text-toned">
          {{ t('inbox.verification.detectedTitle') }}
        </span>
        <div class="flex flex-wrap items-center gap-1.5">
          <UButton
            v-for="(artifact, index) in verificationArtifacts"
            :key="`${artifact.kind}-${index}`"
            size="xs"
            :color="artifact.kind === 'otpCode' ? 'primary' : 'neutral'"
            :variant="artifact.kind === 'otpCode' ? 'soft' : 'outline'"
            :icon="artifact.kind === 'otpCode' ? 'i-lucide-key-round' : 'i-lucide-link'"
            trailing-icon="i-lucide-copy"
            class="max-w-full"
            @click="handleCopyArtifact(artifact)"
          >
            <span class="truncate max-w-64 font-mono">
              {{ artifact.value }}
            </span>
          </UButton>
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
