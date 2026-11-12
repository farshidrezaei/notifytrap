<script setup lang="ts">
type DeviceType = 'iphone' | 'pixel' | 'ipad'

const props = withDefaults(
  defineProps<{
    device?: DeviceType
    showStatusBar?: boolean
    showHomeIndicator?: boolean
  }>(),
  {
    device: 'iphone',
    showStatusBar: true,
    showHomeIndicator: true,
  },
)

const deviceClasses = computed(() => {
  switch (props.device) {
    case 'iphone':
      return {
        wrapper: 'max-w-[393px] rounded-[44px] p-3 shadow-2xl ring-1 ring-default',
        inner: 'rounded-[34px]',
        dynamicIsland: true,
        pixelCamera: false,
      }
    case 'pixel':
      return {
        wrapper: 'max-w-[412px] rounded-[36px] p-2.5 shadow-2xl ring-1 ring-default',
        inner: 'rounded-[28px]',
        dynamicIsland: false,
        pixelCamera: true,
      }
    case 'ipad':
      return {
        wrapper: 'max-w-[768px] rounded-[28px] p-4 shadow-2xl ring-1 ring-default',
        inner: 'rounded-[18px]',
        dynamicIsland: false,
        pixelCamera: false,
      }
    default:
      return {
        wrapper: 'max-w-full rounded-xl p-2 shadow-sm',
        inner: 'rounded-lg',
        dynamicIsland: false,
        pixelCamera: false,
      }
  }
})
</script>

<template>
  <div
    class="mx-auto w-full transition-all duration-200 border-4 border-neutral-800 dark:border-neutral-700 bg-neutral-900"
    :class="deviceClasses.wrapper"
  >
    <div
      class="relative flex flex-col overflow-hidden bg-background text-foreground"
      :class="deviceClasses.inner"
    >
      <!-- Status bar -->
      <div
        v-if="showStatusBar"
        class="relative z-10 flex h-10 w-full shrink-0 items-center justify-between px-6 text-xs font-medium select-none bg-background/90 backdrop-blur-sm"
      >
        <span dir="ltr" class="font-mono text-[11px] font-semibold">09:41</span>

        <!-- iPhone Dynamic Island -->
        <div
          v-if="deviceClasses.dynamicIsland"
          class="absolute inset-x-0 top-2 mx-auto h-6 w-24 rounded-full bg-black flex items-center justify-between px-2.5"
        >
          <div class="size-2.5 rounded-full bg-neutral-900 border border-neutral-800" />
          <div class="size-2 rounded-full bg-emerald-500/80" />
        </div>

        <!-- Pixel camera pinhole -->
        <div
          v-else-if="deviceClasses.pixelCamera"
          class="absolute inset-x-0 top-2.5 mx-auto size-3.5 rounded-full bg-neutral-950 border border-neutral-800"
        />

        <div dir="ltr" class="flex items-center gap-1.5 text-muted">
          <UIcon name="i-lucide-signal" class="size-3" />
          <UIcon name="i-lucide-wifi" class="size-3" />
          <UIcon name="i-lucide-battery-medium" class="size-3.5 text-highlighted" />
        </div>
      </div>

      <!-- Main screen content slot -->
      <div class="relative flex-1 min-h-[380px] overflow-auto">
        <slot />
      </div>

      <!-- Home indicator bar -->
      <div
        v-if="showHomeIndicator"
        class="z-10 flex h-6 w-full shrink-0 items-center justify-center bg-background/90"
      >
        <div class="h-1 w-32 rounded-full bg-neutral-400/60 dark:bg-neutral-600/60" />
      </div>
    </div>
  </div>
</template>
