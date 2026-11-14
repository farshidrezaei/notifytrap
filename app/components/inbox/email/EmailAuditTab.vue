<script setup lang="ts">
import type { AssetCheckResult } from '~/types/application'
import type { auditCapturedEmail } from '~/utils/emailAudit'
import { formatByteSize } from '~/utils/formatters'

const props = defineProps<{
  auditReport: ReturnType<typeof auditCapturedEmail>
  assetCheckResults: readonly AssetCheckResult[]
  isAuditingAssets: boolean
  hasRunAssetAudit: boolean
}>()

const emit = defineEmits<{
  runAudit: []
  copyUrl: [url: string]
}>()

const { t } = useI18n()
</script>

<template>
  <div class="space-y-3">
    <div class="grid grid-cols-2 gap-2 sm:grid-cols-5">
      <div class="rounded-lg border border-default bg-muted/30 p-2.5 text-xs">
        <p class="text-[11px] text-muted">{{ t('inbox.email.audit.metrics.size') }}</p>
        <p dir="ltr" class="mt-0.5 font-mono font-semibold text-highlighted">
          {{ formatByteSize(props.auditReport.rawBytes) }}
        </p>
      </div>
      <div class="rounded-lg border border-default bg-muted/30 p-2.5 text-xs">
        <p class="text-[11px] text-muted">{{ t('inbox.email.audit.metrics.links') }}</p>
        <p class="mt-0.5 font-mono font-semibold text-highlighted">{{ props.auditReport.linkCount }}</p>
      </div>
      <div class="rounded-lg border border-default bg-muted/30 p-2.5 text-xs">
        <p class="text-[11px] text-muted">{{ t('inbox.email.audit.metrics.images') }}</p>
        <p class="mt-0.5 font-mono font-semibold text-highlighted">{{ props.auditReport.imageCount }}</p>
      </div>
      <div class="rounded-lg border border-default bg-muted/30 p-2.5 text-xs">
        <p class="text-[11px] text-muted">{{ t('inbox.email.audit.metrics.missingAlt') }}</p>
        <p class="mt-0.5 font-mono font-semibold text-highlighted">{{ props.auditReport.missingAltCount }}</p>
      </div>
      <div class="rounded-lg border border-default bg-muted/30 p-2.5 text-xs">
        <p class="text-[11px] text-muted">{{ t('inbox.email.audit.metrics.placeholders') }}</p>
        <p class="mt-0.5 font-mono font-semibold" :class="props.auditReport.unrenderedPlaceholderCount > 0 ? 'text-amber-500' : 'text-emerald-500'">
          {{ props.auditReport.unrenderedPlaceholderCount }}
        </p>
      </div>
    </div>

    <div class="space-y-2">
      <div
        v-for="finding in props.auditReport.findings"
        :key="finding.id"
        class="flex items-center justify-between gap-3 rounded-lg border border-default bg-muted/20 px-3 py-2 text-xs"
      >
        <span class="text-highlighted">
          {{ t(finding.translationKey, finding.params ?? {}) }}
        </span>
        <UBadge
          :color="finding.severity === 'pass' ? 'success' : finding.severity === 'warning' ? 'warning' : 'info'"
          variant="subtle"
          size="xs"
        >
          {{ t(`inbox.email.audit.severity.${finding.severity}`) }}
        </UBadge>
      </div>
    </div>

    <!-- Live Link & Remote Asset Health Section -->
    <div class="mt-4 rounded-lg border border-default bg-muted/20 p-3 space-y-3">
      <div class="flex flex-wrap items-center justify-between gap-2">
        <div class="space-y-0.5">
          <h4 class="text-xs font-semibold text-highlighted">
            {{ t('inbox.email.audit.assetHealth.title') }}
          </h4>
          <p class="text-[11px] text-muted">
            {{ t('inbox.email.audit.assetHealth.description') }}
          </p>
        </div>

        <UButton
          size="xs"
          color="primary"
          variant="soft"
          icon="i-lucide-activity"
          :loading="props.isAuditingAssets"
          :label="t('inbox.email.audit.assetHealth.checkButton')"
          @click="emit('runAudit')"
        />
      </div>

      <div v-if="props.hasRunAssetAudit" class="space-y-2 pt-1">
        <div
          v-if="props.assetCheckResults.length === 0"
          class="rounded-md border border-default bg-muted/30 p-2.5 text-center text-xs text-muted"
        >
          {{ t('inbox.email.audit.assetHealth.empty') }}
        </div>

        <div
          v-for="(asset, idx) in props.assetCheckResults"
          :key="`${asset.url}-${idx}`"
          class="flex flex-col gap-1 rounded-md border border-default bg-elevated p-2.5 text-xs shadow-xs"
        >
          <div class="flex items-center justify-between gap-2">
            <div class="flex min-w-0 items-center gap-2">
              <UIcon
                :name="asset.kind === 'link' ? 'i-lucide-link' : 'i-lucide-image'"
                class="size-3.5 shrink-0"
                :class="asset.kind === 'link' ? 'text-primary' : 'text-purple-500'"
              />
              <UBadge
                size="xs"
                variant="subtle"
                :color="asset.kind === 'link' ? 'primary' : 'neutral'"
              >
                {{ t(`inbox.email.audit.assetHealth.kind.${asset.kind}`) }}
              </UBadge>
              <span dir="ltr" class="truncate font-mono text-xs text-highlighted" :title="asset.url">
                {{ asset.url }}
              </span>
            </div>

            <div class="flex shrink-0 items-center gap-1.5">
              <UBadge
                v-if="asset.statusCode"
                size="xs"
                :color="asset.isBroken ? 'error' : 'success'"
                variant="subtle"
              >
                {{ asset.statusCode }} {{ asset.isBroken ? t('inbox.email.audit.assetHealth.status.broken') : 'OK' }}
              </UBadge>
              <UBadge
                v-else
                size="xs"
                color="error"
                variant="subtle"
              >
                {{ t('inbox.email.audit.assetHealth.status.error') }}
              </UBadge>
              <span dir="ltr" class="font-mono text-[11px] text-muted">
                {{ t('inbox.email.audit.assetHealth.latency', { ms: asset.latencyMilliseconds }) }}
              </span>
              <UButton
                size="xs"
                color="neutral"
                variant="ghost"
                square
                icon="i-lucide-copy"
                :aria-label="t('common.copy')"
                @click="emit('copyUrl', asset.url)"
              />
            </div>
          </div>

          <p v-if="asset.errorMessage && asset.isBroken" class="font-mono text-[11px] text-rose-500 ps-5">
            ↳ {{ asset.errorMessage }}
          </p>
        </div>
      </div>
    </div>
  </div>
</template>
