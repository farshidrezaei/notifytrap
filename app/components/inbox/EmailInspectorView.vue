<script setup lang="ts">
import type { TabsItem } from '@nuxt/ui'
import type {
  Notification,
  NotificationAttachmentContent,
  NotificationAttachmentSummary,
} from '~/types/application'
import {
  isPreviewableTextMime,
  isSafePreviewableImageMime,
  resolveInlineCidImages,
  triggerBinaryFileDownload,
} from '~/utils/attachmentPreview'
import { auditCapturedEmail } from '~/utils/emailAudit'
import { buildSandboxedEmailDocument, escapeEmailHtml } from '~/utils/emailSandbox'
import { formatByteSize } from '~/utils/formatters'
import { extractCommandErrorMessage } from '~/utils/tauriClient'

type EmailViewportMode = 'desktop' | 'tablet' | 'mobile'

const props = defineProps<{
  notification: Notification
  attachments: readonly NotificationAttachmentSummary[]
}>()

const { t } = useI18n()
const toast = useToast()
const colorMode = useColorMode()
const { fetchNotificationAttachment } = useNotifications()

const viewportMode = ref<EmailViewportMode>('desktop')
const isEmailDarkSimulated = ref(false)
const inlineCidDataUrls = ref<Map<string, string>>(new Map())
const loadingAttachmentId = ref<string | null>(null)
const previewAttachment = ref<NotificationAttachmentContent | null>(null)
const isAttachmentModalOpen = ref(false)

const payload = computed(() => props.notification.payloadJson)

function readStringList(key: string): string[] {
  const raw = payload.value[key]
  if (Array.isArray(raw)) {
    return raw.map((entry) => String(entry)).filter(Boolean)
  }
  if (typeof raw === 'string' && raw.length > 0) {
    return [raw]
  }
  return []
}

function readString(key: string): string {
  const raw = payload.value[key]
  return typeof raw === 'string' ? raw : ''
}

const fromAddresses = computed(() => readStringList('from'))
const toAddresses = computed(() => readStringList('to'))
const ccAddresses = computed(() => readStringList('cc'))
const bccAddresses = computed(() => readStringList('bcc'))
const sanitizedHtml = computed(() => readString('sanitizedHtml'))
const rawHtml = computed(() => readString('rawHtml'))
const textBody = computed(() => readString('textBody'))

const headerRows = computed<Array<{ name: string; value: string }>>(() => {
  const raw = payload.value.headers
  if (Array.isArray(raw)) {
    return raw
      .filter((entry): entry is Record<string, unknown> => entry !== null && typeof entry === 'object')
      .map((entry) => ({
        name: String(entry.name ?? ''),
        value: String(entry.value ?? ''),
      }))
  }
  if (raw !== null && typeof raw === 'object') {
    return Object.entries(raw as Record<string, unknown>).map(([name, value]) => ({
      name,
      value: String(value ?? ''),
    }))
  }
  return []
})

watch(
  [() => props.notification.id, () => props.attachments],
  async () => {
    const nextMap = new Map<string, string>()
    const inlineImages = props.attachments.filter(
      (item) => item.isInline && Boolean(item.contentId) && isSafePreviewableImageMime(item.contentType),
    )

    for (const item of inlineImages) {
      const normalizedCid = item.contentId?.replace(/^<|>$/g, '').trim().toLowerCase()
      if (!normalizedCid) {
        continue
      }
      try {
        const fetched = await fetchNotificationAttachment(item.id)
        if (fetched) {
          const mime = item.contentType.split(';')[0]?.trim() || 'image/png'
          nextMap.set(normalizedCid, `data:${mime};base64,${fetched.contentBase64}`)
        }
      } catch {
        // Inline CID fallback gracefully leaves the original placeholder untouched.
      }
    }
    inlineCidDataUrls.value = nextMap
  },
  { immediate: true },
)

