<script setup lang="ts">
const { t } = useI18n()
const { locale, localeOptions, changeLocale } = useLocaleController()
const {
  isMinimizeToTrayEnabled,
  isNativeNotificationsEnabled,
  setMinimizeToTray,
  setNativeNotifications,
} = useDesktopPreferences()
</script>

<template>
  <div class="space-y-6">
    <UCard>
      <div class="flex flex-wrap items-center justify-between gap-4">
        <div>
          <h2 class="text-base font-medium text-highlighted">
            {{ t('settings.appearance.title') }}
          </h2>
          <p class="mt-0.5 text-sm text-muted">
            {{ t('settings.appearance.description') }}
          </p>
        </div>
        <UColorModeSelect class="w-44" />
      </div>
    </UCard>

    <UCard>
      <div class="flex flex-wrap items-center justify-between gap-4">
        <div>
          <h2 class="text-base font-medium text-highlighted">
            {{ t('settings.language.title') }}
          </h2>
          <p class="mt-0.5 text-sm text-muted">
            {{ t('settings.language.description') }}
          </p>
        </div>
        <USelect
          :model-value="locale"
          :items="localeOptions"
          value-key="code"
          label-key="label"
          class="w-48"
          @update:model-value="(value) => changeLocale(value as 'en' | 'fa')"
        />
      </div>
    </UCard>

    <UCard>
      <template #header>
        <div class="space-y-1">
          <h2 class="text-sm font-semibold text-highlighted">
            {{ t('settings.desktop.title') }}
          </h2>
          <p class="text-xs text-muted">
            {{ t('settings.desktop.description') }}
          </p>
        </div>
      </template>

      <div class="space-y-3">
        <div class="flex items-center justify-between gap-4 rounded-lg border border-default px-3.5 py-2.5">
          <div class="space-y-0.5">
            <p class="text-xs font-medium text-highlighted">
              {{ t('settings.desktop.minimizeToTrayLabel') }}
            </p>
            <p class="text-xs text-muted">
              {{ t('settings.desktop.minimizeToTrayHint') }}
            </p>
          </div>
          <USwitch
            :model-value="isMinimizeToTrayEnabled"
            @update:model-value="setMinimizeToTray"
          />
        </div>

        <div class="flex items-center justify-between gap-4 rounded-lg border border-default px-3.5 py-2.5">
          <div class="space-y-0.5">
            <p class="text-xs font-medium text-highlighted">
              {{ t('settings.desktop.nativeNotificationsLabel') }}
            </p>
            <p class="text-xs text-muted">
              {{ t('settings.desktop.nativeNotificationsHint') }}
            </p>
          </div>
          <USwitch
            :model-value="isNativeNotificationsEnabled"
            @update:model-value="setNativeNotifications"
          />
        </div>
      </div>
    </UCard>
  </div>
</template>
