<script setup lang="ts">
import type { FormSubmitEvent } from '@nuxt/ui'
import {
  createProjectFormSchema,
  createWorkspaceFormSchema,
  createWorkspaceSearchSchema,
  type ValidatedProjectForm,
  type ValidatedWorkspaceForm,
} from '~/schemas/workspaceForms'
import { buildMonogramLabel, formatSlugIdentifier } from '~/utils/formatters'
import { INPUT_MASKS } from '~/utils/inputMasks'
import { extractCommandErrorMessage } from '~/utils/tauriClient'

const props = withDefaults(
  defineProps<{
    isCollapsed?: boolean
  }>(),
  {
    isCollapsed: false,
  },
)

const { t } = useI18n()
const toast = useToast()
const {
  workspaces,
  projectsByWorkspace,
  activeWorkspace,
  activeProject,
  isInitializing: isWorkspaceInitializing,
  selectWorkspaceAndProject,
  createWorkspace,
  createProject,
} = useWorkspaceContext()

const isPopoverOpen = ref(false)
const searchFormState = reactive({
  searchQuery: '',
})
const workspaceSearchSchema = computed(() => createWorkspaceSearchSchema(t))

const isCreateWorkspaceModalOpen = ref(false)
const isCreatingWorkspace = ref(false)
const hasEditedWorkspaceSlug = ref(false)
const workspaceFormState = reactive({
  name: '',
  slug: '',
  description: '',
})

const isCreateProjectModalOpen = ref(false)
const isCreatingProject = ref(false)
const hasEditedProjectSlug = ref(false)
const projectFormState = reactive({
  workspaceId: '',
  name: '',
  slug: '',
  description: '',
})

const workspaceSchema = computed(() => createWorkspaceFormSchema(t))
const projectSchema = computed(() => createProjectFormSchema(t))

const workspaceSelectOptions = computed(() =>
  workspaces.value.map((workspace) => ({
    label: workspace.name,
    value: workspace.id,
  })),
)

const filteredTree = computed(() => {
  const parsed = workspaceSearchSchema.value.safeParse(searchFormState)
  const normalizedQuery = parsed.success ? parsed.data.searchQuery.toLowerCase() : ''
  return workspaces.value
    .map((workspace) => {
      const projects = projectsByWorkspace.value[workspace.id] ?? []
      if (!normalizedQuery) {
        return { workspace, projects }
      }
      const isWorkspaceMatch =
        workspace.name.toLowerCase().includes(normalizedQuery)
        || workspace.slug.toLowerCase().includes(normalizedQuery)
      const matchingProjects = projects.filter(
        (project) =>
          project.name.toLowerCase().includes(normalizedQuery)
          || project.slug.toLowerCase().includes(normalizedQuery),
      )
      if (!isWorkspaceMatch && matchingProjects.length === 0) {
        return null
      }
      return {
        workspace,
        projects: isWorkspaceMatch ? projects : matchingProjects,
      }
    })
    .filter((entry): entry is NonNullable<typeof entry> => entry !== null)
})

const activeMonogram = computed(() =>
  buildMonogramLabel(activeProject.value?.name ?? activeWorkspace.value?.name),
)

watch(
  () => workspaceFormState.name,
  (nextName) => {
    if (!hasEditedWorkspaceSlug.value) {
      workspaceFormState.slug = formatSlugIdentifier(nextName)
    }
  },
)

watch(
  () => projectFormState.name,
  (nextName) => {
    if (!hasEditedProjectSlug.value) {
      projectFormState.slug = formatSlugIdentifier(nextName)
    }
  },
)

async function handleSelectProject(workspaceId: string, projectId: string): Promise<void> {
  isPopoverOpen.value = false
  await selectWorkspaceAndProject(workspaceId, projectId)
  await navigateTo('/')
}

function openCreateWorkspaceModal(): void {
  isPopoverOpen.value = false
  workspaceFormState.name = ''
  workspaceFormState.slug = ''
  workspaceFormState.description = ''
  hasEditedWorkspaceSlug.value = false
  isCreateWorkspaceModalOpen.value = true
}