const resolvedHtml = computed(() => {
  const baseHtml = sanitizedHtml.value || `<pre>${escapeEmailHtml(textBody.value)}</pre>`
  return resolveInlineCidImages(baseHtml, inlineCidDataUrls.value)
})

const sandboxDocument = computed(() =>
  buildSandboxedEmailDocument(resolvedHtml.value, {
    isDarkTheme: colorMode.value === 'dark',
    simulateEmailDarkMode: isEmailDarkSimulated.value,
  }),
)

const viewportContainerClass = computed(() => {
  if (viewportMode.value === 'mobile') {
    return 'mx-auto w-full max-w-[375px]'
  }
  if (viewportMode.value === 'tablet') {
    return 'mx-auto w-full max-w-[768px]'
  }
  return 'w-full'
})

const auditReport = computed(() => auditCapturedEmail(props.notification, props.attachments))

const activeTab = ref<
  'htmlPreview' | 'plainText' | 'rawHtml' | 'headers' | 'attachments' | 'audit' | 'rawSource'
>('htmlPreview')

watch(
  () => props.notification.id,
  () => {
    activeTab.value = sanitizedHtml.value.trim().length > 0 ? 'htmlPreview' : 'plainText'
  },
  { immediate: true },
)

const tabItems = computed<TabsItem[]>(() => [
  { label: t('inbox.email.tabs.htmlPreview'), value: 'htmlPreview' },
  { label: t('inbox.email.tabs.plainText'), value: 'plainText' },
  { label: t('inbox.email.tabs.rawHtml'), value: 'rawHtml' },
  { label: `${t('inbox.email.tabs.headers')} (${headerRows.value.length})`, value: 'headers' },
  {
    label: `${t('inbox.email.tabs.attachments')} (${props.attachments.length})`,
    value: 'attachments',
  },
  { label: t('inbox.email.tabs.audit'), value: 'audit' },
  { label: t('inbox.email.tabs.rawSource'), value: 'rawSource' },
])

function canPreviewAttachment(attachment: NotificationAttachmentSummary): boolean {
  return isSafePreviewableImageMime(attachment.contentType)
    || isPreviewableTextMime(attachment.contentType)
}

async function handlePreviewAttachment(attachment: NotificationAttachmentSummary): Promise<void> {
  loadingAttachmentId.value = attachment.id
  try {
    const content = await fetchNotificationAttachment(attachment.id)
    if (!content) {
      return
    }
    previewAttachment.value = content
    isAttachmentModalOpen.value = true
  } catch (error) {
    toast.add({
      title: t('common.errorTitle'),
      description: extractCommandErrorMessage(error, t('common.unexpectedError')),
      color: 'error',
      icon: 'i-lucide-alert-circle',
    })
  } finally {
    loadingAttachmentId.value = null
  }
}

async function handleDownloadAttachment(attachment: NotificationAttachmentSummary): Promise<void> {
  loadingAttachmentId.value = attachment.id
  try {
    const content = await fetchNotificationAttachment(attachment.id)
    if (!content) {
      return
    }
    triggerBinaryFileDownload(
      content.summary.filename,
      content.contentBase64,
      content.summary.contentType,
    )
  } catch (error) {
    toast.add({
      title: t('common.errorTitle'),
      description: extractCommandErrorMessage(error, t('common.unexpectedError')),
      color: 'error',
      icon: 'i-lucide-alert-circle',
    })
  } finally {
    loadingAttachmentId.value = null
  }
}
</script>

