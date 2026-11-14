<script setup lang="ts">
import type { ApplicationHealthSnapshot } from '~/types/application'

const props = defineProps<{
  health: ApplicationHealthSnapshot | null
  isHealthInitialLoading: boolean
}>()

const TELEGRAM_CHANNEL_URL = 'https://t.me/notifytrap'
const TELEGRAM_CHANNEL_HANDLE = '@notifytrap'
const GITHUB_REPOSITORY_URL = 'https://github.com/farshidrezaei/notifytrap'

const { t } = useI18n()
</script>

<template>
  <UCard>
    <div class="flex flex-wrap items-center justify-between gap-4">
      <div class="flex items-center gap-3.5">
        <div class="flex size-14 shrink-0 items-center justify-center rounded-xl bg-primary/10 ring-1 ring-primary/20">
          <SharedBrandLogo size="lg" />
        </div>
        <div class="space-y-1">
          <div class="flex flex-wrap items-center gap-2">
            <h2 class="text-sm font-semibold text-highlighted">
              {{ t('application.name') }}
            </h2>
            <UBadge
              color="primary"
              variant="subtle"
              size="xs"
              :label="t('settings.about.architectureBadge')"
            />
          </div>
          <p class="text-xs text-muted">
            {{ t('settings.about.description') }}
          </p>
        </div>
      </div>

      <div class="flex flex-wrap items-center gap-3">
        <div class="rounded-lg border border-default bg-elevated px-3 py-1.5 text-xs">
          <span class="text-muted">{{ t('settings.about.versionLabel') }}:</span>
          <USkeleton v-if="props.isHealthInitialLoading" class="ms-1.5 inline-block h-4 w-10 align-middle rounded" />
          <code v-else class="ms-1.5 font-mono font-medium text-highlighted">v{{ props.health?.applicationVersion ?? '1.0.0' }}</code>
        </div>
        <div class="rounded-lg border border-default bg-elevated px-3 py-1.5 text-xs">
          <span class="text-muted">{{ t('settings.about.schemaLabel') }}:</span>
          <USkeleton v-if="props.isHealthInitialLoading" class="ms-1.5 inline-block h-4 w-8 align-middle rounded" />
          <code v-else class="ms-1.5 font-mono font-medium text-highlighted">v{{ props.health?.schemaMigrationVersion ?? 1 }}</code>
        </div>
      </div>
    </div>

    <div class="mt-4 flex flex-col gap-3 border-t border-default pt-4">
      <!-- Telegram Channel Reference -->
      <div class="flex flex-wrap items-center justify-between gap-3">
        <div class="space-y-0.5">
          <div class="flex items-center gap-2 text-xs font-medium text-highlighted">
            <UIcon name="i-simple-icons-telegram" class="size-4 text-[#24A1DE]" />
            <span>{{ t('settings.about.telegramChannelTitle') }}</span>
          </div>
          <p class="text-xs text-muted">
            {{ t('settings.about.telegramChannelDescription') }}
          </p>
        </div>

        <div class="flex flex-wrap items-center gap-2">
          <span class="rounded-md border border-default bg-muted/40 px-2.5 py-1 font-mono text-xs text-default" dir="ltr">
            {{ TELEGRAM_CHANNEL_HANDLE }}
          </span>

          <UButton
            color="primary"
            variant="subtle"
            size="xs"
            icon="i-simple-icons-telegram"
            :label="t('settings.about.openTelegramAction')"
            :to="TELEGRAM_CHANNEL_URL"
            target="_blank"
            rel="noopener noreferrer"
          />
        </div>
      </div>

      <!-- GitHub Repository -->
      <div class="flex flex-wrap items-center justify-between gap-3 border-t border-default/60 pt-3">
        <div class="space-y-0.5">
          <div class="flex items-center gap-2 text-xs font-medium text-highlighted">
            <UIcon name="i-simple-icons-github" class="size-4 text-highlighted" />
            <span>{{ t('settings.about.githubTitle') }}</span>
          </div>
          <p class="text-xs text-muted">
            {{ t('settings.about.githubDescription') }}
          </p>
        </div>

        <UButton
          color="neutral"
          variant="outline"
          size="xs"
          icon="i-simple-icons-github"
          :label="t('settings.about.openGitHubAction')"
          :to="GITHUB_REPOSITORY_URL"
          target="_blank"
          rel="noopener noreferrer"
        />
      </div>
    </div>
  </UCard>
</template>
