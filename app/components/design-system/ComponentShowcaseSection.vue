<script setup lang="ts">
import type { BreadcrumbItem, DropdownMenuItem, FormSubmitEvent, NavigationMenuItem, TableColumn } from '@nuxt/ui'
import {
  createDesignSystemShowcaseSchema,
  DEFAULT_SHOWCASE_FORM_VALUES,
  type DesignSystemShowcaseFormValues,
  type ValidatedShowcaseForm,
} from '~/schemas/designSystemShowcaseForm'
import { INPUT_MASKS } from '~/utils/inputMasks'

const { t } = useI18n()
const toast = useToast()

const showcaseSchema = computed(() => createDesignSystemShowcaseSchema(t))
const showcaseState = reactive<DesignSystemShowcaseFormValues>({
  ...DEFAULT_SHOWCASE_FORM_VALUES,
})

const currentPage = ref(2)
const isConfirmModalOpen = ref(false)
const isInfoModalOpen = ref(false)

const environmentItems = computed(() => [
  { label: t('workspace.environments.local'), value: 'local' },
  { label: t('workspace.environments.development'), value: 'development' },
  { label: t('workspace.environments.testing'), value: 'testing' },
  { label: t('workspace.environments.staging'), value: 'staging' },
])

const radioItems = computed(() => [
  { label: t('channels.types.email'), value: 'email' },
  { label: t('channels.types.sms'), value: 'sms' },
  { label: t('channels.types.push'), value: 'push' },
  { label: t('channels.types.webhook'), value: 'webhook' },
])

const dropdownItems = computed<DropdownMenuItem[][]>(() => [
  [
    { label: t('common.copy'), icon: 'i-lucide-copy' },
    { label: t('common.refresh'), icon: 'i-lucide-refresh-cw' },
  ],
  [
    { label: t('common.delete'), icon: 'i-lucide-trash-2', color: 'error' },
  ],
])

const tabItems = computed(() => [
  { label: t('designSystem.showcase.tabPreview'), icon: 'i-lucide-eye', slot: 'preview' },
  { label: t('designSystem.showcase.tabSource'), icon: 'i-lucide-code-2', slot: 'source' },
])

interface SampleRow {
  channel: string
  recipient: string
  status: string
}

const tableColumns = computed<TableColumn<SampleRow>[]>(() => [
  { accessorKey: 'channel', header: t('designSystem.showcase.tableChannel') },
  { accessorKey: 'recipient', header: t('designSystem.showcase.tableRecipient') },
  { accessorKey: 'status', header: t('designSystem.showcase.tableStatus') },
])

const tableData = computed<SampleRow[]>(() => [
  { channel: t('channels.types.email'), recipient: 'dev@arsam.local', status: t('application.status.serverRunning') },
  { channel: t('channels.types.sms'), recipient: '+989120000000', status: t('application.status.serverRunning') },
])

const breadcrumbItems = computed<BreadcrumbItem[]>(() => [
  { label: t('navigation.overview'), icon: 'i-lucide-home', to: '/' },
  { label: t('navigation.designSystem'), icon: 'i-lucide-swatch-book', to: '/design-system' },
])

const inlineNavItems = computed<NavigationMenuItem[]>(() => [
  { label: t('navigation.overview'), icon: 'i-lucide-layout-dashboard', active: true },
  { label: t('navigation.inbox'), icon: 'i-lucide-inbox', badge: '4' },
  { label: t('navigation.channels'), icon: 'i-lucide-radio' },
])

function triggerToast(): void {
  toast.add({
    title: t('designSystem.showcase.toastTitle'),
    description: t('designSystem.showcase.toastDescription'),
    icon: 'i-lucide-check-circle-2',
    color: 'success',
  })
}

function handleShowcaseSubmit(event: FormSubmitEvent<ValidatedShowcaseForm>): void {
  toast.add({
    title: t('designSystem.showcase.formValidTitle'),
    description: `${t('designSystem.showcase.formValidDescription')} (${event.data.channelIdentifier}:${event.data.localCapturePort})`,
    icon: 'i-lucide-shield-check',
    color: 'success',
  })
}

function resetShowcaseForm(): void {
  Object.assign(showcaseState, DEFAULT_SHOWCASE_FORM_VALUES)
}
</script>

