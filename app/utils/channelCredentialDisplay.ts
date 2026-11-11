import type { ChannelCredentialReveal, ChannelSummary } from '~/types/application'

export interface ChannelCredentialDisplayModel {
  primaryLabelKey: string
  secondaryLabelKey: string
  revealedSecondaryValue: string | null
  maskedSecondaryValue: string
  isWebhookEndpoint: boolean
}

/**
 * Resolves channel-protocol-specific credential labels and displayed/copyable secret formats
 * so UI credential cards stay aligned with `.env` / HTTP snippet expectations.
 */
export function resolveChannelCredentialDisplay(
  channel: Pick<ChannelSummary, 'channelType' | 'credentialIdentifier'>,
  revealedCredential: ChannelCredentialReveal | null,
  webhookUrl: string,
): ChannelCredentialDisplayModel {
  switch (channel.channelType) {
    case 'email':
      return {
        primaryLabelKey: 'channels.card.smtpUsernameLabel',
        secondaryLabelKey: 'channels.card.smtpPasswordLabel',
        revealedSecondaryValue: revealedCredential?.secretToken ?? null,
        maskedSecondaryValue: 'ntsec_••••••••••••••••••••',
        isWebhookEndpoint: false,
      }

    case 'webhook':
      return {
        primaryLabelKey: 'channels.card.identifierLabel',
        secondaryLabelKey: 'channels.card.webhookUrlLabel',
        revealedSecondaryValue: webhookUrl,
        maskedSecondaryValue: webhookUrl,
        isWebhookEndpoint: true,
      }

    case 'sms':
    case 'push':
      return {
        primaryLabelKey: 'channels.card.identifierLabel',
        secondaryLabelKey: 'channels.card.compositeKeyLabel',
        revealedSecondaryValue: revealedCredential?.compositeKey ?? null,
        maskedSecondaryValue: `${channel.credentialIdentifier}.••••••••••••••••`,
        isWebhookEndpoint: false,
      }
  }
}
