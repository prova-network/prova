import { isProvaError, ProvaError } from './base.ts'

export class InsufficientBalanceError extends ProvaError {
  override name: 'InsufficientBalanceError' = 'InsufficientBalanceError'

  constructor(balance: bigint, required: bigint) {
    super(`Insufficient balance.`, {
      details: `Balance: ${balance}, Required: ${required}`,
    })
  }

  static override is(value: unknown): value is InsufficientBalanceError {
    return isProvaError(value) && value.name === 'InsufficientBalanceError'
  }
}
export class InsufficientAllowanceError extends ProvaError {
  override name: 'InsufficientAllowanceError' = 'InsufficientAllowanceError'

  constructor(allowance: bigint, required: bigint) {
    super(`Insufficient allowance.`, {
      details: `Allowance: ${allowance}, Required: ${required}`,
    })
  }

  static override is(value: unknown): value is InsufficientAllowanceError {
    return isProvaError(value) && value.name === 'InsufficientAllowanceError'
  }
}

export class DepositAmountError extends ProvaError {
  override name: 'DepositAmountError' = 'DepositAmountError'

  constructor(amount: bigint) {
    super(`Deposit amount must be greater than 0.`, {
      details: `Amount: ${amount}`,
    })
  }

  static override is(value: unknown): value is DepositAmountError {
    return isProvaError(value) && value.name === 'DepositAmountError'
  }
}

export class WithdrawAmountError extends ProvaError {
  override name: 'WithdrawAmountError' = 'WithdrawAmountError'

  constructor(amount: bigint) {
    super(`Withdraw amount must be greater than 0.`, {
      details: `Amount: ${amount}`,
    })
  }

  static override is(value: unknown): value is WithdrawAmountError {
    return isProvaError(value) && value.name === 'WithdrawAmountError'
  }
}

export class InsufficientAvailableFundsError extends ProvaError {
  override name: 'InsufficientAvailableFundsError' = 'InsufficientAvailableFundsError'

  constructor(availableFunds: bigint, required: bigint) {
    super(`Insufficient available funds.`, {
      details: `Available funds: ${availableFunds}, Required: ${required}`,
    })
  }

  static override is(value: unknown): value is InsufficientAvailableFundsError {
    return isProvaError(value) && value.name === 'InsufficientAvailableFundsError'
  }
}
