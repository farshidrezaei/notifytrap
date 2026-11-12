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

const CHANNEL_PROTOCOL_ICONS: Record<ChannelType, string> = {
  email: 'i-lucide-mail',
  sms: 'i-lucide-message-square',
  push: 'i-lucide-bell',
  webhook: 'i-lucide-webhook',
}

interface ChannelProtocolCardItem {
  value: ChannelType
  label: string
  description: string
  iconName: string
  transportTag: string
}

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

const channelProtocolItems = computed<ChannelProtocolCardItem[]>(() =>
  NOTIFICATION_CHANNEL_KINDS.map((kind) => ({
    value: kind,
    label: t(`channels.types.${kind}`),
    description: t(`channels.createModal.protocolDescriptions.${kind}`),
    iconName: CHANNEL_PROTOCOL_ICONS[kind],
    transportTag: t(`channels.createModal.protocolTransports.${kind}`),
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

function resolveProtocolCardStyle(channelType: ChannelType, isSelected: boolean): Record<string, string> | undefined {
  if (!isSelected) {
    return undefined
  }
  return {
    borderColor: `var(--nt-channel-${channelType})`,
    backgroundColor: `var(--nt-channel-${channelType}-bg)`,
    boxShadow: `0 0 0 1px var(--nt-channel-${channelType})`,
  }
}

function resolveProtocolIconStyle(channelType: ChannelType): Record<string, string> {
  return {
    color: `var(--nt-channel-${channelType})`,
    backgroundColor: `var(--nt-channel-${channelType}-bg)`,
  }
}

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
    :ui="{ content: 'sm:max-w-xl' }"
  >
    <template #body>
      <UForm
        :schema="channelSchema"
        :state="formState"
        :validate-on="['blur', 'change', 'input']"
        class="space-y-4"
        @submit="handleSubmitChannel"
      >
        <UFormField :label="t('channels.createModal.typeLabel')" name="channelType" required>
          <URadioGroup
            v-model="formState.channelType"
            :items="channelProtocolItems"
            value-key="value"
            variant="list"
            indicator="hidden"
            :ui="{
              fieldset: 'grid grid-cols-1 sm:grid-cols-2 gap-2.5',
              item: 'w-full',
              container: 'sr-only',
              wrapper: 'w-full',
              label: 'block w-full cursor-pointer',
            }"
          >
            <template #label="{ item }">
              <div
                class="flex w-full flex-col gap-2 rounded-lg border border-default bg-default/60 p-3 transition-all hover:border-accented"
                :style="resolveProtocolCardStyle(item.value, formState.channelType === item.value)"
              >
                <div class="flex items-center gap-2.5">
                  <span
                    class="inline-flex size-8 shrink-0 items-center justify-center rounded-md"
                    :style="resolveProtocolIconStyle(item.value)"
                  >
                    <UIcon :name="item.iconName" class="size-4" />
                  </span>
                  <div class="min-w-0 flex-1 text-start">
                    <div class="truncate text-sm font-semibold text-highlighted">
                      {{ item.label }}
                    </div>
                  </div>
                  <span
                    dir="ltr"
                    class="ms-auto shrink-0 rounded px-1.5 py-0.5 font-mono text-[10px] font-semibold uppercase tracking-wide"
                    :style="resolveProtocolIconStyle(item.value)"
                  >
                    {{ item.transportTag }}
                  </span>
                </div>
                <p class="text-start text-xs text-muted">
                  {{ item.description }}
                </p>
              </div>
            </template>

            <template #description>
              <span class="sr-only" />
            </template>
          </URadioGroup>
        </UFormField>

        <UFormField :label="t('channels.createModal.nameLabel')" name="name" required>
          <UInput
            v-model="formState.name"
            v-maska="INPUT_MASKS.displayName"
            :maxlength="80"
            class="w-full"
            :placeholder="t('channels.createModal.namePlaceholder')"
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
          <div class="flex items-center justify-between gap-3 rounded-lg border border-default bg-elevated/40 px-3.5 py-2.5">
            <span class="text-xs text-muted">
              {{ t('channels.createModal.enabledHint') }}
            </span>
            <USwitch v-model="formState.isEnabled" />
          </div>
        </UFormField>

        <div class="flex items-center justify-end gap-2 border-t border-default pt-4">
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
