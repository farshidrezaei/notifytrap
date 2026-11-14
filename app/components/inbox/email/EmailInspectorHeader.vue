<script setup lang="ts">
import type { Notification } from '~/types/application'

const props = defineProps<{
  notification: Notification
  fromAddresses: readonly string[]
  toAddresses: readonly string[]
  ccAddresses: readonly string[]
  bccAddresses: readonly string[]
}>()

const { t } = useI18n()
</script>

<template>
  <div class="grid gap-2 rounded-lg border border-default bg-muted/30 p-3 text-xs">
    <div class="flex flex-wrap gap-2">
      <span class="w-16 shrink-0 font-medium text-muted">{{ t('inbox.email.fields.from') }}</span>
      <span dir="ltr" class="font-mono text-highlighted">
        {{ props.fromAddresses.join(', ') || props.notification.sender }}
      </span>
    </div>
    <div class="flex flex-wrap gap-2">
      <span class="w-16 shrink-0 font-medium text-muted">{{ t('inbox.email.fields.to') }}</span>
      <span dir="ltr" class="font-mono text-highlighted">
        {{ props.toAddresses.join(', ') || props.notification.recipient }}
      </span>
    </div>
    <div v-if="props.ccAddresses.length > 0" class="flex flex-wrap gap-2">
      <span class="w-16 shrink-0 font-medium text-muted">{{ t('inbox.email.fields.cc') }}</span>
      <span dir="ltr" class="font-mono text-toned">{{ props.ccAddresses.join(', ') }}</span>
    </div>
    <div v-if="props.bccAddresses.length > 0" class="flex flex-wrap gap-2">
      <span class="w-16 shrink-0 font-medium text-muted">{{ t('inbox.email.fields.bcc') }}</span>
      <span dir="ltr" class="font-mono text-toned">{{ props.bccAddresses.join(', ') }}</span>
    </div>
  </div>
</template>
