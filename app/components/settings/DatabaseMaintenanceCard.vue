<script setup lang="ts">
import type { DatabaseStorageStatistics } from '~/types/application'
import { copyTextToClipboard } from '~/utils/clipboard'
import { formatByteSize } from '~/utils/formatters'
import {
  extractCommandErrorMessage,
  invokeTauriCommand,
  isTauriRuntime,
} from '~/utils/tauriClient'

const props = defineProps<{
  retentionDays?: number
}>()

const { t } = useI18n()
const toast = useToast()

const storageStatistics = ref<DatabaseStorageStatistics | null>(null)
const isLoadingStatistics = ref(true)
const isVacuuming = ref(false)
const isPruning = ref(false)

async function loadStorageStatistics(): Promise<void> {
  if (!isTauriRuntime()) {
    storageStatistics.value = {
      databasePath: '/home/user/.local/share/notifytrap/notifytrap.sqlite',
      fileSizeBytes: 2_457_600,
      walSizeBytes: 131_072,
      totalNotificationsCount: 142,
      totalAttachmentsCount: 8,
      totalAttachmentsSizeBytes: 524_288,
    }
    isLoadingStatistics.value = false
    return
  }

  isLoadingStatistics.value = true
  try {
    storageStatistics.value = await invokeTauriCommand<DatabaseStorageStatistics>(
      'get_database_storage_statistics',
    )
  } catch (error) {
    toast.add({
      title: t('common.errorTitle'),
      description: extractCommandErrorMessage(error, t('common.unexpectedError')),
      color: 'error',
      icon: 'i-lucide-alert-circle',
    })
  } finally {
    isLoadingStatistics.value = false
  }
}

async function handleVacuumDatabase(): Promise<void> {
  if (!isTauriRuntime()) {
    toast.add({
      title: t('settings.database.vacuumSuccessTitle'),
      description: t('settings.database.vacuumSuccessDescription'),
      color: 'success',
      icon: 'i-lucide-check-circle-2',
    })
    return
  }

  isVacuuming.value = true
  try {
    const updated = await invokeTauriCommand<DatabaseStorageStatistics>('vacuum_database')
    storageStatistics.value = updated
    toast.add({
      title: t('settings.database.vacuumSuccessTitle'),
      description: t('settings.database.vacuumSuccessDescription'),
      color: 'success',
      icon: 'i-lucide-check-circle-2',
    })
  } catch (error) {
    toast.add({
      title: t('common.errorTitle'),
      description: extractCommandErrorMessage(error, t('common.unexpectedError')),
      color: 'error',
      icon: 'i-lucide-alert-circle',
    })
  } finally {
    isVacuuming.value = false
  }
}

async function handlePruneExpiredNow(): Promise<void> {
  const days = props.retentionDays ?? 30
  if (!isTauriRuntime()) {
    toast.add({
      title: t('settings.database.pruneSuccessTitle'),
      description: t('settings.database.pruneSuccessDescription', { count: 12 }),
      color: 'success',
      icon: 'i-lucide-check-circle-2',
    })
    return
  }

  isPruning.value = true
  try {
    const prunedCount = await invokeTauriCommand<number>(
      'prune_expired_notifications_now',
      { retentionDays: days },
    )
    await loadStorageStatistics()
    toast.add({
      title: t('settings.database.pruneSuccessTitle'),
      description: t('settings.database.pruneSuccessDescription', { count: prunedCount }),
      color: 'success',
      icon: 'i-lucide-check-circle-2',
    })
  } catch (error) {
    toast.add({
      title: t('common.errorTitle'),
      description: extractCommandErrorMessage(error, t('common.unexpectedError')),
      color: 'error',
      icon: 'i-lucide-alert-circle',
    })
  } finally {
    isPruning.value = false
  }
}

async function copyPath(path: string): Promise<void> {
  const isCopied = await copyTextToClipboard(path)
  toast.add({
    title: isCopied ? t('common.copiedTitle') : t('common.errorTitle'),
    description: isCopied ? path : t('common.clipboardUnavailable'),
    color: isCopied ? 'success' : 'error',
    icon: isCopied ? 'i-lucide-check-circle-2' : 'i-lucide-alert-circle',
  })
}

