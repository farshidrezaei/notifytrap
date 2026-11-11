<script setup lang="ts">
const localeHead = useLocaleHead()
const { restorePersistedLocale } = useLocaleController()

useHead(() => ({
  htmlAttrs: localeHead.value.htmlAttrs,
}))

onMounted(async () => {
  await restorePersistedLocale()
})
</script>

<template>
  <UApp>
    <NuxtLayout>
      <NuxtPage />
    </NuxtLayout>
  </UApp>
</template>
