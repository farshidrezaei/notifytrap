<script setup lang="ts">
import type { WebhookReplayResult } from '~/types/application'
import {
  buildDefaultLoopbackReplayUrl,
  isValidLoopbackReplayUrl,
} from '~/utils/channelRules'
import { extractCommandErrorMessage } from '~/utils/tauriClient'

const props = defineProps<{
  notificationId: string
  subpath: string
}>()

const isOpen = defineModel<boolean>('open', { default: false })

const { t } = useI18n()
const toast = useToast()
const { replayWebhookNotification } = useNotifications()

const targetUrl = ref('')
const isReplaying = ref(false)
const urlError = ref<string | null>(null)
const replayResult = ref<WebhookReplayResult | null>(null)

watch(isOpen, (opened) => {
  if (!opened) {
    return
  }
  if (!targetUrl.value) {
    targetUrl.value = buildDefaultLoopbackReplayUrl(props.subpath)
  }
  urlError.value = null
  replayResult.value = null
})

async function handleExecuteReplay(): Promise<void> {
  if (!isValidLoopbackReplayUrl(targetUrl.value)) {
    urlError.value = t('inbox.webhook.replayModal.invalidLoopbackUrl')
    return
  }
  urlError.value = null
  isReplaying.value = true

  try {
    replayResult.value = await replayWebhookNotification({
      notificationId: props.notificationId,
      targetUrl: targetUrl.value.trim(),
    })
  } catch (error) {
    toast.add({
      title: t('common.errorTitle'),
      description: extractCommandErrorMessage(error, t('common.unexpectedError')),
      color: 'error',
      icon: 'i-lucide-alert-circle',
    })
  } finally {
    isReplaying.value = false
  }
}
</script>

<template>
  <UModal
    v-model:open="isOpen"
    :title="t('inbox.webhook.replayModal.title')"
    :description="t('inbox.webhook.replayModal.description')"
    :ui="{ content: 'max-w-3xl' }"
  >
    <template #body>
      <div class="space-y-4">
        <UFormField
          :label="t('inbox.webhook.replayModal.targetUrlLabel')"
          :error="urlError ?? undefined"
        >
          <div class="flex flex-wrap items-center gap-2">
            <UInput
              v-model="targetUrl"
              dir="ltr"
              class="min-w-64 flex-1 font-mono text-xs"
              :placeholder="t('inbox.webhook.replayModal.targetUrlPlaceholder')"
            />
            <UButton
              color="primary"
              icon="i-lucide-send"
              :loading="isReplaying"
              :label="t('inbox.webhook.replayModal.sendButton')"
              @click="handleExecuteReplay"
            />
          </div>
        </UFormField>

        <div
          v-if="replayResult"
          class="space-y-3 rounded-lg border border-default bg-muted/20 p-3"
        >
          <div class="flex flex-wrap items-center justify-between gap-2">
            <div class="flex items-center gap-2">
              <UBadge
                :color="replayResult.statusCode < 400 ? 'success' : 'error'"
                variant="subtle"
                size="xs"
                class="font-mono"
              >
                {{ replayResult.method }} {{ replayResult.statusCode }}
              </UBadge>
              <span dir="ltr" class="font-mono text-xs text-muted">
                {{ replayResult.targetUrl }}
              </span>
            </div>
            <span dir="ltr" class="font-mono text-xs text-toned">
              {{ replayResult.durationMilliseconds }} ms
            </span>
          </div>

          <SharedCodeBlock
            v-if="replayResult.responseBody"
            :code="replayResult.responseBody"
            language="auto"
            max-height-class="max-h-64"
          />
          <p v-else class="text-xs text-muted">
            {{ t('inbox.webhook.replayModal.emptyResponseBody') }}
          </p>
        </div>
      </div>
    </template>
  </UModal>
</template>
