<script setup lang="ts">
import type { Notification } from '~/types/application'
import { formatShortTimestamp } from '~/utils/formatters'
import { analyzeSmsMessage } from '~/utils/smsAnalyzer'

type SmsViewMode = 'conversation' | 'lockScreen'

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

const viewMode = ref<SmsViewMode>('conversation')
</script>

<template>
  <div class="flex flex-col gap-4">
    <!-- Sender/Recipient/Metrics Stats Grid -->
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

    <!-- PDU & Encoding Analysis Bar -->
    <div class="flex flex-wrap items-center justify-between gap-2 rounded-lg border border-default bg-muted/20 px-3 py-2 text-xs">
      <div class="flex flex-wrap items-center gap-2">
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

      <!-- View Switcher -->
      <div class="flex items-center gap-1">
        <UButton
          size="xs"
          :color="viewMode === 'conversation' ? 'primary' : 'neutral'"
          :variant="viewMode === 'conversation' ? 'soft' : 'ghost'"
          icon="i-lucide-message-square"
          :label="t('inbox.sms.views.conversation')"
          @click="viewMode = 'conversation'"
        />
        <UButton
          size="xs"
          :color="viewMode === 'lockScreen' ? 'primary' : 'neutral'"
          :variant="viewMode === 'lockScreen' ? 'soft' : 'ghost'"
          icon="i-lucide-smartphone"
          :label="t('inbox.sms.views.lockScreen')"
          @click="viewMode = 'lockScreen'"
        />
      </div>
    </div>

    <!-- Realistic Mobile Simulation -->
    <div class="rounded-xl border border-default bg-muted/20 p-6 flex justify-center items-center">
      <!-- 1. Conversation Chat Bubble View -->
      <div
        v-if="viewMode === 'conversation'"
        class="w-full max-w-sm overflow-hidden rounded-[36px] border-4 border-neutral-800 bg-background text-foreground shadow-2xl ring-1 ring-default min-h-[440px] flex flex-col justify-between"
      >
        <!-- Contact Header -->
        <div class="border-b border-default bg-muted/40 px-4 py-3 text-center space-y-1.5 select-none">
          <div class="mx-auto flex size-12 items-center justify-center rounded-full bg-primary/20 text-primary font-semibold text-base">
            {{ sender.charAt(0).toUpperCase() || '#' }}
          </div>
          <div class="space-y-0.5">
            <h4 dir="ltr" class="text-xs font-semibold text-highlighted font-mono">
              {{ sender }}
            </h4>
            <p class="text-[10px] text-muted">{{ t('inbox.sms.toLabel') }}: {{ recipient }}</p>
          </div>
        </div>

        <!-- Chat Bubble Thread -->
        <div class="flex-1 p-4 space-y-3 overflow-auto flex flex-col justify-end">
          <div class="text-center text-[10px] font-medium text-muted select-none">
            {{ formatShortTimestamp(notification.receivedAt, locale) }}
          </div>

          <!-- Received Message Bubble (incoming to user) -->
          <div class="flex items-end gap-2 max-w-[85%]">
            <div class="space-y-1">
              <div class="rounded-2xl rounded-es-sm bg-neutral-200 dark:bg-neutral-800 text-neutral-900 dark:text-neutral-100 px-3.5 py-2.5 text-xs leading-relaxed shadow-sm">
                {{ messageText }}
              </div>
              <div class="text-[10px] text-muted ps-1">
                {{ t('inbox.sms.simulator.delivered') }}
              </div>
            </div>
          </div>
        </div>

        <!-- Simulated Input Bar -->
        <div class="border-t border-default bg-muted/30 p-2.5 flex items-center gap-2 select-none">
          <div class="flex-1 rounded-full border border-default bg-background px-3 py-1.5 text-xs text-muted">
            iMessage / SMS
          </div>
          <div class="flex size-7 items-center justify-center rounded-full bg-primary text-white">
            <UIcon name="i-lucide-arrow-up" class="size-4" />
          </div>
        </div>
      </div>

      <!-- 2. Lock Screen Notification View -->
      <div
        v-else
        class="relative w-full max-w-sm overflow-hidden rounded-[38px] border-4 border-neutral-800 bg-gradient-to-b from-stone-900 via-neutral-900 to-zinc-950 p-5 text-white shadow-2xl ring-1 ring-default min-h-[440px] flex flex-col justify-between"
      >
        <div class="space-y-3 text-center select-none pt-2">
          <div class="flex items-center justify-between text-[11px] text-white/70 px-2">
            <span dir="ltr" class="font-mono">09:41</span>
            <div dir="ltr" class="flex items-center gap-1">
              <UIcon name="i-lucide-wifi" class="size-3" />
              <UIcon name="i-lucide-battery-medium" class="size-3.5" />
            </div>
          </div>

          <div class="space-y-1">
            <p class="text-xs font-medium text-white/75">
              {{ formatShortTimestamp(notification.receivedAt, locale) }}
            </p>
            <h2 dir="ltr" class="text-5xl font-light tracking-tight font-mono text-white/95">
              09:41
            </h2>
          </div>
        </div>

        <!-- Lock Screen SMS Card -->
        <div class="my-auto space-y-2">
          <div class="rounded-2xl border border-white/15 bg-white/10 p-3.5 backdrop-blur-md shadow-lg">
            <div class="flex items-center justify-between text-xs text-white/80 pb-2">
              <div class="flex items-center gap-2">
                <div class="flex size-5 items-center justify-center rounded-md bg-emerald-500 text-white shadow-sm">
                  <UIcon name="i-lucide-message-square" class="size-3" />
                </div>
                <span class="font-semibold text-white/95 text-[13px]">{{ t('inbox.sms.simulator.notificationTitle') }}</span>
              </div>
              <span class="text-[11px] text-white/70">{{ formatShortTimestamp(notification.receivedAt, locale) }}</span>
            </div>

            <div class="space-y-1 pt-0.5">
              <h3 dir="ltr" class="text-xs font-semibold text-white font-mono">
                {{ sender }}
              </h3>
              <p class="text-xs leading-relaxed text-white/90">
                {{ messageText }}
              </p>
            </div>

            <div class="mt-3 flex items-center justify-end gap-2 border-t border-white/10 pt-2">
              <UButton
                size="xs"
                variant="soft"
                color="primary"
                :label="t('inbox.sms.simulator.replyAction')"
              />
            </div>
          </div>
        </div>

        <!-- Home bar -->
        <div class="flex justify-center pb-2">
          <div class="h-1 w-28 rounded-full bg-white/40" />
        </div>
      </div>
    </div>

    <!-- Metadata Section -->
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
