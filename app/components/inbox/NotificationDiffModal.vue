<script setup lang="ts">
import type { Notification, NotificationSummary } from '~/types/application'
import { formatShortTimestamp } from '~/utils/formatters'
import { extractCommandErrorMessage } from '~/utils/tauriClient'
import {
  buildNotificationComparableText,
  computeLineDiff,
  type NotificationCompareMode,
} from '~/utils/textDiff'

const props = defineProps<{
  baselineNotification: Notification | null
  candidates: readonly NotificationSummary[]
}>()

const isOpen = defineModel<boolean>('open', { default: false })
const { t, locale } = useI18n()
const toast = useToast()
const { fetchNotificationDetailById } = useNotifications()

const selectedTargetId = ref<string>('')
const compareMode = ref<NotificationCompareMode>('prettyPayload')
const comparisonNotification = ref<Notification | null>(null)
const isLoadingTarget = ref(false)

const targetOptions = computed(() =>
  props.candidates
    .filter((item) => item.id !== props.baselineNotification?.id)
    .map((item) => ({
      label: `${item.subject || item.recipient || t('inbox.noSubject')} (${formatShortTimestamp(item.receivedAt, locale.value)})`,
      value: item.id,
    })),
)

const modeOptions = computed<Array<{ label: string; value: NotificationCompareMode }>>(() => [
  { label: t('inbox.diff.modes.prettyPayload'), value: 'prettyPayload' },
  { label: t('inbox.diff.modes.rawPayload'), value: 'rawPayload' },
  { label: t('inbox.diff.modes.metadata'), value: 'metadata' },
])

watch(
  [isOpen, () => props.baselineNotification?.id ?? null, targetOptions],
  async ([opened, baselineId]) => {
    if (!opened || !baselineId) {
      if (!baselineId && opened) {
        isOpen.value = false
      }
      return
    }
    const firstOption = targetOptions.value[0]?.value ?? ''
    if (!selectedTargetId.value || selectedTargetId.value === baselineId) {
      selectedTargetId.value = firstOption
    }
    if (selectedTargetId.value) {
      await loadTargetDetail(selectedTargetId.value)
    } else {
      comparisonNotification.value = null
    }
  },
)

watch(selectedTargetId, async (nextId) => {
  if (isOpen.value && nextId) {
    await loadTargetDetail(nextId)
  }
})

async function loadTargetDetail(notificationId: string): Promise<void> {
  isLoadingTarget.value = true
  try {
    const detail = await fetchNotificationDetailById(notificationId)
    comparisonNotification.value = detail?.notification ?? null
  } catch (error) {
    toast.add({
      title: t('common.errorTitle'),
      description: extractCommandErrorMessage(error, t('common.unexpectedError')),
      color: 'error',
      icon: 'i-lucide-alert-circle',
    })
  } finally {
    isLoadingTarget.value = false
  }
}

const diffLines = computed(() => {
  if (!props.baselineNotification || !comparisonNotification.value) {
    return []
  }
  const baselineText = buildNotificationComparableText(props.baselineNotification, compareMode.value)
  const comparisonText = buildNotificationComparableText(comparisonNotification.value, compareMode.value)
  return computeLineDiff(baselineText, comparisonText)
})

const diffCounts = computed(() => {
  let added = 0
  let removed = 0
  for (const line of diffLines.value) {
    if (line.kind === 'added') {
      added += 1
    } else if (line.kind === 'removed') {
      removed += 1
    }
  }
  return { added, removed }
})
</script>

<template>
  <UModal
    v-model:open="isOpen"
    :title="t('inbox.diff.title')"
    :description="t('inbox.diff.description')"
    :ui="{ content: 'max-w-4xl' }"
  >
    <template #body>
      <div class="space-y-4">
        <div class="flex flex-wrap items-center justify-between gap-2">
          <USelect
            v-model="selectedTargetId"
            :items="targetOptions"
            value-key="value"
            size="xs"
            class="min-w-64 flex-1"
            :placeholder="t('inbox.diff.selectPlaceholder')"
          />

          <div class="flex items-center gap-2">
            <USelect
              v-model="compareMode"
              :items="modeOptions"
              value-key="value"
              size="xs"
              class="w-40"
            />
            <UBadge color="success" variant="subtle" size="xs">
              +{{ diffCounts.added }}
            </UBadge>
            <UBadge color="error" variant="subtle" size="xs">
              -{{ diffCounts.removed }}
            </UBadge>
          </div>
        </div>

        <div
          v-if="isLoadingTarget"
          class="flex items-center justify-center rounded-lg border border-default py-12 text-xs text-muted"
        >
          {{ t('common.loading') }}
        </div>

        <UEmpty
          v-else-if="!comparisonNotification"
          icon="i-lucide-git-compare-arrows"
          :title="t('inbox.diff.emptyTitle')"
          :description="t('inbox.diff.emptyDescription')"
        />

        <div
          v-else
          dir="ltr"
          class="max-h-[58vh] overflow-auto rounded-lg border border-default bg-muted/20 font-mono text-xs"
        >
          <div
            v-for="(line, index) in diffLines"
            :key="index"
            class="flex items-stretch border-b border-default/40 last:border-b-0"
            :class="line.kind === 'added'
              ? 'bg-success/10 text-success'
              : line.kind === 'removed'
                ? 'bg-error/10 text-error'
                : 'text-toned'"
          >
            <span class="w-10 shrink-0 select-none border-e border-default/40 px-2 py-0.5 text-end text-[11px] text-muted">
              {{ line.leftLineNumber ?? '' }}
            </span>
            <span class="w-10 shrink-0 select-none border-e border-default/40 px-2 py-0.5 text-end text-[11px] text-muted">
              {{ line.rightLineNumber ?? '' }}
            </span>
            <span class="w-5 shrink-0 select-none px-1 py-0.5 text-center font-bold">
              {{ line.kind === 'added' ? '+' : line.kind === 'removed' ? '-' : ' ' }}
            </span>
            <pre class="flex-1 whitespace-pre-wrap break-all px-2 py-0.5">{{ line.text }}</pre>
          </div>
        </div>
      </div>
    </template>
  </UModal>
</template>
