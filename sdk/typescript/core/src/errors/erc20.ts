import { isProvaError, ProvaError } from './base.ts'

export class AllowanceAmountError extends ProvaError {
  override name: 'AllowanceAmountError' = 'AllowanceAmountError'

  constructor(amount: bigint) {
    super(`Allowance amount must be positive.`, {
      details: `Amount: ${amount}`,
    })
  }

  static override is(value: unknown): value is AllowanceAmountError {
    return isProvaError(value) && value.name === 'AllowanceAmountError'
  }
}
