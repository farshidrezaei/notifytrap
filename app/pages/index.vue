<script setup lang="ts">
import type { ChannelType } from '~/types/application'
import { NOTIFICATION_CHANNEL_KINDS } from '~/schemas/workspaceForms'
import { copyTextToClipboard } from '~/utils/clipboard'
import { formatShortTimestamp } from '~/utils/formatters'
import {
  buildChannelIntegrationSnippet,
  resolveChannelIntegrationSnippetLanguage,
} from '~/utils/integrationSnippets'

const { t, locale } = useI18n()
const toast = useToast()
const { health, isInitialLoading: isHealthInitialLoading, isDesktopRuntime } = useApplicationHealth()
const {
  activeWorkspace,
  activeProject,
  activeEnvironment,
  hasActiveScope,
  hasInitialized,
  isInitializing: isWorkspaceInitializing,
} = useWorkspaceContext()
const { channels, revealedCredentials, isLoading: isChannelsLoading } = useChannels()
const { notificationPage, unreadCount, isLoadingList: isInboxLoading, selectNotification } = useNotifications()

const isCreateChannelModalOpen = ref(false)

const CHANNEL_ICON_MAP: Record<ChannelType, string> = {
  email: 'i-lucide-mail',
  sms: 'i-lucide-message-square',
  push: 'i-lucide-bell',
  webhook: 'i-lucide-webhook',
}

const channelCountByType = computed<Record<ChannelType, number>>(() => {
  const counts: Record<ChannelType, number> = {
    email: 0,
    sms: 0,
    push: 0,
    webhook: 0,
  }
  for (const channel of channels.value) {
    counts[channel.channelType] += 1
  }
  return counts
})

const recentNotifications = computed(() => (notificationPage.value?.items ?? []).slice(0, 5))
const primaryChannel = computed(() => channels.value[0] ?? null)

const quickSnippetLanguage = computed(() =>
  primaryChannel.value
    ? resolveChannelIntegrationSnippetLanguage(primaryChannel.value.channelType)
    : 'bash',
)

const quickSnippet = computed(() => {
  if (!primaryChannel.value) {
    return ''
  }
  return buildChannelIntegrationSnippet({
    channel: primaryChannel.value,
    credential: revealedCredentials.value[primaryChannel.value.id] ?? null,
    httpEndpoint: health.value?.httpEndpoint ?? '127.0.0.1:4080',
    smtpEndpoint: health.value?.smtpEndpoint ?? '127.0.0.1:1025',
  })
})

async function handleCopyEndpoint(endpoint: string, label: string): Promise<void> {
  const isCopied = await copyTextToClipboard(endpoint)
  toast.add({
    title: isCopied ? t('common.copiedTitle') : t('common.copyFailedTitle'),
    description: `${label}: ${endpoint}`,
    color: isCopied ? 'success' : 'warning',
    icon: isCopied ? 'i-lucide-clipboard-check' : 'i-lucide-alert-triangle',
  })
}

async function handleInspectRecent(notificationId: string): Promise<void> {
  await selectNotification(notificationId)
  await navigateTo('/inbox')
}
</script>

