import { z } from 'zod'
import {
  createBoundedIntegerField,
  createIpv4AddressField,
  createTcpPortField,
  MAXIMUM_PAYLOAD_MEGABYTES,
  MAXIMUM_RETENTION_DAYS,
  MINIMUM_PAYLOAD_MEGABYTES,
  MINIMUM_RETENTION_DAYS,
  type ValidationTranslator,
} from './commonValidators'

export interface ServerSettingsFormValues {
  bindAddress: string
  httpPort: string
  smtpPort: string
  maximumHttpPayloadMegabytes: string
  maximumSmtpMessageMegabytes: string
  retentionDays: string
}

export const DEFAULT_SERVER_SETTINGS_FORM_VALUES: ServerSettingsFormValues = {
  bindAddress: '127.0.0.1',
  httpPort: '4080',
  smtpPort: '1025',
  maximumHttpPayloadMegabytes: '2',
  maximumSmtpMessageMegabytes: '10',
  retentionDays: '30',
}

/**
 * Builds the localized Zod schema for local HTTP and SMTP capture server configuration.
 */
export function createServerSettingsFormSchema(translate: ValidationTranslator) {
  return z
    .object({
      bindAddress: createIpv4AddressField(translate),
      httpPort: createTcpPortField(translate),
      smtpPort: createTcpPortField(translate),
      maximumHttpPayloadMegabytes: createBoundedIntegerField(translate, {
        min: MINIMUM_PAYLOAD_MEGABYTES,
        max: MAXIMUM_PAYLOAD_MEGABYTES,
      }),
      maximumSmtpMessageMegabytes: createBoundedIntegerField(translate, {
        min: MINIMUM_PAYLOAD_MEGABYTES,
        max: MAXIMUM_PAYLOAD_MEGABYTES,
      }),
      retentionDays: createBoundedIntegerField(translate, {
        min: MINIMUM_RETENTION_DAYS,
        max: MAXIMUM_RETENTION_DAYS,
      }),
    })
    .superRefine((formValues, refinementContext) => {
      if (formValues.httpPort === formValues.smtpPort) {
        refinementContext.addIssue({
          code: 'custom',
          path: ['smtpPort'],
          message: translate('validation.portConflict'),
        })
      }
    })
}

export type ValidatedServerSettings = z.output<
  ReturnType<typeof createServerSettingsFormSchema>
>
