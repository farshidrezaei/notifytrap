<script setup lang="ts">
const props = withDefaults(
  defineProps<{
    label: string
    hint?: string
    name?: string
    required?: boolean
    description?: string
  }>(),
  {
    hint: undefined,
    name: undefined,
    required: false,
    description: undefined,
  },
)
</script>

<template>
  <UFormField
    :name="props.name"
    :required="props.required"
    :description="props.description"
    :ui="{ labelWrapper: 'min-h-5 justify-start gap-1.5', label: 'whitespace-nowrap' }"
  >
    <template #label>
      <span>{{ props.label }}</span>
    </template>

    <template v-if="props.hint" #hint>
      <UPopover :content="{ side: 'top', align: 'start', sideOffset: 6 }">
        <button
          type="button"
          class="inline-flex size-4 cursor-pointer items-center justify-center rounded-full text-muted transition-colors hover:bg-elevated hover:text-primary focus-visible:outline-2 focus-visible:outline-primary"
          :aria-label="props.hint"
          @click.stop
        >
          <UIcon name="i-lucide-info" class="size-3.5" />
        </button>

        <template #content>
          <p class="max-w-xs px-3 py-2 text-xs leading-relaxed text-highlighted">
            {{ props.hint }}
          </p>
        </template>
      </UPopover>
    </template>

    <template #default="slotProps">
      <slot v-bind="slotProps" />
    </template>
  </UFormField>
</template>
