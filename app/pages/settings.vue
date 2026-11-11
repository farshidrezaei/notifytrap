<script setup lang="ts">
import type { FormSubmitEvent } from '@nuxt/ui'
import {
  createServerSettingsFormSchema,
  DEFAULT_SERVER_SETTINGS_FORM_VALUES,
  type ServerSettingsFormValues,
  type ValidatedServerSettings,
} from '~/schemas/serverSettingsForm'
import type { ApplicationSettingEntry, ServerConfiguration } from '~/types/application'
import { INPUT_MASKS } from '~/utils/inputMasks'
import {
  extractCommandErrorMessage,
  invokeTauriCommand,
  isTauriRuntime,
} from '~/utils/tauriClient'

const RETENTION_DAYS_SETTING_KEY = 'inbox.retention_days'
const BYTES_PER_MEGABYTE = 1024 * 1024

const { t } = useI18n()
const toast = useToast()
const { locale, localeOptions, changeLocale } = useLocaleController()
const { health, isInitialLoading: isHealthInitialLoading, refreshHealth } = useApplicationHealth()

const serverSettingsSchema = computed(() => createServerSettingsFormSchema(t))
const isLoadingConfiguration = ref(isTauriRuntime())
const isSavingConfiguration = ref(false)
const shutdownGracePeriodSeconds = ref(5)
const serverFormState = reactive<ServerSettingsFormValues>({
  ...DEFAULT_SERVER_SETTINGS_FORM_VALUES,
})

function extractEndpointParts(endpoint: string): { host: string; port: string } | null {
  const separatorIndex = endpoint.lastIndexOf(':')
  if (separatorIndex <= 0) {
    return null
  }
  return {
    host: endpoint.slice(0, separatorIndex),
    port: endpoint.slice(separatorIndex + 1),
  }
}

watch(
  health,
  (snapshot) => {
    if (!snapshot) {
      return
    }
    const httpParts = extractEndpointParts(snapshot.httpEndpoint)
    const smtpParts = extractEndpointParts(snapshot.smtpEndpoint)
    if (httpParts) {
      serverFormState.bindAddress = httpParts.host
      serverFormState.httpPort = httpParts.port
    }
    if (smtpParts) {
      serverFormState.smtpPort = smtpParts.port
    }
  },
  { immediate: true },
)

onMounted(async () => {
  if (!isTauriRuntime()) {
    isLoadingConfiguration.value = false
    return
  }
  isLoadingConfiguration.value = true
  try {
    const [configuration, retentionSetting] = await Promise.all([
      invokeTauriCommand<ServerConfiguration>('get_server_configuration'),
      invokeTauriCommand<ApplicationSettingEntry<number> | null>('get_application_setting', {
        key: RETENTION_DAYS_SETTING_KEY,
      }),
    ])
    serverFormState.bindAddress = configuration.bindAddress
    serverFormState.httpPort = String(configuration.httpPort)
    serverFormState.smtpPort = String(configuration.smtpPort)
    serverFormState.maximumHttpPayloadMegabytes = String(
      Math.max(1, Math.round(configuration.maximumHttpPayloadBytes / BYTES_PER_MEGABYTE)),
    )
    serverFormState.maximumSmtpMessageMegabytes = String(
      Math.max(1, Math.round(configuration.maximumSmtpMessageBytes / BYTES_PER_MEGABYTE)),
    )
    shutdownGracePeriodSeconds.value = configuration.shutdownGracePeriodSeconds
    if (retentionSetting && typeof retentionSetting.value === 'number') {
      serverFormState.retentionDays = String(retentionSetting.value)
    }
  } catch {
    // Fallback to reactive health endpoint defaults when running outside desktop shell.
  } finally {
    isLoadingConfiguration.value = false
  }
})

