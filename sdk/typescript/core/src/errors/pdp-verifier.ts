import { isProvaError, ProvaError } from './base.ts'

export class LimitMustBeGreaterThanZeroError extends ProvaError {
  override name: 'LimitMustBeGreaterThanZeroError' = 'LimitMustBeGreaterThanZeroError'
  constructor() {
    super('Limit must be greater than zero')
  }

  static override is(value: unknown): value is LimitMustBeGreaterThanZeroError {
    return isProvaError(value) && value.name === 'LimitMustBeGreaterThanZeroError'
  }
}