function openCreateProjectModal(targetWorkspaceId?: string): void {
  isPopoverOpen.value = false
  projectFormState.workspaceId =
    targetWorkspaceId ?? activeWorkspace.value?.id ?? workspaces.value[0]?.id ?? ''
  projectFormState.name = ''
  projectFormState.slug = ''
  projectFormState.description = ''
  hasEditedProjectSlug.value = false
  isCreateProjectModalOpen.value = true
}

async function handleCreateWorkspaceSubmit(
  event: FormSubmitEvent<ValidatedWorkspaceForm>,
): Promise<void> {
  isCreatingWorkspace.value = true
  try {
    const createdWorkspace = await createWorkspace({
      name: event.data.name,
      slug: event.data.slug,
      description: event.data.description || undefined,
    })
    isCreateWorkspaceModalOpen.value = false
    toast.add({
      title: t('workspace.toast.workspaceCreatedTitle'),
      description: createdWorkspace.name,
      color: 'success',
      icon: 'i-lucide-check-circle-2',
    })
    openCreateProjectModal(createdWorkspace.id)
  } catch (error) {
    toast.add({
      title: t('common.errorTitle'),
      description: extractCommandErrorMessage(error, t('common.unexpectedError')),
      color: 'error',
      icon: 'i-lucide-alert-circle',
    })
  } finally {
    isCreatingWorkspace.value = false
  }
}

async function handleCreateProjectSubmit(event: FormSubmitEvent<ValidatedProjectForm>): Promise<void> {
  isCreatingProject.value = true
  try {
    const createdProject = await createProject({
      workspaceId: event.data.workspaceId,
      name: event.data.name,
      slug: event.data.slug,
      description: event.data.description || undefined,
    })
    await selectWorkspaceAndProject(createdProject.workspaceId, createdProject.id)
    isCreateProjectModalOpen.value = false
    toast.add({
      title: t('workspace.toast.projectCreatedTitle'),
      description: createdProject.name,
      color: 'success',
      icon: 'i-lucide-check-circle-2',
    })
    await navigateTo('/')
  } catch (error) {
    toast.add({
      title: t('common.errorTitle'),
      description: extractCommandErrorMessage(error, t('common.unexpectedError')),
      color: 'error',
      icon: 'i-lucide-alert-circle',
    })
  } finally {
    isCreatingProject.value = false
  }
}
</script>