<template>
  <div class="flex flex-col gap-4">
    <div class="grid gap-2 rounded-lg border border-default bg-muted/30 p-3 text-xs">
      <div class="flex flex-wrap gap-2">
        <span class="w-16 shrink-0 font-medium text-muted">{{ t('inbox.email.fields.from') }}</span>
        <span dir="ltr" class="font-mono text-highlighted">
          {{ fromAddresses.join(', ') || notification.sender }}
        </span>
      </div>
      <div class="flex flex-wrap gap-2">
        <span class="w-16 shrink-0 font-medium text-muted">{{ t('inbox.email.fields.to') }}</span>
        <span dir="ltr" class="font-mono text-highlighted">
          {{ toAddresses.join(', ') || notification.recipient }}
        </span>
      </div>
      <div v-if="ccAddresses.length > 0" class="flex flex-wrap gap-2">
        <span class="w-16 shrink-0 font-medium text-muted">{{ t('inbox.email.fields.cc') }}</span>
        <span dir="ltr" class="font-mono text-toned">{{ ccAddresses.join(', ') }}</span>
      </div>
      <div v-if="bccAddresses.length > 0" class="flex flex-wrap gap-2">
        <span class="w-16 shrink-0 font-medium text-muted">{{ t('inbox.email.fields.bcc') }}</span>
        <span dir="ltr" class="font-mono text-toned">{{ bccAddresses.join(', ') }}</span>
      </div>
    </div>

    <UTabs v-model="activeTab" :items="tabItems" size="xs" />

    <div v-if="activeTab === 'htmlPreview'" key="htmlPreview" class="space-y-2">
      <div class="flex flex-wrap items-center justify-between gap-2 rounded-lg border border-default bg-muted/30 px-2.5 py-1.5">
        <div class="flex items-center gap-1">
          <UButton
            size="xs"
            :color="viewportMode === 'desktop' ? 'primary' : 'neutral'"
            :variant="viewportMode === 'desktop' ? 'soft' : 'ghost'"
            icon="i-lucide-monitor"
            :label="t('inbox.email.viewport.desktop')"
            @click="viewportMode = 'desktop'"
          />
          <UButton
            size="xs"
            :color="viewportMode === 'tablet' ? 'primary' : 'neutral'"
            :variant="viewportMode === 'tablet' ? 'soft' : 'ghost'"
            icon="i-lucide-tablet"
            :label="t('inbox.email.viewport.tablet')"
            @click="viewportMode = 'tablet'"
          />
          <UButton
            size="xs"
            :color="viewportMode === 'mobile' ? 'primary' : 'neutral'"
            :variant="viewportMode === 'mobile' ? 'soft' : 'ghost'"
            icon="i-lucide-smartphone"
            :label="t('inbox.email.viewport.mobile')"
            @click="viewportMode = 'mobile'"
          />
        </div>

        <UButton
          size="xs"
          :color="isEmailDarkSimulated ? 'primary' : 'neutral'"
          :variant="isEmailDarkSimulated ? 'soft' : 'outline'"
          :icon="isEmailDarkSimulated ? 'i-lucide-moon' : 'i-lucide-sun'"
          :label="t('inbox.email.viewport.simulateDark')"
          @click="isEmailDarkSimulated = !isEmailDarkSimulated"
        />
      </div>

      <div class="overflow-x-auto rounded-lg border border-default bg-muted/20 p-2">
        <div
          class="overflow-hidden rounded-md border border-default transition-all"
          :class="viewportContainerClass"
        >
          <iframe
            sandbox=""
            referrerpolicy="no-referrer"
            :title="notification.subject || t('inbox.email.tabs.htmlPreview')"
            :srcdoc="sandboxDocument"
            class="h-[420px] w-full border-0 bg-default"
          />
        </div>
      </div>
    </div>

    <pre
      v-else-if="activeTab === 'plainText'"
      key="plainText"
      class="max-h-[420px] overflow-auto whitespace-pre-wrap rounded-lg border border-default bg-muted/40 p-3 text-xs leading-relaxed text-highlighted"
    >{{ textBody || t('inbox.email.emptyPlainText') }}</pre>

    <div v-else-if="activeTab === 'rawHtml'" key="rawHtml">
      <SharedCodeBlock
        v-if="rawHtml"
        :code="rawHtml"
        language="html"
        max-height-class="max-h-[420px]"
      />
      <div
        v-else
        class="rounded-lg border border-default bg-muted/30 p-3 text-xs text-muted"
      >
        {{ t('inbox.email.emptyRawHtml') }}
      </div>
    </div>

    <div v-else-if="activeTab === 'headers'" key="headers" class="max-h-[420px] overflow-auto rounded-lg border border-default">
      <table class="w-full text-xs">
        <tbody class="divide-y divide-default">
          <tr v-for="(headerEntry, index) in headerRows" :key="`${headerEntry.name}-${index}`">
            <td dir="ltr" class="w-48 bg-muted/30 px-3 py-2 font-mono font-medium text-muted">
              {{ headerEntry.name }}
            </td>
            <td dir="ltr" class="break-all px-3 py-2 font-mono text-highlighted">
              {{ headerEntry.value }}
            </td>
          </tr>
        </tbody>
      </table>
    </div>

    <div v-else-if="activeTab === 'attachments'" key="attachments" class="space-y-2">
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
            :loading="loadingAttachmentId === attachment.id"
            :label="t('inbox.email.attachments.preview')"
            @click="handlePreviewAttachment(attachment)"
          />
          <UButton
            size="xs"
            color="primary"
            variant="soft"
            icon="i-lucide-download"
            :loading="loadingAttachmentId === attachment.id"
            :label="t('inbox.email.attachments.download')"
            @click="handleDownloadAttachment(attachment)"
          />
        </div>
      </div>
    </div>

    <div v-else-if="activeTab === 'audit'" key="audit" class="space-y-3">
      <div class="grid grid-cols-2 gap-2 sm:grid-cols-4">
        <div class="rounded-lg border border-default bg-muted/30 p-2.5 text-xs">
          <p class="text-[11px] text-muted">{{ t('inbox.email.audit.metrics.size') }}</p>
          <p dir="ltr" class="mt-0.5 font-mono font-semibold text-highlighted">
            {{ formatByteSize(auditReport.rawBytes) }}
          </p>
        </div>
        <div class="rounded-lg border border-default bg-muted/30 p-2.5 text-xs">
          <p class="text-[11px] text-muted">{{ t('inbox.email.audit.metrics.links') }}</p>
          <p class="mt-0.5 font-mono font-semibold text-highlighted">{{ auditReport.linkCount }}</p>
        </div>
        <div class="rounded-lg border border-default bg-muted/30 p-2.5 text-xs">
          <p class="text-[11px] text-muted">{{ t('inbox.email.audit.metrics.images') }}</p>
          <p class="mt-0.5 font-mono font-semibold text-highlighted">{{ auditReport.imageCount }}</p>
        </div>
        <div class="rounded-lg border border-default bg-muted/30 p-2.5 text-xs">
          <p class="text-[11px] text-muted">{{ t('inbox.email.audit.metrics.missingAlt') }}</p>
          <p class="mt-0.5 font-mono font-semibold text-highlighted">{{ auditReport.missingAltCount }}</p>
        </div>
      </div>

      <div class="space-y-2">
        <div
          v-for="finding in auditReport.findings"
          :key="finding.id"
          class="flex items-center justify-between gap-3 rounded-lg border border-default bg-muted/20 px-3 py-2 text-xs"
        >
          <span class="text-highlighted">
            {{ t(finding.translationKey, finding.params ?? {}) }}
          </span>
          <UBadge
            :color="finding.severity === 'pass' ? 'success' : finding.severity === 'warning' ? 'warning' : 'info'"
            variant="subtle"
            size="xs"
          >
            {{ t(`inbox.email.audit.severity.${finding.severity}`) }}
          </UBadge>
        </div>
      </div>
    </div>

    <div v-else key="rawSource">
      <SharedCodeBlock
        :code="notification.rawPayload"
        language="http"
        max-height-class="max-h-[420px]"
      />
    </div>

    <InboxEmailAttachmentModal
      v-model:open="isAttachmentModalOpen"
      :attachment="previewAttachment"
    />
  </div>
</template>
