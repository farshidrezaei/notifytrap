<script setup lang="ts">
import type { FormSubmitEvent } from '@nuxt/ui'
import AboutSettingsCard from '~/components/settings/AboutSettingsCard.vue'
import ApplicationUpdatesCard from '~/components/settings/ApplicationUpdatesCard.vue'
import DatabaseMaintenanceCard from '~/components/settings/DatabaseMaintenanceCard.vue'
import GeneralSettingsCard from '~/components/settings/GeneralSettingsCard.vue'
import McpSettingsCard from '~/components/settings/McpSettingsCard.vue'
import PublicTunnelCard from '~/components/settings/PublicTunnelCard.vue'
import ServerSettingsCard from '~/components/settings/ServerSettingsCard.vue'
import {
  createServerSettingsFormSchema,
  DEFAULT_SERVER_SETTINGS_FORM_VALUES,
  type ServerSettingsFormValues,
  type ValidatedServerSettings,
} from '~/schemas/serverSettingsForm'
import type { ApplicationSettingEntry, ServerConfiguration } from '~/types/application'
import { copyTextToClipboard } from '~/utils/clipboard'
import {
  extractCommandErrorMessage,
  invokeTauriCommand,
  isTauriRuntime,
} from '~/utils/tauriClient'

type SettingsCategory = 'general' | 'server' | 'storage' | 'tunnel' | 'mcp' | 'about'

const RETENTION_DAYS_SETTING_KEY = 'inbox.retention_days'
const BYTES_PER_MEGABYTE = 1024 * 1024

const route = useRoute()
const router = useRouter()
const { t } = useI18n()
const toast = useToast()
const { health, isInitialLoading: isHealthInitialLoading, refreshHealth } = useApplicationHealth()

const activeCategory = ref<SettingsCategory>('general')

const navigationCategories = computed<Array<{ id: SettingsCategory; labelKey: string; icon: string }>>(() => [
  { id: 'general', labelKey: 'settings.categories.general', icon: 'i-lucide-settings-2' },
  { id: 'server', labelKey: 'settings.categories.server', icon: 'i-lucide-server' },
  { id: 'storage', labelKey: 'settings.categories.storage', icon: 'i-lucide-database' },
  { id: 'tunnel', labelKey: 'settings.categories.tunnel', icon: 'i-lucide-globe' },
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
  if (initialTab && ['general', 'server', 'storage', 'tunnel', 'mcp', 'about'].includes(initialTab)) {
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
        <GeneralSettingsCard v-if="activeCategory === 'general'" />

        <!-- 2. SERVER & NETWORK -->
        <ServerSettingsCard
          v-else-if="activeCategory === 'server'"
          :health="health"
          :is-health-initial-loading="isHealthInitialLoading"
          :is-loading-configuration="isLoadingConfiguration"
          :is-saving-configuration="isSavingConfiguration"
          :server-form-state="serverFormState"
          :server-settings-schema="serverSettingsSchema"
          @submit="handleSaveServerSettings"
          @reset="resetServerForm"
        />

        <!-- 3. STORAGE & LIFECYCLE -->
        <template v-else-if="activeCategory === 'storage'">
          <DatabaseMaintenanceCard :retention-days="Number(serverFormState.retentionDays)" />
        </template>

        <!-- 4. PUBLIC TUNNEL / WEBHOOK INGRESS -->
        <PublicTunnelCard v-else-if="activeCategory === 'tunnel'" />

        <!-- 5. AI & INTEGRATIONS (MCP) -->
        <McpSettingsCard
          v-else-if="activeCategory === 'mcp'"
          :mcp-endpoint-url="mcpEndpointUrl"
          :mcp-cli-snippet="mcpCliSnippet"
          @copy="copyText"
        />

        <!-- 6. ABOUT & UPDATES -->
        <template v-else-if="activeCategory === 'about'">
          <ApplicationUpdatesCard />
          <AboutSettingsCard
            :health="health"
            :is-health-initial-loading="isHealthInitialLoading"
          />
        </template>
      </div>
    </div>
  </div>
</template>
