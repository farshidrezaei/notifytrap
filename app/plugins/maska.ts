import { vMaska } from 'maska/vue'

/**
 * Registers the Maska input masking directive globally for all form inputs.
 */
export default defineNuxtPlugin((nuxtApp) => {
  nuxtApp.vueApp.directive('maska', vMaska)
})