async function handleSaveServerSettings(
  event: FormSubmitEvent<ValidatedServerSettings>,
): Promise<void> {
  if (!isTauriRuntime()) {
    toast.add({
      title: t('settings.server.form.savedTitle'),
      description: `${t('settings.server.form.savedDescription')} (${event.data.bindAddress}:${event.data.httpPort} / ${event.data.smtpPort})`,
      icon: 'i-lucide-check-circle-2',
      color: 'success',
    })
    return
  }

  isSavingConfiguration.value = true
  try {
    const savedConfiguration = await invokeTauriCommand<ServerConfiguration>(
      'save_server_configuration',
      {
        configuration: {
          bindAddress: event.data.bindAddress,
          httpPort: event.data.httpPort,
          smtpPort: event.data.smtpPort,
          maximumHttpPayloadBytes: event.data.maximumHttpPayloadMegabytes * BYTES_PER_MEGABYTE,
          maximumSmtpMessageBytes: event.data.maximumSmtpMessageMegabytes * BYTES_PER_MEGABYTE,
          shutdownGracePeriodSeconds: shutdownGracePeriodSeconds.value,
        },
      },
    )
    await invokeTauriCommand('set_application_setting', {
      key: RETENTION_DAYS_SETTING_KEY,
      value: event.data.retentionDays,
    })
    await refreshHealth()
    toast.add({
      title: t('settings.server.form.savedTitle'),
      description: `${t('settings.server.form.savedDescription')} (${savedConfiguration.bindAddress}:${savedConfiguration.httpPort} / ${savedConfiguration.smtpPort})`,
      icon: 'i-lucide-check-circle-2',
      color: 'success',
    })
  } catch (error) {
    toast.add({
      title: t('common.errorTitle'),
      description: extractCommandErrorMessage(error, t('common.unexpectedError')),
      icon: 'i-lucide-alert-circle',
      color: 'error',
    })
  } finally {
    isSavingConfiguration.value = false
  }
}

function resetServerForm(): void {
  Object.assign(serverFormState, DEFAULT_SERVER_SETTINGS_FORM_VALUES)
}
</script>

