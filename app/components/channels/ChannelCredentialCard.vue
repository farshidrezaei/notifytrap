<script setup lang="ts">
import type { ChannelSummary } from '~/types/application'
import { resolveChannelCredentialDisplay } from '~/utils/channelCredentialDisplay'
import { copyTextToClipboard } from '~/utils/clipboard'
import { formatShortTimestamp } from '~/utils/formatters'
import {
  buildChannelIntegrationSnippet,
  resolveChannelIntegrationSnippetLanguage,
} from '~/utils/integrationSnippets'
import { extractCommandErrorMessage } from '~/utils/tauriClient'

const props = defineProps<{
  channel: ChannelSummary
}>()

const { t, locale } = useI18n()
const toast = useToast()
const { health } = useApplicationHealth()
const {
  revealedCredentials,
  toggleChannelEnabled,
  revealCredential,
  hideCredential,
  rotateCredential,
  deleteChannel,
} = useChannels()

const isTogglingEnabled = ref(false)
const isLoadingSecret = ref(false)
const isRotatingSecret = ref(false)
const isDeletingChannel = ref(false)
const isSnippetVisible = ref(false)
const isRulesModalOpen = ref(false)
const isRotateConfirmOpen = ref(false)
const isDeleteConfirmOpen = ref(false)

const revealedCredential = computed(() => revealedCredentials.value[props.channel.id] ?? null)

const httpEndpoint = computed(() => health.value?.httpEndpoint ?? '127.0.0.1:4080')
const smtpEndpoint = computed(() => health.value?.smtpEndpoint ?? '127.0.0.1:1025')

const webhookUrl = computed(
  () => `http://${httpEndpoint.value}/api/v1/webhooks/${props.channel.credentialIdentifier}/events`,
)

const credentialDisplay = computed(() =>
  resolveChannelCredentialDisplay(props.channel, revealedCredential.value, webhookUrl.value),
)

const integrationSnippetLanguage = computed(() =>
  resolveChannelIntegrationSnippetLanguage(props.channel.channelType),
)

const integrationSnippet = computed(() =>
  buildChannelIntegrationSnippet({
    channel: props.channel,
    credential: revealedCredential.value,
    httpEndpoint: httpEndpoint.value,
    smtpEndpoint: smtpEndpoint.value,
  }),
)

