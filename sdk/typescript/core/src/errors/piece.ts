import { isProvaError, ProvaError, type ProvaErrorOptions } from './base.ts'

export class InvalidPieceCIDError extends ProvaError {
  override name: 'InvalidPieceCIDError' = 'InvalidPieceCIDError'

  constructor(input: unknown, options?: ProvaErrorOptions) {
    let msg = 'Invalid piece CID'
    if (typeof input === 'object' && input != null && 'toString' in input && typeof input.toString === 'function') {
      msg = `Invalid piece CID: ${input.toString()}`
    } else if (typeof input === 'string') {
      msg = `Invalid piece CID: ${input}`
    }
    super(msg, options)
  }

  static override is(value: unknown): value is InvalidPieceCIDError {
    return isProvaError(value) && value.name === 'InvalidPieceCIDError'
  }
}
