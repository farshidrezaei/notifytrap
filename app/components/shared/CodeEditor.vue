<script setup lang="ts">
import {
  renderHighlightedCode,
  type CodeLanguageOption,
} from '~/utils/codeHighlighting'

const modelValue = defineModel<string>({ required: true })

const props = withDefaults(
  defineProps<{
    language?: CodeLanguageOption
    minRows?: number
    maxHeightClass?: string
    placeholder?: string
  }>(),
  {
    language: 'json',
    minRows: 4,
    maxHeightClass: 'max-h-56',
    placeholder: '',
  },
)

const { t } = useI18n()
const underlayRef = ref<HTMLElement | null>(null)
const textareaRef = ref<HTMLTextAreaElement | null>(null)

const highlighted = computed(() => renderHighlightedCode(modelValue.value || ' ', props.language))

const lineCount = computed(() => {
  if (!modelValue.value) {
    return 1
  }
  return modelValue.value.split('\n').length
})

const minHeightStyle = computed(() => `${props.minRows * 1.25 + 1.5}rem`)

const canFormatJson = computed(() => {
  const trimmed = modelValue.value.trim()
  if (!trimmed.startsWith('{') && !trimmed.startsWith('[')) {
    return false
  }
  try {
    JSON.parse(trimmed)
    return true
  } catch {
    return false
  }
})

function syncScroll(): void {
  if (!underlayRef.value || !textareaRef.value) {
    return
  }
  underlayRef.value.scrollTop = textareaRef.value.scrollTop
  underlayRef.value.scrollLeft = textareaRef.value.scrollLeft
}

function handleTabKey(event: KeyboardEvent): void {
  const target = event.target as HTMLTextAreaElement | null
  if (!target) {
    return
  }
  const start = target.selectionStart
  const end = target.selectionEnd
  const value = modelValue.value
  modelValue.value = `${value.slice(0, start)}  ${value.slice(end)}`
  nextTick(() => {
    target.selectionStart = start + 2
    target.selectionEnd = start + 2
  })
}

function formatJsonContent(): void {
  try {
    const parsed: unknown = JSON.parse(modelValue.value)
    modelValue.value = JSON.stringify(parsed, null, 2)
  } catch {
    // Keep current text when JSON is invalid
  }
}
</script>

<template>
  <div
    dir="ltr"
    class="overflow-hidden rounded-lg border border-default bg-[var(--nt-code-bg)] text-start focus-within:border-primary focus-within:ring-1 focus-within:ring-primary"
  >
    <div class="flex items-center justify-between gap-2 border-b border-default/60 bg-muted/40 px-2.5 py-1">
      <div class="flex items-center gap-2">
        <span class="font-mono text-[10px] font-semibold uppercase tracking-wide text-muted">
          {{ highlighted.label }}
        </span>
        <span class="font-mono text-[10px] text-muted/80">
          {{ lineCount }}L
        </span>
      </div>

      <UButton
        v-if="canFormatJson"
        type="button"
        size="xs"
        color="neutral"
        variant="ghost"
        icon="i-lucide-wand-sparkles"
        :label="t('common.formatJson')"
        @click="formatJsonContent"
      />
    </div>

    <div class="relative" :class="maxHeightClass" :style="{ minHeight: minHeightStyle }">
      <pre
        ref="underlayRef"
        aria-hidden="true"
        class="nt-code-block hljs pointer-events-none m-0 h-full w-full overflow-auto whitespace-pre-wrap break-words bg-transparent p-3 font-mono text-xs leading-5"
        :style="{ minHeight: minHeightStyle }"
      ><code :class="`language-${highlighted.language}`" v-html="`${highlighted.html}\n`" /></pre>

      <textarea
        ref="textareaRef"
        v-model="modelValue"
        :placeholder="placeholder"
        spellcheck="false"
        autocapitalize="off"
        autocomplete="off"
        autocorrect="off"
        class="nt-code-editor-textarea absolute inset-0 h-full w-full resize-none overflow-auto whitespace-pre-wrap break-words bg-transparent p-3 font-mono text-xs leading-5 outline-none"
        style="color: var(--ui-text-highlighted); caret-color: var(--ui-text-highlighted); -webkit-text-fill-color: transparent;"
        @scroll="syncScroll"
        @keydown.tab.prevent="handleTabKey"
      />
    </div>
  </div>
</template>

<style scoped>
.nt-code-editor-textarea::selection {
  background-color: color-mix(in srgb, var(--nt-primary) 28%, transparent);
}
</style>
