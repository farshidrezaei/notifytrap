<script setup lang="ts">
import type { TabsItem } from '@nuxt/ui'
import EmailAttachmentsTab from '~/components/inbox/email/EmailAttachmentsTab.vue'
import EmailAuditTab from '~/components/inbox/email/EmailAuditTab.vue'
import EmailHtmlTab, { type EmailViewportMode } from '~/components/inbox/email/EmailHtmlTab.vue'
import EmailInspectorHeader from '~/components/inbox/email/EmailInspectorHeader.vue'
import EmailMjmlTab from '~/components/inbox/email/EmailMjmlTab.vue'
import type {
  AssetCheckResult,
  Notification,
  NotificationAttachmentContent,
  NotificationAttachmentSummary,
} from '~/types/application'
import {
  isSafePreviewableImageMime,
  resolveInlineCidImages,
  triggerBinaryFileDownload,
} from '~/utils/attachmentPreview'
import { copyTextToClipboard } from '~/utils/clipboard'
import { auditCapturedEmail } from '~/utils/emailAudit'
import { buildSandboxedEmailDocument, escapeEmailHtml } from '~/utils/emailSandbox'
import { inspectMjmlContent } from '~/utils/mjmlInspector'
import {
  extractCommandErrorMessage,
  invokeTauriCommand,
  isTauriRuntime,
} from '~/utils/tauriClient'

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

const mjmlInspection = computed(() =>
  inspectMjmlContent(rawHtml.value || textBody.value || props.notification.rawPayload),
)

const mjmlSandboxDocument = computed(() =>
  buildSandboxedEmailDocument(mjmlInspection.value.compiledHtml, {
    isDarkTheme: colorMode.value === 'dark',
    simulateEmailDarkMode: isEmailDarkSimulated.value,
  }),
)

const auditReport = computed(() => auditCapturedEmail(props.notification, props.attachments))

const assetCheckResults = ref<AssetCheckResult[]>([])
const isAuditingAssets = ref(false)
const hasRunAssetAudit = ref(false)

async function handleAuditLinksAndAssets(): Promise<void> {
  if (!isTauriRuntime()) {
    isAuditingAssets.value = true
    setTimeout(() => {
      assetCheckResults.value = [
        {
          url: 'https://notifytrap.dev/verify',
          kind: 'link',
          statusCode: 200,
          isBroken: false,
          latencyMilliseconds: 38,
          errorMessage: null,
        },
      ]
      hasRunAssetAudit.value = true
      isAuditingAssets.value = false
    }, 300)
    return
  }

  isAuditingAssets.value = true
  try {
    const results = await invokeTauriCommand<AssetCheckResult[]>(
      'audit_notification_links_and_assets',
      { notificationId: props.notification.id },
    )
    assetCheckResults.value = results
    hasRunAssetAudit.value = true
  } catch (error) {
    toast.add({
      title: t('common.errorTitle'),
      description: extractCommandErrorMessage(error, t('common.unexpectedError')),
      color: 'error',
      icon: 'i-lucide-alert-circle',
    })
  } finally {
    isAuditingAssets.value = false
  }
}

async function copyAssetUrl(url: string): Promise<void> {
  const isCopied = await copyTextToClipboard(url)
  toast.add({
    title: isCopied ? t('common.copiedTitle') : t('common.errorTitle'),
    description: isCopied ? url : t('common.clipboardUnavailable'),
    color: isCopied ? 'success' : 'error',
    icon: isCopied ? 'i-lucide-check-circle-2' : 'i-lucide-alert-circle',
  })
}

const activeTab = ref<
  'htmlPreview' | 'plainText' | 'rawHtml' | 'headers' | 'attachments' | 'audit' | 'rawSource' | 'mjml'
>('htmlPreview')

watch(
  () => props.notification.id,
  () => {
    activeTab.value = sanitizedHtml.value.trim().length > 0 ? 'htmlPreview' : 'plainText'
    assetCheckResults.value = []
    hasRunAssetAudit.value = false
  },
  { immediate: true },
)

const tabItems = computed<TabsItem[]>(() => {
  const items: TabsItem[] = [
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
  ]

  if (mjmlInspection.value.isMjml) {
    items.splice(3, 0, {
      label: `${t('inbox.email.tabs.mjml')} (${mjmlInspection.value.totalTagCount})`,
      value: 'mjml',
    })
  }

  return items
})

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

async function handleCopyCompiledMjml(): Promise<void> {
  try {
    await navigator.clipboard.writeText(mjmlInspection.value.compiledHtml)
    toast.add({
      title: t('inbox.email.mjml.copiedSuccess'),
      color: 'success',
      icon: 'i-lucide-check',
    })
  } catch {
    // Clipboard permission denied fallback
  }
}
</script>

<template>
  <div class="flex flex-col gap-4">
    <EmailInspectorHeader
      :notification="props.notification"
      :from-addresses="fromAddresses"
      :to-addresses="toAddresses"
      :cc-addresses="ccAddresses"
      :bcc-addresses="bccAddresses"
    />

    <UTabs v-model="activeTab" :items="tabItems" size="xs" />

    <EmailHtmlTab
      v-if="activeTab === 'htmlPreview'"
      key="htmlPreview"
      v-model:viewport-mode="viewportMode"
      v-model:is-email-dark-simulated="isEmailDarkSimulated"
      :notification="props.notification"
      :sandbox-document="sandboxDocument"
    />

    <EmailMjmlTab
      v-else-if="activeTab === 'mjml'"
      key="mjml"
      :mjml-inspection="mjmlInspection"
      :raw-source="rawHtml || textBody"
      :mjml-sandbox-document="mjmlSandboxDocument"
      @copy-compiled="handleCopyCompiledMjml"
    />

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

    <EmailAttachmentsTab
      v-else-if="activeTab === 'attachments'"
      key="attachments"
      :attachments="props.attachments"
      :loading-attachment-id="loadingAttachmentId"
      @preview="handlePreviewAttachment"
      @download="handleDownloadAttachment"
    />

    <EmailAuditTab
      v-else-if="activeTab === 'audit'"
      key="audit"
      :audit-report="auditReport"
      :asset-check-results="assetCheckResults"
      :is-auditing-assets="isAuditingAssets"
      :has-run-asset-audit="hasRunAssetAudit"
      @run-audit="handleAuditLinksAndAssets"
      @copy-url="copyAssetUrl"
    />

    <div v-else key="rawSource">
      <SharedCodeBlock
        :code="props.notification.rawPayload"
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
