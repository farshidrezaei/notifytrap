<script setup lang="ts">
const { t } = useI18n()
const { locale } = useLocaleController()
const { health, isLoading, isInitialLoading: isHealthInitialLoading, refreshHealth } = useApplicationHealth()
</script>

<template>
  <footer
    class="flex h-8 shrink-0 select-none items-center justify-between gap-4 border-t border-default/60 bg-muted px-3 text-[11px] text-muted"
  >
    <!-- Start: Persistence & Refresh -->
    <div class="flex min-w-0 items-center gap-3">
      <div v-if="isHealthInitialLoading" class="flex items-center gap-1.5">
        <USkeleton class="size-1.5 rounded-full" />
        <USkeleton class="h-3 w-28 rounded" />
      </div>
      <div v-else class="flex items-center gap-1.5">
        <span
          class="size-1.5 shrink-0 rounded-full"
          :class="health?.isDatabaseConnected ? 'bg-success' : 'bg-neutral-400'"
        />
        <span class="truncate">
          {{
            health?.isDatabaseConnected
              ? t('application.status.databaseConnected')
              : t('application.status.databaseDisconnected')
          }}
        </span>
        <span v-if="health" class="font-mono text-dimmed" dir="ltr">
          ({{ t('overview.cards.migrationVersion', { version: health.schemaMigrationVersion }) }})
        </span>
      </div>

      <UButton
        size="xs"
        color="neutral"
        variant="ghost"
        icon="i-lucide-refresh-cw"
        :loading="isLoading"
        :aria-label="t('common.refresh')"
        @click="refreshHealth"
      />
    </div>

    <!-- End: Capture Server Endpoints & Locale/Version -->
    <div class="flex shrink-0 items-center gap-3">
      <div class="hidden items-center gap-1.5 sm:flex">
        <span
          class="size-1.5 rounded-full"
          :class="health?.isSmtpServerRunning ? 'bg-success' : 'bg-neutral-400'"
        />
        <span>{{ t('footer.smtpLabel') }}</span>
        <USkeleton v-if="isHealthInitialLoading" class="h-3 w-20 rounded" />
        <span v-else class="font-mono text-toned" dir="ltr">
          {{ health?.smtpEndpoint ?? '127.0.0.1:1025' }}
        </span>
      </div>

      <div class="hidden items-center gap-1.5 sm:flex">
        <span
          class="size-1.5 rounded-full"
          :class="health?.isHttpServerRunning ? 'bg-success' : 'bg-neutral-400'"
        />
        <span>{{ t('footer.httpLabel') }}</span>
        <USkeleton v-if="isHealthInitialLoading" class="h-3 w-20 rounded" />
        <span v-else class="font-mono text-toned" dir="ltr">
          {{ health?.httpEndpoint ?? '127.0.0.1:4080' }}
        </span>
      </div>

      <span class="hidden h-3 w-px bg-default sm:inline-block" aria-hidden="true" />

      <span class="font-mono uppercase text-toned" dir="ltr">
        {{ locale === 'fa' ? 'FA · RTL' : 'EN · LTR' }}
      </span>

      <USkeleton v-if="isHealthInitialLoading" class="h-3 w-10 rounded" />
      <span v-else class="font-mono text-dimmed" dir="ltr">
        v{{ health?.applicationVersion ?? '1.0.0' }}
      </span>
    </div>
  </footer>
</template>
