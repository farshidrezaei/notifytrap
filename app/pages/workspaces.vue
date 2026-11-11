<script setup lang="ts">
import type { FormSubmitEvent } from '@nuxt/ui'
import {
  createProjectFormSchema,
  createWorkspaceFormSchema,
  type ValidatedProjectForm,
  type ValidatedWorkspaceForm,
} from '~/schemas/workspaceForms'
import type { Workspace } from '~/types/application'
import { formatShortTimestamp, formatSlugIdentifier } from '~/utils/formatters'
import { INPUT_MASKS } from '~/utils/inputMasks'
import { extractCommandErrorMessage } from '~/utils/tauriClient'

const { t, locale } = useI18n()
const toast = useToast()
const {
  workspaces,
  projectsByWorkspace,
  activeWorkspaceId,
  activeProjectId,
  hasActiveScope,
  isLoading,
  isInitializing: isWorkspaceInitializing,
  selectWorkspaceAndProject,
  createWorkspace,
  updateWorkspace,
  deleteWorkspace,
  createProject,
} = useWorkspaceContext()

const workspaceSchema = computed(() => createWorkspaceFormSchema(t))
const projectSchema = computed(() => createProjectFormSchema(t))

const isWorkspaceModalOpen = ref(false)
const editingWorkspaceId = ref<string | null>(null)
const isSavingWorkspace = ref(false)
const hasEditedWorkspaceSlug = ref(false)
const workspaceFormState = reactive({
  name: '',
  slug: '',
  description: '',
})

const isProjectModalOpen = ref(false)
const isCreatingProject = ref(false)
const hasEditedProjectSlug = ref(false)
const projectFormState = reactive({
  workspaceId: '',
  name: '',
  slug: '',
  description: '',
})

const workspacePendingDelete = ref<Workspace | null>(null)
const isDeleteConfirmOpen = ref(false)
const isDeletingWorkspace = ref(false)