async function handleToggleEnabled(nextValue: boolean): Promise<void> {
  isTogglingEnabled.value = true
  try {
    await toggleChannelEnabled(props.channel.id, nextValue)
    toast.add({
      title: nextValue ? t('channels.toast.enabledTitle') : t('channels.toast.disabledTitle'),
      description: props.channel.name,
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
    isTogglingEnabled.value = false
  }
}

async function handleToggleReveal(): Promise<void> {
  if (revealedCredential.value) {
    hideCredential(props.channel.id)
    return
  }
  isLoadingSecret.value = true
  try {
    await revealCredential(props.channel.id)
  } catch (error) {
    toast.add({
      title: t('common.errorTitle'),
      description: extractCommandErrorMessage(error, t('common.unexpectedError')),
      color: 'error',
      icon: 'i-lucide-alert-circle',
    })
  } finally {
    isLoadingSecret.value = false
  }
}

async function handleConfirmRotate(): Promise<void> {
  isRotatingSecret.value = true
  try {
    await rotateCredential(props.channel.id)
    isRotateConfirmOpen.value = false
    toast.add({
      title: t('channels.toast.rotatedTitle'),
      description: props.channel.name,
      color: 'success',
      icon: 'i-lucide-refresh-cw',
    })
  } catch (error) {
    toast.add({
      title: t('common.errorTitle'),
      description: extractCommandErrorMessage(error, t('common.unexpectedError')),
      color: 'error',
      icon: 'i-lucide-alert-circle',
    })
  } finally {
    isRotatingSecret.value = false
  }
}

async function handleConfirmDelete(): Promise<void> {
  isDeletingChannel.value = true
  try {
    await deleteChannel(props.channel.id)
    isDeleteConfirmOpen.value = false
    toast.add({
      title: t('channels.toast.deletedTitle'),
      description: props.channel.name,
      color: 'success',
      icon: 'i-lucide-trash-2',
    })
  } catch (error) {
    toast.add({
      title: t('common.errorTitle'),
      description: extractCommandErrorMessage(error, t('common.unexpectedError')),
      color: 'error',
      icon: 'i-lucide-alert-circle',
    })
  } finally {
    isDeletingChannel.value = false
  }
}

async function handleCopyValue(value: string, label: string): Promise<void> {
  const isCopied = await copyTextToClipboard(value)
  toast.add({
    title: isCopied ? t('common.copiedTitle') : t('common.copyFailedTitle'),
    description: label,
    color: isCopied ? 'success' : 'warning',
    icon: isCopied ? 'i-lucide-clipboard-check' : 'i-lucide-alert-triangle',
  })
}
</script>

<template>
  <UCard>
    <div class="flex flex-col gap-4">
      <div class="flex flex-wrap items-start justify-between gap-3">
        <div class="min-w-0 space-y-1">
          <div class="flex flex-wrap items-center gap-2">
            <SharedChannelBadge :channel-type="props.channel.channelType" />
            <h3 class="truncate text-sm font-semibold text-highlighted">
              {{ props.channel.name }}
            </h3>
            <UBadge
              :color="props.channel.isEnabled ? 'success' : 'neutral'"
              variant="subtle"
              size="xs"
            >
              {{ props.channel.isEnabled ? t('channels.status.enabled') : t('channels.status.disabled') }}
            </UBadge>
            <UBadge
              v-if="props.channel.webhookMock?.isEnabled"
              color="info"
              variant="subtle"
              size="xs"
            >
              {{ t('channels.rulesModal.mockBadge', { code: props.channel.webhookMock.statusCode }) }}
            </UBadge>
            <UBadge
              v-if="props.channel.chaosRule?.isEnabled"
              color="warning"
              variant="subtle"
              size="xs"
            >
              {{ t('channels.rulesModal.chaosBadge', { rate: props.channel.chaosRule.failureRatePercent }) }}
            </UBadge>
          </div>
          <p class="text-xs text-muted">
            {{ t('channels.card.updatedAt', { date: formatShortTimestamp(props.channel.updatedAt, locale) }) }}
          </p>
        </div>

        <div class="flex items-center gap-2">
          <USwitch
            :model-value="props.channel.isEnabled"
            :disabled="isTogglingEnabled"
            :aria-label="t('channels.actions.toggleEnabled')"
            @update:model-value="handleToggleEnabled"
          />

          <UButton
            size="xs"
            color="neutral"
            variant="outline"
            icon="i-lucide-sliders-horizontal"
            :label="t('channels.actions.rules')"
            @click="isRulesModalOpen = true"
          />

          <UTooltip :text="t('channels.actions.rotateCredential')">
            <UButton
              size="xs"
              color="neutral"
              variant="outline"
              square
              icon="i-lucide-refresh-cw"
              :aria-label="t('channels.actions.rotateCredential')"
              @click="isRotateConfirmOpen = true"
            />
          </UTooltip>

          <UTooltip :text="t('channels.actions.deleteChannel')">
            <UButton
              size="xs"
              color="error"
              variant="ghost"
              square
              icon="i-lucide-trash-2"
              :aria-label="t('channels.actions.deleteChannel')"
              @click="isDeleteConfirmOpen = true"
            />
          </UTooltip>
        </div>
      </div>

      <div class="grid gap-3 md:grid-cols-2">
        <div class="rounded-md border border-default bg-muted/40 p-2.5">
          <div class="mb-1 flex items-center justify-between gap-2">
            <span class="text-[11px] font-medium text-muted">
              {{ t(credentialDisplay.primaryLabelKey) }}
            </span>
            <UButton
              size="xs"
              color="neutral"
              variant="ghost"
              icon="i-lucide-copy"
              @click="handleCopyValue(props.channel.credentialIdentifier, t(credentialDisplay.primaryLabelKey))"
            />
          </div>
          <code dir="ltr" class="block truncate font-mono text-xs text-highlighted">
            {{ props.channel.credentialIdentifier }}
          </code>
        </div>

        <div class="rounded-md border border-default bg-muted/40 p-2.5">
          <div class="mb-1 flex items-center justify-between gap-2">
            <span class="text-[11px] font-medium text-muted">
              {{ t(credentialDisplay.secondaryLabelKey) }}
            </span>
            <div class="flex items-center gap-1">
              <UButton
                size="xs"
                color="neutral"
                variant="ghost"
                :loading="isLoadingSecret"
                :icon="revealedCredential ? 'i-lucide-eye-off' : 'i-lucide-eye'"
                :label="revealedCredential ? t('channels.actions.hideSecret') : t('channels.actions.revealSecret')"
                @click="handleToggleReveal"
              />
              <UButton
                v-if="credentialDisplay.revealedSecondaryValue"
                size="xs"
                color="neutral"
                variant="ghost"
                icon="i-lucide-copy"
                @click="
                  handleCopyValue(
                    credentialDisplay.revealedSecondaryValue,
                    t(credentialDisplay.secondaryLabelKey),
                  )
                "
              />
            </div>
          </div>

          <USkeleton v-if="isLoadingSecret" class="h-4 w-48 rounded" />
          <code
            v-else-if="credentialDisplay.revealedSecondaryValue"
            dir="ltr"
            class="block truncate font-mono text-xs text-highlighted"
          >
            {{ credentialDisplay.revealedSecondaryValue }}
          </code>
          <code v-else dir="ltr" class="block truncate font-mono text-xs text-muted">
            {{ credentialDisplay.maskedSecondaryValue }}
          </code>

          <div
            v-if="credentialDisplay.isWebhookEndpoint && revealedCredential"
            class="mt-2 flex items-center justify-between gap-2 border-t border-default/60 pt-2"
          >
            <code dir="ltr" class="truncate font-mono text-[11px] text-toned">
              X-Channel-Key: {{ revealedCredential.secretToken }}
            </code>
            <UButton
              size="xs"
              color="neutral"
              variant="ghost"
              icon="i-lucide-copy"
              @click="handleCopyValue(revealedCredential.secretToken, t('channels.card.secretTokenLabel'))"
            />
          </div>
        </div>
      </div>

      <div class="flex flex-wrap items-center justify-between gap-2 border-t border-default/70 pt-3">
        <UButton
          size="xs"
          color="neutral"
          variant="ghost"
          :icon="isSnippetVisible ? 'i-lucide-chevron-up' : 'i-lucide-terminal'"
          :label="isSnippetVisible ? t('channels.actions.hideSnippet') : t('channels.actions.showSnippet')"
          @click="isSnippetVisible = !isSnippetVisible"
        />

        <UButton
          v-if="isSnippetVisible"
          size="xs"
          color="primary"
          variant="soft"
          icon="i-lucide-copy"
          :label="t('channels.actions.copySnippet')"
          @click="handleCopyValue(integrationSnippet, t('channels.card.snippetLabel'))"
        />
      </div>

      <SharedCodeBlock
        v-if="isSnippetVisible"
        :code="integrationSnippet"
        :language="integrationSnippetLanguage"
        :show-copy-button="false"
      />
    </div>

    <UModal
      v-model:open="isRotateConfirmOpen"
      :title="t('channels.rotateModal.title')"
      :description="t('channels.rotateModal.description', { name: props.channel.name })"
    >
      <template #body>
        <div class="flex justify-end gap-2">
          <UButton
            color="neutral"
            variant="ghost"
            :label="t('common.cancel')"
            @click="isRotateConfirmOpen = false"
          />
          <UButton
            color="warning"
            :loading="isRotatingSecret"
            :label="t('channels.actions.rotateCredential')"
            @click="handleConfirmRotate"
          />
        </div>
      </template>
    </UModal>

    <UModal
      v-model:open="isDeleteConfirmOpen"
      :title="t('channels.deleteModal.title')"
      :description="t('channels.deleteModal.description', { name: props.channel.name })"
    >
      <template #body>
        <div class="flex justify-end gap-2">
          <UButton
            color="neutral"
            variant="ghost"
            :label="t('common.cancel')"
            @click="isDeleteConfirmOpen = false"
          />
          <UButton
            color="error"
            :loading="isDeletingChannel"
            :label="t('channels.actions.deleteChannel')"
            @click="handleConfirmDelete"
          />
        </div>
      </template>
    </UModal>

    <ChannelsChannelRulesModal
      v-model:open="isRulesModalOpen"
      :channel="props.channel"
    />
  </UCard>
</template>
