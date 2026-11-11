import { z } from 'zod'
import {
  createOptionalSlugField,
  createOptionalTextField,
  createRequiredTextField,
  createSearchQueryField,
  createSlugField,
  createWebhookUrlField,
  MAXIMUM_DESCRIPTION_LENGTH,
  MAXIMUM_NAME_LENGTH,
  MINIMUM_NAME_LENGTH,
  type ValidationTranslator,
} from './commonValidators'

export const WORKSPACE_ENVIRONMENT_KINDS = [
  'local',
  'development',
  'testing',
  'staging',
] as const

export const NOTIFICATION_CHANNEL_KINDS = [
  'email',
  'sms',
  'push',
  'webhook',
] as const

/**
 * Builds the localized Zod schema for creating or editing a Workspace.
 */
export function createWorkspaceFormSchema(translate: ValidationTranslator) {
  return z.object({
    name: createRequiredTextField(translate, {
      minLength: MINIMUM_NAME_LENGTH,
      maxLength: MAXIMUM_NAME_LENGTH,
    }),
    slug: createSlugField(translate),
    description: createOptionalTextField(translate, MAXIMUM_DESCRIPTION_LENGTH),
  })
}

/**
 * Builds the localized Zod schema for creating or editing a Project within a Workspace.
 */
export function createProjectFormSchema(translate: ValidationTranslator) {
  return z.object({
    workspaceId: z
      .string({ error: translate('validation.required') })
      .trim()
      .uuid({ error: translate('validation.invalidUuid') }),
    name: createRequiredTextField(translate, {
      minLength: MINIMUM_NAME_LENGTH,
      maxLength: MAXIMUM_NAME_LENGTH,
    }),
    slug: createSlugField(translate),
    description: createOptionalTextField(translate, MAXIMUM_DESCRIPTION_LENGTH),
  })
}

/**
 * Builds the localized Zod schema for one-step First-Run Onboarding (initial Workspace + Project).
 */
export function createWorkspaceOnboardingSchema(translate: ValidationTranslator) {
  return z.object({
    workspaceName: createRequiredTextField(translate, {
      minLength: MINIMUM_NAME_LENGTH,
      maxLength: MAXIMUM_NAME_LENGTH,
    }),
    workspaceSlug: createSlugField(translate),
    projectName: createRequiredTextField(translate, {
      minLength: MINIMUM_NAME_LENGTH,
      maxLength: MAXIMUM_NAME_LENGTH,
    }),
    projectSlug: createSlugField(translate),
  })
}

/**
 * Builds the localized Zod schema for creating a scoped Notification Channel inside active project/environment.
 */
export function createChannelFormSchema(translate: ValidationTranslator) {
  return z
    .object({
      projectId: z
        .string({ error: translate('validation.required') })
        .trim()
        .uuid({ error: translate('validation.invalidUuid') }),
      environmentKind: z.enum(WORKSPACE_ENVIRONMENT_KINDS, {
        error: translate('validation.invalidSelection'),
      }),
      name: createRequiredTextField(translate, {
        minLength: MINIMUM_NAME_LENGTH,
        maxLength: MAXIMUM_NAME_LENGTH,
      }),
      channelType: z.enum(NOTIFICATION_CHANNEL_KINDS, {
        error: translate('validation.invalidSelection'),
      }),
      credentialIdentifier: createSlugField(translate),
      targetEndpointUrl: z.string().trim().optional().default(''),
      isEnabled: z.boolean(),
    })
    .superRefine((formValues, refinementContext) => {
      if (formValues.channelType === 'webhook' && formValues.targetEndpointUrl.length > 0) {
        const urlResult = createWebhookUrlField(translate).safeParse(formValues.targetEndpointUrl)
        if (!urlResult.success) {
          for (const issue of urlResult.error.issues) {
            refinementContext.addIssue({
              code: 'custom',
              path: ['targetEndpointUrl'],
              message: issue.message,
            })
          }
        }
      }
    })
}

/**
 * Builds the localized Zod schema for the scoped Add Channel modal (optional custom identifier).
 */
export function createScopedChannelModalSchema(translate: ValidationTranslator) {
  return z.object({
    name: createRequiredTextField(translate, {
      minLength: MINIMUM_NAME_LENGTH,
      maxLength: MAXIMUM_NAME_LENGTH,
    }),
    channelType: z.enum(NOTIFICATION_CHANNEL_KINDS, {
      error: translate('validation.invalidSelection'),
    }),
    credentialIdentifier: createOptionalSlugField(translate),
    isEnabled: z.boolean(),
  })
}

export const INBOX_STATUS_FILTER_OPTIONS = [
  'all',
  'captured',
  'read',
  'archived',
] as const

export const INBOX_CHANNEL_FILTER_OPTIONS = [
  'all',
  ...NOTIFICATION_CHANNEL_KINDS,
] as const

/**
 * Builds the localized Zod schema for Inbox channel, status, and keyword filters.
 */
export function createInboxFilterFormSchema(translate: ValidationTranslator) {
  return z.object({
    channelType: z.enum(INBOX_CHANNEL_FILTER_OPTIONS, {
      error: translate('validation.invalidSelection'),
    }),
    status: z.enum(INBOX_STATUS_FILTER_OPTIONS, {
      error: translate('validation.invalidSelection'),
    }),
    searchQuery: createSearchQueryField(translate),
  })
}

/**
 * Builds the localized Zod schema for workspace switcher search filtering.
 */
export function createWorkspaceSearchSchema(translate: ValidationTranslator) {
  return z.object({
    searchQuery: createSearchQueryField(translate),
  })
}

export type ValidatedWorkspaceForm = z.output<ReturnType<typeof createWorkspaceFormSchema>>
export type ValidatedProjectForm = z.output<ReturnType<typeof createProjectFormSchema>>
export type ValidatedOnboardingForm = z.output<ReturnType<typeof createWorkspaceOnboardingSchema>>
export type ValidatedChannelForm = z.output<ReturnType<typeof createChannelFormSchema>>
export type ValidatedScopedChannelModalForm = z.output<
  ReturnType<typeof createScopedChannelModalSchema>
>
export type ValidatedInboxFilterForm = z.output<ReturnType<typeof createInboxFilterFormSchema>>
export type ValidatedWorkspaceSearchForm = z.output<
  ReturnType<typeof createWorkspaceSearchSchema>
>
