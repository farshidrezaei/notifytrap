<script setup lang="ts">
const { t } = useI18n()
const { locale, changeLocale } = useLocaleController()
const colorMode = useColorMode()

function setTheme(mode: 'light' | 'dark'): void {
  colorMode.preference = mode
}
</script>

<template>
  <UCard>
    <template #header>
      <SharedSectionHeader
        :title="t('designSystem.sections.themeMatrix')"
        :description="t('designSystem.descriptions.themeMatrix')"
      />
    </template>

    <div class="grid grid-cols-1 gap-4 md:grid-cols-2">
      <div class="flex flex-col gap-3 rounded-md border border-default bg-muted/30 p-4">
        <p class="text-xs font-semibold text-highlighted">
          {{ t('designSystem.matrix.themeControls') }}
        </p>
        <div class="flex flex-wrap gap-2">
          <UButton
            size="sm"
            :color="colorMode.value === 'light' ? 'primary' : 'neutral'"
            :variant="colorMode.value === 'light' ? 'solid' : 'outline'"
            icon="i-lucide-sun"
            :label="t('settings.appearance.light')"
            @click="setTheme('light')"
          />
          <UButton
            size="sm"
            :color="colorMode.value === 'dark' ? 'primary' : 'neutral'"
            :variant="colorMode.value === 'dark' ? 'solid' : 'outline'"
            icon="i-lucide-moon"
            :label="t('settings.appearance.dark')"
            @click="setTheme('dark')"
          />
        </div>
      </div>

      <div class="flex flex-col gap-3 rounded-md border border-default bg-muted/30 p-4">
        <p class="text-xs font-semibold text-highlighted">
          {{ t('designSystem.matrix.directionControls') }}
        </p>
        <div class="flex flex-wrap gap-2">
          <UButton
            size="sm"
            :color="locale === 'en' ? 'primary' : 'neutral'"
            :variant="locale === 'en' ? 'solid' : 'outline'"
            icon="i-lucide-align-left"
            :label="t('designSystem.matrix.englishLtr')"
            @click="changeLocale('en')"
          />
          <UButton
            size="sm"
            :color="locale === 'fa' ? 'primary' : 'neutral'"
            :variant="locale === 'fa' ? 'solid' : 'outline'"
            icon="i-lucide-align-right"
            :label="t('designSystem.matrix.persianRtl')"
            @click="changeLocale('fa')"
          />
        </div>
      </div>
    </div>
  </UCard>
</template>
