import type {
  ChannelCredentialReveal,
  ChannelSummary,
  CreateChannelPayload,
  CreatedChannelResult,
} from '~/types/application'
import { invokeTauriCommand, isTauriRuntime } from '~/utils/tauriClient'

/**
 * Manages capture channels and credential reveal/rotation scoped to the active workspace hierarchy.
 */
export function useChannels() {
  const { activeWorkspaceId, activeProjectId, activeEnvironmentId } = useWorkspaceContext()

  const channels = useState<ChannelSummary[]>('scoped-channels', () => [])
  const revealedCredentials = useState<Record<string, ChannelCredentialReveal>>(
    'revealed-channel-credentials',
    () => ({}),
  )
  const isLoading = useState<boolean>('scoped-channels-loading', () => false)

  async function refreshChannels(): Promise<void> {
    if (
      !isTauriRuntime()
      || !activeWorkspaceId.value
      || !activeProjectId.value
      || !activeEnvironmentId.value
    ) {
      channels.value = []
      return
    }

    isLoading.value = true
    try {
      channels.value = await invokeTauriCommand<ChannelSummary[]>('list_channels', {
        workspaceId: activeWorkspaceId.value,
        projectId: activeProjectId.value,
        environmentId: activeEnvironmentId.value,
      })
    } finally {
      isLoading.value = false
    }
  }

  async function createChannel(
    payload: Omit<CreateChannelPayload, 'workspaceId' | 'projectId' | 'environmentId'>,
  ): Promise<CreatedChannelResult> {
    if (!activeWorkspaceId.value || !activeProjectId.value || !activeEnvironmentId.value) {
      throw new Error('Active workspace scope is required')
    }

    const result = await invokeTauriCommand<CreatedChannelResult>('create_channel', {
      input: {
        ...payload,
        workspaceId: activeWorkspaceId.value,
        projectId: activeProjectId.value,
        environmentId: activeEnvironmentId.value,
      },
    })

    channels.value = [...channels.value, result.channel]
    revealedCredentials.value = {
      ...revealedCredentials.value,
      [result.channel.id]: result.credential,
    }
    return result
  }

  async function toggleChannelEnabled(channelId: string, isEnabled: boolean): Promise<ChannelSummary> {
    const updated = await invokeTauriCommand<ChannelSummary>('set_channel_enabled', {
      channelId,
      isEnabled,
    })
    channels.value = channels.value.map((channel) =>
      channel.id === updated.id ? updated : channel,
    )
    return updated
  }

  async function revealCredential(channelId: string): Promise<ChannelCredentialReveal> {
    const reveal = await invokeTauriCommand<ChannelCredentialReveal>('reveal_channel_credential', {
      channelId,
    })
    revealedCredentials.value = {
      ...revealedCredentials.value,
      [channelId]: reveal,
    }
    return reveal
  }

  async function rotateCredential(channelId: string): Promise<ChannelCredentialReveal> {
    const rotated = await invokeTauriCommand<ChannelCredentialReveal>('rotate_channel_credential', {
      channelId,
    })
    revealedCredentials.value = {
      ...revealedCredentials.value,
      [channelId]: rotated,
    }
    return rotated
  }

  function hideCredential(channelId: string): void {
    const next = { ...revealedCredentials.value }
    delete next[channelId]
    revealedCredentials.value = next
  }

  async function deleteChannel(channelId: string): Promise<void> {
    await invokeTauriCommand('delete_channel', { channelId })
    channels.value = channels.value.filter((channel) => channel.id !== channelId)
    hideCredential(channelId)
  }

  return {
    channels: readonly(channels),
    revealedCredentials: readonly(revealedCredentials),
    isLoading: readonly(isLoading),
    refreshChannels,
    createChannel,
    toggleChannelEnabled,
    revealCredential,
    rotateCredential,
    hideCredential,
    deleteChannel,
  }
}
