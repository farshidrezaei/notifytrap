<script setup lang="ts">
import type { Notification } from '~/types/application'
import { formatShortTimestamp } from '~/utils/formatters'

type PushPreviewDevice = 'ios' | 'android' | 'web'

const props = defineProps<{
  notification: Notification
}>()

const { t, locale } = useI18n()

const payload = computed(() => props.notification.payloadJson)
const platform = computed(() => String(payload.value.platform ?? props.notification.sender))
const deviceToken = computed(() => String(payload.value.deviceToken ?? props.notification.recipient))
const pushTitle = computed(() => String(payload.value.title ?? props.notification.subject))
const pushBody = computed(() => String(payload.value.body ?? props.notification.previewText))
const badgeValue = computed(() => payload.value.badge ?? null)
const soundValue = computed(() => (payload.value.sound ? String(payload.value.sound) : null))
const customDataJson = computed(() => JSON.stringify(payload.value.data ?? {}, null, 2))

const previewDevice = ref<PushPreviewDevice>('ios')

watch(
  platform,
  (nextPlatform) => {
    const lower = nextPlatform.toLowerCase()
    if (lower.includes('android') || lower.includes('fcm')) {
      previewDevice.value = 'android'
    } else if (lower.includes('web')) {
      previewDevice.value = 'web'
    } else {
      previewDevice.value = 'ios'
    }
  },
  { immediate: true },
)
</script>

<template>
  <div class="flex flex-col gap-4">
    <div class="flex flex-wrap items-center justify-between gap-2 rounded-lg border border-default bg-muted/30 p-3">
      <div class="flex min-w-0 flex-wrap items-center gap-2">
        <UBadge color="primary" variant="subtle" size="xs" class="uppercase">
          {{ platform }}
        </UBadge>
        <span dir="ltr" class="truncate font-mono text-xs text-toned">
          {{ deviceToken }}
        </span>
        <UBadge v-if="badgeValue !== null" color="neutral" variant="outline" size="xs">
          {{ t('inbox.push.badgeLabel', { value: badgeValue }) }}
        </UBadge>
        <UBadge v-if="soundValue" color="neutral" variant="outline" size="xs">
          {{ t('inbox.push.soundLabel', { value: soundValue }) }}
        </UBadge>
      </div>

      <div class="flex items-center gap-1">
        <UButton
          size="xs"
          :color="previewDevice === 'ios' ? 'primary' : 'neutral'"
          :variant="previewDevice === 'ios' ? 'soft' : 'ghost'"
          :label="t('inbox.push.devices.ios')"
          @click="previewDevice = 'ios'"
        />
        <UButton
          size="xs"
          :color="previewDevice === 'android' ? 'primary' : 'neutral'"
          :variant="previewDevice === 'android' ? 'soft' : 'ghost'"
          :label="t('inbox.push.devices.android')"
          @click="previewDevice = 'android'"
        />
        <UButton
          size="xs"
          :color="previewDevice === 'web' ? 'primary' : 'neutral'"
          :variant="previewDevice === 'web' ? 'soft' : 'ghost'"
          :label="t('inbox.push.devices.web')"
          @click="previewDevice = 'web'"
        />
      </div>
    </div>

    <div class="rounded-xl border border-default bg-muted/20 p-4">
      <div
        class="mx-auto max-w-md border border-default bg-elevated p-3.5 shadow-sm"
        :class="previewDevice === 'ios'
          ? 'rounded-2xl'
          : previewDevice === 'android'
            ? 'rounded-xl'
            : 'rounded-lg'"
      >
        <div class="mb-2 flex items-center justify-between text-[11px] text-muted">
          <div class="flex items-center gap-1.5">
            <UIcon
              :name="previewDevice === 'web' ? 'i-lucide-globe' : 'i-lucide-bell'"
              class="size-3.5 text-primary"
            />
            <span class="font-medium uppercase tracking-wide">
              {{ t(`inbox.push.devices.${previewDevice}`) }}
            </span>
          </div>
          <span>{{ formatShortTimestamp(notification.receivedAt, locale) }}</span>
        </div>

        <div class="flex items-start gap-3">
          <div class="flex size-9 shrink-0 items-center justify-center rounded-lg bg-primary/15 text-primary">
            <UIcon name="i-lucide-bell" class="size-4" />
          </div>
          <div class="min-w-0 flex-1 space-y-1">
            <h3 class="text-sm font-semibold text-highlighted">
              {{ pushTitle }}
            </h3>
            <p class="text-xs leading-relaxed text-toned">
              {{ pushBody }}
            </p>
          </div>
          <UBadge v-if="badgeValue !== null" color="primary" variant="solid" size="xs">
            {{ badgeValue }}
          </UBadge>
        </div>
      </div>
    </div>

    <div class="space-y-1.5">
      <h3 class="text-xs font-semibold text-muted">
        {{ t('inbox.push.dataLabel') }}
      </h3>
      <SharedCodeBlock
        :code="customDataJson"
        language="json"
        max-height-class="max-h-56"
      />
    </div>
  </div>
</template>
