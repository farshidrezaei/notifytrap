<script setup lang="ts">
const props = defineProps<{
  mcpEndpointUrl: string
  mcpCliSnippet: string
}>()

const emit = defineEmits<{
  copy: [value: string, label: string]
}>()

const { t } = useI18n()
</script>

<template>
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
            {{ props.mcpEndpointUrl }}
          </code>
          <UButton
            size="xs"
            color="neutral"
            variant="ghost"
            square
            icon="i-lucide-copy"
            :aria-label="t('common.copy')"
            @click="emit('copy', props.mcpEndpointUrl, t('settings.mcp.httpEndpointLabel'))"
          />
        </div>
      </div>

      <div class="space-y-1">
        <span class="text-xs font-medium text-toned">
          {{ t('settings.mcp.cliBridgeLabel') }}
        </span>
        <div class="flex items-center gap-2 rounded-md border border-default bg-muted/40 px-3 py-2">
          <code dir="ltr" class="min-w-0 flex-1 truncate font-mono text-xs text-default">
            {{ props.mcpCliSnippet }}
          </code>
          <UButton
            size="xs"
            color="neutral"
            variant="ghost"
            square
            icon="i-lucide-copy"
            :aria-label="t('common.copy')"
            @click="emit('copy', props.mcpCliSnippet, t('settings.mcp.cliBridgeLabel'))"
          />
        </div>
      </div>
    </div>
  </UCard>
</template>
