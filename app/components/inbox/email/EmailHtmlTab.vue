<script setup lang="ts">
import InboxDeviceFrameWrapper, { type SimulatedDeviceType } from '~/components/inbox/DeviceFrameWrapper.vue'
import type { Notification } from '~/types/application'

export type EmailViewportMode = SimulatedDeviceType

const props = defineProps<{
  notification: Notification
  sandboxDocument: string
  viewportMode: EmailViewportMode
  isEmailDarkSimulated: boolean
}>()

const emit = defineEmits<{
  'update:viewportMode': [mode: EmailViewportMode]
  'update:isEmailDarkSimulated': [value: boolean]
}>()

const { t } = useI18n()
</script>

<template>
  <div class="space-y-3">
    <div class="flex flex-wrap items-center justify-between gap-2 rounded-lg border border-default bg-muted/30 px-2.5 py-1.5">
      <div class="flex flex-wrap items-center gap-1">
        <UButton
          size="xs"
          :color="props.viewportMode === 'desktop' ? 'primary' : 'neutral'"
          :variant="props.viewportMode === 'desktop' ? 'soft' : 'ghost'"
          icon="i-lucide-monitor"
          :label="t('inbox.email.viewport.desktop')"
          @click="emit('update:viewportMode', 'desktop')"
        />
        <UButton
          size="xs"
          :color="props.viewportMode === 'iphone18Pro' ? 'primary' : 'neutral'"
          :variant="props.viewportMode === 'iphone18Pro' ? 'soft' : 'ghost'"
          icon="i-lucide-smartphone"
          :label="t('inbox.email.viewport.iphone18Pro')"
          @click="emit('update:viewportMode', 'iphone18Pro')"
        />
        <UButton
          size="xs"
          :color="props.viewportMode === 'galaxyS28Plus' ? 'primary' : 'neutral'"
          :variant="props.viewportMode === 'galaxyS28Plus' ? 'soft' : 'ghost'"
          icon="i-lucide-smartphone"
          :label="t('inbox.email.viewport.galaxyS28Plus')"
          @click="emit('update:viewportMode', 'galaxyS28Plus')"
        />
        <UButton
          size="xs"
          :color="props.viewportMode === 'ipadPro' ? 'primary' : 'neutral'"
          :variant="props.viewportMode === 'ipadPro' ? 'soft' : 'ghost'"
          icon="i-lucide-tablet"
          :label="t('inbox.email.viewport.ipadPro')"
          @click="emit('update:viewportMode', 'ipadPro')"
        />
      </div>

      <UButton
        size="xs"
        :color="props.isEmailDarkSimulated ? 'primary' : 'neutral'"
        :variant="props.isEmailDarkSimulated ? 'soft' : 'outline'"
        :icon="props.isEmailDarkSimulated ? 'i-lucide-moon' : 'i-lucide-sun'"
        :label="props.isEmailDarkSimulated ? t('inbox.email.viewport.simulateLight') : t('inbox.email.viewport.simulateDark')"
        @click="emit('update:isEmailDarkSimulated', !props.isEmailDarkSimulated)"
      />
    </div>

    <div class="overflow-x-auto rounded-lg border border-default bg-muted/20 p-4">
      <InboxDeviceFrameWrapper
        :device="props.viewportMode"
        :show-status-bar="true"
        :show-home-indicator="true"
      >
        <iframe
          sandbox=""
          referrerpolicy="no-referrer"
          :title="props.notification.subject || t('inbox.email.tabs.htmlPreview')"
          :srcdoc="props.sandboxDocument"
          class="h-[480px] w-full border-0 bg-default"
        />
      </InboxDeviceFrameWrapper>
    </div>
  </div>
</template>