onMounted(() => {
  loadStorageStatistics()
})
</script>

<template>
  <UCard>
    <template #header>
      <div class="flex flex-wrap items-center justify-between gap-3">
        <div class="space-y-1">
          <div class="flex items-center gap-2">
            <h2 class="text-sm font-semibold text-highlighted">
              {{ t('settings.database.title') }}
            </h2>
            <UBadge color="primary" variant="subtle" size="xs">
              SQLite WAL
            </UBadge>
          </div>
          <p class="text-xs text-muted">
            {{ t('settings.database.description') }}
          </p>
        </div>

        <div class="flex flex-wrap items-center gap-2">
          <UButton
            size="xs"
            color="neutral"
            variant="outline"
            icon="i-lucide-archive"
            :loading="isPruning"
            :label="t('settings.database.pruneNowButton')"
            @click="handlePruneExpiredNow"
          />
          <UButton
            size="xs"
            color="primary"
            variant="soft"
            icon="i-lucide-sparkles"
            :loading="isVacuuming"
            :label="t('settings.database.vacuumButton')"
            @click="handleVacuumDatabase"
          />
        </div>
      </div>
    </template>

    <div class="space-y-4">
      <!-- Storage Metrics 4-col grid -->
      <div class="grid grid-cols-2 gap-3 sm:grid-cols-4">
        <div class="rounded-lg border border-default bg-muted/30 p-3">
          <p class="text-[11px] font-medium text-muted">
            {{ t('settings.database.metrics.databaseSize') }}
          </p>
          <p v-if="!isLoadingStatistics" dir="ltr" class="mt-1 font-mono text-sm font-semibold text-highlighted">
            {{ formatByteSize(storageStatistics?.fileSizeBytes ?? 0) }}
          </p>
          <USkeleton v-else class="mt-1 h-5 w-20 rounded" />
        </div>

        <div class="rounded-lg border border-default bg-muted/30 p-3">
          <p class="text-[11px] font-medium text-muted">
            {{ t('settings.database.metrics.walSize') }}
          </p>
          <p v-if="!isLoadingStatistics" dir="ltr" class="mt-1 font-mono text-sm font-semibold text-highlighted">
            {{ formatByteSize(storageStatistics?.walSizeBytes ?? 0) }}
          </p>
          <USkeleton v-else class="mt-1 h-5 w-20 rounded" />
        </div>

        <div class="rounded-lg border border-default bg-muted/30 p-3">
          <p class="text-[11px] font-medium text-muted">
            {{ t('settings.database.metrics.totalNotifications') }}
          </p>
          <p v-if="!isLoadingStatistics" class="mt-1 font-mono text-sm font-semibold text-highlighted">
            {{ storageStatistics?.totalNotificationsCount ?? 0 }}
          </p>
          <USkeleton v-else class="mt-1 h-5 w-16 rounded" />
        </div>

        <div class="rounded-lg border border-default bg-muted/30 p-3">
          <p class="text-[11px] font-medium text-muted">
            {{ t('settings.database.metrics.attachmentsSize') }}
          </p>
          <p v-if="!isLoadingStatistics" dir="ltr" class="mt-1 font-mono text-sm font-semibold text-highlighted">
            {{ formatByteSize(storageStatistics?.totalAttachmentsSizeBytes ?? 0) }}
          </p>
          <USkeleton v-else class="mt-1 h-5 w-20 rounded" />
        </div>
      </div>

      <!-- File Path Bar -->
      <div v-if="storageStatistics?.databasePath" class="space-y-1">
        <span class="text-xs font-medium text-toned">
          {{ t('settings.database.filePathLabel') }}
        </span>
        <div class="flex items-center gap-2 rounded-md border border-default bg-muted/40 px-3 py-2">
          <code dir="ltr" class="min-w-0 flex-1 truncate font-mono text-xs text-default">
            {{ storageStatistics.databasePath }}
          </code>
          <UButton
            size="xs"
            color="neutral"
            variant="ghost"
            square
            icon="i-lucide-copy"
            :aria-label="t('common.copy')"
            @click="copyPath(storageStatistics.databasePath)"
          />
        </div>
      </div>
    </div>
  </UCard>
</template>
