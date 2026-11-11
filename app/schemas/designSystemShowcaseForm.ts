import { z } from 'zod'
import {
  createJsonPayloadField,
  createOptionalTextField,
  createPhoneNumberField,
  createSlugField,
  createTcpPortField,
  createVerificationCodeField,
  MAXIMUM_SEARCH_QUERY_LENGTH,
  type ValidationTranslator,
} from './commonValidators'
import {
  NOTIFICATION_CHANNEL_KINDS,
  WORKSPACE_ENVIRONMENT_KINDS,
} from './workspaceForms'

export interface DesignSystemShowcaseFormValues {
  channelIdentifier: string
  recipientPhone: string
  localCapturePort: string
  verificationCode: string
  environmentKind: (typeof WORKSPACE_ENVIRONMENT_KINDS)[number]
  searchFilter: string
  payloadJson: string
  selectedChannel: (typeof NOTIFICATION_CHANNEL_KINDS)[number]
  isCaptureEnabled: boolean
  isAutoScrollEnabled: boolean
}

export const DEFAULT_SHOWCASE_FORM_VALUES: DesignSystemShowcaseFormValues = {
  channelIdentifier: 'smtp-local-channel',
  recipientPhone: '+98 912 000 0000',
  localCapturePort: '1025',
  verificationCode: '482-910',
  environmentKind: 'local',
  searchFilter: '',
  payloadJson: '{"event":"notification.captured","status":"ok"}',
  selectedChannel: 'email',
  isCaptureEnabled: true,
  isAutoScrollEnabled: true,
}

/**
 * Builds the localized Zod schema validating the interactive Component Showcase form.
 */
export function createDesignSystemShowcaseSchema(translate: ValidationTranslator) {
  return z.object({
    channelIdentifier: createSlugField(translate),
    recipientPhone: createPhoneNumberField(translate),
    localCapturePort: createTcpPortField(translate),
    verificationCode: createVerificationCodeField(translate),
    environmentKind: z.enum(WORKSPACE_ENVIRONMENT_KINDS, {
      error: translate('validation.invalidSelection'),
    }),
    searchFilter: createOptionalTextField(translate, MAXIMUM_SEARCH_QUERY_LENGTH),
    payloadJson: createJsonPayloadField(translate),
    selectedChannel: z.enum(NOTIFICATION_CHANNEL_KINDS, {
      error: translate('validation.invalidSelection'),
    }),
    isCaptureEnabled: z.boolean().refine((isEnabled) => isEnabled, {
      error: translate('validation.mustBeEnabled'),
    }),
    isAutoScrollEnabled: z.boolean(),
  })
}

export type ValidatedShowcaseForm = z.output<
  ReturnType<typeof createDesignSystemShowcaseSchema>
>
