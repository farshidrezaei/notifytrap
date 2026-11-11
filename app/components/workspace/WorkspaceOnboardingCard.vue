<script setup lang="ts">
import type { FormSubmitEvent } from '@nuxt/ui'
import {
  createWorkspaceOnboardingSchema,
  NOTIFICATION_CHANNEL_KINDS,
  type ValidatedOnboardingForm,
} from '~/schemas/workspaceForms'
import { formatSlugIdentifier } from '~/utils/formatters'
import { INPUT_MASKS } from '~/utils/inputMasks'
import { extractCommandErrorMessage } from '~/utils/tauriClient'

const { t } = useI18n()
const toast = useToast()
const { health } = useApplicationHealth()
const { completeFirstRunOnboarding } = useWorkspaceContext()
const { refreshChannels } = useChannels()
const { refreshInbox } = useNotifications()

const onboardingSchema = computed(() => createWorkspaceOnboardingSchema(t))
const isSubmitting = ref(false)
const hasEditedWorkspaceSlug = ref(false)
const hasEditedProjectSlug = ref(false)

const formState = reactive({
  workspaceName: '',
  workspaceSlug: '',
  projectName: '',
  projectSlug: '',
})

watch(
  () => formState.workspaceName,
  (nextName) => {
    if (!hasEditedWorkspaceSlug.value) {
      formState.workspaceSlug = formatSlugIdentifier(nextName)
    }
  },
)

watch(
  () => formState.projectName,
  (nextName) => {
    if (!hasEditedProjectSlug.value) {
      formState.projectSlug = formatSlugIdentifier(nextName)
    }
  },
)

async function handleCompleteOnboarding(
  event: FormSubmitEvent<ValidatedOnboardingForm>,
): Promise<void> {
  isSubmitting.value = true
  try {
    await completeFirstRunOnboarding(
      {
        name: event.data.workspaceName,
        slug: event.data.workspaceSlug,
      },
      event.data.projectName,
      event.data.projectSlug,
    )
    await Promise.all([refreshChannels(), refreshInbox()])
    toast.add({
      title: t('onboarding.successTitle'),
      description: t('onboarding.successDescription'),
      color: 'success',
      icon: 'i-lucide-sparkles',
    })
  } catch (error) {
    toast.add({
      title: t('common.errorTitle'),
      description: extractCommandErrorMessage(error, t('common.unexpectedError')),
      color: 'error',
      icon: 'i-lucide-alert-circle',
    })
  } finally {
    isSubmitting.value = false
  }
}
</script>

