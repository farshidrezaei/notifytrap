<script setup lang="ts">
import type { NotificationAttachmentContent } from '~/types/application'
import {
  decodeBase64ToUtf8Text,
  isPreviewableTextMime,
  isSafePreviewableImageMime,
  triggerBinaryFileDownload,
} from '~/utils/attachmentPreview'
import { formatByteSize } from '~/utils/formatters'

const props = defineProps<{
  attachment: NotificationAttachmentContent | null
}>()

const isOpen = defineModel<boolean>('open', { default: false })
const { t } = useI18n()

const isImagePreview = computed(() =>
  props.attachment ? isSafePreviewableImageMime(props.attachment.summary.contentType) : false,
)

const isTextPreview = computed(() =>
  props.attachment ? isPreviewableTextMime(props.attachment.summary.contentType) : false,
)

const imageDataUrl = computed(() => {
  if (!props.attachment || !isImagePreview.value) {
    return ''
  }
  const mime = props.attachment.summary.contentType.split(';')[0]?.trim() || 'image/png'
  return `data:${mime};base64,${props.attachment.contentBase64}`
})

const textContent = computed(() => {
  if (!props.attachment || !isTextPreview.value) {
    return ''
  }
  return decodeBase64ToUtf8Text(props.attachment.contentBase64)
})

const textLanguage = computed(() => {
  const mime = props.attachment?.summary.contentType.toLowerCase() ?? ''
  if (mime.includes('json')) {
    return 'json'
  }
  if (mime.includes('html') || mime.includes('xml')) {
    return 'html'
  }
  return 'plaintext'
})

function handleDownload(): void {
  if (!props.attachment) {
    return
  }
  triggerBinaryFileDownload(
    props.attachment.summary.filename,
    props.attachment.contentBase64,
    props.attachment.summary.contentType,
  )
}
</script>

<template>
  <UModal
    v-model:open="isOpen"
    :title="attachment?.summary.filename ?? t('inbox.email.attachments.previewTitle')"
    :description="attachment ? `${attachment.summary.contentType} • ${formatByteSize(attachment.summary.sizeBytes)}` : ''"
    :ui="{ content: 'max-w-3xl' }"
  >
    <template #body>
      <div v-if="attachment" class="space-y-4">
        <div
          v-if="isImagePreview"
          class="flex max-h-[60vh] items-center justify-center overflow-auto rounded-lg border border-default bg-muted/30 p-4"
        >
          <img
            :src="imageDataUrl"
            :alt="attachment.summary.filename"
            class="max-h-[52vh] max-w-full rounded object-contain"
          >
        </div>

        <SharedCodeBlock
          v-else-if="isTextPreview"
          :code="textContent"
          :language="textLanguage"
          max-height-class="max-h-[55vh]"
        />

        <UEmpty
          v-else
          icon="i-lucide-file"
          :title="t('inbox.email.attachments.binaryTitle')"
          :description="t('inbox.email.attachments.binaryDescription')"
        />

        <div class="flex justify-end gap-2">
          <UButton
            color="neutral"
            variant="ghost"
            :label="t('common.cancel')"
            @click="isOpen = false"
          />
          <UButton
            color="primary"
            icon="i-lucide-download"
            :label="t('inbox.email.attachments.download')"
            @click="handleDownload"
          />
        </div>
      </div>
    </template>
  </UModal>
</template>
