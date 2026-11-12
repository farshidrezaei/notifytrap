<script setup lang="ts">
import type { Notification } from '~/types/application'
import { formatShortTimestamp } from '~/utils/formatters'

type PushPreviewDevice = 'ios' | 'android' | 'web'
type PushViewMode = 'banner' | 'lockScreen' | 'notificationShade'

const props = defineProps<{
  notification: Notification
}>()

const { t, locale } = useI18n()

const payload = computed(() => props.notification.payloadJson)
const platform = computed(() => String(payload.value.platform ?? props.notification.sender))
const deviceToken = computed(() => String(payload.value.deviceToken ?? props.notification.recipient))
const pushTitle = computed(() => String(payload.value.title ?? props.notification.subject ?? 'Notification'))
const pushBody = computed(() => String(payload.value.body ?? props.notification.previewText ?? ''))
const badgeValue = computed(() => payload.value.badge ?? null)
const soundValue = computed(() => (payload.value.sound ? String(payload.value.sound) : null))
const customDataJson = computed(() => JSON.stringify(payload.value.data ?? {}, null, 2))

const previewDevice = ref<PushPreviewDevice>('ios')
const viewMode = ref<PushViewMode>('lockScreen')

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
    <!-- Header bar with platform, tokens, and mode controls -->
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

      <div class="flex flex-wrap items-center gap-1.5">
        <!-- View mode switcher -->
        <div class="flex items-center gap-1 border-e border-default pe-1.5 me-1.5">
          <UButton
            size="xs"
            :color="viewMode === 'lockScreen' ? 'primary' : 'neutral'"
            :variant="viewMode === 'lockScreen' ? 'soft' : 'ghost'"
            icon="i-lucide-smartphone"
            :label="t('inbox.push.views.lockScreen')"
            @click="viewMode = 'lockScreen'"
          />
          <UButton
            size="xs"
            :color="viewMode === 'banner' ? 'primary' : 'neutral'"
            :variant="viewMode === 'banner' ? 'soft' : 'ghost'"
            icon="i-lucide-bell-ring"
            :label="t('inbox.push.views.banner')"
            @click="viewMode = 'banner'"
          />
          <UButton
            size="xs"
            :color="viewMode === 'notificationShade' ? 'primary' : 'neutral'"
            :variant="viewMode === 'notificationShade' ? 'soft' : 'ghost'"
            icon="i-lucide-panel-top-open"
            :label="t('inbox.push.views.notificationShade')"
            @click="viewMode = 'notificationShade'"
          />
        </div>

        <!-- Platform simulator switcher -->
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
    </div>

    <!-- Interactive Device Simulation Canvas -->
    <div class="rounded-xl border border-default bg-muted/20 p-6 flex justify-center items-center">
      <!-- 1. Lock Screen View -->
      <div
        v-if="viewMode === 'lockScreen'"
        class="relative w-full max-w-sm overflow-hidden rounded-[38px] border-4 border-neutral-800 bg-gradient-to-b from-slate-900 via-indigo-950 to-neutral-950 p-5 text-white shadow-2xl ring-1 ring-default min-h-[460px] flex flex-col justify-between"
      >
        <!-- Lock Screen Status & Time -->
        <div class="space-y-4 text-center select-none pt-2">
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

        <!-- Floating Lock Screen Notification Card -->
        <div class="my-auto space-y-2">
          <div class="rounded-2xl border border-white/15 bg-white/10 p-3.5 backdrop-blur-md shadow-lg transition-all hover:bg-white/15">
            <div class="flex items-center justify-between text-xs text-white/80 pb-2">
              <div class="flex items-center gap-2">
                <div class="flex size-5 items-center justify-center rounded-md bg-primary text-white shadow-sm">
                  <UIcon name="i-lucide-bell" class="size-3" />
                </div>
                <span class="font-semibold text-white/95 capitalize text-[13px]">{{ platform }}</span>
              </div>
              <span class="text-[11px] text-white/70">{{ t('inbox.push.simulator.now') }}</span>
            </div>

            <div class="space-y-1 pt-0.5">
              <h3 class="text-sm font-semibold text-white">
                {{ pushTitle }}
              </h3>
              <p class="text-xs leading-relaxed text-white/85">
                {{ pushBody }}
              </p>
            </div>

            <div class="mt-3 flex items-center justify-end gap-2 border-t border-white/10 pt-2">
              <UButton
                size="xs"
                variant="ghost"
                color="neutral"
                class="text-white/80 hover:text-white"
                :label="t('inbox.push.simulator.actionDismiss')"
              />
              <UButton
                size="xs"
                variant="soft"
                color="primary"
                :label="t('inbox.push.simulator.actionOpen')"
              />
            </div>
          </div>
        </div>

        <!-- Lock screen bottom utilities -->
        <div class="flex items-center justify-between px-4 pb-2 text-white/80 select-none">
          <div class="flex size-10 items-center justify-center rounded-full bg-white/10 backdrop-blur-md">
            <UIcon name="i-lucide-flashlight" class="size-4" />
          </div>
          <div class="h-1 w-28 rounded-full bg-white/40" />
          <div class="flex size-10 items-center justify-center rounded-full bg-white/10 backdrop-blur-md">
            <UIcon name="i-lucide-camera" class="size-4" />
          </div>
        </div>
      </div>

      <!-- 2. Dynamic Banner View -->
      <div
        v-else-if="viewMode === 'banner'"
        class="w-full max-w-md space-y-4"
      >
        <div class="relative overflow-hidden rounded-2xl border border-default bg-elevated p-4 shadow-xl transition-all">
          <div class="flex items-start gap-3">
            <div class="flex size-10 shrink-0 items-center justify-center rounded-xl bg-primary/20 text-primary">
              <UIcon
                :name="previewDevice === 'web' ? 'i-lucide-globe' : 'i-lucide-bell'"
                class="size-5"
              />
            </div>
            <div class="min-w-0 flex-1 space-y-1">
              <div class="flex items-center justify-between text-xs text-muted">
                <span class="font-semibold uppercase text-highlighted">{{ platform }}</span>
                <span>{{ t('inbox.push.simulator.now') }}</span>
              </div>
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

      <!-- 3. Notification Shade View (Android / OS Center) -->
      <div
        v-else
        class="w-full max-w-md rounded-2xl border border-default bg-muted/40 p-3 space-y-2"
      >
        <div class="rounded-xl border border-default bg-elevated p-3.5 space-y-2 shadow-sm">
          <div class="flex items-center justify-between text-xs text-muted">
            <div class="flex items-center gap-1.5">
              <UIcon name="i-lucide-bell" class="size-3.5 text-primary" />
              <span class="font-medium uppercase">{{ platform }}</span>
              <span>&bull;</span>
              <span>{{ formatShortTimestamp(notification.receivedAt, locale) }}</span>
            </div>
            <UBadge v-if="soundValue" color="neutral" variant="subtle" size="xs">
              {{ soundValue }}
            </UBadge>
          </div>

          <div class="space-y-1">
            <h4 class="text-sm font-semibold text-highlighted">{{ pushTitle }}</h4>
            <p class="text-xs leading-relaxed text-toned">{{ pushBody }}</p>
          </div>

          <div class="flex items-center gap-2 pt-2 border-t border-default/50">
            <UButton
              size="xs"
              variant="outline"
              color="neutral"
              :label="t('inbox.push.simulator.actionDismiss')"
            />
            <UButton
              size="xs"
              variant="solid"
              color="primary"
              :label="t('inbox.push.simulator.actionOpen')"
            />
          </div>
        </div>
      </div>
    </div>

    <!-- Custom Data Payload Section -->
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
