<script setup lang="ts">
import type { Notification } from '~/types/application'
import { formatShortTimestamp } from '~/utils/formatters'
import { analyzeSmsMessage } from '~/utils/smsAnalyzer'

const props = defineProps<{
  notification: Notification
}>()

const { t, locale } = useI18n()

const payload = computed(() => props.notification.payloadJson)
const sender = computed(() => String(payload.value.from ?? props.notification.sender))
const recipient = computed(() => String(payload.value.to ?? props.notification.recipient))
const messageText = computed(() => String(payload.value.message ?? props.notification.previewText))
const metadataJson = computed(() => JSON.stringify(payload.value.metadata ?? {}, null, 2))
const smsAnalysis = computed(() => analyzeSmsMessage(messageText.value))
</script>

<template>
  <div class="flex flex-col gap-4">
    <div class="grid gap-3 rounded-lg border border-default bg-muted/30 p-3 sm:grid-cols-3">
      <div>
        <p class="text-[11px] font-medium text-muted">{{ t('inbox.sms.fromLabel') }}</p>
        <p dir="ltr" class="mt-0.5 font-mono text-xs font-semibold text-highlighted">{{ sender }}</p>
      </div>
      <div>
        <p class="text-[11px] font-medium text-muted">{{ t('inbox.sms.toLabel') }}</p>
        <p dir="ltr" class="mt-0.5 font-mono text-xs font-semibold text-highlighted">{{ recipient }}</p>
      </div>
      <div>
        <p class="text-[11px] font-medium text-muted">{{ t('inbox.sms.lengthLabel') }}</p>
        <p class="mt-0.5 text-xs font-semibold text-highlighted">
          {{ t('inbox.sms.charactersCount', { count: smsAnalysis.characterCount }) }}
        </p>
      </div>
    </div>

    <div class="flex flex-wrap items-center gap-2 rounded-lg border border-default bg-muted/20 px-3 py-2 text-xs">
      <UBadge
        :color="smsAnalysis.encoding === 'GSM-7' ? 'success' : 'warning'"
        variant="subtle"
        size="xs"
      >
        {{ smsAnalysis.encoding }}
      </UBadge>
      <UBadge color="primary" variant="subtle" size="xs">
        {{ t('inbox.sms.segmentsCount', { count: smsAnalysis.segmentCount }) }}
      </UBadge>
      <span class="text-muted">
        {{ t('inbox.sms.unitsSummary', {
          units: smsAnalysis.unitCount,
          limit: smsAnalysis.segmentCount <= 1 ? smsAnalysis.maxSingleSegmentUnits : smsAnalysis.maxMultiSegmentUnits,
          remaining: smsAnalysis.charactersRemainingInSegment,
        }) }}
      </span>
    </div>

    <div class="rounded-xl border border-default bg-elevated p-4">
      <div class="mx-auto max-w-sm rounded-2xl border border-default bg-muted/30 p-3 space-y-2">
        <div class="flex items-center justify-between border-b border-default/60 pb-2 text-[11px] text-muted">
          <span class="font-semibold text-highlighted">{{ sender }}</span>
          <span>{{ formatShortTimestamp(notification.receivedAt, locale) }}</span>
        </div>
        <div class="rounded-2xl rounded-ss-sm bg-primary/15 px-3.5 py-2.5 text-sm leading-relaxed text-highlighted">
          {{ messageText }}
        </div>
      </div>
    </div>

    <div class="space-y-1.5">
      <h3 class="text-xs font-semibold text-muted">
        {{ t('inbox.sms.metadataLabel') }}
      </h3>
      <SharedCodeBlock
        :code="metadataJson"
        language="json"
        max-height-class="max-h-56"
      />
    </div>
  </div>
</template>
