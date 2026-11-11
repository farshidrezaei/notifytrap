<script setup lang="ts">
const { t } = useI18n()
const { locale, changeLocale } = useLocaleController()
</script>

<template>
  <div class="mx-auto flex max-w-6xl flex-col gap-6">
    <div class="flex flex-wrap items-start justify-between gap-4">
      <div class="flex items-center gap-3">
        <SharedBrandLogo size="md" is-decorative />
        <div>
          <h1 class="text-xl font-semibold text-highlighted">
            {{ t('designSystem.title') }}
          </h1>
          <p class="mt-1 text-sm text-muted">
            {{ t('designSystem.subtitle') }}
          </p>
        </div>
      </div>

      <div class="flex items-center gap-2">
        <UButton
          color="neutral"
          variant="outline"
          icon="i-lucide-languages"
          :label="t('designSystem.controls.languageToggle')"
          @click="changeLocale(locale === 'en' ? 'fa' : 'en')"
        />
        <UColorModeButton />
      </div>
    </div>

    <DesignSystemThemeDirectionMatrix />
    <DesignSystemTokenColorSection />
    <DesignSystemTokenTypographySection />
    <DesignSystemTokenScaleSection />
    <DesignSystemComponentShowcaseSection />
  </div>
</template>
