// SPDX-License-Identifier: Apache-2.0 OR MIT
// Copyright (c) 2024-2026 Rod Vagg, FilOzone contributors (upstream: @filoz/synapse-sdk).
// Copyright (c) 2026 Prova Network contributors.
//
// Adapted from FilOzone/synapse-sdk packages/synapse-sdk/src/synapse.ts
// (https://github.com/FilOzone/synapse-sdk). Originally under Permissive
// License Stack (Apache-2.0 OR MIT). Attribution preserved per license.
//
// Adaptations for Prova:
//   - Renamed class Synapse -> Prova
//   - Dropped FilBeamService (Filecoin CDN, not used in Prova v1)
//   - Renamed warm-storage internals -> storage/marketplace
//
// Remaining internal modules retain their upstream structure and are
// covered by the Apache-2.0 OR MIT header inherited from this file.

import { asChain, type Chain } from '@prova-network/core/chains'
import type { SessionKeyAccount } from '@prova-network/core/session-key'
import * as SessionKey from '@prova-network/core/session-key'
import {
  type Account,
  type Address,
  type Client,
  createClient,
  http,
  isAddress,
  type PublicActions,
  type PublicRpcSchema,
  publicActions,
  type Transport,
} from 'viem'
import { PaymentsService } from './payments/index.ts'
import { SPRegistryService } from './sp-registry/index.ts'
import { StorageManager } from './storage/manager.ts'
import type { PDPProvider, ProvaFromClientOptions, ProvaOptions } from './types.ts'
import { DEFAULT_CHAIN } from './utils/constants.ts'
import { StorageService } from './storage/index.ts'

/**
 * Class for interacting with Filecoin storage and other on-chain services
 */
export class Prova {
  private readonly _withCDN: boolean
  private readonly _source: string | null
  private readonly _payments: PaymentsService
  private readonly _storageService: StorageService
  private readonly _storageManager: StorageManager
  private readonly _providers: SPRegistryService

  private readonly _client: Client<Transport, Chain, Account, PublicRpcSchema, PublicActions<Transport, Chain>>
  private readonly _sessionClient: Client<Transport, Chain, SessionKeyAccount<'Secp256k1'>> | undefined
  private readonly _chain: Chain

  /**
   * Create a new Prova instance.
   *
   * @param options - Configuration options for Prova
   * @returns A fully initialized Prova instance
   */
  static create(options: ProvaOptions) {
    const client = createClient({
      // todo: change to mainnet chain for GA
      chain: options.chain ?? DEFAULT_CHAIN,
      // todo: add better fallback transport
      transport: options.transport ?? http(),
      account: options.account,
      name: 'Prova Client',
      key: 'prova-client',
    })

    if (client.account.type === 'json-rpc' && client.transport.type !== 'custom') {
      throw new Error('Transport must be a custom transport. See https://viem.sh/docs/clients/transports/custom.')
    }

    if (options.sessionKey != null && !options.sessionKey.hasPermissions(SessionKey.DefaultFwssPermissions)) {
      throw new Error(
        'Session key does not have the required permissions. Please login and sync expirations with the session key first.'
      )
    }

    return new Prova({
      client,
      withCDN: options.withCDN,
      source: options.source,
      sessionClient: options.sessionKey?.client,
    })
  }

  public constructor(options: ProvaFromClientOptions) {
    this._client = options.client.extend(publicActions)
    this._sessionClient = options.sessionClient
    this._chain = asChain(options.client.chain)
    this._withCDN = options.withCDN ?? false
    this._source = options.source ?? null
    this._providers = new SPRegistryService({ client: options.client })
    this._storageService = new StorageService({ client: options.client })
    this._payments = new PaymentsService({ client: options.client })

    // Initialize StorageManager
    this._storageManager = new StorageManager({
      prova: this,
      storageService: this._storageService,
      withCDN: this._withCDN,
      source: this._source,
    })
  }

  get client(): Client<Transport, Chain, Account, PublicRpcSchema, PublicActions<Transport, Chain>> {
    return this._client
  }

  get sessionClient(): Client<Transport, Chain, SessionKeyAccount<'Secp256k1'>> | undefined {
    return this._sessionClient
  }

  get chain(): Chain {
    return this._chain
  }

  /**
   * Gets the payment service instance
   * @returns The payment service
   */
  get payments(): PaymentsService {
    return this._payments
  }

  /**
   * Gets the storage manager instance
   *
   * @returns The storage manager for all storage operations
   */
  get storage(): StorageManager {
    return this._storageManager
  }

  /**
   * Gets the service provider registry instance
   *
   * @returns The service provider registry for interacting with service providers
   */
  get providers(): SPRegistryService {
    return this._providers
  }

  /**
   * Get detailed information about a specific service provider
   * @param providerAddress - The provider's address or provider ID
   * @returns Provider information including URLs and pricing
   */
  async getProviderInfo(providerAddress: Address | bigint): Promise<PDPProvider> {
    try {
      // Validate address format if string provided
      if (typeof providerAddress === 'string') {
        try {
          isAddress(providerAddress) // Will throw if invalid
        } catch {
          throw new Error(`Invalid provider address: ${providerAddress}`)
        }
      }

      let providerInfo: PDPProvider | null
      if (typeof providerAddress === 'string') {
        providerInfo = await this._providers.getProviderByAddress({ address: providerAddress })
      } else {
        providerInfo = await this._providers.getProvider({ providerId: providerAddress })
      }

      // Check if provider was found in registry
      if (providerInfo == null) {
        throw new Error(`Provider ${providerAddress} not found in registry`)
      }

      return providerInfo
    } catch (error) {
      if (error instanceof Error && error.message.includes('Invalid provider address')) {
        throw error
      }
      if (error instanceof Error && error.message.includes('not found')) {
        throw error
      }
      throw new Error(`Failed to get provider info: ${error instanceof Error ? error.message : String(error)}`)
    }
  }
}