<template>
  <div class="mx-auto flex max-w-6xl flex-col gap-6">
    <SkeletonDashboardView v-if="isWorkspaceInitializing" />

    <WorkspaceOnboardingCard
      v-else-if="isDesktopRuntime && hasInitialized && !hasActiveScope"
    />

    <template v-else>
      <SharedSectionHeader
        :title="
          hasActiveScope && activeProject
            ? activeProject.name
            : t('overview.title')
        "
        :description="
          hasActiveScope && activeWorkspace && activeEnvironment
            ? t('dashboard.scopedSubtitle', {
                workspace: activeWorkspace.name,
                environment: t(`workspace.environments.${activeEnvironment.kind}`),
              })
            : t('overview.subtitle')
        "
      >
        <template #actions>
          <div class="flex flex-wrap items-center gap-2">
            <UButton
              v-if="hasActiveScope"
              size="sm"
              color="neutral"
              variant="outline"
              icon="i-lucide-inbox"
              :label="t('dashboard.actions.openInbox', { count: unreadCount })"
              to="/inbox"
            />
            <UButton
              v-if="hasActiveScope"
              size="sm"
              color="primary"
              icon="i-lucide-plus"
              :label="t('channels.actions.addChannel')"
              @click="isCreateChannelModalOpen = true"
            />
          </div>
        </template>
      </SharedSectionHeader>

      <!-- Capture Servers & Database Status -->
      <div class="grid gap-4 md:grid-cols-3">
        <UCard>
          <div class="flex items-start justify-between gap-2">
            <div>
              <p class="text-xs font-medium text-muted">
                {{ t('overview.cards.httpEndpoint') }}
              </p>
              <USkeleton v-if="isHealthInitialLoading" class="mt-2 h-5 w-32 rounded" />
              <code v-else dir="ltr" class="mt-2 block font-mono text-sm font-semibold text-highlighted">
                {{ health?.httpEndpoint ?? '127.0.0.1:4080' }}
              </code>
            </div>
            <div class="flex items-center gap-1">
              <USkeleton v-if="isHealthInitialLoading" class="h-4 w-14 rounded-full" />
              <UBadge
                v-else
                :color="health?.isHttpServerRunning ? 'success' : 'neutral'"
                variant="subtle"
                size="xs"
              >
                {{ health?.isHttpServerRunning ? t('dashboard.serverLive') : t('dashboard.serverStandby') }}
              </UBadge>
              <UButton
                size="xs"
                color="neutral"
                variant="ghost"
                square
                icon="i-lucide-copy"
                :disabled="isHealthInitialLoading"
                :aria-label="t('common.copy')"
                @click="
                  handleCopyEndpoint(
                    `http://${health?.httpEndpoint ?? '127.0.0.1:4080'}`,
                    t('overview.cards.httpEndpoint'),
                  )
                "
              />
            </div>
          </div>
        </UCard>

        <UCard>
          <div class="flex items-start justify-between gap-2">
            <div>
              <p class="text-xs font-medium text-muted">
                {{ t('overview.cards.smtpEndpoint') }}
              </p>
              <USkeleton v-if="isHealthInitialLoading" class="mt-2 h-5 w-32 rounded" />
              <code v-else dir="ltr" class="mt-2 block font-mono text-sm font-semibold text-highlighted">
                {{ health?.smtpEndpoint ?? '127.0.0.1:1025' }}
              </code>
            </div>
            <div class="flex items-center gap-1">
              <USkeleton v-if="isHealthInitialLoading" class="h-4 w-14 rounded-full" />
              <UBadge
                v-else
                :color="health?.isSmtpServerRunning ? 'success' : 'neutral'"
                variant="subtle"
                size="xs"
              >
                {{ health?.isSmtpServerRunning ? t('dashboard.serverLive') : t('dashboard.serverStandby') }}
              </UBadge>
              <UButton
                size="xs"
                color="neutral"
                variant="ghost"
                square
                icon="i-lucide-copy"
                :disabled="isHealthInitialLoading"
                :aria-label="t('common.copy')"
                @click="
                  handleCopyEndpoint(
                    health?.smtpEndpoint ?? '127.0.0.1:1025',
                    t('overview.cards.smtpEndpoint'),
                  )
                "
              />
            </div>
          </div>
        </UCard>

        <UCard>
          <div class="flex items-start justify-between gap-2">
            <div>
              <p class="text-xs font-medium text-muted">
                {{ t('overview.cards.databaseStatus') }}
              </p>
              <USkeleton v-if="isHealthInitialLoading" class="mt-2 h-5 w-36 rounded" />
              <p v-else class="mt-2 text-sm font-semibold text-highlighted">
                {{
                  health?.isDatabaseConnected
                    ? t('overview.cards.databaseConnected')
                    : isDesktopRuntime
                      ? t('overview.cards.databaseStandby')
                      : t('application.status.browserPreviewMode')
                }}
              </p>
            </div>
            <USkeleton v-if="isHealthInitialLoading" class="h-4 w-10 rounded-full" />
            <UBadge v-else color="info" variant="subtle" size="xs">
              v{{ health?.schemaMigrationVersion ?? 0 }}
            </UBadge>
          </div>
        </UCard>
      </div>

      <!-- Scoped Channel Matrix -->
      <SkeletonDashboardChannelCards v-if="isChannelsLoading" />
      <div v-else class="grid gap-3 sm:grid-cols-2 lg:grid-cols-4">
        <UCard v-for="kind in NOTIFICATION_CHANNEL_KINDS" :key="kind">
          <div class="flex items-center justify-between gap-3">
            <div class="flex items-center gap-2.5">
              <div
                class="flex size-9 items-center justify-center rounded-lg"
                :style="{
                  color: `var(--nt-channel-${kind})`,
                  backgroundColor: `var(--nt-channel-${kind}-bg)`,
                }"
              >
                <UIcon :name="CHANNEL_ICON_MAP[kind]" class="size-4" />
              </div>
              <div>
                <p class="text-xs text-muted">
                  {{ t(`channels.types.${kind}`) }}
                </p>
                <p class="text-lg font-semibold text-highlighted">
                  {{ channelCountByType[kind] }}
                </p>
              </div>
            </div>
            <UButton
              size="xs"
              color="neutral"
              variant="ghost"
              icon="i-lucide-arrow-up-right"
              to="/channels"
              :aria-label="t('navigation.channels')"
            />
          </div>
        </UCard>
      </div>

      <div class="grid gap-6 lg:grid-cols-2">
        <!-- Quick Integration Snippet -->
        <SkeletonDashboardSnippetCard v-if="isChannelsLoading" />
        <UCard v-else>
          <template #header>
            <div class="flex items-center justify-between gap-2">
              <h2 class="text-sm font-semibold text-highlighted">
                {{ t('dashboard.quickIntegration.title') }}
              </h2>
              <UButton
                v-if="primaryChannel"
                size="xs"
                color="primary"
                variant="soft"
                icon="i-lucide-copy"
                :label="t('channels.actions.copySnippet')"
                @click="handleCopyEndpoint(quickSnippet, primaryChannel.name)"
              />
            </div>
          </template>

          <div v-if="primaryChannel" class="space-y-3">
            <div class="flex items-center gap-2">
              <SharedChannelBadge :channel-type="primaryChannel.channelType" />
              <span class="text-xs font-medium text-highlighted">
                {{ primaryChannel.name }}
              </span>
            </div>
            <SharedCodeBlock
              :code="quickSnippet"
              :language="quickSnippetLanguage"
              :show-copy-button="false"
            />
          </div>

          <UEmpty
            v-else
            icon="i-lucide-radio"
            :title="t('dashboard.quickIntegration.emptyTitle')"
            :description="t('dashboard.quickIntegration.emptyDescription')"
          >
            <template #actions>
              <UButton
                size="xs"
                color="primary"
                icon="i-lucide-plus"
                :disabled="!hasActiveScope"
                :label="t('channels.actions.addChannel')"
                @click="isCreateChannelModalOpen = true"
              />
            </template>
          </UEmpty>
        </UCard>

        <!-- Recent Captured Notifications -->
        <SkeletonDashboardRecentCard v-if="isInboxLoading" />
        <UCard v-else>
          <template #header>
            <div class="flex items-center justify-between gap-2">
              <h2 class="text-sm font-semibold text-highlighted">
                {{ t('dashboard.recentActivity.title') }}
              </h2>
              <UButton
                size="xs"
                color="neutral"
                variant="ghost"
                :label="t('dashboard.recentActivity.viewAll')"
                to="/inbox"
              />
            </div>
          </template>

          <div v-if="recentNotifications.length > 0" class="divide-y divide-default">
            <button
              v-for="item in recentNotifications"
              :key="item.id"
              type="button"
              class="flex w-full items-center justify-between gap-3 py-2.5 text-start transition-colors hover:bg-muted/40"
              @click="handleInspectRecent(item.id)"
            >
              <div class="min-w-0 space-y-1">
                <div class="flex items-center gap-2">
                  <SharedChannelBadge :channel-type="item.channelType" />
                  <span class="truncate text-xs font-semibold text-highlighted">
                    {{ item.subject || item.recipient || t('inbox.noSubject') }}
                  </span>
                </div>
                <p class="truncate text-xs text-muted">
                  {{ item.previewText || item.recipient }}
                </p>
              </div>
              <span class="shrink-0 text-[11px] text-muted">
                {{ formatShortTimestamp(item.receivedAt, locale) }}
              </span>
            </button>
          </div>

          <UEmpty
            v-else
            icon="i-lucide-inbox"
            :title="t('dashboard.recentActivity.emptyTitle')"
            :description="t('dashboard.recentActivity.emptyDescription')"
          />
        </UCard>
      </div>
    </template>

    <ChannelsCreateChannelModal v-model:open="isCreateChannelModalOpen" />
  </div>
</template>
