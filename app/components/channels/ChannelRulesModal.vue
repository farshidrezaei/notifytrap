<script setup lang="ts">
import type {
  ChannelChaosConfiguration,
  ChannelSummary,
  WebhookMockConfiguration,
} from '~/types/application'
import {
  DEFAULT_CHANNEL_CHAOS_CONFIGURATION,
  DEFAULT_WEBHOOK_MOCK_CONFIGURATION,
  parseMockHeadersJson,
} from '~/utils/channelRules'
import { extractCommandErrorMessage } from '~/utils/tauriClient'

const props = defineProps<{
  channel: ChannelSummary
}>()

const isOpen = defineModel<boolean>('open', { default: false })

const { t } = useI18n()
const toast = useToast()
const { updateChannelRules } = useChannels()

const isSaving = ref(false)
const mockHeadersText = ref('{}')
const headersError = ref<string | null>(null)

const mockState = reactive<WebhookMockConfiguration>({
  ...DEFAULT_WEBHOOK_MOCK_CONFIGURATION,
})

const chaosState = reactive<ChannelChaosConfiguration>({
  ...DEFAULT_CHANNEL_CHAOS_CONFIGURATION,
})

const isWebhookChannel = computed(() => props.channel.channelType === 'webhook')
const isEmailChannel = computed(() => props.channel.channelType === 'email')

watch(isOpen, (opened) => {
  if (!opened) {
    return
  }
  const mock = props.channel.webhookMock ?? DEFAULT_WEBHOOK_MOCK_CONFIGURATION
  mockState.isEnabled = mock.isEnabled
  mockState.statusCode = mock.statusCode
  mockState.body = mock.body
  mockState.delayMilliseconds = mock.delayMilliseconds
  mockState.headers = { ...(mock.headers ?? {}) }
  mockHeadersText.value = JSON.stringify(mock.headers ?? {}, null, 2)
  headersError.value = null

  const chaos = props.channel.chaosRule ?? DEFAULT_CHANNEL_CHAOS_CONFIGURATION
  chaosState.isEnabled = chaos.isEnabled
  chaosState.latencyMilliseconds = chaos.latencyMilliseconds
  chaosState.failureRatePercent = chaos.failureRatePercent
  chaosState.httpErrorStatusCode = chaos.httpErrorStatusCode
  chaosState.smtpErrorCode = chaos.smtpErrorCode
  chaosState.errorMessage = chaos.errorMessage
})

