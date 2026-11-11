<script setup lang="ts">
import {
  BRAND_AND_STATUS_COLOR_TOKENS,
  CHANNEL_TYPES,
  SURFACE_AND_CONTROL_COLOR_TOKENS,
} from '~/utils/designTokens'

const { t } = useI18n()
</script>

<template>
  <UCard>
    <template #header>
      <SharedSectionHeader
        :title="t('designSystem.sections.colors')"
        :description="t('designSystem.descriptions.colors')"
      />
    </template>

    <div class="flex flex-col gap-6">
      <div>
        <h3 class="mb-3 text-xs font-semibold uppercase tracking-wider text-muted">
          {{ t('designSystem.colors.semanticGroup') }}
        </h3>
        <div class="grid grid-cols-2 gap-3 sm:grid-cols-4 lg:grid-cols-7">
          <div
            v-for="item in BRAND_AND_STATUS_COLOR_TOKENS"
            :key="item.id"
            class="overflow-hidden rounded-md border border-default bg-elevated"
          >
            <div class="h-12 w-full" :style="item.surfaceStyle" />
            <div class="p-2">
              <p class="text-xs font-medium text-highlighted">
                {{ t(`designSystem.colors.tokens.${item.id}`) }}
              </p>
              <p class="mt-0.5 truncate font-mono text-[11px] text-muted" dir="ltr">
                {{ item.token }}
              </p>
            </div>
          </div>
        </div>
      </div>

      <div>
        <h3 class="mb-3 text-xs font-semibold uppercase tracking-wider text-muted">
          {{ t('designSystem.colors.surfaceGroup') }}
        </h3>
        <div class="grid grid-cols-2 gap-3 sm:grid-cols-3 lg:grid-cols-6">
          <div
            v-for="item in SURFACE_AND_CONTROL_COLOR_TOKENS"
            :key="item.id"
            class="overflow-hidden rounded-md border border-default bg-elevated"
          >
            <div class="h-12 w-full border-b border-default" :style="item.surfaceStyle" />
            <div class="p-2">
              <p class="text-xs font-medium text-highlighted">
                {{ t(`designSystem.colors.tokens.${item.id}`) }}
              </p>
              <p class="mt-0.5 truncate font-mono text-[11px] text-muted" dir="ltr">
                {{ item.token }}
              </p>
            </div>
          </div>
        </div>
      </div>

      <div>
        <h3 class="mb-3 text-xs font-semibold uppercase tracking-wider text-muted">
          {{ t('designSystem.colors.channelGroup') }}
        </h3>
        <div class="flex flex-wrap items-center gap-3">
          <SharedChannelBadge
            v-for="channelType in CHANNEL_TYPES"
            :key="channelType"
            :channel-type="channelType"
          />
        </div>
      </div>
    </div>
  </UCard>
</template>
