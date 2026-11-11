<script setup lang="ts">
import type { ChannelKind } from '~/utils/designTokens'

const props = defineProps<{
  channelType: ChannelKind
}>()

const { t } = useI18n()

const CHANNEL_ICON_MAP: Record<ChannelKind, string> = {
  email: 'i-lucide-mail',
  sms: 'i-lucide-message-square',
  push: 'i-lucide-bell',
  webhook: 'i-lucide-webhook',
}

const badgeStyle = computed<Record<string, string>>(() => ({
  color: `var(--nt-channel-${props.channelType})`,
  backgroundColor: `var(--nt-channel-${props.channelType}-bg)`,
}))
</script>

<template>
  <span
    class="inline-flex items-center gap-1.5 rounded-md px-2 py-0.5 text-xs font-medium"
    :style="badgeStyle"
  >
    <UIcon :name="CHANNEL_ICON_MAP[channelType]" class="size-3.5 shrink-0" />
    <span>{{ t(`channels.types.${channelType}`) }}</span>
  </span>
</template>
