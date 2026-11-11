import type { ChannelCredentialReveal, ChannelSummary, ChannelType } from '~/types/application'
import type { SupportedCodeLanguage } from '~/utils/codeHighlighting'

export interface SnippetOptions {
  channel: ChannelSummary
  credential?: ChannelCredentialReveal | null
  httpEndpoint: string
  smtpEndpoint: string
}

/**
 * Resolves the syntax highlighting language corresponding to a channel integration snippet.
 */
export function resolveChannelIntegrationSnippetLanguage(
  channelType: ChannelType,
): SupportedCodeLanguage {
  return channelType === 'email' ? 'dotenv' : 'bash'
}

/**
 * Generates copy-ready integration snippets bound cleanly to local capture endpoints.
 */
export function buildChannelIntegrationSnippet(options: SnippetOptions): string {
  const { channel, credential, httpEndpoint, smtpEndpoint } = options
  const compositeKey = credential?.compositeKey ?? `${channel.credentialIdentifier}.<SECRET_TOKEN>`
  const secretValue = credential?.secretToken ?? '<SECRET_TOKEN>'
  const [smtpHost, smtpPort] = splitHostAndPort(smtpEndpoint, '127.0.0.1', '1025')
  const [httpHost, httpPort] = splitHostAndPort(httpEndpoint, '127.0.0.1', '4080')
  const baseHttpUrl = `http://${httpHost}:${httpPort}`

  switch (channel.channelType) {
    case 'email':
      return [
        `MAIL_MAILER=smtp`,
        `MAIL_HOST=${smtpHost}`,
        `MAIL_PORT=${smtpPort}`,
        `MAIL_USERNAME=${channel.credentialIdentifier}`,
        `MAIL_PASSWORD=${secretValue}`,
        `MAIL_ENCRYPTION=null`,
      ].join('\n')

    case 'sms':
      return [
        `curl -X POST ${baseHttpUrl}/api/v1/sms \\`,
        `  -H "Content-Type: application/json" \\`,
        `  -H "X-Channel-Key: ${compositeKey}" \\`,
        `  -d '{"to":"+989120000000","from":"NotifyApp","message":"Verification code: 482-910"}'`,
      ].join('\n')

    case 'push':
      return [
        `curl -X POST ${baseHttpUrl}/api/v1/push \\`,
        `  -H "Content-Type: application/json" \\`,
        `  -H "X-Channel-Key: ${compositeKey}" \\`,
        `  -d '{"device_token":"demo-device-1","platform":"android","title":"Hello","body":"Captured push notification"}'`,
      ].join('\n')

    case 'webhook':
      return [
        `curl -X POST ${baseHttpUrl}/api/v1/webhooks/${channel.credentialIdentifier}/events \\`,
        `  -H "Content-Type: application/json" \\`,
        `  -H "X-Channel-Key: ${secretValue}" \\`,
        `  -d '{"event":"invoice.paid","amount":4900}'`,
      ].join('\n')
  }
}

function splitHostAndPort(endpoint: string, fallbackHost: string, fallbackPort: string): [string, string] {
  const separatorIndex = endpoint.lastIndexOf(':')
  if (separatorIndex <= 0) {
    return [fallbackHost, fallbackPort]
  }
  return [endpoint.slice(0, separatorIndex), endpoint.slice(separatorIndex + 1)]
}