<template>
  <div class="mx-auto flex max-w-5xl flex-col gap-6 py-4">
    <div class="flex flex-col gap-3 border-b border-default pb-5 sm:flex-row sm:items-center sm:justify-between">
      <div class="flex items-start gap-3.5">
        <div class="flex size-14 shrink-0 items-center justify-center rounded-xl bg-primary/10 ring-1 ring-primary/20">
          <SharedBrandLogo size="lg" is-decorative />
        </div>
        <div class="space-y-1">
          <h1 class="text-xl font-semibold tracking-tight text-highlighted">
            {{ t('onboarding.title') }}
          </h1>
          <p class="max-w-2xl text-sm text-muted">
            {{ t('onboarding.subtitle') }}
          </p>
        </div>
      </div>

      <UBadge color="success" variant="subtle" size="sm" class="self-start sm:self-center">
        {{ t('onboarding.offlineBadge') }}
      </UBadge>
    </div>

    <div class="grid gap-6 lg:grid-cols-5">
      <UCard class="lg:col-span-3">
        <template #header>
          <div class="space-y-0.5">
            <h2 class="text-sm font-semibold text-highlighted">
              {{ t('onboarding.formTitle') }}
            </h2>
            <p class="text-xs text-muted">
              {{ t('onboarding.formDescription') }}
            </p>
          </div>
        </template>

        <UForm
          :schema="onboardingSchema"
          :state="formState"
          :validate-on="['blur', 'change', 'input']"
          class="space-y-5"
          @submit="handleCompleteOnboarding"
        >
          <div class="grid gap-4 sm:grid-cols-2">
            <UFormField :label="t('onboarding.workspaceNameLabel')" name="workspaceName" required>
              <UInput
                v-model="formState.workspaceName"
                v-maska="INPUT_MASKS.displayName"
                :maxlength="80"
                class="w-full"
                :placeholder="t('onboarding.workspaceNamePlaceholder')"
              />
            </UFormField>

            <SharedHintedFormField
              :label="t('onboarding.workspaceSlugLabel')"
              :hint="t('workspace.form.slugHint')"
              name="workspaceSlug"
              required
            >
              <UInput
                v-model="formState.workspaceSlug"
                v-maska="INPUT_MASKS.singleLineAsciiSafe"
                :maxlength="64"
                dir="ltr"
                class="w-full font-mono"
                :placeholder="t('onboarding.workspaceSlugPlaceholder')"
                @input="hasEditedWorkspaceSlug = true"
              />
            </SharedHintedFormField>
          </div>

          <div class="grid gap-4 sm:grid-cols-2">
            <UFormField :label="t('onboarding.projectNameLabel')" name="projectName" required>
              <UInput
                v-model="formState.projectName"
                v-maska="INPUT_MASKS.displayName"
                :maxlength="80"
                class="w-full"
                :placeholder="t('onboarding.projectNamePlaceholder')"
              />
            </UFormField>

            <SharedHintedFormField
              :label="t('onboarding.projectSlugLabel')"
              :hint="t('workspace.form.slugHint')"
              name="projectSlug"
              required
            >
              <UInput
                v-model="formState.projectSlug"
                v-maska="INPUT_MASKS.singleLineAsciiSafe"
                :maxlength="64"
                dir="ltr"
                class="w-full font-mono"
                :placeholder="t('onboarding.projectSlugPlaceholder')"
                @input="hasEditedProjectSlug = true"
              />
            </SharedHintedFormField>
          </div>

          <div class="flex flex-wrap items-center justify-between gap-3 border-t border-default pt-4">
            <p class="text-xs text-muted">
              {{ t('onboarding.defaultEnvironmentsNote') }}
            </p>
            <UButton
              type="submit"
              color="primary"
              icon="i-lucide-arrow-right"
              :loading="isSubmitting"
              :label="t('onboarding.submit')"
            />
          </div>
        </UForm>
      </UCard>

      <div class="flex flex-col gap-4 lg:col-span-2">
        <UCard>
          <template #header>
            <div class="flex items-center justify-between gap-2">
              <h2 class="text-sm font-semibold text-highlighted">
                {{ t('onboarding.captureReadyTitle') }}
              </h2>
              <UBadge
                :color="health?.isHttpServerRunning && health?.isSmtpServerRunning ? 'success' : 'neutral'"
                variant="subtle"
                size="xs"
              >
                {{
                  health?.isHttpServerRunning && health?.isSmtpServerRunning
                    ? t('dashboard.serverLive')
                    : t('dashboard.serverStandby')
                }}
              </UBadge>
            </div>
          </template>

          <div class="space-y-3">
            <div class="rounded-md border border-default bg-muted/40 px-3 py-2">
              <p class="text-[11px] font-medium text-muted">{{ t('overview.cards.httpEndpoint') }}</p>
              <code dir="ltr" class="mt-0.5 block font-mono text-xs font-semibold text-highlighted">
                http://{{ health?.httpEndpoint ?? '127.0.0.1:4080' }}
              </code>
            </div>

            <div class="rounded-md border border-default bg-muted/40 px-3 py-2">
              <p class="text-[11px] font-medium text-muted">{{ t('overview.cards.smtpEndpoint') }}</p>
              <code dir="ltr" class="mt-0.5 block font-mono text-xs font-semibold text-highlighted">
                {{ health?.smtpEndpoint ?? '127.0.0.1:1025' }}
              </code>
            </div>

            <div class="flex flex-wrap gap-1.5 pt-1">
              <SharedChannelBadge
                v-for="kind in NOTIFICATION_CHANNEL_KINDS"
                :key="kind"
                :channel-type="kind"
              />
            </div>
          </div>
        </UCard>

        <UCard>
          <div class="space-y-2.5 text-xs text-toned">
            <h3 class="text-xs font-semibold text-highlighted">
              {{ t('onboarding.highlightsTitle') }}
            </h3>
            <ul class="space-y-2">
              <li class="flex items-start gap-2">
                <UIcon name="i-lucide-layers" class="mt-0.5 size-3.5 shrink-0 text-primary" />
                <span>{{ t('onboarding.highlightEnvironments') }}</span>
              </li>
              <li class="flex items-start gap-2">
                <UIcon name="i-lucide-shield-check" class="mt-0.5 size-3.5 shrink-0 text-primary" />
                <span>{{ t('onboarding.highlightSecurity') }}</span>
              </li>
              <li class="flex items-start gap-2">
                <UIcon name="i-lucide-wifi-off" class="mt-0.5 size-3.5 shrink-0 text-primary" />
                <span>{{ t('onboarding.highlightOffline') }}</span>
              </li>
            </ul>
          </div>
        </UCard>
      </div>
    </div>
  </div>
</template>
