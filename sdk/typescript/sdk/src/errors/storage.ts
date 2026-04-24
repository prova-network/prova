import { isProvaError, ProvaError, type ProvaErrorOptions } from '@prova-network/core/errors'

export interface StorageErrorOptions extends ProvaErrorOptions {
  providerId?: bigint
  endpoint?: string
}

/**
 * Primary store failed - no data stored anywhere.
 * Thrown when the initial upload to the primary provider fails.
 */
export class StoreError extends ProvaError {
  override name: 'StoreError' = 'StoreError'
  providerId?: string
  endpoint?: string

  /**
   * Create a new StoreError
   *
   * @param message - The error message
   * @param options - {@link StorageErrorOptions}
   */
  constructor(message: string, options?: StorageErrorOptions) {
    super(message, options)
    this.providerId = options?.providerId?.toString()
    this.endpoint = options?.endpoint
  }

  override toJSON() {
    return {
      ...super.toJSON(),
      providerId: this.providerId,
      endpoint: this.endpoint,
    }
  }

  static override is(value: unknown): value is StoreError {
    return isProvaError(value) && value.name === 'StoreError'
  }
}

/**
 * All commits failed - data stored but not on-chain.
 * Thrown when on-chain commit fails on every provider after successful store.
 */
export class CommitError extends ProvaError {
  override name: 'CommitError' = 'CommitError'
  providerId?: string
  endpoint?: string

  /**
   * Create a new CommitError
   *
   * @param message - The error message
   * @param options - {@link StorageErrorOptions}
   */
  constructor(message: string, options?: StorageErrorOptions) {
    super(message, options)
    this.providerId = options?.providerId?.toString()
    this.endpoint = options?.endpoint
  }

  override toJSON() {
    return {
      ...super.toJSON(),
      providerId: this.providerId,
      endpoint: this.endpoint,
    }
  }

  static override is(value: unknown): value is CommitError {
    return isProvaError(value) && value.name === 'CommitError'
  }
}