watch(
  () => workspaceFormState.name,
  (nextName) => {
    if (!editingWorkspaceId.value && !hasEditedWorkspaceSlug.value) {
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

function openCreateWorkspaceModal(): void {
  editingWorkspaceId.value = null
  workspaceFormState.name = ''
  workspaceFormState.slug = ''
  workspaceFormState.description = ''
  hasEditedWorkspaceSlug.value = false
  isWorkspaceModalOpen.value = true
}

function openEditWorkspaceModal(workspace: Workspace): void {
  editingWorkspaceId.value = workspace.id
  workspaceFormState.name = workspace.name
  workspaceFormState.slug = workspace.slug
  workspaceFormState.description = workspace.description
  hasEditedWorkspaceSlug.value = true
  isWorkspaceModalOpen.value = true
}

function openCreateProjectForWorkspace(workspaceId: string): void {
  projectFormState.workspaceId = workspaceId
  projectFormState.name = ''
  projectFormState.slug = ''
  projectFormState.description = ''
  hasEditedProjectSlug.value = false
  isProjectModalOpen.value = true
}

function requestDeleteWorkspace(workspace: Workspace): void {
  workspacePendingDelete.value = workspace
  isDeleteConfirmOpen.value = true
}

async function handleSaveWorkspace(event: FormSubmitEvent<ValidatedWorkspaceForm>): Promise<void> {
  isSavingWorkspace.value = true
  try {
    if (editingWorkspaceId.value) {
      const updated = await updateWorkspace({
        id: editingWorkspaceId.value,
        name: event.data.name,
        slug: event.data.slug,
        description: event.data.description || undefined,
      })
      toast.add({
        title: t('workspace.toast.workspaceUpdatedTitle'),
        description: updated.name,
        color: 'success',
        icon: 'i-lucide-check-circle-2',
      })
    } else {
      const created = await createWorkspace({
        name: event.data.name,
        slug: event.data.slug,
        description: event.data.description || undefined,
      })
      toast.add({
        title: t('workspace.toast.workspaceCreatedTitle'),
        description: created.name,
        color: 'success',
        icon: 'i-lucide-check-circle-2',
      })
    }
    isWorkspaceModalOpen.value = false
  } catch (error) {
    toast.add({
      title: t('common.errorTitle'),
      description: extractCommandErrorMessage(error, t('common.unexpectedError')),
      color: 'error',
      icon: 'i-lucide-alert-circle',
    })
  } finally {
    isSavingWorkspace.value = false
  }
}

async function handleCreateProject(event: FormSubmitEvent<ValidatedProjectForm>): Promise<void> {
  isCreatingProject.value = true
  try {
    const created = await createProject({
      workspaceId: event.data.workspaceId,
      name: event.data.name,
      slug: event.data.slug,
      description: event.data.description || undefined,
    })
    await selectWorkspaceAndProject(created.workspaceId, created.id)
    isProjectModalOpen.value = false
    toast.add({
      title: t('workspace.toast.projectCreatedTitle'),
      description: created.name,
      color: 'success',
      icon: 'i-lucide-check-circle-2',
    })
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

async function handleConfirmDeleteWorkspace(): Promise<void> {
  if (!workspacePendingDelete.value) {
    return
  }
  isDeletingWorkspace.value = true
  try {
    const deletedName = workspacePendingDelete.value.name
    await deleteWorkspace(workspacePendingDelete.value.id)
    isDeleteConfirmOpen.value = false
    workspacePendingDelete.value = null
    toast.add({
      title: t('workspace.toast.workspaceDeletedTitle'),
      description: deletedName,
      color: 'success',
      icon: 'i-lucide-trash-2',
    })
  } catch (error) {
    toast.add({
      title: t('common.errorTitle'),
      description: extractCommandErrorMessage(error, t('common.unexpectedError')),
      color: 'error',
      icon: 'i-lucide-alert-circle',
    })
  } finally {
    isDeletingWorkspace.value = false
  }
}

async function handleOpenProjectDashboard(workspaceId: string, projectId: string): Promise<void> {
  await selectWorkspaceAndProject(workspaceId, projectId)
  await navigateTo('/')
}
</script>

<template>
  <div class="mx-auto flex max-w-5xl flex-col gap-6">
    <SharedSectionHeader
      :title="t('navigation.workspaces')"
      :description="t('workspaces.subtitle')"
    >
      <template #actions>
        <div class="flex items-center gap-2">
          <UButton
            v-if="hasActiveScope"
            size="sm"
            color="neutral"
            variant="outline"
            icon="i-lucide-folder-kanban"
            :label="t('navigation.projects')"
            to="/projects"
          />
          <UButton
            size="sm"
            color="primary"
            icon="i-lucide-plus"
            :label="t('workspace.actions.newWorkspace')"
            @click="openCreateWorkspaceModal"
          />
        </div>
      </template>
    </SharedSectionHeader>

    <SkeletonWorkspaceCardList v-if="isWorkspaceInitializing || isLoading" />

    <UEmpty
      v-else-if="workspaces.length === 0"
      icon="i-lucide-building-2"
      :title="t('workspaces.emptyTitle')"
      :description="t('workspaces.emptyDescription')"
    >
      <template #actions>
        <UButton
          color="primary"
          icon="i-lucide-plus"
          :label="t('workspace.actions.newWorkspace')"
          @click="openCreateWorkspaceModal"
        />
      </template>
    </UEmpty>

    <div v-else class="space-y-4">
      <UCard v-for="workspace in workspaces" :key="workspace.id">
        <div class="flex flex-col gap-4">
          <div class="flex flex-wrap items-start justify-between gap-3">
            <div class="min-w-0 space-y-1">
              <div class="flex flex-wrap items-center gap-2">
                <h2 class="text-base font-semibold text-highlighted">
                  {{ workspace.name }}
                </h2>
                <code dir="ltr" class="rounded bg-muted px-1.5 py-0.5 font-mono text-[11px] text-muted">
                  {{ workspace.slug }}
                </code>
                <UBadge
                  v-if="activeWorkspaceId === workspace.id"
                  color="primary"
                  variant="subtle"
                  size="xs"
                >
                  {{ t('workspace.activeBadge') }}
                </UBadge>
              </div>
              <p v-if="workspace.description" class="text-xs text-muted">
                {{ workspace.description }}
              </p>
              <p class="text-[11px] text-muted">
                {{ t('workspaces.updatedAt', { date: formatShortTimestamp(workspace.updatedAt, locale) }) }}
              </p>
            </div>

            <div class="flex items-center gap-1.5">
              <UButton
                size="xs"
                color="primary"
                variant="soft"
                icon="i-lucide-folder-plus"
                :label="t('workspace.actions.newProject')"
                @click="openCreateProjectForWorkspace(workspace.id)"
              />
              <UButton
                size="xs"
                color="neutral"
                variant="outline"
                square
                icon="i-lucide-pencil"
                :aria-label="t('common.edit')"
                @click="openEditWorkspaceModal(workspace)"
              />
              <UButton
                size="xs"
                color="error"
                variant="ghost"
                square
                icon="i-lucide-trash-2"
                :aria-label="t('common.delete')"
                @click="requestDeleteWorkspace(workspace)"
              />
            </div>
          </div>

          <div class="border-t border-default pt-3">
            <div v-if="(projectsByWorkspace[workspace.id] ?? []).length === 0" class="text-xs text-muted">
              {{ t('workspace.switcher.noProjectsInWorkspace') }}
            </div>

            <div v-else class="grid gap-2 sm:grid-cols-2 lg:grid-cols-3">
              <button
                v-for="project in projectsByWorkspace[workspace.id] ?? []"
                :key="project.id"
                type="button"
                class="flex items-center justify-between gap-2 rounded-lg border border-default px-3 py-2 text-start transition-colors hover:bg-muted/50"
                :class="activeProjectId === project.id ? 'border-primary/60 bg-primary/5' : ''"
                @click="handleOpenProjectDashboard(workspace.id, project.id)"
              >
                <div class="min-w-0">
                  <p class="truncate text-xs font-semibold text-highlighted">
                    {{ project.name }}
                  </p>
                  <code dir="ltr" class="truncate font-mono text-[11px] text-muted">
                    {{ project.slug }}
                  </code>
                </div>
                <UIcon name="i-lucide-arrow-right" class="size-3.5 shrink-0 text-muted" />
              </button>
            </div>
          </div>
        </div>
      </UCard>
    </div>

    <UModal
      v-model:open="isWorkspaceModalOpen"
      :title="
        editingWorkspaceId
          ? t('workspace.modals.editWorkspaceTitle')
          : t('workspace.modals.createWorkspaceTitle')
      "
      :description="t('workspace.modals.createWorkspaceDescription')"
    >
      <template #body>
        <UForm
          :schema="workspaceSchema"
          :state="workspaceFormState"
          :validate-on="['blur', 'change', 'input']"
          class="space-y-4"
          @submit="handleSaveWorkspace"
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
              @click="isWorkspaceModalOpen = false"
            />
            <UButton
              type="submit"
              color="primary"
              :loading="isSavingWorkspace"
              :label="editingWorkspaceId ? t('common.save') : t('workspace.actions.createWorkspace')"
            />
          </div>
        </UForm>
      </template>
    </UModal>

    <UModal
      v-model:open="isProjectModalOpen"
      :title="t('workspace.modals.createProjectTitle')"
      :description="t('workspace.modals.createProjectDescription')"
    >
      <template #body>
        <UForm
          :schema="projectSchema"
          :state="projectFormState"
          :validate-on="['blur', 'change', 'input']"
          class="space-y-4"
          @submit="handleCreateProject"
        >
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
              @click="isProjectModalOpen = false"
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

    <UModal
      v-model:open="isDeleteConfirmOpen"
      :title="t('workspaces.deleteModal.title')"
      :description="t('workspaces.deleteModal.description', { name: workspacePendingDelete?.name ?? '' })"
    >
      <template #body>
        <div class="flex justify-end gap-2">
          <UButton
            color="neutral"
            variant="ghost"
            :label="t('common.cancel')"
            @click="isDeleteConfirmOpen = false"
          />
          <UButton
            color="error"
            :loading="isDeletingWorkspace"
            :label="t('common.delete')"
            @click="handleConfirmDeleteWorkspace"
          />
        </div>
      </template>
    </UModal>
  </div>
</template>
