<script setup lang="ts">
import type { NotificationAttachmentSummary } from '~/types/application'
import {
  isPreviewableTextMime,
  isSafePreviewableImageMime,
} from '~/utils/attachmentPreview'
import { formatByteSize } from '~/utils/formatters'

const props = defineProps<{
  attachments: readonly NotificationAttachmentSummary[]
  loadingAttachmentId: string | null
}>()

const emit = defineEmits<{
  preview: [attachment: NotificationAttachmentSummary]
  download: [attachment: NotificationAttachmentSummary]
}>()

const { t } = useI18n()

function canPreviewAttachment(attachment: NotificationAttachmentSummary): boolean {
  return isSafePreviewableImageMime(attachment.contentType)
    || isPreviewableTextMime(attachment.contentType)
}
</script>

<template>
  <div class="space-y-2">
    <UEmpty
      v-if="props.attachments.length === 0"
      icon="i-lucide-paperclip"
      :title="t('inbox.email.emptyAttachmentsTitle')"
      :description="t('inbox.email.emptyAttachmentsDescription')"
    />
    <div
      v-for="attachment in props.attachments"
      :key="attachment.id"
      class="flex items-center justify-between gap-3 rounded-lg border border-default bg-muted/30 px-3 py-2"
    >
      <div class="flex min-w-0 items-center gap-2.5">
        <UIcon name="i-lucide-paperclip" class="size-4 shrink-0 text-primary" />
        <div class="min-w-0">
          <p class="truncate text-xs font-medium text-highlighted">
            {{ attachment.filename }}
          </p>
          <p dir="ltr" class="truncate font-mono text-[11px] text-muted">
            {{ attachment.contentType }}
          </p>
        </div>
      </div>
      <div class="flex items-center gap-2">
        <UBadge v-if="attachment.isInline" color="info" variant="subtle" size="xs">
          {{ t('inbox.email.inlineAttachment') }}
        </UBadge>
        <span dir="ltr" class="font-mono text-xs text-toned">
          {{ formatByteSize(attachment.sizeBytes) }}
        </span>
        <UButton
          v-if="canPreviewAttachment(attachment)"
          size="xs"
          color="neutral"
          variant="outline"
          icon="i-lucide-eye"
          :loading="props.loadingAttachmentId === attachment.id"
          :label="t('inbox.email.attachments.preview')"
          @click="emit('preview', attachment)"
        />
        <UButton
          size="xs"
          color="primary"
          variant="soft"
          icon="i-lucide-download"
          :loading="props.loadingAttachmentId === attachment.id"
          :label="t('inbox.email.attachments.download')"
          @click="emit('download', attachment)"
        />
      </div>
    </div>
  </div>
</template>
