<script setup lang="ts">
import type { FormSubmitEvent } from '@nuxt/ui'
import {
  createProjectFormSchema,
  type ValidatedProjectForm,
} from '~/schemas/workspaceForms'
import type { Project } from '~/types/application'
import { formatShortTimestamp, formatSlugIdentifier } from '~/utils/formatters'
import { INPUT_MASKS } from '~/utils/inputMasks'
import { extractCommandErrorMessage } from '~/utils/tauriClient'

const { t, locale } = useI18n()
const toast = useToast()
const {
  workspaces,
  activeWorkspace,
  activeWorkspaceProjects,
  activeProjectId,
  isLoading: isWorkspaceLoading,
  isInitializing: isWorkspaceInitializing,
  selectWorkspaceAndProject,
  createProject,
  updateProject,
  deleteProject,
} = useWorkspaceContext()

const projectSchema = computed(() => createProjectFormSchema(t))

const isProjectModalOpen = ref(false)
const editingProjectId = ref<string | null>(null)
const isSavingProject = ref(false)
const hasEditedSlug = ref(false)

const projectFormState = reactive({
  workspaceId: '',
  name: '',
  slug: '',
  description: '',
})

const projectPendingDelete = ref<Project | null>(null)
const isDeleteConfirmOpen = ref(false)
const isDeletingProject = ref(false)

const workspaceSelectOptions = computed(() =>
  workspaces.value.map((workspace) => ({
    label: workspace.name,
    value: workspace.id,
  })),
)

watch(
  () => projectFormState.name,
  (nextName) => {
    if (!editingProjectId.value && !hasEditedSlug.value) {
      projectFormState.slug = formatSlugIdentifier(nextName)
    }
  },
)

function openCreateProjectModal(): void {
  editingProjectId.value = null
  projectFormState.workspaceId = activeWorkspace.value?.id ?? workspaces.value[0]?.id ?? ''
  projectFormState.name = ''
  projectFormState.slug = ''
  projectFormState.description = ''
  hasEditedSlug.value = false
  isProjectModalOpen.value = true
}

function openEditProjectModal(project: Project): void {
  editingProjectId.value = project.id
  projectFormState.workspaceId = project.workspaceId
  projectFormState.name = project.name
  projectFormState.slug = project.slug
  projectFormState.description = project.description
  hasEditedSlug.value = true
  isProjectModalOpen.value = true
}

function requestDeleteProject(project: Project): void {
  projectPendingDelete.value = project
  isDeleteConfirmOpen.value = true
}

async function handleSaveProject(event: FormSubmitEvent<ValidatedProjectForm>): Promise<void> {
  isSavingProject.value = true
  try {
    if (editingProjectId.value) {
      const updated = await updateProject({
        id: editingProjectId.value,
        name: event.data.name,
        slug: event.data.slug,
        description: event.data.description || undefined,
      })
      toast.add({
        title: t('workspace.toast.projectUpdatedTitle'),
        description: updated.name,
        color: 'success',
        icon: 'i-lucide-check-circle-2',
      })
    } else {
      const created = await createProject({
        workspaceId: event.data.workspaceId,
        name: event.data.name,
        slug: event.data.slug,
        description: event.data.description || undefined,
      })
      await selectWorkspaceAndProject(created.workspaceId, created.id)
      toast.add({
        title: t('workspace.toast.projectCreatedTitle'),
        description: created.name,
        color: 'success',
        icon: 'i-lucide-check-circle-2',
      })
    }
    isProjectModalOpen.value = false
  } catch (error) {
    toast.add({
      title: t('common.errorTitle'),
      description: extractCommandErrorMessage(error, t('common.unexpectedError')),
      color: 'error',
      icon: 'i-lucide-alert-circle',
    })
  } finally {
    isSavingProject.value = false
  }
}

