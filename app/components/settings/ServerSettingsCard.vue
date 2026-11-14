<script setup lang="ts">
import type { FormSubmitEvent } from '@nuxt/ui'
import type { ValidatedServerSettings } from '~/schemas/serverSettingsForm'
import type { ApplicationHealthSnapshot } from '~/types/application'
import { INPUT_MASKS } from '~/utils/inputMasks'

const props = defineProps<{
  health: ApplicationHealthSnapshot | null
  isHealthInitialLoading: boolean
  isLoadingConfiguration: boolean
  isSavingConfiguration: boolean
  serverFormState: Record<string, any>
  serverSettingsSchema: unknown
}>()

const emit = defineEmits<{
  submit: [event: FormSubmitEvent<ValidatedServerSettings>]
  reset: []
}>()

const { t } = useI18n()
</script>

<template>
  <UCard>
    <h2 class="text-base font-medium text-highlighted">
      {{ t('settings.server.title') }}
    </h2>
    <p class="mt-0.5 text-sm text-muted">
      {{ t('settings.server.description') }}
    </p>

    <div class="mt-4 grid grid-cols-1 gap-3 sm:grid-cols-2">
      <div class="rounded-md border border-default bg-muted/40 p-3">
        <div class="text-xs text-muted">{{ t('settings.server.httpEndpoint') }}</div>
        <USkeleton v-if="props.isHealthInitialLoading || props.isLoadingConfiguration" class="mt-1 h-5 w-32 rounded" />
        <div v-else class="mt-1 font-mono text-sm text-highlighted" dir="ltr">
          {{ props.health?.httpEndpoint ?? '127.0.0.1:4080' }}
        </div>
      </div>
      <div class="rounded-md border border-default bg-muted/40 p-3">
        <div class="text-xs text-muted">{{ t('settings.server.smtpEndpoint') }}</div>
        <USkeleton v-if="props.isHealthInitialLoading || props.isLoadingConfiguration" class="mt-1 h-5 w-32 rounded" />
        <div v-else class="mt-1 font-mono text-sm text-highlighted" dir="ltr">
          {{ props.health?.smtpEndpoint ?? '127.0.0.1:1025' }}
        </div>
      </div>
    </div>

    <SkeletonSettingsFormGrid v-if="props.isLoadingConfiguration" />

    <UForm
      v-else
      :schema="props.serverSettingsSchema"
      :state="props.serverFormState"
      class="mt-6 space-y-5"
      @submit="(event) => emit('submit', event as FormSubmitEvent<ValidatedServerSettings>)"
    >
      <div class="grid grid-cols-1 gap-4 sm:grid-cols-3">
        <SharedHintedFormField
          :label="t('settings.server.form.bindAddress')"
          name="bindAddress"
          required
          :hint="t('settings.server.form.bindAddressHint')"
        >
          <UInput
            v-model="props.serverFormState.bindAddress"
            v-maska="INPUT_MASKS.ipv4Address"
            dir="ltr"
            class="w-full font-mono"
            placeholder="127.0.0.1"
          />
        </SharedHintedFormField>

        <SharedHintedFormField
          :label="t('settings.server.form.httpPort')"
          name="httpPort"
          required
          :hint="t('settings.server.form.httpPortHint')"
        >
          <UInput
            v-model="props.serverFormState.httpPort"
            v-maska="INPUT_MASKS.tcpPort"
            dir="ltr"
            class="w-full font-mono"
            placeholder="4080"
          />
        </SharedHintedFormField>

        <SharedHintedFormField
          :label="t('settings.server.form.smtpPort')"
          name="smtpPort"
          required
          :hint="t('settings.server.form.smtpPortHint')"
        >
          <UInput
            v-model="props.serverFormState.smtpPort"
            v-maska="INPUT_MASKS.tcpPort"
            dir="ltr"
            class="w-full font-mono"
            placeholder="1025"
          />
        </SharedHintedFormField>
      </div>

      <div class="grid grid-cols-1 gap-4 sm:grid-cols-3">
        <SharedHintedFormField
          :label="t('settings.server.form.maxHttpPayload')"
          name="maximumHttpPayloadMegabytes"
          required
          :hint="t('settings.server.form.maxHttpPayloadHint')"
        >
          <UInput
            v-model="props.serverFormState.maximumHttpPayloadMegabytes"
            v-maska="INPUT_MASKS.payloadMegabytes"
            dir="ltr"
            class="w-full font-mono"
            placeholder="10"
          />
        </SharedHintedFormField>

        <SharedHintedFormField
          :label="t('settings.server.form.maxSmtpMessage')"
          name="maximumSmtpMessageMegabytes"
          required
          :hint="t('settings.server.form.maxSmtpMessageHint')"
        >
          <UInput
            v-model="props.serverFormState.maximumSmtpMessageMegabytes"
            v-maska="INPUT_MASKS.payloadMegabytes"
            dir="ltr"
            class="w-full font-mono"
            placeholder="25"
          />
        </SharedHintedFormField>

        <SharedHintedFormField
          :label="t('settings.server.form.retentionDays')"
          name="retentionDays"
          required
          :hint="t('settings.server.form.retentionDaysHint')"
        >
          <UInput
            v-model="props.serverFormState.retentionDays"
            v-maska="INPUT_MASKS.retentionDays"
            dir="ltr"
            class="w-full font-mono"
            placeholder="30"
          />
        </SharedHintedFormField>

        <div class="sm:col-span-3 flex items-center justify-between rounded-lg border border-default bg-muted/20 p-3">
          <div>
            <div class="text-xs font-semibold text-highlighted">
              {{ t('settings.server.form.smtpTls') }}
            </div>
            <div class="text-[11px] text-muted">
              {{ t('settings.server.form.smtpTlsHint') }}
            </div>
          </div>
          <USwitch
            v-model="props.serverFormState.isSmtpTlsEnabled"
            size="sm"
          />
        </div>
      </div>

      <div class="flex items-center justify-between border-t border-default pt-5">
        <UButton
          type="button"
          color="neutral"
          variant="ghost"
          :label="t('common.cancel')"
          @click="emit('reset')"
        />

        <UButton
          type="submit"
          color="primary"
          :loading="props.isSavingConfiguration"
          :label="t('common.saveChanges')"
        />
      </div>
    </UForm>
  </UCard>
</template>
