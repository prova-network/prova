import * as z from 'zod'

const symbol = Symbol.for('prova-error')

export interface ProvaErrorOptions extends ErrorOptions {
  cause?: Error
  details?: string
}

/**
 * Check if a value is a ProvaError
 *
 */
export function isProvaError(value: unknown): value is ProvaError {
  return value instanceof Error && symbol in value
}

export class ProvaError extends Error {
  [symbol]: boolean = true

  override name = 'ProvaError'
  override cause?: Error
  details?: string
  shortMessage: string

  constructor(message: string, options?: ProvaErrorOptions) {
    const details = options?.details
      ? options.details
      : options?.cause instanceof Error
        ? options.cause.message
        : undefined

    const msg = [
      message || 'An error occurred.',
      ...(details ? [''] : []),
      ...(details ? [`Details: ${details}`] : []),
    ].join('\n')
    super(msg, options)

    this.cause = options?.cause ?? undefined
    this.details = details ?? undefined
    this.shortMessage = message
  }

  toJSON(): Record<string, unknown> {
    return {
      name: this.name,
      message: this.message,
      ...(this.details ? { details: this.details } : {}),
      ...(this.cause ? { cause: serializeErrorCause(this.cause) } : {}),
    }
  }

  static is(value: unknown): value is ProvaError {
    return isProvaError(value) && value.name === 'ProvaError'
  }
}

function serializeErrorCause(error: Error, depth: number = 0): Record<string, unknown> {
  if (depth > 4) {
    return { name: error.name, message: error.message }
  }
  return {
    name: error.name,
    message: error.message,
    ...(error.cause instanceof Error ? { cause: serializeErrorCause(error.cause, depth + 1) } : {}),
  }
}

/**
 * Validation error thrown when a value does not match the expected Zod schema.
 */
export class ZodValidationError extends ProvaError {
  override name: 'ZodValidationError' = 'ZodValidationError'

  constructor(zodError: z.ZodError, message: string = 'Validation failed.') {
    super(message, {
      cause: zodError,
      details: z.prettifyError(zodError),
    })
  }

  static override is(value: unknown): value is ZodValidationError {
    return isProvaError(value) && value.name === 'ZodValidationError'
  }
}

export class ValidationError extends ProvaError {
  override name: 'ValidationError' = 'ValidationError'

  static override is(value: unknown): value is ValidationError {
    return isProvaError(value) && value.name === 'ValidationError'
  }
}
