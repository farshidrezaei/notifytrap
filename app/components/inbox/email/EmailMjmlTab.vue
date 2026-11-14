<script setup lang="ts">
import type { MjmlInspectionResult } from '~/utils/mjmlInspector'

const props = defineProps<{
  mjmlInspection: MjmlInspectionResult
  rawSource: string
  mjmlSandboxDocument: string
}>()

const emit = defineEmits<{
  copyCompiled: []
}>()

const { t } = useI18n()
</script>

<template>
  <div class="space-y-3">
    <div class="flex flex-wrap items-center justify-between gap-2 rounded-lg border border-default bg-muted/30 px-3 py-2">
      <div class="flex flex-wrap items-center gap-2">
        <UBadge color="primary" variant="subtle" size="xs">
          {{ t('inbox.email.mjml.badge') }}
        </UBadge>
        <span class="text-xs text-muted">
          {{ t('inbox.email.mjml.tagsCount', { count: props.mjmlInspection.totalTagCount }) }}
        </span>
        <div class="flex flex-wrap gap-1">
          <UBadge
            v-for="tag in props.mjmlInspection.tagSummaries.slice(0, 5)"
            :key="tag.name"
            color="neutral"
            variant="outline"
            size="xs"
            class="font-mono"
          >
            &lt;{{ tag.name }}&gt; ({{ tag.count }})
          </UBadge>
        </div>
      </div>

      <UButton
        size="xs"
        color="primary"
        variant="soft"
        icon="i-lucide-copy"
        :label="t('inbox.email.mjml.copyCompiled')"
        @click="emit('copyCompiled')"
      />
    </div>

    <div class="grid grid-cols-1 gap-4 lg:grid-cols-2">
      <div class="space-y-1.5">
        <h4 class="text-xs font-semibold text-muted">
          {{ t('inbox.email.mjml.sourceTitle') }}
        </h4>
        <SharedCodeBlock
          :code="props.rawSource"
          language="html"
          max-height-class="max-h-[420px]"
        />
      </div>

      <div class="space-y-1.5">
        <h4 class="text-xs font-semibold text-muted">
          {{ t('inbox.email.mjml.previewTitle') }}
        </h4>
        <div class="overflow-hidden rounded-lg border border-default bg-muted/20">
          <iframe
            sandbox=""
            referrerpolicy="no-referrer"
            :title="t('inbox.email.mjml.previewTitle')"
            :srcdoc="props.mjmlSandboxDocument"
            class="h-[420px] w-full border-0 bg-default"
          />
        </div>
      </div>
    </div>
  </div>
</template>
