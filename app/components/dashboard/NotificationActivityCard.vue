<script setup lang="ts">
import type { ChannelType, NotificationSummary } from '~/types/application'

const props = defineProps<{
  notifications: readonly NotificationSummary[]
}>()

const { t } = useI18n()

interface HourlyBucket {
  label: string
  total: number
  email: number
  sms: number
  push: number
  webhook: number
}

const CHANNEL_COLORS: Record<ChannelType, string> = {
  email: 'bg-sky-500',
  sms: 'bg-emerald-500',
  push: 'bg-indigo-500',
  webhook: 'bg-amber-500',
}

const buckets = computed<HourlyBucket[]>(() => {
  const result: HourlyBucket[] = []
  const now = Date.now()
  const intervalMs = 2 * 60 * 60 * 1000 // 2-hour buckets across 24h (12 buckets)

  for (let i = 11; i >= 0; i -= 1) {
    const bucketStart = now - (i + 1) * intervalMs
    const bucketEnd = now - i * intervalMs
    const dateObj = new Date(bucketEnd)
    const hours = String(dateObj.getHours()).padStart(2, '0')
    const label = `${hours}:00`

    let email = 0
    let sms = 0
    let push = 0
    let webhook = 0

    for (const item of props.notifications) {
      const itemTime = new Date(item.receivedAt).getTime()
      if (itemTime >= bucketStart && itemTime < bucketEnd) {
        if (item.channelType === 'email') email += 1
        else if (item.channelType === 'sms') sms += 1
        else if (item.channelType === 'push') push += 1
        else if (item.channelType === 'webhook') webhook += 1
      }
    }

    result.push({
      label,
      total: email + sms + push + webhook,
      email,
      sms,
      push,
      webhook,
    })
  }

  return result
})

const maxBucketTotal = computed(() => {
  const highest = Math.max(...buckets.value.map((bucket) => bucket.total), 0)
  return highest > 0 ? highest : 10
})

const totalActivityCount = computed(() =>
  buckets.value.reduce((acc, curr) => acc + curr.total, 0),
)

const peakVolume = computed(() =>
  Math.max(...buckets.value.map((bucket) => bucket.total), 0),
)
</script>

<template>
  <UCard>
    <div class="flex flex-wrap items-center justify-between gap-4 border-b border-default pb-3.5">
      <div class="space-y-0.5">
        <h2 class="text-sm font-semibold text-highlighted">
          {{ t('dashboard.activity.title') }}
        </h2>
        <p class="text-xs text-muted">
          {{ t('dashboard.activity.subtitle') }}
        </p>
      </div>

      <div class="flex flex-wrap items-center gap-3 text-xs">
        <div class="flex items-center gap-1.5 rounded-md border border-default bg-muted/40 px-2.5 py-1">
          <span class="text-muted">{{ t('dashboard.activity.totalCaptured') }}:</span>
          <span class="font-mono font-semibold text-highlighted">{{ totalActivityCount }}</span>
        </div>
        <div class="flex items-center gap-1.5 rounded-md border border-default bg-muted/40 px-2.5 py-1">
          <span class="text-muted">{{ t('dashboard.activity.peakHour') }}:</span>
          <span class="font-mono font-semibold text-primary">{{ peakVolume }}/2h</span>
        </div>
      </div>
    </div>

    <!-- Activity Bar Chart Visualization -->
    <div class="pt-4">
      <div class="flex h-36 items-end gap-1.5 sm:gap-2">
        <div
          v-for="(bucket, idx) in buckets"
          :key="`${bucket.label}-${idx}`"
          class="group relative flex flex-1 flex-col items-center justify-end h-full"
        >
          <!-- Tooltip on hover -->
          <div
            class="pointer-events-none absolute -top-8 z-10 hidden whitespace-nowrap rounded-md bg-elevated px-2 py-1 text-[11px] font-medium text-highlighted shadow-md ring-1 ring-default group-hover:block"
          >
            {{ bucket.total }} {{ t('dashboard.activity.hourlyTooltip', { count: bucket.total }) }} ({{ bucket.label }})
          </div>

          <!-- Stacked Bar Column -->
          <div
            class="w-full max-w-[28px] overflow-hidden rounded-t transition-all duration-200 group-hover:opacity-80 flex flex-col-reverse"
            :style="{
              height: bucket.total > 0
                ? `${Math.max(8, Math.round((bucket.total / maxBucketTotal) * 100))}%`
                : '3px',
            }"
            :class="bucket.total === 0 ? 'bg-muted/40' : ''"
          >
            <div
              v-if="bucket.email > 0"
              class="w-full bg-sky-500"
              :style="{ height: `${(bucket.email / bucket.total) * 100}%` }"
            />
            <div
              v-if="bucket.sms > 0"
              class="w-full bg-emerald-500"
              :style="{ height: `${(bucket.sms / bucket.total) * 100}%` }"
            />
            <div
              v-if="bucket.push > 0"
              class="w-full bg-indigo-500"
              :style="{ height: `${(bucket.push / bucket.total) * 100}%` }"
            />
            <div
              v-if="bucket.webhook > 0"
              class="w-full bg-amber-500"
              :style="{ height: `${(bucket.webhook / bucket.total) * 100}%` }"
            />
          </div>

          <!-- Time Label -->
          <span class="mt-2 text-[10px] font-mono text-muted select-none">
            {{ bucket.label }}
          </span>
        </div>
      </div>

      <!-- Legend -->
      <div class="mt-4 flex flex-wrap items-center justify-center gap-4 border-t border-default/60 pt-3 text-xs text-muted">
        <div class="flex items-center gap-1.5">
          <span class="size-2 rounded-full bg-sky-500" />
          <span>{{ t('channels.types.email') }}</span>
        </div>
        <div class="flex items-center gap-1.5">
          <span class="size-2 rounded-full bg-emerald-500" />
          <span>{{ t('channels.types.sms') }}</span>
        </div>
        <div class="flex items-center gap-1.5">
          <span class="size-2 rounded-full bg-indigo-500" />
          <span>{{ t('channels.types.push') }}</span>
        </div>
        <div class="flex items-center gap-1.5">
          <span class="size-2 rounded-full bg-amber-500" />
          <span>{{ t('channels.types.webhook') }}</span>
        </div>
      </div>
    </div>
  </UCard>
</template>