<template>
  <UCard>
    <template #header>
      <SharedSectionHeader
        :title="t('designSystem.sections.components')"
        :description="t('designSystem.descriptions.components')"
      />
    </template>

    <div class="flex flex-col gap-8">
      <!-- Buttons & Icon Buttons -->
      <div>
        <h3 class="mb-3 text-xs font-semibold uppercase tracking-wider text-muted">
          {{ t('designSystem.showcase.buttons') }}
        </h3>
        <div class="flex flex-wrap items-center gap-2.5">
          <UButton color="primary" :label="t('common.save')" />
          <UButton color="secondary" variant="soft" :label="t('common.refresh')" />
          <UButton color="neutral" variant="outline" :label="t('common.cancel')" />
          <UButton color="error" variant="subtle" :label="t('common.delete')" />
          <UButton color="primary" loading :label="t('common.loading')" />
          <UButton color="neutral" disabled :label="t('designSystem.showcase.disabledState')" />
          <UButton color="neutral" variant="outline" icon="i-lucide-copy" :aria-label="t('common.copy')" />
          <UButton color="primary" variant="soft" icon="i-lucide-plus" :aria-label="t('common.save')" />
        </div>
      </div>

      <!-- Standardized Zod + Maska Form Controls -->
      <div>
        <h3 class="mb-3 text-xs font-semibold uppercase tracking-wider text-muted">
          {{ t('designSystem.showcase.formsTitle') }}
        </h3>

        <UForm
          :schema="showcaseSchema"
          :state="showcaseState"
          :validate-on="['blur', 'change', 'input']"
          class="flex flex-col gap-5"
          @submit="handleShowcaseSubmit"
        >
          <div class="grid grid-cols-1 gap-4 md:grid-cols-2">
            <SharedHintedFormField
              :label="t('designSystem.showcase.inputDefault')"
              :hint="t('designSystem.showcase.inputSlugHint')"
              name="channelIdentifier"
              required
            >
              <UInput
                v-model="showcaseState.channelIdentifier"
                v-maska="INPUT_MASKS.singleLineAsciiSafe"
                icon="i-lucide-terminal"
                dir="ltr"
                class="w-full font-mono"
              />
            </SharedHintedFormField>

            <UFormField
              :label="t('designSystem.showcase.phoneLabel')"
              name="recipientPhone"
              required
            >
              <UInput
                v-model="showcaseState.recipientPhone"
                v-maska="INPUT_MASKS.phoneE164"
                icon="i-lucide-phone"
                dir="ltr"
                placeholder="+98 912 000 0000"
                class="w-full font-mono"
              />
            </UFormField>

            <UFormField
              :label="t('designSystem.showcase.portLabel')"
              name="localCapturePort"
              required
            >
              <UInput
                v-model="showcaseState.localCapturePort"
                v-maska="INPUT_MASKS.tcpPort"
                icon="i-lucide-network"
                dir="ltr"
                placeholder="1025"
                class="w-full font-mono"
              />
            </UFormField>

            <UFormField
              :label="t('designSystem.showcase.verificationCodeLabel')"
              name="verificationCode"
              required
            >
              <UInput
                v-model="showcaseState.verificationCode"
                v-maska="INPUT_MASKS.verificationCode"
                icon="i-lucide-key-round"
                dir="ltr"
                placeholder="123-456"
                class="w-full font-mono"
              />
            </UFormField>

            <UFormField :label="t('designSystem.showcase.selectLabel')" name="environmentKind">
              <USelect
                v-model="showcaseState.environmentKind"
                :items="environmentItems"
                value-key="value"
                label-key="label"
                class="w-full"
              />
            </UFormField>

            <UFormField :label="t('designSystem.showcase.searchLabel')" name="searchFilter">
              <UInput
                v-model="showcaseState.searchFilter"
                v-maska="INPUT_MASKS.searchKeyword"
                :maxlength="64"
                icon="i-lucide-search"
                :placeholder="t('designSystem.showcase.searchPlaceholder')"
                class="w-full"
              />
            </UFormField>

            <div class="md:col-span-2">
              <UFormField :label="t('designSystem.showcase.textareaLabel')" name="payloadJson" required>
                <UTextarea
                  v-model="showcaseState.payloadJson"
                  v-maska="INPUT_MASKS.descriptionText"
                  :rows="2"
                  class="w-full font-mono text-xs"
                  dir="ltr"
                />
              </UFormField>
            </div>
          </div>

          <div class="grid grid-cols-1 gap-6 md:grid-cols-3">
            <div class="flex flex-col gap-3">
              <UFormField name="isCaptureEnabled">
                <UCheckbox
                  v-model="showcaseState.isCaptureEnabled"
                  :label="t('designSystem.showcase.checkboxEnabled')"
                />
              </UFormField>
              <UCheckbox disabled :label="t('designSystem.showcase.disabledState')" />
              <USwitch
                v-model="showcaseState.isAutoScrollEnabled"
                :label="t('designSystem.showcase.switchLabel')"
              />
            </div>

            <UFormField name="selectedChannel">
              <URadioGroup
                v-model="showcaseState.selectedChannel"
                :items="radioItems"
                value-key="value"
                label-key="label"
              />
            </UFormField>

            <div class="flex flex-col justify-between gap-3">
              <div class="flex flex-wrap items-center gap-2">
                <UBadge color="primary" variant="solid">{{ t('designSystem.colors.tokens.primary') }}</UBadge>
                <UBadge color="success" variant="subtle">{{ t('designSystem.colors.tokens.success') }}</UBadge>
                <UBadge color="warning" variant="subtle">{{ t('designSystem.colors.tokens.warning') }}</UBadge>
                <UBadge color="error" variant="subtle">{{ t('designSystem.colors.tokens.error') }}</UBadge>
              </div>
              <div class="flex items-center gap-3">
                <UAvatar icon="i-lucide-building-2" size="sm" />
                <UAvatar text="NT" size="md" />
                <UAvatar icon="i-lucide-shield-check" size="lg" />
              </div>
            </div>
          </div>

          <div class="flex flex-wrap items-center justify-end gap-2 border-t border-default pt-4">
            <UButton
              type="button"
              color="neutral"
              variant="outline"
              :label="t('common.reset')"
              @click="resetShowcaseForm"
            />
            <UButton
              type="submit"
              color="primary"
              icon="i-lucide-shield-check"
              :label="t('designSystem.showcase.submitForm')"
            />
          </div>
        </UForm>
      </div>

      <!-- Overlays: Tooltip, Dropdown, Popover, Modals, Toast -->
      <div class="flex flex-wrap items-center gap-3">
        <UTooltip :text="t('designSystem.showcase.tooltipHint')">
          <UButton color="neutral" variant="outline" :label="t('designSystem.showcase.tooltipTrigger')" />
        </UTooltip>

        <UDropdownMenu :items="dropdownItems">
          <UButton color="neutral" variant="outline" trailing-icon="i-lucide-chevron-down" :label="t('designSystem.showcase.dropdownTrigger')" />
        </UDropdownMenu>

        <UPopover>
          <UButton color="neutral" variant="outline" :label="t('designSystem.showcase.popoverTrigger')" />
          <template #content>
            <div class="p-3 text-xs text-toned" style="max-width: var(--nt-popover-width);">
              {{ t('designSystem.showcase.popoverBody') }}
            </div>
          </template>
        </UPopover>

        <UModal
          v-model:open="isInfoModalOpen"
          :title="t('designSystem.showcase.dialogTitle')"
          :description="t('designSystem.showcase.dialogDescription')"
        >
          <UButton color="primary" variant="soft" :label="t('designSystem.showcase.dialogTrigger')" />
          <template #footer>
            <UButton color="primary" :label="t('common.save')" @click="isInfoModalOpen = false" />
          </template>
        </UModal>

        <UModal
          v-model:open="isConfirmModalOpen"
          :title="t('designSystem.showcase.confirmTitle')"
          :description="t('designSystem.showcase.confirmDescription')"
        >
          <UButton color="error" variant="subtle" :label="t('designSystem.showcase.confirmTrigger')" />
          <template #footer>
            <div class="flex justify-end gap-2">
              <UButton color="neutral" variant="outline" :label="t('common.cancel')" @click="isConfirmModalOpen = false" />
              <UButton color="error" :label="t('common.delete')" @click="isConfirmModalOpen = false" />
            </div>
          </template>
        </UModal>

        <UButton color="success" variant="subtle" icon="i-lucide-bell" :label="t('designSystem.showcase.toastTrigger')" @click="triggerToast" />
      </div>

      <!-- Tabs & Table -->
      <div class="grid grid-cols-1 gap-6 lg:grid-cols-2">
        <UTabs :items="tabItems">
          <template #preview>
            <div class="mt-3 rounded-md border border-default bg-muted/30 p-3 text-sm text-default">
              {{ t('designSystem.showcase.tabPreviewBody') }}
            </div>
          </template>
          <template #source>
            <SharedCodeBlock
              code='{ "subject": "Welcome to NotifyTrap", "channel": "email" }'
              language="json"
              class="mt-3"
            />
          </template>
        </UTabs>

        <UTable :data="tableData" :columns="tableColumns" class="rounded-md border border-default" />
      </div>

      <!-- Progress, Skeleton, Alerts, Empty & Error States -->
      <div class="grid grid-cols-1 gap-6 lg:grid-cols-2">
        <div class="flex flex-col gap-4">
          <UProgress :model-value="68" color="primary" />
          <div class="flex items-center gap-3 rounded-md border border-default p-3">
            <USkeleton class="size-10 rounded-full" />
            <div class="flex-1 space-y-2">
              <USkeleton class="h-3.5 w-3/4" />
              <USkeleton class="h-3 w-1/2" />
            </div>
          </div>
          <UAlert color="success" variant="subtle" icon="i-lucide-check-circle-2" :description="t('designSystem.showcase.alertSuccess')" />
          <UAlert color="error" variant="subtle" icon="i-lucide-alert-triangle" :description="t('designSystem.showcase.alertError')" />
        </div>

        <UEmpty
          icon="i-lucide-inbox"
          size="sm"
          :title="t('inbox.emptyTitle')"
          :description="t('inbox.emptyDescription')"
        />
      </div>

      <!-- Navigation, Breadcrumbs, Pagination -->
      <div class="flex flex-col gap-4 rounded-md border border-default bg-muted/20 p-4">
        <UBreadcrumb :items="breadcrumbItems" />
        <UNavigationMenu :items="inlineNavItems" orientation="horizontal" />
        <div class="flex justify-start">
          <UPagination v-model:page="currentPage" :total="50" :items-per-page="10" :sibling-count="1" />
        </div>
      </div>
    </div>
  </UCard>
</template>
