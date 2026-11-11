<script setup lang="ts">
import type { FormSubmitEvent } from '@nuxt/ui'
import {
  createScopedChannelModalSchema,
  NOTIFICATION_CHANNEL_KINDS,
  type ValidatedScopedChannelModalForm,
} from '~/schemas/workspaceForms'
import type { ChannelType } from '~/types/application'
import { INPUT_MASKS } from '~/utils/inputMasks'
import { extractCommandErrorMessage } from '~/utils/tauriClient'

const isModalOpen = defineModel<boolean>('open', { required: true })

const { t } = useI18n()
const toast = useToast()
const { createChannel } = useChannels()

const channelSchema = computed(() => createScopedChannelModalSchema(t))
const isSubmitting = ref(false)

const formState = reactive<{
  name: string
  channelType: ChannelType
  credentialIdentifier: string
  isEnabled: boolean
}>({
  name: '',
  channelType: 'email',
  credentialIdentifier: '',
  isEnabled: true,
})

const channelTypeOptions = computed(() =>
  NOTIFICATION_CHANNEL_KINDS.map((kind) => ({
    label: t(`channels.types.${kind}`),
    value: kind,
  })),
)

watch(isModalOpen, (opened) => {
  if (opened) {
    formState.name = ''
    formState.channelType = 'email'
    formState.credentialIdentifier = ''
    formState.isEnabled = true
  }
})

async function handleSubmitChannel(
  event: FormSubmitEvent<ValidatedScopedChannelModalForm>,
): Promise<void> {
  isSubmitting.value = true
  try {
    const result = await createChannel({
      name: event.data.name,
      channelType: event.data.channelType,
      credentialIdentifier: event.data.credentialIdentifier || undefined,
      isEnabled: event.data.isEnabled,
    })
    isModalOpen.value = false
    toast.add({
      title: t('channels.toast.createdTitle'),
      description: `${result.channel.name} (${result.credential.credentialIdentifier})`,
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
    isSubmitting.value = false
  }
}
</script>

<template>
  <UModal
    v-model:open="isModalOpen"
    :title="t('channels.createModal.title')"
    :description="t('channels.createModal.description')"
  >
    <template #body>
      <UForm
        :schema="channelSchema"
        :state="formState"
        :validate-on="['blur', 'change', 'input']"
        class="space-y-4"
        @submit="handleSubmitChannel"
      >
        <UFormField :label="t('channels.createModal.nameLabel')" name="name" required>
          <UInput
            v-model="formState.name"
            v-maska="INPUT_MASKS.displayName"
            :maxlength="80"
            class="w-full"
            :placeholder="t('channels.createModal.namePlaceholder')"
          />
        </UFormField>

        <UFormField :label="t('channels.createModal.typeLabel')" name="channelType" required>
          <USelect
            v-model="formState.channelType"
            :items="channelTypeOptions"
            value-key="value"
            class="w-full"
          />
        </UFormField>

        <SharedHintedFormField
          :label="t('channels.createModal.identifierLabel')"
          :hint="t('channels.createModal.identifierHint')"
          name="credentialIdentifier"
        >
          <UInput
            v-model="formState.credentialIdentifier"
            v-maska="INPUT_MASKS.singleLineAsciiSafe"
            :maxlength="64"
            dir="ltr"
            class="w-full font-mono"
            :placeholder="t('channels.createModal.identifierPlaceholder')"
          />
        </SharedHintedFormField>

        <UFormField :label="t('channels.createModal.enabledLabel')" name="isEnabled">
          <div class="flex items-center justify-between rounded-md border border-default px-3 py-2">
            <span class="text-xs text-muted">
              {{ t('channels.createModal.enabledHint') }}
            </span>
            <USwitch v-model="formState.isEnabled" />
          </div>
        </UFormField>

        <div class="flex justify-end gap-2 pt-2">
          <UButton
            color="neutral"
            variant="ghost"
            :label="t('common.cancel')"
            @click="isModalOpen = false"
          />
          <UButton
            type="submit"
            color="primary"
            :loading="isSubmitting"
            :label="t('channels.createModal.submit')"
          />
        </div>
      </UForm>
    </template>
  </UModal>
</template>
