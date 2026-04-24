import { decodePDPError } from '../utils/decode-pdp-errors.ts'
import { isProvaError, ProvaError } from './base.ts'

export class PullError extends ProvaError {
  override name: 'PullError' = 'PullError'

  constructor(error: string) {
    const decodedError = decodePDPError(error)
    super(`Failed to pull pieces from storage provider.`, {
      details: decodedError,
    })
  }

  static override is(value: unknown): value is PullError {
    return isProvaError(value) && value.name === 'PullError'
  }
}
