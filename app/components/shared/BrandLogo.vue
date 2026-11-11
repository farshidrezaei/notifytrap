<script setup lang="ts">
type BrandLogoSize = 'xs' | 'sm' | 'md' | 'lg'

const props = withDefaults(
  defineProps<{
    size?: BrandLogoSize
    isDecorative?: boolean
  }>(),
  {
    size: 'md',
    isDecorative: false,
  },
)

const { t } = useI18n()

const SIZE_CLASSES: Record<BrandLogoSize, string> = {
  xs: 'size-5',
  sm: 'size-6',
  md: 'size-10',
  lg: 'size-12',
}

const resolvedSizeClass = computed(() => SIZE_CLASSES[props.size])
</script>

<template>
  <img
    src="/notifytrap-logo.png"
    :alt="isDecorative ? '' : t('application.name')"
    :aria-hidden="isDecorative ? 'true' : undefined"
    :class="[resolvedSizeClass, 'shrink-0 select-none object-contain']"
    draggable="false"
  >
</template>
