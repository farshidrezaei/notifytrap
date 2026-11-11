<script setup lang="ts">
import type { TabsItem } from '@nuxt/ui'
import type { Notification } from '~/types/application'
import { formatByteSize } from '~/utils/formatters'

const props = defineProps<{
  notification: Notification
}>()

const { t } = useI18n()

const payload = computed(() => props.notification.payloadJson)
const methodName = computed(() => String(payload.value.method ?? 'POST'))
const subpath = computed(() => String(payload.value.subpath ?? '/'))
const fullPath = computed(() => String(payload.value.path ?? props.notification.recipient))
const contentType = computed(() => String(payload.value.contentType ?? 'application/json'))
const bodyByteCount = computed(() => Number(payload.value.bodyBytes ?? 0))
const rawTextBody = computed(() => String(payload.value.textBody ?? ''))

const formattedJsonBody = computed(() => {
  if (payload.value.jsonBody !== null && payload.value.jsonBody !== undefined) {
    return JSON.stringify(payload.value.jsonBody, null, 2)
  }
  return ''
})

const headerRows = computed<Array<{ name: string; value: string }>>(() => {
  const raw = payload.value.headers
  if (raw !== null && typeof raw === 'object' && !Array.isArray(raw)) {
    return Object.entries(raw as Record<string, unknown>).map(([name, value]) => ({
      name,
      value: String(value ?? ''),
    }))
  }
  return []
})

const queryText = computed(() => {
  const raw = payload.value.query
  if (!raw) {
    return ''
  }
  return typeof raw === 'string' ? raw : JSON.stringify(raw, null, 2)
})

const activeTab = ref<'body' | 'headers' | 'query'>('body')

const tabItems = computed<TabsItem[]>(() => [
  { label: t('inbox.webhook.tabs.body'), value: 'body' },
  { label: `${t('inbox.webhook.tabs.headers')} (${headerRows.value.length})`, value: 'headers' },
  { label: t('inbox.webhook.tabs.query'), value: 'query' },
])
</script>

<template>
  <div class="flex flex-col gap-4">
    <div class="flex flex-wrap items-center justify-between gap-2 rounded-lg border border-default bg-muted/30 p-3">
      <div class="flex min-w-0 items-center gap-2">
        <UBadge color="warning" variant="subtle" size="xs" class="font-mono">
          {{ methodName }}
        </UBadge>
        <code dir="ltr" class="truncate font-mono text-xs font-semibold text-highlighted">
          {{ subpath }}
        </code>
      </div>
      <div class="flex items-center gap-2 text-[11px] text-muted">
        <code dir="ltr" class="font-mono">{{ contentType }}</code>
        <span>•</span>
        <span dir="ltr" class="font-mono">{{ formatByteSize(bodyByteCount) }}</span>
      </div>
    </div>

    <p dir="ltr" class="truncate font-mono text-xs text-muted">
      {{ fullPath }}
    </p>

    <UTabs v-model="activeTab" :items="tabItems" size="xs" />

    <template v-if="activeTab === 'body'">
      <SharedCodeBlock
        v-if="formattedJsonBody || rawTextBody"
        :code="formattedJsonBody || rawTextBody"
        :language="formattedJsonBody ? 'json' : 'auto'"
        :content-type="contentType"
        max-height-class="max-h-[380px]"
      />
      <div
        v-else
        class="rounded-lg border border-default bg-muted/30 p-3 text-xs text-muted"
      >
        {{ t('inbox.webhook.emptyBody') }}
      </div>
    </template>

    <div v-else-if="activeTab === 'headers'" class="max-h-[380px] overflow-auto rounded-lg border border-default">
      <table class="w-full text-xs">
        <tbody class="divide-y divide-default">
          <tr v-for="entry in headerRows" :key="entry.name">
            <td dir="ltr" class="w-48 bg-muted/30 px-3 py-2 font-mono font-medium text-muted">
              {{ entry.name }}
            </td>
            <td dir="ltr" class="break-all px-3 py-2 font-mono text-highlighted">
              {{ entry.value }}
            </td>
          </tr>
        </tbody>
      </table>
    </div>

    <template v-else>
      <SharedCodeBlock
        v-if="queryText"
        :code="queryText"
        language="auto"
        max-height-class="max-h-[380px]"
      />
      <div
        v-else
        class="rounded-lg border border-default bg-muted/30 p-3 text-xs text-muted"
      >
        {{ t('inbox.webhook.emptyQuery') }}
      </div>
    </template>
  </div>
</template>