async function handleConfirmDeleteProject(): Promise<void> {
  if (!projectPendingDelete.value) {
    return
  }
  isDeletingProject.value = true
  try {
    const deleted = projectPendingDelete.value
    await deleteProject(deleted.id, deleted.workspaceId)
    isDeleteConfirmOpen.value = false
    projectPendingDelete.value = null
    toast.add({
      title: t('workspace.toast.projectDeletedTitle'),
      description: deleted.name,
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
    isDeletingProject.value = false
  }
}

async function handleOpenDashboard(project: Project): Promise<void> {
  await selectWorkspaceAndProject(project.workspaceId, project.id)
  await navigateTo('/')
}
</script>

<template>
  <div class="mx-auto flex max-w-5xl flex-col gap-6">
    <SharedSectionHeader
      :title="t('projects.title')"
      :description="
        activeWorkspace
          ? t('projects.scopedSubtitle', { workspace: activeWorkspace.name })
          : t('projects.subtitle')
      "
    >
      <template #actions>
        <div class="flex items-center gap-2">
          <UButton
            size="sm"
            color="neutral"
            variant="outline"
            icon="i-lucide-building-2"
            :label="t('navigation.workspaces')"
            to="/workspaces"
          />
          <UButton
            size="sm"
            color="primary"
            icon="i-lucide-plus"
            :disabled="workspaces.length === 0"
            :label="t('workspace.actions.newProject')"
            @click="openCreateProjectModal"
          />
        </div>
      </template>
    </SharedSectionHeader>

    <SkeletonProjectGrid v-if="isWorkspaceInitializing || isWorkspaceLoading" />

    <UEmpty
      v-else-if="activeWorkspaceProjects.length === 0"
      icon="i-lucide-folder-kanban"
      :title="t('projects.emptyTitle')"
      :description="t('projects.emptyDescription')"
    >
      <template #actions>
        <UButton
          color="primary"
          icon="i-lucide-plus"
          :disabled="workspaces.length === 0"
          :label="t('workspace.actions.newProject')"
          @click="openCreateProjectModal"
        />
      </template>
    </UEmpty>

    <div v-else class="grid gap-4 sm:grid-cols-2">
      <UCard v-for="project in activeWorkspaceProjects" :key="project.id">
        <div class="flex flex-col gap-3">
          <div class="flex items-start justify-between gap-2">
            <div class="min-w-0 space-y-1">
              <div class="flex flex-wrap items-center gap-2">
                <h2 class="truncate text-base font-semibold text-highlighted">
                  {{ project.name }}
                </h2>
                <UBadge
                  v-if="activeProjectId === project.id"
                  color="primary"
                  variant="subtle"
                  size="xs"
                >
                  {{ t('workspace.activeBadge') }}
                </UBadge>
              </div>
              <code dir="ltr" class="block font-mono text-xs text-muted">
                {{ project.slug }}
              </code>
            </div>

            <div class="flex items-center gap-1">
              <UButton
                size="xs"
                color="neutral"
                variant="outline"
                square
                icon="i-lucide-pencil"
                :aria-label="t('common.edit')"
                @click="openEditProjectModal(project)"
              />
              <UButton
                size="xs"
                color="error"
                variant="ghost"
                square
                icon="i-lucide-trash-2"
                :aria-label="t('common.delete')"
                @click="requestDeleteProject(project)"
              />
            </div>
          </div>

          <p v-if="project.description" class="text-xs text-muted">
            {{ project.description }}
          </p>

          <div class="flex items-center justify-between border-t border-default pt-3">
            <span class="text-[11px] text-muted">
              {{ formatShortTimestamp(project.updatedAt, locale) }}
            </span>
            <UButton
              size="xs"
              color="primary"
              variant="soft"
              icon="i-lucide-layout-dashboard"
              :label="t('projects.actions.openDashboard')"
              @click="handleOpenDashboard(project)"
            />
          </div>
        </div>
      </UCard>
    </div>

    <UModal
      v-model:open="isProjectModalOpen"
      :title="
        editingProjectId
          ? t('workspace.modals.editProjectTitle')
          : t('workspace.modals.createProjectTitle')
      "
      :description="t('workspace.modals.createProjectDescription')"
    >
      <template #body>
        <UForm
          :schema="projectSchema"
          :state="projectFormState"
          :validate-on="['blur', 'change', 'input']"
          class="space-y-4"
          @submit="handleSaveProject"
        >
          <UFormField :label="t('projects.form.workspaceLabel')" name="workspaceId" required>
            <USelect
              v-model="projectFormState.workspaceId"
              :items="workspaceSelectOptions"
              value-key="value"
              :disabled="Boolean(editingProjectId)"
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
              @input="hasEditedSlug = true"
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
              :loading="isSavingProject"
              :label="editingProjectId ? t('common.save') : t('workspace.actions.createProject')"
            />
          </div>
        </UForm>
      </template>
    </UModal>

    <UModal
      v-model:open="isDeleteConfirmOpen"
      :title="t('projects.deleteModal.title')"
      :description="t('projects.deleteModal.description', { name: projectPendingDelete?.name ?? '' })"
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
            :loading="isDeletingProject"
            :label="t('common.delete')"
            @click="handleConfirmDeleteProject"
          />
        </div>
      </template>
    </UModal>
  </div>
</template>
