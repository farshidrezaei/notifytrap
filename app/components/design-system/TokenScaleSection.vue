<script setup lang="ts">
import {
  MOTION_TOKENS,
  RADIUS_TOKENS,
  SHADOW_TOKENS,
  SIZING_TOKENS,
  SPACING_TOKENS,
} from '~/utils/designTokens'

const { t } = useI18n()
const isMotionActive = ref(false)
</script>

<template>
  <div class="flex flex-col gap-6">
    <UCard>
      <template #header>
        <SharedSectionHeader
          :title="t('designSystem.sections.spacing')"
          :description="t('designSystem.descriptions.spacing')"
        />
      </template>

      <div class="grid grid-cols-1 gap-2 sm:grid-cols-2 lg:grid-cols-3">
        <div
          v-for="item in SPACING_TOKENS"
          :key="item.step"
          class="flex items-center justify-between gap-3 rounded-md border border-default bg-muted/30 px-3 py-2"
        >
          <div class="flex items-center gap-2.5 min-w-0">
            <span class="font-mono text-xs font-medium text-highlighted" dir="ltr">
              {{ item.step }}
            </span>
            <span class="truncate font-mono text-[11px] text-muted" dir="ltr">
              {{ item.pixelValue }}px
            </span>
          </div>
          <div
            class="h-2.5 shrink-0 rounded-xs bg-primary"
            :style="{ width: `${Math.min(item.pixelValue, 96)}px` }"
          />
        </div>
      </div>
    </UCard>

    <div class="grid grid-cols-1 gap-6 lg:grid-cols-2">
      <UCard>
        <template #header>
          <SharedSectionHeader :title="t('designSystem.sections.radius')" />
        </template>

        <div class="grid grid-cols-2 gap-3 sm:grid-cols-3">
          <div
            v-for="item in RADIUS_TOKENS"
            :key="item.id"
            class="flex flex-col items-center justify-center gap-2 border border-default bg-muted/30 p-3"
          >
            <div
              class="size-12 border-2 border-primary bg-primary/15"
              :style="{ borderRadius: `var(${item.token})` }"
            />
            <span class="text-xs font-medium text-highlighted">
              {{ t(`designSystem.radius.${item.id}`) }}
            </span>
            <span class="font-mono text-[11px] text-muted" dir="ltr">{{ item.value }}</span>
          </div>
        </div>
      </UCard>

      <UCard>
        <template #header>
          <SharedSectionHeader :title="t('designSystem.sections.shadows')" />
        </template>

        <div class="grid grid-cols-2 gap-3 sm:grid-cols-3">
          <div
            v-for="item in SHADOW_TOKENS"
            :key="item.id"
            class="flex flex-col items-center justify-center gap-2 rounded-md border border-default bg-elevated p-4"
            :style="{ boxShadow: `var(${item.token})` }"
          >
            <span class="text-xs font-medium text-highlighted">
              {{ t(`designSystem.shadows.${item.id}`) }}
            </span>
            <span class="truncate font-mono text-[11px] text-muted" dir="ltr">
              {{ item.token }}
            </span>
          </div>
        </div>
      </UCard>
    </div>

    <div class="grid grid-cols-1 gap-6 lg:grid-cols-2">
      <UCard>
        <template #header>
          <SharedSectionHeader :title="t('designSystem.sections.sizing')" />
        </template>

        <div class="divide-y divide-default rounded-md border border-default">
          <div
            v-for="item in SIZING_TOKENS"
            :key="item.id"
            class="flex items-center justify-between gap-3 px-3 py-2 text-xs"
          >
            <span class="font-medium text-highlighted">{{ t(`designSystem.sizing.${item.id}`) }}</span>
            <span class="font-mono text-muted" dir="ltr">{{ item.value }}</span>
          </div>
        </div>
      </UCard>

      <UCard>
        <template #header>
          <SharedSectionHeader
            :title="t('designSystem.sections.motion')"
            :description="t('designSystem.descriptions.motion')"
          >
            <template #actions>
              <UButton
                size="xs"
                color="primary"
                variant="subtle"
                :label="t('designSystem.motion.trigger')"
                @click="isMotionActive = !isMotionActive"
              />
            </template>
          </SharedSectionHeader>
        </template>

        <div class="flex flex-col gap-3">
          <div
            v-for="item in MOTION_TOKENS"
            :key="item.id"
            class="flex items-center justify-between gap-3 rounded-md border border-default bg-muted/30 px-3 py-2"
          >
            <div>
              <p class="text-xs font-medium text-highlighted">{{ t(`designSystem.motion.${item.id}`) }}</p>
              <p class="font-mono text-[11px] text-muted" dir="ltr">{{ item.value }}</p>
            </div>
            <div dir="ltr" class="h-3 w-28 overflow-hidden rounded-full bg-accented p-0.5">
              <div
                class="h-full w-8 rounded-full bg-primary transition-transform"
                :style="{
                  transitionDuration: item.id === 'easing' ? 'var(--nt-duration-standard)' : `var(${item.token})`,
                  transitionTimingFunction: 'var(--ease-out)',
                  transform: isMotionActive ? 'translateX(4.75rem)' : 'translateX(0)',
                }"
              />
            </div>
          </div>
        </div>
      </UCard>
    </div>
  </div>
</template>
