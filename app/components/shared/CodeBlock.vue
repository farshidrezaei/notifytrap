<script setup lang="ts">
import {
  renderHighlightedCode,
  type CodeLanguageOption,
} from '~/utils/codeHighlighting'
import { copyTextToClipboard } from '~/utils/clipboard'

const props = withDefaults(
  defineProps<{
    code: string
    language?: CodeLanguageOption
    contentType?: string
    maxHeightClass?: string
    showCopyButton?: boolean
    copyDescription?: string
  }>(),
  {
    language: 'auto',
    contentType: undefined,
    maxHeightClass: '',
    showCopyButton: true,
    copyDescription: undefined,
  },
)

const { t } = useI18n()
const toast = useToast()
const isCopied = ref(false)
let copiedResetTimer: ReturnType<typeof setTimeout> | null = null

const highlighted = computed(() =>
  renderHighlightedCode(props.code, props.language, props.contentType),
)

onBeforeUnmount(() => {
  if (copiedResetTimer) {
    clearTimeout(copiedResetTimer)
  }
})

async function handleCopyCode(): Promise<void> {
  const didCopy = await copyTextToClipboard(props.code)
  toast.add({
    title: didCopy ? t('common.copiedTitle') : t('common.copyFailedTitle'),
    description: props.copyDescription ?? highlighted.value.label,
    color: didCopy ? 'success' : 'warning',
    icon: didCopy ? 'i-lucide-clipboard-check' : 'i-lucide-alert-triangle',
  })

  if (!didCopy) {
    return
  }

  isCopied.value = true
  if (copiedResetTimer) {
    clearTimeout(copiedResetTimer)
  }
  copiedResetTimer = setTimeout(() => {
    isCopied.value = false
  }, 1500)
}
</script>

<template>
  <div class="overflow-hidden rounded-lg border border-default bg-muted/40">
    <div class="flex items-center justify-between gap-2 border-b border-default/80 bg-muted/70 px-3 py-1.5">
      <div class="flex items-center gap-2">
        <UIcon name="i-lucide-code-2" class="size-3.5 shrink-0 text-primary" />
        <UBadge
          color="primary"
          variant="subtle"
          size="xs"
          class="font-mono uppercase tracking-wider"
        >
          {{ highlighted.label }}
        </UBadge>
      </div>

      <UButton
        v-if="showCopyButton"
        size="xs"
        color="neutral"
        variant="ghost"
        :icon="isCopied ? 'i-lucide-check' : 'i-lucide-copy'"
        :label="isCopied ? t('common.copiedTitle') : t('common.copy')"
        @click="handleCopyCode"
      />
    </div>

    <pre
      dir="ltr"
      :class="[
        'nt-code-block overflow-auto p-3 font-mono text-xs leading-relaxed text-highlighted',
        maxHeightClass,
      ]"
    ><code :class="`language-${highlighted.language}`" v-html="highlighted.html" /></pre>
  </div>
</template>
