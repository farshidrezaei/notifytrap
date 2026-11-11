import type {
  ApplicationSettingEntry,
  CreateProjectPayload,
  CreateWorkspacePayload,
  Environment,
  Project,
  UpdateProjectPayload,
  UpdateWorkspacePayload,
  Workspace,
  WorkspaceSelectionPreference,
} from '~/types/application'
import { invokeTauriCommand, isTauriRuntime } from '~/utils/tauriClient'

const WORKSPACE_SELECTION_SETTING_KEY = 'workspace.active_selection'

/**
 * Manages hierarchical workspace context (Workspace -> Project -> Environment) with SQLite persistence.
 */
export function useWorkspaceContext() {
  const workspaces = useState<Workspace[]>('workspace-list', () => [])
  const projectsByWorkspace = useState<Record<string, Project[]>>(
    'workspace-projects-by-workspace',
    () => ({}),
  )
  const environments = useState<Environment[]>('workspace-environments', () => [])

  const activeWorkspaceId = useState<string | null>('workspace-active-workspace-id', () => null)
  const activeProjectId = useState<string | null>('workspace-active-project-id', () => null)
  const activeEnvironmentId = useState<string | null>('workspace-active-environment-id', () => null)

  const isLoading = useState<boolean>('workspace-loading', () => false)
  const hasInitialized = useState<boolean>('workspace-initialized', () => false)

  const activeWorkspace = computed<Workspace | null>(
    () => workspaces.value.find((workspace) => workspace.id === activeWorkspaceId.value) ?? null,
  )

  const activeWorkspaceProjects = computed<Project[]>(() => {
    if (!activeWorkspaceId.value) {
      return []
    }
    return projectsByWorkspace.value[activeWorkspaceId.value] ?? []
  })

  const activeProject = computed<Project | null>(() => {
    if (!activeWorkspaceId.value || !activeProjectId.value) {
      return null
    }
    const list = projectsByWorkspace.value[activeWorkspaceId.value] ?? []
    return list.find((project) => project.id === activeProjectId.value) ?? null
  })

  const activeEnvironment = computed<Environment | null>(
    () => environments.value.find((environment) => environment.id === activeEnvironmentId.value) ?? null,
  )

  const hasActiveScope = computed<boolean>(
    () => activeWorkspace.value !== null && activeProject.value !== null && activeEnvironment.value !== null,
  )

  const isInitializing = computed<boolean>(() => isTauriRuntime() && !hasInitialized.value)

  async function persistSelection(): Promise<void> {
    if (!isTauriRuntime()) {
      return
    }
    const preference: WorkspaceSelectionPreference = {
      activeWorkspaceId: activeWorkspaceId.value,
      activeProjectId: activeProjectId.value,
      activeEnvironmentId: activeEnvironmentId.value,
    }
    try {
      await invokeTauriCommand('set_application_setting', {
        key: WORKSPACE_SELECTION_SETTING_KEY,
        value: preference,
      })
    } catch {
      // Non-fatal persistence error; in-memory state remains usable.
    }
  }

  async function loadEnvironmentsForProject(
    projectId: string,
    preferredEnvironmentId?: string | null,
  ): Promise<void> {
    const loaded = await invokeTauriCommand<Environment[]>('list_environments', {
      projectId,
    })
    environments.value = loaded

    const preferred = preferredEnvironmentId
      ? loaded.find((environment) => environment.id === preferredEnvironmentId)
      : undefined
    const fallback = loaded.find((environment) => environment.isDefault) ?? loaded[0] ?? null
    activeEnvironmentId.value = (preferred ?? fallback)?.id ?? null
  }

  async function initializeWorkspace(shouldForceReload = false): Promise<void> {
    if (!isTauriRuntime()) {
      hasInitialized.value = true
      return
    }
    if (hasInitialized.value && !shouldForceReload) {
      return
    }

    isLoading.value = true
    try {
      const loadedWorkspaces = await invokeTauriCommand<Workspace[]>('list_workspaces')
      workspaces.value = loadedWorkspaces

      const nextProjectsMap: Record<string, Project[]> = {}
      await Promise.all(
        loadedWorkspaces.map(async (workspace) => {
          const workspaceProjects = await invokeTauriCommand<Project[]>('list_projects', {
            workspaceId: workspace.id,
          })
          nextProjectsMap[workspace.id] = workspaceProjects
        }),
      )
      projectsByWorkspace.value = nextProjectsMap

      const savedSetting = await invokeTauriCommand<ApplicationSettingEntry<WorkspaceSelectionPreference> | null>(
        'get_application_setting',
        { key: WORKSPACE_SELECTION_SETTING_KEY },
      )
      const saved = savedSetting?.value ?? null

      const selectedWorkspace =
        (saved?.activeWorkspaceId
          ? loadedWorkspaces.find((workspace) => workspace.id === saved.activeWorkspaceId)
          : undefined) ?? loadedWorkspaces[0] ?? null

      activeWorkspaceId.value = selectedWorkspace?.id ?? null

      if (selectedWorkspace) {
        const workspaceProjects = nextProjectsMap[selectedWorkspace.id] ?? []
        const selectedProject =
          (saved?.activeProjectId
            ? workspaceProjects.find((project) => project.id === saved.activeProjectId)
            : undefined) ?? workspaceProjects[0] ?? null

        activeProjectId.value = selectedProject?.id ?? null

        if (selectedProject) {
          await loadEnvironmentsForProject(selectedProject.id, saved?.activeEnvironmentId)
        } else {
          environments.value = []
          activeEnvironmentId.value = null
        }
      } else {
        activeProjectId.value = null
        environments.value = []
        activeEnvironmentId.value = null
      }

      hasInitialized.value = true
    } finally {
      isLoading.value = false
    }
  }

  async function selectWorkspaceAndProject(workspaceId: string, projectId?: string | null): Promise<void> {
    activeWorkspaceId.value = workspaceId
    const workspaceProjects = projectsByWorkspace.value[workspaceId] ?? []
    const targetProject = projectId
      ? workspaceProjects.find((project) => project.id === projectId) ?? workspaceProjects[0] ?? null
      : workspaceProjects[0] ?? null

    activeProjectId.value = targetProject?.id ?? null
    if (targetProject) {
      await loadEnvironmentsForProject(targetProject.id)
    } else {
      environments.value = []
      activeEnvironmentId.value = null
    }
    await persistSelection()
  }

  async function selectEnvironment(environmentId: string): Promise<void> {
    activeEnvironmentId.value = environmentId
    await persistSelection()
  }

  async function setDefaultEnvironment(environmentId: string): Promise<void> {
    if (!activeProjectId.value) {
      return
    }
    environments.value = await invokeTauriCommand<Environment[]>('set_default_environment', {
      projectId: activeProjectId.value,
      environmentId,
    })
    activeEnvironmentId.value = environmentId
    await persistSelection()
  }

  async function createWorkspace(payload: CreateWorkspacePayload): Promise<Workspace> {
    const created = await invokeTauriCommand<Workspace>('create_workspace', { input: payload })
    workspaces.value = [...workspaces.value, created]
    projectsByWorkspace.value = {
      ...projectsByWorkspace.value,
      [created.id]: [],
    }
    return created
  }

  async function updateWorkspace(payload: UpdateWorkspacePayload): Promise<Workspace> {
    const updated = await invokeTauriCommand<Workspace>('update_workspace', { input: payload })
    workspaces.value = workspaces.value.map((workspace) =>
      workspace.id === updated.id ? updated : workspace,
    )
    return updated
  }

  async function deleteWorkspace(workspaceId: string): Promise<void> {
    await invokeTauriCommand('delete_workspace', { workspaceId })
    workspaces.value = workspaces.value.filter((workspace) => workspace.id !== workspaceId)
    const nextProjects = { ...projectsByWorkspace.value }
    delete nextProjects[workspaceId]
    projectsByWorkspace.value = nextProjects

    if (activeWorkspaceId.value === workspaceId) {
      const nextWorkspace = workspaces.value[0] ?? null
      if (nextWorkspace) {
        await selectWorkspaceAndProject(nextWorkspace.id)
      } else {
        activeWorkspaceId.value = null
        activeProjectId.value = null
        environments.value = []
        activeEnvironmentId.value = null
        await persistSelection()
      }
    }
  }

  async function createProject(payload: CreateProjectPayload): Promise<Project> {
    const created = await invokeTauriCommand<Project>('create_project', { input: payload })
    const existingList = projectsByWorkspace.value[created.workspaceId] ?? []
    projectsByWorkspace.value = {
      ...projectsByWorkspace.value,
      [created.workspaceId]: [...existingList, created],
    }
    return created
  }

  async function updateProject(payload: UpdateProjectPayload): Promise<Project> {
    const updated = await invokeTauriCommand<Project>('update_project', { input: payload })
    const existingList = projectsByWorkspace.value[updated.workspaceId] ?? []
    projectsByWorkspace.value = {
      ...projectsByWorkspace.value,
      [updated.workspaceId]: existingList.map((project) =>
        project.id === updated.id ? updated : project,
      ),
    }
    return updated
  }

  async function deleteProject(projectId: string, workspaceId: string): Promise<void> {
    await invokeTauriCommand('delete_project', { projectId })
    const remaining = (projectsByWorkspace.value[workspaceId] ?? []).filter(
      (project) => project.id !== projectId,
    )
    projectsByWorkspace.value = {
      ...projectsByWorkspace.value,
      [workspaceId]: remaining,
    }

    if (activeProjectId.value === projectId) {
      const nextProject = remaining[0] ?? null
      await selectWorkspaceAndProject(workspaceId, nextProject?.id ?? null)
    }
  }

  async function completeFirstRunOnboarding(
    workspacePayload: CreateWorkspacePayload,
    projectName: string,
    projectSlug: string,
  ): Promise<void> {
    const workspace = await createWorkspace(workspacePayload)
    const project = await createProject({
      workspaceId: workspace.id,
      name: projectName,
      slug: projectSlug,
    })
    await selectWorkspaceAndProject(workspace.id, project.id)
  }

  return {
    workspaces: readonly(workspaces),
    projectsByWorkspace: readonly(projectsByWorkspace),
    environments: readonly(environments),
    activeWorkspaceId: readonly(activeWorkspaceId),
    activeProjectId: readonly(activeProjectId),
    activeEnvironmentId: readonly(activeEnvironmentId),
    activeWorkspace,
    activeWorkspaceProjects,
    activeProject,
    activeEnvironment,
    hasActiveScope,
    isLoading: readonly(isLoading),
    hasInitialized: readonly(hasInitialized),
    isInitializing,
    initializeWorkspace,
    selectWorkspaceAndProject,
    selectEnvironment,
    setDefaultEnvironment,
    createWorkspace,
    updateWorkspace,
    deleteWorkspace,
    createProject,
    updateProject,
    deleteProject,
    completeFirstRunOnboarding,
  }
}
