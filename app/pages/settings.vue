<script setup lang="ts">
import type { FormSubmitEvent } from '@nuxt/ui'
import {
  createServerSettingsFormSchema,
  DEFAULT_SERVER_SETTINGS_FORM_VALUES,
  type ServerSettingsFormValues,
  type ValidatedServerSettings,
} from '~/schemas/serverSettingsForm'
import type { ApplicationSettingEntry, ServerConfiguration } from '~/types/application'
import { copyTextToClipboard } from '~/utils/clipboard'
import { INPUT_MASKS } from '~/utils/inputMasks'
import {
  extractCommandErrorMessage,
  invokeTauriCommand,
  isTauriRuntime,
} from '~/utils/tauriClient'

type SettingsCategory = 'general' | 'server' | 'storage' | 'mcp' | 'about'

const RETENTION_DAYS_SETTING_KEY = 'inbox.retention_days'
const BYTES_PER_MEGABYTE = 1024 * 1024

const route = useRoute()
const router = useRouter()
const { t } = useI18n()
const toast = useToast()
const { locale, localeOptions, changeLocale } = useLocaleController()
const { health, isInitialLoading: isHealthInitialLoading, refreshHealth } = useApplicationHealth()
const {
  isMinimizeToTrayEnabled,
  isNativeNotificationsEnabled,
  setMinimizeToTray,
  setNativeNotifications,
} = useDesktopPreferences()

const activeCategory = ref<SettingsCategory>('general')

const navigationCategories = computed<Array<{ id: SettingsCategory; labelKey: string; icon: string }>>(() => [
  { id: 'general', labelKey: 'settings.categories.general', icon: 'i-lucide-settings-2' },
  { id: 'server', labelKey: 'settings.categories.server', icon: 'i-lucide-server' },
  { id: 'storage', labelKey: 'settings.categories.storage', icon: 'i-lucide-database' },
  { id: 'mcp', labelKey: 'settings.categories.mcp', icon: 'i-lucide-bot' },
  { id: 'about', labelKey: 'settings.categories.about', icon: 'i-lucide-info' },
])

function selectCategory(category: SettingsCategory): void {
  activeCategory.value = category
  router.replace({ query: { ...route.query, category } })
}

async function copyText(value: string, label: string): Promise<void> {
  const isCopied = await copyTextToClipboard(value)
  toast.add({
    title: isCopied ? t('common.copiedTitle') : t('common.errorTitle'),
    description: isCopied ? label : t('common.clipboardUnavailable'),
    color: isCopied ? 'success' : 'error',
    icon: isCopied ? 'i-lucide-check-circle-2' : 'i-lucide-alert-circle',
  })
}

