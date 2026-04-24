import { isProvaError, ProvaError } from './base.ts'

export class DataSetNotFoundError extends ProvaError {
  override name: 'DataSetNotFoundError' = 'DataSetNotFoundError'
  constructor(dataSetId: bigint) {
    super(`Data set ${dataSetId} not found.`)
  }

  static override is(value: unknown): value is DataSetNotFoundError {
    return isProvaError(value) && value.name === 'DataSetNotFoundError'
  }
}

export class AtLeastOnePieceRequiredError extends ProvaError {
  override name: 'AtLeastOnePieceRequiredError' = 'AtLeastOnePieceRequiredError'
  constructor() {
    super('At least one piece must be provided')
  }

  static override is(value: unknown): value is AtLeastOnePieceRequiredError {
    return isProvaError(value) && value.name === 'AtLeastOnePieceRequiredError'
  }
}
