<script setup lang="ts">
import type { Notification } from '~/types/application'

const props = defineProps<{
  notification: Notification
}>()

const { t } = useI18n()

const payload = computed(() => props.notification.payloadJson)
const sender = computed(() => String(payload.value.from ?? props.notification.sender))
const recipient = computed(() => String(payload.value.to ?? props.notification.recipient))
const messageText = computed(() => String(payload.value.message ?? props.notification.previewText))
const metadataJson = computed(() => JSON.stringify(payload.value.metadata ?? {}, null, 2))
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
          {{ t('inbox.sms.charactersCount', { count: messageText.length }) }}
        </p>
      </div>
    </div>

    <div class="rounded-xl border border-default bg-elevated p-4">
      <div class="max-w-md rounded-2xl rounded-ss-sm bg-primary/15 px-4 py-3 text-sm leading-relaxed text-highlighted">
        {{ messageText }}
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
