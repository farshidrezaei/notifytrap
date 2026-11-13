<script setup lang="ts">
import type { TunnelProvider } from '~/types/application'

const { t } = useI18n()
const toast = useToast()
const {
  tunnelStatus,
  isOperating,
  tunnelErrorMessage,
  isTunnelActive,
  isTunnelConnected,
  publicTunnelUrl,
  startTunnel,
  stopTunnel,
} = useTunnel()

const selectedProvider = ref<TunnelProvider>('cloudflared')
const customBinaryPath = ref('')
const authenticationToken = ref('')
const customArguments = ref('')

const providerOptions = [
  { label: 'Cloudflare Tunnel (cloudflared)', value: 'cloudflared' },
  { label: 'Ngrok', value: 'ngrok' },
  { label: 'Bore (bore.pub)', value: 'bore' },
  { label: 'Custom Binary', value: 'custom' },
]

async function handleToggleTunnel(): Promise<void> {
  if (isTunnelActive.value) {
    try {
      await stopTunnel()
      toast.add({
        title: t('settings.tunnel.stoppedTitle'),
        color: 'neutral',
        icon: 'i-lucide-square',
      })
    } catch {
      // Error handled by composable
    }
  } else {
    try {
      await startTunnel({
        provider: selectedProvider.value,
        customBinaryPath: customBinaryPath.value.trim() || null,
        authenticationToken: authenticationToken.value.trim() || null,
        customArguments: customArguments.value.trim() || null,
      })
      toast.add({
        title: t('settings.tunnel.startedTitle'),
        color: 'success',
        icon: 'i-lucide-radio',
      })
    } catch {
      // Error handled by composable
    }
  }
}

async function copyPublicUrl(): Promise<void> {
  if (!publicTunnelUrl.value) {
    return
  }
  await navigator.clipboard.writeText(publicTunnelUrl.value)
  toast.add({
    title: t('common.copied'),
    color: 'success',
    icon: 'i-lucide-check',
  })
}
</script>

<template>
  <UCard class="border border-default bg-muted/10">
    <template #header>
      <div class="flex items-center justify-between gap-3">
        <div class="flex items-center gap-2">
          <UIcon name="i-lucide-globe" class="size-5 text-primary" />
          <h3 class="text-base font-semibold text-highlighted">
            {{ t('settings.tunnel.title') }}
          </h3>
        </div>
        <div class="flex items-center gap-2">
          <UBadge
            :color="isTunnelConnected ? 'success' : isTunnelActive ? 'warning' : 'neutral'"
            variant="subtle"
            size="sm"
          >
            {{ t(`settings.tunnel.status.${tunnelStatus.status}`) }}
          </UBadge>
        </div>
      </div>
      <p class="mt-1 text-xs text-muted">
        {{ t('settings.tunnel.description') }}
      </p>
    </template>

    <div class="space-y-4">
      <div v-if="publicTunnelUrl" class="rounded-lg border border-success/30 bg-success/10 p-3">
        <div class="flex items-center justify-between gap-2">
          <div class="space-y-0.5">
            <span class="text-xs font-semibold text-success">
              {{ t('settings.tunnel.publicEndpoint') }}
            </span>
            <div class="font-mono text-xs text-highlighted break-all select-all">
              {{ publicTunnelUrl }}
            </div>
          </div>
          <UButton
            color="success"
            variant="ghost"
            size="xs"
            icon="i-lucide-copy"
            :label="t('common.copy')"
            @click="copyPublicUrl"
          />
        </div>
      </div>

      <div v-if="tunnelErrorMessage" class="rounded-lg border border-error/30 bg-error/10 p-3 text-xs text-error">
        {{ tunnelErrorMessage }}
      </div>

      <div class="grid gap-3 sm:grid-cols-2">
        <UFormField :label="t('settings.tunnel.providerLabel')">
          <USelect
            v-model="selectedProvider"
            :items="providerOptions"
            :disabled="isTunnelActive || isOperating"
            class="w-full"
          />
        </UFormField>

        <UFormField
          v-if="selectedProvider === 'ngrok'"
          :label="t('settings.tunnel.authTokenLabel')"
        >
          <UInput
            v-model="authenticationToken"
            type="password"
            :disabled="isTunnelActive || isOperating"
            placeholder="ngrok auth token"
            class="w-full font-mono text-xs"
          />
        </UFormField>

        <UFormField
          v-if="selectedProvider === 'custom'"
          :label="t('settings.tunnel.customBinaryLabel')"
        >
          <UInput
            v-model="customBinaryPath"
            :disabled="isTunnelActive || isOperating"
            placeholder="/usr/local/bin/cloudflared"
            class="w-full font-mono text-xs"
          />
        </UFormField>
      </div>

      <UFormField
        v-if="selectedProvider === 'custom'"
        :label="t('settings.tunnel.customArgumentsLabel')"
      >
        <UInput
          v-model="customArguments"
          :disabled="isTunnelActive || isOperating"
          placeholder="tunnel --url http://127.0.0.1:{port}"
          class="w-full font-mono text-xs"
        />
      </UFormField>

      <div class="flex items-center justify-between border-t border-default pt-3">
        <span class="text-xs text-muted">
          {{ t('settings.tunnel.targetNote', { target: tunnelStatus.localTarget }) }}
        </span>
        <UButton
          :color="isTunnelActive ? 'error' : 'primary'"
          :variant="isTunnelActive ? 'outline' : 'solid'"
          :icon="isTunnelActive ? 'i-lucide-square' : 'i-lucide-play'"
          :loading="isOperating"
          :label="isTunnelActive ? t('settings.tunnel.stopAction') : t('settings.tunnel.startAction')"
          @click="handleToggleTunnel"
        />
      </div>
    </div>
  </UCard>
</template>
