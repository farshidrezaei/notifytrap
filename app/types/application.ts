export type EnvironmentKind = 'local' | 'development' | 'testing' | 'staging'
export type ChannelType = 'email' | 'sms' | 'push' | 'webhook'
export type NotificationStatus = 'captured' | 'read' | 'archived'

export interface ServerConfiguration {
  bindAddress: string
  httpPort: number
  smtpPort: number
  maximumHttpPayloadBytes: number
  maximumSmtpMessageBytes: number
  shutdownGracePeriodSeconds: number
}

export interface ApplicationHealthSnapshot {
  applicationName: string
  applicationVersion: string
  isDatabaseConnected: boolean
  schemaMigrationVersion: number
  isHttpServerRunning: boolean
  isSmtpServerRunning: boolean
  httpEndpoint: string
  smtpEndpoint: string
}

export interface CommandErrorPayload {
  code: string
  message: string
}

export interface Workspace {
  id: string
  name: string
  slug: string
  description: string
  createdAt: string
  updatedAt: string
}

export interface CreateWorkspacePayload {
  name: string
  slug?: string
  description?: string
}

export interface UpdateWorkspacePayload {
  id: string
  name: string
  slug: string
  description?: string
}

export interface Project {
  id: string
  workspaceId: string
  name: string
  slug: string
  description: string
  createdAt: string
  updatedAt: string
}

export interface CreateProjectPayload {
  workspaceId: string
  name: string
  slug?: string
  description?: string
}

export interface UpdateProjectPayload {
  id: string
  name: string
  slug: string
  description?: string
}

export interface Environment {
  id: string
  projectId: string
  name: string
  slug: string
  kind: EnvironmentKind
  isDefault: boolean
  createdAt: string
  updatedAt: string
}

export interface WebhookMockConfiguration {
  isEnabled: boolean
  statusCode: number
  headers: Record<string, string>
  body: string
  delayMilliseconds: number
}

export interface ChannelChaosConfiguration {
  isEnabled: boolean
  latencyMilliseconds: number
  failureRatePercent: number
  httpErrorStatusCode: number
  smtpErrorCode: number
  errorMessage: string
}

export interface ChannelForwardingConfiguration {
  isEnabled: boolean
  targetUrl: string
  customHeaders: Record<string, string>
  includeOriginalHeaders: boolean
  filterEventName?: string | null
  timeoutSeconds: number
}

export type TunnelProvider = 'cloudflared' | 'ngrok' | 'bore' | 'custom'
export type TunnelStatus = 'disconnected' | 'starting' | 'connected' | 'failed'

export interface TunnelStatusReport {
  isActive: boolean
  status: TunnelStatus
  provider: TunnelProvider
  publicUrl?: string | null
  localTarget: string
  errorMessage?: string | null
  startedAt?: string | null
}

export interface TunnelStartConfiguration {
  provider: TunnelProvider
  customBinaryPath?: string | null
  authenticationToken?: string | null
  customArguments?: string | null
}

export interface UpdateChannelRulesInput {
  channelId: string
  webhookMock: WebhookMockConfiguration
  chaosRule: ChannelChaosConfiguration
  forwardingRule: ChannelForwardingConfiguration
}

export interface ReplayHeaderEntry {
  name: string
  value: string
}

export interface WebhookReplayInput {
  notificationId: string
  targetUrl: string
}

export interface WebhookReplayResult {
  targetUrl: string
  method: string
  statusCode: number
  durationMilliseconds: number
  responseHeaders: ReplayHeaderEntry[]
  responseBody: string
}

export interface ChannelSummary {
  id: string
  workspaceId: string
  projectId: string
  environmentId: string
  name: string
  channelType: ChannelType
  isEnabled: boolean
  credentialIdentifier: string
  webhookMock: WebhookMockConfiguration
  chaosRule: ChannelChaosConfiguration
  forwardingRule?: ChannelForwardingConfiguration
  createdAt: string
  updatedAt: string
}

export interface ChannelCredentialReveal {
  channelId: string
  credentialIdentifier: string
  secretToken: string
  compositeKey: string
}

export interface CreatedChannelResult {
  channel: ChannelSummary
  credential: ChannelCredentialReveal
}

export interface CreateChannelPayload {
  workspaceId: string
  projectId: string
  environmentId: string
  name: string
  channelType: ChannelType
  credentialIdentifier?: string
  isEnabled?: boolean
}

export interface NotificationAttachmentSummary {
  id: string
  notificationId: string
  filename: string
  contentType: string
  sizeBytes: number
  contentId: string | null
  isInline: boolean
  createdAt: string
}

export interface NotificationAttachmentContent {
  summary: NotificationAttachmentSummary
  contentBase64: string
}

export type VerificationArtifactKind = 'otpCode' | 'verificationLink'

export interface VerificationArtifact {
  kind: VerificationArtifactKind
  value: string
  label: string
}

export interface NotificationSummary {
  id: string
  workspaceId: string
  projectId: string
  environmentId: string
  channelId: string
  channelType: ChannelType
  status: NotificationStatus
  recipient: string
  sender: string
  subject: string
  previewText: string
  testRunIdentifier: string
  verificationArtifacts: readonly VerificationArtifact[]
  receivedAt: string
}

export interface Notification {
  id: string
  workspaceId: string
  projectId: string
  environmentId: string
  channelId: string
  channelType: ChannelType
  status: NotificationStatus
  recipient: string
  sender: string
  subject: string
  previewText: string
  payloadJson: Record<string, unknown>
  rawPayload: string
  testRunIdentifier: string
  verificationArtifacts: readonly VerificationArtifact[]
  receivedAt: string
  createdAt: string
}

export interface NotificationDetail {
  notification: Notification
  attachments: readonly NotificationAttachmentSummary[]
}

export interface NotificationQuery {
  workspaceId: string
  projectId: string
  environmentId: string
  channelId?: string
  channelType?: ChannelType
  status?: NotificationStatus
  testRunIdentifier?: string
  searchQuery?: string
  limit?: number
  offset?: number
}

export interface NotificationPage {
  items: NotificationSummary[]
  totalCount: number
  limit: number
  offset: number
}

export interface ApplicationSettingEntry<TValue = unknown> {
  key: string
  value: TValue
  updatedAt: string
}

export interface WorkspaceSelectionPreference {
  activeWorkspaceId: string | null
  activeProjectId: string | null
  activeEnvironmentId: string | null
}

export interface NotificationCapturedEvent {
  notification: NotificationSummary
  attachmentCount: number
}

export interface ServerStatusChangedEvent {
  isHttpServerRunning: boolean
  isSmtpServerRunning: boolean
  httpEndpoint: string
  smtpEndpoint: string
}

export interface DatabaseStorageStatistics {
  databasePath: string | null
  fileSizeBytes: number
  walSizeBytes: number
  totalNotificationsCount: number
  totalAttachmentsCount: number
  totalAttachmentsSizeBytes: number
}

export interface AssetCheckResult {
  url: string
  kind: 'link' | 'image'
  statusCode: number | null
  isBroken: boolean
  latencyMilliseconds: number
  errorMessage: string | null
}
