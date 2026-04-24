import { calibration, devnet, mainnet } from '../chains.ts'
import { isProvaError, ProvaError } from './base.ts'

export class UnsupportedChainError extends ProvaError {
  override name: 'UnsupportedChainError' = 'UnsupportedChainError'

  constructor(chainId: number) {
    super(
      `Unsupported chain: ${chainId} (only Filecoin mainnet (${mainnet.id}), calibration (${calibration.id}), and devnet (${devnet.id}) are supported). Import chains from @prova-network/core/chains to get the correct chain.`
    )
  }

  static override is(value: unknown): value is UnsupportedChainError {
    return isProvaError(value) && value.name === 'UnsupportedChainError'
  }
}
