<script setup lang="ts">
export type SimulatedDeviceType = 'iphone18Pro' | 'galaxyS28Plus' | 'ipadPro' | 'desktop'

const props = withDefaults(
  defineProps<{
    device?: SimulatedDeviceType
    showStatusBar?: boolean
    showHomeIndicator?: boolean
  }>(),
  {
    device: 'iphone18Pro',
    showStatusBar: true,
    showHomeIndicator: true,
  },
)
</script>

<template>
  <!-- Desktop Browser Window Frame -->
  <div
    v-if="props.device === 'desktop'"
    class="mx-auto w-full max-w-4xl overflow-hidden rounded-xl border border-default bg-background shadow-2xl transition-all duration-200"
  >
    <div class="flex h-9 shrink-0 items-center justify-between border-b border-default bg-muted/40 px-3 select-none">
      <div class="flex items-center gap-1.5">
        <span class="size-3 rounded-full border border-[#e0443e] bg-[#ff5f56]" />
        <span class="size-3 rounded-full border border-[#dea123] bg-[#ffbd2e]" />
        <span class="size-3 rounded-full border border-[#1aab29] bg-[#27c93f]" />
      </div>

      <div class="flex h-6 max-w-sm flex-1 items-center justify-center gap-1.5 rounded-md border border-default bg-background/90 px-3 text-[11px] text-muted">
        <UIcon name="i-lucide-lock" class="size-3 text-success" />
        <span dir="ltr" class="font-mono text-[11px]">notifytrap://email-sandbox/preview</span>
      </div>

      <div class="flex items-center gap-2 text-muted">
        <UIcon name="i-lucide-rotate-cw" class="size-3.5" />
      </div>
    </div>

    <div class="relative h-[620px] w-full overflow-hidden bg-default">
      <slot />
    </div>
  </div>

  <!-- Apple iPhone 18 Pro Chassis (19.5:9 aspect ratio, Dynamic Island, ultra-slim bezel) -->
  <div
    v-else-if="props.device === 'iphone18Pro'"
    class="mx-auto w-[393px] max-w-full rounded-[52px] border-[6px] border-neutral-800 bg-neutral-900 p-2.5 shadow-2xl ring-1 ring-white/10 transition-all duration-200 dark:border-neutral-700"
  >
    <div class="relative flex h-[810px] w-full flex-col overflow-hidden rounded-[42px] bg-background text-foreground">
      <!-- Status bar with Dynamic Island -->
      <div
        v-if="props.showStatusBar"
        class="relative z-10 flex h-11 w-full shrink-0 select-none items-center justify-between bg-background/90 px-7 text-xs font-medium backdrop-blur-sm"
      >
        <span dir="ltr" class="font-mono text-[12px] font-semibold">09:41</span>

        <!-- Dynamic Island Pill -->
        <div class="absolute inset-x-0 top-2 mx-auto flex h-6 w-24 items-center justify-between rounded-full bg-black px-2.5 shadow-md">
          <span class="size-2.5 rounded-full border border-neutral-800/80 bg-neutral-900" />
          <span class="size-2 rounded-full bg-emerald-500/80" />
        </div>

        <div dir="ltr" class="flex items-center gap-1.5 text-muted">
          <UIcon name="i-lucide-signal" class="size-3 text-highlighted" />
          <span class="text-[10px] font-semibold text-highlighted">5G</span>
          <UIcon name="i-lucide-wifi" class="size-3 text-highlighted" />
          <UIcon name="i-lucide-battery-full" class="size-3.5 text-highlighted" />
        </div>
      </div>

      <!-- Main screen content -->
      <div class="relative min-h-0 w-full flex-1 overflow-hidden">
        <slot />
      </div>

      <!-- iOS Home indicator bar -->
      <div
        v-if="props.showHomeIndicator"
        class="z-10 flex h-6 w-full shrink-0 items-center justify-center bg-background/90 backdrop-blur-sm"
      >
        <div class="h-1 w-32 rounded-full bg-neutral-400/60 dark:bg-neutral-600/60" />
      </div>
    </div>
  </div>

  <!-- Samsung Galaxy S28+ Chassis (19.5:9 aspect ratio, center punch-hole camera) -->
  <div
    v-else-if="props.device === 'galaxyS28Plus'"
    class="mx-auto w-[390px] max-w-full rounded-[40px] border-[5px] border-neutral-800 bg-neutral-900 p-2 shadow-2xl ring-1 ring-white/10 transition-all duration-200 dark:border-neutral-700"
  >
    <div class="relative flex h-[810px] w-full flex-col overflow-hidden rounded-[32px] bg-background text-foreground">
      <!-- Status bar with centered camera punch-hole -->
      <div
        v-if="props.showStatusBar"
        class="relative z-10 flex h-10 w-full shrink-0 select-none items-center justify-between bg-background/90 px-6 text-xs font-medium backdrop-blur-sm"
      >
        <span dir="ltr" class="font-sans text-[12px] font-medium">10:00</span>

        <!-- Pinhole Camera -->
        <div class="absolute inset-x-0 top-2.5 mx-auto flex size-3.5 items-center justify-center rounded-full border border-neutral-800 bg-neutral-950 shadow-inner">
          <span class="size-1.5 rounded-full bg-neutral-900" />
        </div>

        <div dir="ltr" class="flex items-center gap-1.5 text-muted">
          <span class="text-[10px] font-bold text-highlighted">5G</span>
          <UIcon name="i-lucide-signal" class="size-3 text-highlighted" />
          <UIcon name="i-lucide-wifi" class="size-3 text-highlighted" />
          <span class="font-mono text-[10px] text-highlighted">100%</span>
          <UIcon name="i-lucide-battery" class="size-3 text-highlighted" />
        </div>
      </div>

      <!-- Main screen content -->
      <div class="relative min-h-0 w-full flex-1 overflow-hidden">
        <slot />
      </div>

      <!-- Android Navigation Line -->
      <div
        v-if="props.showHomeIndicator"
        class="z-10 flex h-5 w-full shrink-0 items-center justify-center bg-background/90 backdrop-blur-sm"
      >
        <div class="h-1 w-28 rounded-full bg-neutral-400/60 dark:bg-neutral-600/60" />
      </div>
    </div>
  </div>

  <!-- Apple iPad Pro Chassis (3:4 aspect ratio tablet) -->
  <div
    v-else-if="props.device === 'ipadPro'"
    class="mx-auto w-full max-w-[620px] rounded-[30px] border-[8px] border-neutral-800 bg-neutral-900 p-3 shadow-2xl ring-1 ring-white/10 transition-all duration-200 dark:border-neutral-700"
  >
    <div class="relative flex h-[820px] w-full flex-col overflow-hidden rounded-[18px] bg-background text-foreground">
      <!-- Tablet status bar -->
      <div
        v-if="props.showStatusBar"
        class="relative z-10 flex h-9 w-full shrink-0 select-none items-center justify-between bg-background/90 px-6 text-xs font-medium backdrop-blur-sm"
      >
        <span dir="ltr" class="font-sans text-[11px] font-medium text-highlighted">09:41</span>

        <!-- Top Bezel Center Camera Sensor -->
        <div class="absolute inset-x-0 top-2 mx-auto size-2.5 rounded-full border border-neutral-800 bg-neutral-950 shadow-inner" />

        <div dir="ltr" class="flex items-center gap-2 text-muted">
          <UIcon name="i-lucide-wifi" class="size-3.5 text-highlighted" />
          <span class="font-mono text-[10px] font-medium text-highlighted">100%</span>
          <UIcon name="i-lucide-battery-full" class="size-4 text-highlighted" />
        </div>
      </div>

      <!-- Main screen content -->
      <div class="relative min-h-0 w-full flex-1 overflow-hidden">
        <slot />
      </div>

      <!-- Tablet Home indicator bar -->
      <div
        v-if="props.showHomeIndicator"
        class="z-10 flex h-6 w-full shrink-0 items-center justify-center bg-background/90 backdrop-blur-sm"
      >
        <div class="h-1 w-44 rounded-full bg-neutral-400/60 dark:bg-neutral-600/60" />
      </div>
    </div>
  </div>
</template>
