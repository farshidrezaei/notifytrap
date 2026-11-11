<script setup lang="ts">
import { TYPOGRAPHY_SCALE } from '~/utils/designTokens'

const { t } = useI18n()
</script>

<template>
  <UCard>
    <template #header>
      <SharedSectionHeader
        :title="t('designSystem.sections.typography')"
        :description="t('designSystem.descriptions.typography')"
      />
    </template>

    <div class="flex flex-col gap-5">
      <div class="grid grid-cols-1 gap-3 md:grid-cols-3">
        <div class="rounded-md border border-default bg-muted/40 p-3">
          <p class="text-xs font-medium text-muted">{{ t('designSystem.typography.latinFont') }}</p>
          <p class="mt-1 text-sm font-semibold text-highlighted" dir="ltr">IBM Plex Sans Variable</p>
        </div>
        <div class="rounded-md border border-default bg-muted/40 p-3">
          <p class="text-xs font-medium text-muted">{{ t('designSystem.typography.persianFont') }}</p>
          <p class="mt-1 text-sm font-semibold text-highlighted" dir="rtl">وزیرمتن متغیر (Vazirmatn)</p>
        </div>
        <div class="rounded-md border border-default bg-muted/40 p-3">
          <p class="text-xs font-medium text-muted">{{ t('designSystem.typography.monoFont') }}</p>
          <p class="mt-1 font-mono text-sm font-medium text-highlighted" dir="ltr">JetBrains Mono Variable</p>
        </div>
      </div>

      <div class="divide-y divide-default rounded-md border border-default">
        <div
          v-for="entry in TYPOGRAPHY_SCALE"
          :key="entry.id"
          class="flex flex-col gap-2 p-3 sm:flex-row sm:items-center sm:justify-between"
        >
          <div class="min-w-0 flex-1">
            <span :class="entry.className">
              {{ t(`designSystem.typography.scale.${entry.id}`) }}
            </span>
          </div>
          <div class="flex flex-wrap items-center gap-3 font-mono text-xs text-muted" dir="ltr">
            <span>{{ entry.fontSize }}</span>
            <span>LH {{ entry.lineHeight }}</span>
            <span>W {{ entry.fontWeight }}</span>
            <span>LS {{ entry.letterSpacing }}</span>
          </div>
        </div>
      </div>

      <div class="rounded-md border border-default bg-muted/40 p-3 font-mono text-xs text-toned" dir="ltr">
        POST http://127.0.0.1:4080/api/v1/push | X-Channel-Key: nt_ch_arsam.9f4a21c8 | 202 Accepted
      </div>
    </div>
  </UCard>
</template>