<template>
  <div class="w-full">
    <template v-if="isWorkspaceInitializing">
      <USkeleton
        v-if="props.isCollapsed"
        class="mx-auto size-9 rounded-lg border border-default/70"
      />
      <div
        v-else
        class="flex w-full items-center justify-between gap-2 rounded-lg border border-default/70 bg-elevated/70 px-2.5 py-2"
      >
        <div class="flex min-w-0 items-center gap-2.5">
          <USkeleton class="size-7 shrink-0 rounded-md" />
          <div class="space-y-1.5">
            <USkeleton class="h-3.5 w-24 rounded" />
            <USkeleton class="h-3 w-16 rounded" />
          </div>
        </div>
        <USkeleton class="size-3.5 shrink-0 rounded" />
      </div>
    </template>

    <UPopover
      v-else
      v-model:open="isPopoverOpen"
      :content="{ side: 'bottom', align: 'start', sideOffset: 6 }"
    >
      <UTooltip
        v-if="props.isCollapsed"
        :text="activeProject ? `${activeWorkspace?.name} / ${activeProject.name}` : t('workspace.switcher.emptyTitle')"
      >
        <UButton
          color="neutral"
          variant="soft"
          square
          class="mx-auto flex size-9 items-center justify-center rounded-lg border border-default/70"
          :aria-label="t('workspace.switcher.ariaLabel')"
        >
          <span v-if="activeProject" class="text-xs font-semibold text-primary">
            {{ activeMonogram }}
          </span>
          <SharedBrandLogo v-else size="xs" is-decorative />
        </UButton>
      </UTooltip>

      <UButton
        v-else
        color="neutral"
        variant="ghost"
        block
        class="h-auto justify-between gap-2 rounded-lg border border-default/70 bg-elevated/70 px-2.5 py-2 hover:bg-elevated"
        :aria-label="t('workspace.switcher.ariaLabel')"
      >
        <div class="flex min-w-0 items-center gap-2.5 text-start">
          <div class="flex size-7 shrink-0 items-center justify-center rounded-md bg-primary/15 text-xs font-semibold text-primary">
            <span v-if="activeProject">{{ activeMonogram }}</span>
            <SharedBrandLogo v-else size="xs" is-decorative />
          </div>
          <div class="min-w-0">
            <p class="truncate text-xs font-semibold text-highlighted">
              {{ activeProject?.name ?? t('workspace.switcher.emptyTitle') }}
            </p>
            <p class="truncate text-[11px] text-muted">
              {{ activeWorkspace?.name ?? t('workspace.switcher.emptySubtitle') }}
            </p>
          </div>
        </div>
        <UIcon name="i-lucide-chevrons-up-down" class="size-3.5 shrink-0 text-muted" />
      </UButton>

      <template #content>
        <div class="flex w-72 flex-col gap-2 p-2">
          <UForm
            :schema="workspaceSearchSchema"
            :state="searchFormState"
            :validate-on="['change', 'input']"
          >
            <UFormField name="searchQuery">
              <UInput
                v-model="searchFormState.searchQuery"
                v-maska="INPUT_MASKS.searchKeyword"
                :maxlength="64"
                size="xs"
                icon="i-lucide-search"
                class="w-full"
                :placeholder="t('workspace.switcher.searchPlaceholder')"
              />
            </UFormField>
          </UForm>

          <div class="max-h-72 space-y-2 overflow-y-auto pe-0.5">
            <div v-if="filteredTree.length === 0" class="px-2 py-4 text-center text-xs text-muted">
              {{ t('workspace.switcher.noMatches') }}
            </div>

            <div
              v-for="group in filteredTree"
              :key="group.workspace.id"
              class="space-y-0.5"
            >
              <div class="flex items-center justify-between gap-2 px-1.5 py-1">
                <div class="flex min-w-0 items-center gap-1.5">
                  <UIcon name="i-lucide-building-2" class="size-3.5 shrink-0 text-muted" />
                  <span class="truncate text-[11px] font-semibold uppercase tracking-wide text-muted">
                    {{ group.workspace.name }}
                  </span>
                </div>
                <UTooltip :text="t('workspace.actions.newProject')">
                  <UButton
                    size="xs"
                    color="neutral"
                    variant="ghost"
                    square
                    icon="i-lucide-plus"
                    :aria-label="t('workspace.actions.newProject')"
                    @click.stop="openCreateProjectModal(group.workspace.id)"
                  />
                </UTooltip>
              </div>

              <div v-if="group.projects.length === 0" class="ps-6 pe-2 py-1 text-xs text-muted">
                {{ t('workspace.switcher.noProjectsInWorkspace') }}
              </div>

              <button
                v-for="project in group.projects"
                :key="project.id"
                type="button"
                class="flex w-full items-center justify-between gap-2 rounded-md ps-5 pe-2 py-1.5 text-start text-xs transition-colors hover:bg-elevated"
                :class="
                  activeProject?.id === project.id
                    ? 'bg-primary/10 font-semibold text-primary'
                    : 'text-toned'
                "
                @click="handleSelectProject(group.workspace.id, project.id)"
              >
                <span class="flex min-w-0 items-center gap-2">
                  <UIcon name="i-lucide-folder-git-2" class="size-3.5 shrink-0" />
                  <span class="truncate">{{ project.name }}</span>
                </span>
                <UIcon
                  v-if="activeProject?.id === project.id"
                  name="i-lucide-check"
                  class="size-3.5 shrink-0"
                />
              </button>
            </div>
          </div>

          <div class="flex items-center justify-between gap-1 border-t border-default pt-2">
            <UButton
              size="xs"
              color="primary"
              variant="soft"
              icon="i-lucide-plus"
              :label="t('workspace.actions.newWorkspace')"
              @click="openCreateWorkspaceModal"
            />
            <UButton
              size="xs"
              color="neutral"
              variant="ghost"
              icon="i-lucide-sliders-horizontal"
              :label="t('navigation.workspaces')"
              to="/workspaces"
              @click="isPopoverOpen = false"
            />
          </div>
        </div>
      </template>
    </UPopover>

    <UModal
      v-model:open="isCreateWorkspaceModalOpen"
      :title="t('workspace.modals.createWorkspaceTitle')"
      :description="t('workspace.modals.createWorkspaceDescription')"
    >
      <template #body>
        <UForm
          :schema="workspaceSchema"
          :state="workspaceFormState"
          :validate-on="['blur', 'change', 'input']"
          class="space-y-4"
          @submit="handleCreateWorkspaceSubmit"
        >
          <UFormField :label="t('workspaces.form.nameLabel')" name="name" required>
            <UInput
              v-model="workspaceFormState.name"
              v-maska="INPUT_MASKS.displayName"
              :maxlength="80"
              class="w-full"
              :placeholder="t('workspaces.form.namePlaceholder')"
            />
          </UFormField>

          <SharedHintedFormField
            :label="t('workspaces.form.slugLabel')"
            :hint="t('workspace.form.slugHint')"
            name="slug"
            required
          >
            <UInput
              v-model="workspaceFormState.slug"
              v-maska="INPUT_MASKS.singleLineAsciiSafe"
              :maxlength="64"
              dir="ltr"
              class="w-full font-mono"
              :placeholder="t('workspaces.form.slugPlaceholder')"
              @input="hasEditedWorkspaceSlug = true"
            />
          </SharedHintedFormField>

          <UFormField :label="t('workspaces.form.descriptionLabel')" name="description">
            <UTextarea
              v-model="workspaceFormState.description"
              v-maska="INPUT_MASKS.descriptionText"
              :maxlength="280"
              class="w-full"
              :rows="2"
              :placeholder="t('workspaces.form.descriptionPlaceholder')"
            />
          </UFormField>

          <div class="flex justify-end gap-2 pt-2">
            <UButton
              color="neutral"
              variant="ghost"
              :label="t('common.cancel')"
              @click="isCreateWorkspaceModalOpen = false"
            />
            <UButton
              type="submit"
              color="primary"
              :loading="isCreatingWorkspace"
              :label="t('workspace.actions.createWorkspace')"
            />
          </div>
        </UForm>
      </template>
    </UModal>

    <UModal
      v-model:open="isCreateProjectModalOpen"
      :title="t('workspace.modals.createProjectTitle')"
      :description="t('workspace.modals.createProjectDescription')"
    >
      <template #body>
        <UForm
          :schema="projectSchema"
          :state="projectFormState"
          :validate-on="['blur', 'change', 'input']"
          class="space-y-4"
          @submit="handleCreateProjectSubmit"
        >
          <UFormField :label="t('projects.form.workspaceLabel')" name="workspaceId" required>
            <USelect
              v-model="projectFormState.workspaceId"
              :items="workspaceSelectOptions"
              value-key="value"
              class="w-full"
            />
          </UFormField>

          <UFormField :label="t('projects.form.nameLabel')" name="name" required>
            <UInput
              v-model="projectFormState.name"
              v-maska="INPUT_MASKS.displayName"
              :maxlength="80"
              class="w-full"
              :placeholder="t('projects.form.namePlaceholder')"
            />
          </UFormField>

          <SharedHintedFormField
            :label="t('projects.form.slugLabel')"
            :hint="t('workspace.form.slugHint')"
            name="slug"
            required
          >
            <UInput
              v-model="projectFormState.slug"
              v-maska="INPUT_MASKS.singleLineAsciiSafe"
              :maxlength="64"
              dir="ltr"
              class="w-full font-mono"
              :placeholder="t('projects.form.slugPlaceholder')"
              @input="hasEditedProjectSlug = true"
            />
          </SharedHintedFormField>

          <UFormField :label="t('projects.form.descriptionLabel')" name="description">
            <UTextarea
              v-model="projectFormState.description"
              v-maska="INPUT_MASKS.descriptionText"
              :maxlength="280"
              class="w-full"
              :rows="2"
              :placeholder="t('projects.form.descriptionPlaceholder')"
            />
          </UFormField>

          <div class="flex justify-end gap-2 pt-2">
            <UButton
              color="neutral"
              variant="ghost"
              :label="t('common.cancel')"
              @click="isCreateProjectModalOpen = false"
            />
            <UButton
              type="submit"
              color="primary"
              :loading="isCreatingProject"
              :label="t('workspace.actions.createProject')"
            />
          </div>
        </UForm>
      </template>
    </UModal>
  </div>
</template>