const mcpEndpointUrl = computed(
  () => `http://${health.value?.httpEndpoint ?? '127.0.0.1:4080'}/api/v1/mcp`,
)
const mcpCliSnippet = computed(
  () => `notifytrap-cli mcp --base-url http://${health.value?.httpEndpoint ?? '127.0.0.1:4080'} --channel-key <CHANNEL_KEY>`,
)

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
  const initialTab = route.query.category as string | undefined
  if (initialTab && ['general', 'server', 'storage', 'mcp', 'about'].includes(initialTab)) {
    activeCategory.value = initialTab as SettingsCategory
  }

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

    serverFormState.bindAddress = savedConfiguration.bindAddress
    serverFormState.httpPort = String(savedConfiguration.httpPort)
    serverFormState.smtpPort = String(savedConfiguration.smtpPort)
    serverFormState.maximumHttpPayloadMegabytes = String(
      Math.max(1, Math.round(savedConfiguration.maximumHttpPayloadBytes / BYTES_PER_MEGABYTE)),
    )
    serverFormState.maximumSmtpMessageMegabytes = String(
      Math.max(1, Math.round(savedConfiguration.maximumSmtpMessageBytes / BYTES_PER_MEGABYTE)),
    )
    serverFormState.retentionDays = String(event.data.retentionDays)
    shutdownGracePeriodSeconds.value = savedConfiguration.shutdownGracePeriodSeconds

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
  <div class="mx-auto flex max-w-5xl flex-col gap-6">
    <div>
      <h1 class="text-xl font-semibold text-highlighted">
        {{ t('settings.title') }}
      </h1>
      <p class="mt-1 text-sm text-muted">
        {{ t('settings.subtitle') }}
      </p>
    </div>

    <!-- Responsive Categorized Settings Layout -->
    <div class="grid grid-cols-1 items-start gap-6 lg:grid-cols-4">
      <!-- Sidebar Navigation Menu (1 col on desktop, sticky) -->
      <div class="lg:col-span-1">
        <div class="sticky top-4 rounded-xl border border-default bg-elevated p-1.5 shadow-xs space-y-1">
          <button
            v-for="category in navigationCategories"
            :key="category.id"
            type="button"
            class="flex w-full items-center gap-2.5 rounded-lg px-3 py-2.5 text-xs font-medium transition-all text-start"
            :class="activeCategory === category.id
              ? 'bg-primary/10 text-primary font-semibold shadow-xs'
              : 'text-toned hover:bg-muted/50 hover:text-highlighted'"
            @click="selectCategory(category.id)"
          >
            <UIcon
              :name="category.icon"
              class="size-4 shrink-0"
              :class="activeCategory === category.id ? 'text-primary' : 'text-muted'"
            />
            <span class="min-w-0 flex-1 truncate">
              {{ t(category.labelKey) }}
            </span>
            <UIcon
              v-if="activeCategory === category.id"
              name="i-lucide-chevron-right"
              class="size-3.5 shrink-0 rtl:rotate-180 text-primary"
            />
          </button>
        </div>
      </div>

      <!-- Main Content Area for Active Category (3 cols on desktop) -->
      <div class="space-y-6 lg:col-span-3">
        <!-- 1. GENERAL & APPEARANCE -->
        <template v-if="activeCategory === 'general'">
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
            <template #header>
              <div class="space-y-1">
                <h2 class="text-sm font-semibold text-highlighted">
                  {{ t('settings.desktop.title') }}
                </h2>
                <p class="text-xs text-muted">
                  {{ t('settings.desktop.description') }}
                </p>
              </div>
            </template>

            <div class="space-y-3">
              <div class="flex items-center justify-between gap-4 rounded-lg border border-default px-3.5 py-2.5">
                <div class="space-y-0.5">
                  <p class="text-xs font-medium text-highlighted">
                    {{ t('settings.desktop.minimizeToTrayLabel') }}
                  </p>
                  <p class="text-xs text-muted">
                    {{ t('settings.desktop.minimizeToTrayHint') }}
                  </p>
                </div>
                <USwitch
                  :model-value="isMinimizeToTrayEnabled"
                  @update:model-value="setMinimizeToTray"
                />
              </div>

              <div class="flex items-center justify-between gap-4 rounded-lg border border-default px-3.5 py-2.5">
                <div class="space-y-0.5">
                  <p class="text-xs font-medium text-highlighted">
                    {{ t('settings.desktop.nativeNotificationsLabel') }}
                  </p>
                  <p class="text-xs text-muted">
                    {{ t('settings.desktop.nativeNotificationsHint') }}
                  </p>
                </div>
                <USwitch
                  :model-value="isNativeNotificationsEnabled"
                  @update:model-value="setNativeNotifications"
                />
              </div>
            </div>
          </UCard>
        </template>

        <!-- 2. SERVER & NETWORK -->
        <template v-else-if="activeCategory === 'server'">
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
              class="mt-6 space-y-5"
              @submit="handleSaveServerSettings"
            >
              <div class="grid grid-cols-1 gap-4 sm:grid-cols-3">
                <SharedHintedFormField
                  :label="t('settings.server.form.bindAddress')"
                  name="bindAddress"
                  required
                  :hint="t('settings.server.form.bindAddressHint')"
                >
                  <UInput
                    v-model="serverFormState.bindAddress"
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
                    v-model="serverFormState.httpPort"
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
                    v-model="serverFormState.smtpPort"
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
                    v-model="serverFormState.maximumHttpPayloadMegabytes"
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
                    v-model="serverFormState.maximumSmtpMessageMegabytes"
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
                    v-model="serverFormState.retentionDays"
                    v-maska="INPUT_MASKS.retentionDays"
                    dir="ltr"
                    class="w-full font-mono"
                    placeholder="30"
                  />
                </SharedHintedFormField>
              </div>

              <div class="flex items-center justify-between border-t border-default pt-5">
                <UButton
                  type="button"
                  color="neutral"
                  variant="ghost"
                  :label="t('common.cancel')"
                  @click="resetServerForm"
                />

                <UButton
                  type="submit"
                  color="primary"
                  :loading="isSavingConfiguration"
                  :label="t('common.saveChanges')"
                />
              </div>
            </UForm>
          </UCard>
        </template>

        <!-- 3. STORAGE & LIFECYCLE -->
        <template v-else-if="activeCategory === 'storage'">
          <SettingsDatabaseMaintenanceCard :retention-days="Number(serverFormState.retentionDays)" />
        </template>

        <!-- 4. AI & INTEGRATIONS (MCP) -->
        <template v-else-if="activeCategory === 'mcp'">
          <UCard>
            <template #header>
              <div class="space-y-1">
                <h2 class="text-sm font-semibold text-highlighted">
                  {{ t('settings.mcp.title') }}
                </h2>
                <p class="text-xs text-muted">
                  {{ t('settings.mcp.description') }}
                </p>
              </div>
            </template>

            <div class="space-y-3">
              <div class="space-y-1">
                <span class="text-xs font-medium text-toned">
                  {{ t('settings.mcp.httpEndpointLabel') }}
                </span>
                <div class="flex items-center gap-2 rounded-md border border-default bg-muted/40 px-3 py-2">
                  <code dir="ltr" class="min-w-0 flex-1 truncate font-mono text-xs text-default">
                    {{ mcpEndpointUrl }}
                  </code>
                  <UButton
                    size="xs"
                    color="neutral"
                    variant="ghost"
                    square
                    icon="i-lucide-copy"
                    :aria-label="t('common.copy')"
                    @click="copyText(mcpEndpointUrl, t('settings.mcp.httpEndpointLabel'))"
                  />
                </div>
              </div>

              <div class="space-y-1">
                <span class="text-xs font-medium text-toned">
                  {{ t('settings.mcp.cliBridgeLabel') }}
                </span>
                <div class="flex items-center gap-2 rounded-md border border-default bg-muted/40 px-3 py-2">
                  <code dir="ltr" class="min-w-0 flex-1 truncate font-mono text-xs text-default">
                    {{ mcpCliSnippet }}
                  </code>
                  <UButton
                    size="xs"
                    color="neutral"
                    variant="ghost"
                    square
                    icon="i-lucide-copy"
                    :aria-label="t('common.copy')"
                    @click="copyText(mcpCliSnippet, t('settings.mcp.cliBridgeLabel'))"
                  />
                </div>
              </div>
            </div>
          </UCard>
        </template>

        <!-- 5. ABOUT & UPDATES -->
        <template v-else-if="activeCategory === 'about'">
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
        </template>
      </div>
    </div>
  </div>
</template>
