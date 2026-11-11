<script setup lang="ts">
const { t } = useI18n()
const { health } = useApplicationHealth()
const {
  isAutoUpdateCheckEnabled,
  isCheckingForUpdates,
  isDownloadingUpdate,
  isRestartPending,
  availableUpdate,
  downloadProgressPercentage,
  updateErrorMessage,
  loadAutoUpdatePreference,
  setAutoUpdateCheckEnabled,
  checkForUpdates,
  downloadAndInstallUpdate,
  restartApplication,
} = useAppUpdater()

const currentVersionLabel = computed(
  () => availableUpdate.value?.currentVersion ?? health.value?.applicationVersion ?? '1.0.0',
)

const statusBadgeColor = computed<'success' | 'primary' | 'warning' | 'neutral'>(() => {
  if (isRestartPending.value) {
    return 'success'
  }
  if (isDownloadingUpdate.value) {
    return 'warning'
  }
  if (availableUpdate.value) {
    return 'primary'
  }
  return 'neutral'
})

const statusBadgeLabel = computed(() => {
  if (isRestartPending.value) {
    return t('settings.updates.statusRestartReady')
  }
  if (isDownloadingUpdate.value) {
    return t('settings.updates.statusDownloading')
  }
  if (availableUpdate.value) {
    return t('settings.updates.statusAvailable', { version: availableUpdate.value.version })
  }
  return t('settings.updates.statusUpToDate')
})

onMounted(() => {
  void loadAutoUpdatePreference()
})
</script>

<template>
  <UCard id="application-updates">
    <div class="flex flex-wrap items-start justify-between gap-4">
      <div class="space-y-1">
        <div class="flex flex-wrap items-center gap-2">
          <h2 class="text-base font-medium text-highlighted">
            {{ t('settings.updates.title') }}
          </h2>
          <UBadge
            :color="statusBadgeColor"
            variant="subtle"
            size="xs"
            :label="statusBadgeLabel"
          />
        </div>
        <p class="text-sm text-muted">
          {{ t('settings.updates.description') }}
        </p>
      </div>

      <div class="flex flex-wrap items-center gap-2">
        <UButton
          v-if="isRestartPending"
          color="success"
          icon="i-lucide-rotate-cw"
          :label="t('settings.updates.actions.restartNow')"
          @click="restartApplication()"
        />
        <UButton
          v-else-if="availableUpdate"
          color="primary"
          icon="i-lucide-download"
          :loading="isDownloadingUpdate"
          :label="t('settings.updates.actions.installNow')"
          @click="downloadAndInstallUpdate()"
        />
        <UButton
          color="neutral"
          variant="outline"
          icon="i-lucide-refresh-cw"
          :loading="isCheckingForUpdates"
          :disabled="isDownloadingUpdate || isRestartPending"
          :label="t('settings.updates.actions.checkNow')"
          @click="checkForUpdates({ isBackground: false })"
        />
      </div>
    </div>

    <div class="mt-4 flex flex-col gap-3 border-t border-default pt-4">
      <div class="flex flex-wrap items-center justify-between gap-3">
        <div>
          <div class="text-sm font-medium text-highlighted">
            {{ t('settings.updates.autoCheckLabel') }}
          </div>
          <p class="text-xs text-muted">
            {{ t('settings.updates.autoCheckHint') }}
          </p>
        </div>

        <USwitch
          :model-value="isAutoUpdateCheckEnabled"
          :aria-label="t('settings.updates.autoCheckLabel')"
          @update:model-value="(value) => setAutoUpdateCheckEnabled(Boolean(value))"
        />
      </div>

      <div class="flex flex-wrap items-center gap-3 text-xs text-muted">
        <span>
          {{ t('settings.updates.currentVersionLabel') }}:
          <code class="ms-1 font-mono font-medium text-highlighted" dir="ltr">
            v{{ currentVersionLabel }}
          </code>
        </span>
        <span v-if="availableUpdate">
          {{ t('settings.updates.latestVersionLabel') }}:
          <code class="ms-1 font-mono font-medium text-primary" dir="ltr">
            v{{ availableUpdate.version }}
          </code>
        </span>
      </div>

      <div v-if="isDownloadingUpdate" class="space-y-1.5">
        <UProgress
          :model-value="downloadProgressPercentage"
          color="primary"
          size="sm"
          status
        />
      </div>

      <UAlert
        v-if="updateErrorMessage"
        color="error"
        variant="subtle"
        icon="i-lucide-alert-circle"
        :description="updateErrorMessage"
      />

      <div
        v-if="availableUpdate?.releaseNotes"
        class="rounded-lg border border-default bg-muted/40 p-3 text-xs"
      >
        <div class="mb-1 font-medium text-highlighted">
          {{ t('settings.updates.releaseNotesLabel') }}
        </div>
        <p class="whitespace-pre-wrap leading-relaxed text-toned">
          {{ availableUpdate.releaseNotes }}
        </p>
      </div>

      <p class="text-xs text-muted">
        {{ t('settings.updates.linuxPackageHint') }}
      </p>
    </div>
  </UCard>
</template>
