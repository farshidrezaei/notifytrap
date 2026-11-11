<script setup lang="ts">
import type { TabsItem } from '@nuxt/ui'
import type { Notification, NotificationAttachmentSummary } from '~/types/application'
import { buildSandboxedEmailDocument, escapeEmailHtml } from '~/utils/emailSandbox'
import { formatByteSize } from '~/utils/formatters'

const props = defineProps<{
  notification: Notification
  attachments: readonly NotificationAttachmentSummary[]
}>()

const { t } = useI18n()
const colorMode = useColorMode()

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

const sandboxDocument = computed(() =>
  buildSandboxedEmailDocument(
    sanitizedHtml.value || `<pre>${escapeEmailHtml(textBody.value)}</pre>`,
    colorMode.value === 'dark',
  ),
)

const activeTab = ref<'htmlPreview' | 'plainText' | 'rawHtml' | 'headers' | 'attachments' | 'rawSource'>(
  'htmlPreview',
)

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
  { label: t('inbox.email.tabs.rawSource'), value: 'rawSource' },
])
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

    <div v-if="activeTab === 'htmlPreview'" class="overflow-hidden rounded-lg border border-default">
      <iframe
        sandbox=""
        referrerpolicy="no-referrer"
        :title="notification.subject || t('inbox.email.tabs.htmlPreview')"
        :srcdoc="sandboxDocument"
        class="h-[420px] w-full border-0 bg-default"
      />
    </div>

    <pre
      v-else-if="activeTab === 'plainText'"
      class="max-h-[420px] overflow-auto whitespace-pre-wrap rounded-lg border border-default bg-muted/40 p-3 text-xs leading-relaxed text-highlighted"
    >{{ textBody || t('inbox.email.emptyPlainText') }}</pre>

    <template v-else-if="activeTab === 'rawHtml'">
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
    </template>

    <div v-else-if="activeTab === 'headers'" class="max-h-[420px] overflow-auto rounded-lg border border-default">
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

    <div v-else-if="activeTab === 'attachments'" class="space-y-2">
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
        </div>
      </div>
    </div>

    <SharedCodeBlock
      v-else
      :code="notification.rawPayload"
      language="http"
      max-height-class="max-h-[420px]"
    />
  </div>
</template>