<template>
  <div class="mx-auto flex max-w-4xl flex-col gap-6">
    <div>
      <h1 class="text-xl font-semibold text-highlighted">
        {{ t('settings.title') }}
      </h1>
      <p class="mt-1 text-sm text-muted">
        {{ t('settings.subtitle') }}
      </p>
    </div>

    <UCard>
      <div class="flex flex-wrap items-center justify-between gap-4">
        <div>
          <h2 class="text-base font-medium text-highlighted">
            {{ t('settings.appearance.title') }}
          </h2>
          <p class="mt-0.5 text-sm text-muted">
            {{ t('settings.appearance.description') }}
          </p>
        </div>
        <UColorModeSelect class="w-44" />
      </div>
    </UCard>

    <UCard>
      <div class="flex flex-wrap items-center justify-between gap-4">
        <div>
          <h2 class="text-base font-medium text-highlighted">
            {{ t('settings.language.title') }}
          </h2>
          <p class="mt-0.5 text-sm text-muted">
            {{ t('settings.language.description') }}
          </p>
        </div>
        <USelect
          :model-value="locale"
          :items="localeOptions"
          value-key="code"
          label-key="label"
          class="w-48"
          @update:model-value="(value) => changeLocale(value as 'en' | 'fa')"
        />
      </div>
    </UCard>

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
          <USkeleton v-if="isHealthInitialLoading || isLoadingConfiguration" class="mt-1 h-5 w-32 rounded" />
          <div v-else class="mt-1 font-mono text-sm text-highlighted" dir="ltr">
            {{ health?.httpEndpoint ?? '127.0.0.1:4080' }}
          </div>
        </div>
        <div class="rounded-md border border-default bg-muted/40 p-3">
          <div class="text-xs text-muted">{{ t('settings.server.smtpEndpoint') }}</div>
          <USkeleton v-if="isHealthInitialLoading || isLoadingConfiguration" class="mt-1 h-5 w-32 rounded" />
          <div v-else class="mt-1 font-mono text-sm text-highlighted" dir="ltr">
            {{ health?.smtpEndpoint ?? '127.0.0.1:1025' }}
          </div>
        </div>
      </div>

      <SkeletonSettingsFormGrid v-if="isLoadingConfiguration" />

      <UForm
        v-else
        :schema="serverSettingsSchema"
        :state="serverFormState"
        :validate-on="['blur', 'change', 'input']"
        class="mt-5 flex flex-col gap-4 border-t border-default pt-5"
        @submit="handleSaveServerSettings"
      >
        <div class="grid grid-cols-1 gap-4 sm:grid-cols-3">
          <SharedHintedFormField
            :label="t('settings.server.form.bindAddressLabel')"
            :hint="t('settings.server.form.bindAddressHint')"
            name="bindAddress"
            required
          >
            <UInput
              v-model="serverFormState.bindAddress"
              v-maska="INPUT_MASKS.ipv4Address"
              icon="i-lucide-globe"
              dir="ltr"
              placeholder="127.0.0.1"
              class="w-full font-mono"
            />
          </SharedHintedFormField>

          <UFormField
            :label="t('settings.server.form.httpPortLabel')"
            name="httpPort"
            required
          >
            <UInput
              v-model="serverFormState.httpPort"
              v-maska="INPUT_MASKS.tcpPort"
              icon="i-lucide-server"
              dir="ltr"
              placeholder="4080"
              class="w-full font-mono"
            />
          </UFormField>

          <UFormField
            :label="t('settings.server.form.smtpPortLabel')"
            name="smtpPort"
            required
          >
            <UInput
              v-model="serverFormState.smtpPort"
              v-maska="INPUT_MASKS.tcpPort"
              icon="i-lucide-mail"
              dir="ltr"
              placeholder="1025"
              class="w-full font-mono"
            />
          </UFormField>
        </div>

        <div class="grid grid-cols-1 gap-4 sm:grid-cols-3">
          <UFormField
            :label="t('settings.server.form.httpPayloadLimitLabel')"
            name="maximumHttpPayloadMegabytes"
            required
          >
            <UInput
              v-model="serverFormState.maximumHttpPayloadMegabytes"
              v-maska="INPUT_MASKS.payloadMegabytes"
              :maxlength="3"
              icon="i-lucide-hard-drive"
              dir="ltr"
              class="w-full font-mono"
            />
          </UFormField>

          <UFormField
            :label="t('settings.server.form.smtpMessageLimitLabel')"
            name="maximumSmtpMessageMegabytes"
            required
          >
            <UInput
              v-model="serverFormState.maximumSmtpMessageMegabytes"
              v-maska="INPUT_MASKS.payloadMegabytes"
              :maxlength="3"
              icon="i-lucide-file-archive"
              dir="ltr"
              class="w-full font-mono"
            />
          </UFormField>

          <UFormField
            :label="t('settings.server.form.retentionDaysLabel')"
            name="retentionDays"
            required
          >
            <UInput
              v-model="serverFormState.retentionDays"
              v-maska="INPUT_MASKS.retentionDays"
              :maxlength="4"
              icon="i-lucide-calendar-clock"
              dir="ltr"
              class="w-full font-mono"
            />
          </UFormField>
        </div>

        <div class="flex flex-wrap items-center justify-end gap-2 pt-2">
          <UButton
            type="button"
            color="neutral"
            variant="outline"
            :label="t('common.reset')"
            @click="resetServerForm"
          />
          <UButton
            type="submit"
            color="primary"
            icon="i-lucide-check"
            :loading="isSavingConfiguration"
            :label="t('settings.server.form.saveLabel')"
          />
        </div>
      </UForm>
    </UCard>

    <SettingsApplicationUpdatesCard />

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
            <USkeleton v-if="isHealthInitialLoading" class="ms-1.5 inline-block h-4 w-10 align-middle rounded" />
            <code v-else class="ms-1.5 font-mono font-medium text-highlighted">v{{ health?.applicationVersion ?? '1.0.0' }}</code>
          </div>
          <div class="rounded-lg border border-default bg-elevated px-3 py-1.5 text-xs">
            <span class="text-muted">{{ t('settings.about.schemaLabel') }}:</span>
            <USkeleton v-if="isHealthInitialLoading" class="ms-1.5 inline-block h-4 w-8 align-middle rounded" />
            <code v-else class="ms-1.5 font-mono font-medium text-highlighted">v{{ health?.schemaMigrationVersion ?? 1 }}</code>
          </div>
        </div>
      </div>
    </UCard>
  </div>
</template>