async function handleSaveRules(): Promise<void> {
  const parsedHeaders = parseMockHeadersJson(mockHeadersText.value)
  if (!parsedHeaders.isValid || !parsedHeaders.headers) {
    headersError.value = t('channels.rulesModal.invalidHeadersJson')
    return
  }
  headersError.value = null
  isSaving.value = true

  try {
    await updateChannelRules({
      channelId: props.channel.id,
      webhookMock: {
        isEnabled: isWebhookChannel.value ? mockState.isEnabled : false,
        statusCode: Number(mockState.statusCode) || 200,
        headers: parsedHeaders.headers,
        body: mockState.body,
        delayMilliseconds: Math.max(0, Number(mockState.delayMilliseconds) || 0),
      },
      chaosRule: {
        isEnabled: chaosState.isEnabled,
        latencyMilliseconds: Math.max(0, Number(chaosState.latencyMilliseconds) || 0),
        failureRatePercent: Math.min(100, Math.max(0, Number(chaosState.failureRatePercent) || 0)),
        httpErrorStatusCode: Number(chaosState.httpErrorStatusCode) || 503,
        smtpErrorCode: Number(chaosState.smtpErrorCode) || 451,
        errorMessage: chaosState.errorMessage.trim() || DEFAULT_CHANNEL_CHAOS_CONFIGURATION.errorMessage,
      },
    })
    isOpen.value = false
    toast.add({
      title: t('channels.rulesModal.savedTitle'),
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
    isSaving.value = false
  }
}
</script>

<template>
  <UModal
    v-model:open="isOpen"
    :title="t('channels.rulesModal.title', { name: channel.name })"
    :description="t('channels.rulesModal.description')"
    :ui="{ content: 'max-w-2xl' }"
  >
    <template #body>
      <div class="space-y-5">
        <div
          v-if="isWebhookChannel"
          class="space-y-3 rounded-lg border border-default bg-muted/20 p-4"
        >
          <div class="flex items-center justify-between gap-3">
            <div>
              <h4 class="text-sm font-semibold text-highlighted">
                {{ t('channels.rulesModal.mockSectionTitle') }}
              </h4>
              <p class="text-xs text-muted">
                {{ t('channels.rulesModal.mockSectionDescription') }}
              </p>
            </div>
            <USwitch
              v-model="mockState.isEnabled"
              :aria-label="t('channels.rulesModal.mockEnabledLabel')"
            />
          </div>

          <div class="grid gap-3 sm:grid-cols-2">
            <UFormField :label="t('channels.rulesModal.mockStatusLabel')">
              <UInput
                v-model.number="mockState.statusCode"
                type="number"
                :min="100"
                :max="599"
                dir="ltr"
                class="w-full font-mono"
              />
            </UFormField>
            <UFormField :label="t('channels.rulesModal.mockDelayLabel')">
              <UInput
                v-model.number="mockState.delayMilliseconds"
                type="number"
                :min="0"
                :max="10000"
                dir="ltr"
                class="w-full font-mono"
              />
            </UFormField>
          </div>

          <UFormField
            :label="t('channels.rulesModal.mockHeadersLabel')"
            :error="headersError ?? undefined"
          >
            <UTextarea
              v-model="mockHeadersText"
              :rows="3"
              dir="ltr"
              class="w-full font-mono text-xs"
            />
          </UFormField>

          <UFormField :label="t('channels.rulesModal.mockBodyLabel')">
            <UTextarea
              v-model="mockState.body"
              :rows="4"
              dir="ltr"
              class="w-full font-mono text-xs"
            />
          </UFormField>
        </div>

        <div class="space-y-3 rounded-lg border border-default bg-muted/20 p-4">
          <div class="flex items-center justify-between gap-3">
            <div>
              <h4 class="text-sm font-semibold text-highlighted">
                {{ t('channels.rulesModal.chaosSectionTitle') }}
              </h4>
              <p class="text-xs text-muted">
                {{ t('channels.rulesModal.chaosSectionDescription') }}
              </p>
            </div>
            <USwitch
              v-model="chaosState.isEnabled"
              :aria-label="t('channels.rulesModal.chaosEnabledLabel')"
            />
          </div>

          <div class="grid gap-3 sm:grid-cols-3">
            <UFormField :label="t('channels.rulesModal.chaosFailureRateLabel')">
              <UInput
                v-model.number="chaosState.failureRatePercent"
                type="number"
                :min="0"
                :max="100"
                dir="ltr"
                class="w-full font-mono"
              />
            </UFormField>
            <UFormField :label="t('channels.rulesModal.chaosLatencyLabel')">
              <UInput
                v-model.number="chaosState.latencyMilliseconds"
                type="number"
                :min="0"
                :max="10000"
                dir="ltr"
                class="w-full font-mono"
              />
            </UFormField>
            <UFormField
              :label="isEmailChannel
                ? t('channels.rulesModal.chaosSmtpCodeLabel')
                : t('channels.rulesModal.chaosHttpCodeLabel')"
            >
              <UInput
                v-if="isEmailChannel"
                v-model.number="chaosState.smtpErrorCode"
                type="number"
                :min="400"
                :max="599"
                dir="ltr"
                class="w-full font-mono"
              />
              <UInput
                v-else
                v-model.number="chaosState.httpErrorStatusCode"
                type="number"
                :min="400"
                :max="599"
                dir="ltr"
                class="w-full font-mono"
              />
            </UFormField>
          </div>

          <UFormField :label="t('channels.rulesModal.chaosMessageLabel')">
            <UInput
              v-model="chaosState.errorMessage"
              :maxlength="240"
              class="w-full"
            />
          </UFormField>
        </div>

        <div class="flex justify-end gap-2">
          <UButton
            color="neutral"
            variant="ghost"
            :label="t('common.cancel')"
            @click="isOpen = false"
          />
          <UButton
            color="primary"
            :loading="isSaving"
            :label="t('common.save')"
            @click="handleSaveRules"
          />
        </div>
      </div>
    </template>
  </UModal>
</template>
