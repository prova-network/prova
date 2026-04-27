// SPDX-License-Identifier: MIT
// Copyright (c) 2026 Prova Network contributors.
//
// On-chain bridge for the desktop UI. Wraps ethers v6 around the
// active-network preset's RPC + the wallet's signer so the renderer
// can read balances/stake and submit stake/unstake/withdraw txs
// without holding any private keys itself.
//
// Every method here returns plain JSON (numbers as decimal strings to
// avoid BigInt serialization across IPC) so the renderer can consume
// them directly through the preload bridge.

'use strict'

const electronLog = require('electron-log')
const { ethers } = require('ethers')

const wallet = require('./wallet')
const provaConfig = require('./prova-config')

const log = electronLog.scope('chain')

// ─── ABIs we use ────────────────────────────────────────────────────
//
// These are hand-trimmed to the surface the desktop calls. Full ABIs
// live in prover/pkg/contracts/* (auto-generated from the Solidity).

const ERC20_ABI = [
  'function balanceOf(address owner) view returns (uint256)',
  'function decimals() view returns (uint8)',
  'function approve(address spender, uint256 amount) returns (bool)',
  'function allowance(address owner, address spender) view returns (uint256)'
]

const PROVER_STAKING_ABI = [
  'function getStake(address prover) view returns (tuple(uint256 staked, uint256 unbonding, uint256 unbondingEndsAt, uint256 committedBytes))',
  'function UNBONDING_PERIOD() view returns (uint256)',
  'function minStakePerTiB() view returns (uint256)',
  'function minStakeUsdPerTiB() view returns (uint256)',
  'function stake(uint256 amount) external',
  'function requestUnstake(uint256 amount) external',
  'function withdraw() external',
  'event Staked(address indexed prover, uint256 amount, uint256 newTotal)',
  'event UnstakeRequested(address indexed prover, uint256 amount, uint256 endsAt)',
  'event Withdrawn(address indexed prover, uint256 amount)'
]

// Mirrors the Prover struct in src/ProverRegistry.sol exactly.
const PROVER_REGISTRY_ABI = [
  'function getProver(address prover) view returns (tuple(address owner, string endpoint, uint64 features, uint128 pricePerGibDay, uint128 pricePerByteServed, uint64 registeredAt, uint64 updatedAt, bool active, bytes32 ensNode, string metadata))',
  'function register(string endpoint, uint64 features, uint128 pricePerGibDay, uint128 pricePerByteServed, string metadata) external',
  'function FEATURE_PDP() view returns (uint64)',
  'function FEATURE_HTTPS_SERVING() view returns (uint64)'
]

// ─── Lazy-built provider / signer ───────────────────────────────────

/** @type {{key: string, provider: ethers.JsonRpcProvider, signer: ethers.Wallet} | null} */
let cached = null

async function getProviderAndSigner () {
  const network = provaConfig.getNetworkConfig()
  if (cached && cached.key === network.key) {
    return cached
  }
  const provider = new ethers.JsonRpcProvider(network.rpcUrl)
  // The desktop wallet's HD account 0 private key is the signing key.
  const privHex = await wallet.exportPrivateKey()
  const signer = new ethers.Wallet(privHex, provider)
  cached = { key: network.key, provider, signer }
  log.info(`chain bridge online: ${network.label} (${network.rpcUrl})`)
  return cached
}

function resetProvider () {
  cached = null
}

/**
 * Snapshot the prover's wallet/stake state for the dashboard.
 *
 * Returns null if the active network preset has placeholder contract
 * addresses (e.g. base-sepolia / base-mainnet before deploy). Returns
 * a partial snapshot if any individual read fails so the UI can still
 * render the parts that succeeded.
 *
 * @returns {Promise<null | {
 *   address: string,
 *   ethWei: string,
 *   provaWei: string,
 *   provaDecimals: number,
 *   stakedWei: string,
 *   unbondingWei: string,
 *   unbondingEndsAt: number,
 *   committedBytes: string,
 *   minStakePerTiBWei: string,
 *   unbondingPeriodSeconds: number,
 *   isRegistered: boolean,
 *   registeredEndpoint: string
 * }>}
 */
async function getStakeSnapshot () {
  const network = provaConfig.getNetworkConfig()
  const c = network.contracts
  const hasContracts =
    c.proverStaking && c.proverStaking.length > 0 &&
    c.provaToken && c.provaToken.length > 0
  if (!hasContracts) return null

  const { provider, signer } = await getProviderAndSigner()
  const me = await signer.getAddress()

  const token = new ethers.Contract(c.provaToken, ERC20_ABI, provider)
  const staking = new ethers.Contract(c.proverStaking, PROVER_STAKING_ABI, provider)
  const registry = c.proverRegistry && c.proverRegistry.length > 0
    ? new ethers.Contract(c.proverRegistry, PROVER_REGISTRY_ABI, provider)
    : null

  // Run reads in parallel; failures fall back to zero/empty so the UI
  // can render a degraded but non-empty snapshot.
  const [
    ethBal, provaBal, provaDec, stakeInfo, unbondingPeriod,
    minPerTiB, prov
  ] = await Promise.all([
    provider.getBalance(me).catch(() => 0n),
    token.balanceOf(me).catch(() => 0n),
    token.decimals().catch(() => 18),
    staking.getStake(me).catch(() => ({ staked: 0n, unbonding: 0n, unbondingEndsAt: 0n, committedBytes: 0n })),
    staking.UNBONDING_PERIOD().catch(() => 0n),
    staking.minStakePerTiB().catch(() => 0n),
    registry ? registry.getProver(me).catch(() => null) : null
  ])

  return {
    address: me,
    ethWei: ethBal.toString(),
    provaWei: provaBal.toString(),
    provaDecimals: Number(provaDec),
    stakedWei: stakeInfo.staked.toString(),
    unbondingWei: stakeInfo.unbonding.toString(),
    unbondingEndsAt: Number(stakeInfo.unbondingEndsAt),
    committedBytes: stakeInfo.committedBytes.toString(),
    minStakePerTiBWei: minPerTiB.toString(),
    unbondingPeriodSeconds: Number(unbondingPeriod),
    // Registry uses a soft-delete `active` flag; treat that as the
    // ground truth for whether this prover has registered.
    isRegistered: !!(prov && prov.active),
    registeredEndpoint: prov ? prov.endpoint : ''
  }
}

/**
 * Stake `amountWei` PROVA. Approves the staking contract first if the
 * current allowance is insufficient, then calls `stake()`.
 *
 * @param {string} amountWei  decimal string of PROVA wei (18 decimals)
 * @returns {Promise<{txHash: string, approveTxHash?: string}>}
 */
async function stake (amountWei) {
  const network = provaConfig.getNetworkConfig()
  const c = network.contracts
  if (!c.proverStaking || !c.provaToken) {
    throw new Error('chain: contracts not configured for the active network')
  }
  const { signer } = await getProviderAndSigner()
  const amount = BigInt(amountWei)
  if (amount <= 0n) throw new Error('amount must be > 0')

  const token = new ethers.Contract(c.provaToken, ERC20_ABI, signer)
  const staking = new ethers.Contract(c.proverStaking, PROVER_STAKING_ABI, signer)

  let approveTxHash
  const allowance = await token.allowance(await signer.getAddress(), c.proverStaking)
  if (allowance < amount) {
    log.info(`approving ${amount} PROVA on ProverStaking (${c.proverStaking})`)
    const tx = await token.approve(c.proverStaking, amount)
    approveTxHash = tx.hash
    await tx.wait()
  }

  log.info(`staking ${amount} PROVA`)
  const stakeTx = await staking.stake(amount)
  await stakeTx.wait()
  return { txHash: stakeTx.hash, approveTxHash }
}

/**
 * Move `amountWei` from staked → unbonding. Withdrawable after
 * UNBONDING_PERIOD elapses.
 *
 * @param {string} amountWei
 * @returns {Promise<{txHash: string}>}
 */
async function requestUnstake (amountWei) {
  const network = provaConfig.getNetworkConfig()
  const c = network.contracts
  if (!c.proverStaking) throw new Error('chain: contracts not configured')
  const { signer } = await getProviderAndSigner()
  const staking = new ethers.Contract(c.proverStaking, PROVER_STAKING_ABI, signer)
  const tx = await staking.requestUnstake(BigInt(amountWei))
  await tx.wait()
  return { txHash: tx.hash }
}

/**
 * Claim the prover's fully-unbonded stake back to their wallet. Reverts
 * on-chain if the unbonding period hasn't elapsed.
 */
async function withdrawUnbonded () {
  const network = provaConfig.getNetworkConfig()
  const c = network.contracts
  if (!c.proverStaking) throw new Error('chain: contracts not configured')
  const { signer } = await getProviderAndSigner()
  const staking = new ethers.Contract(c.proverStaking, PROVER_STAKING_ABI, signer)
  const tx = await staking.withdraw()
  await tx.wait()
  return { txHash: tx.hash }
}

/**
 * Register this prover in the registry. Required before the marketplace
 * will route deals to this address.
 *
 * The on-chain ProverRegistry.register signature is
 *   register(string endpoint, uint64 features, uint128 pricePerGibDay,
 *            uint128 pricePerByteServed, string metadata)
 * `features` must include `FEATURE_PDP = 1`. The desktop ships
 * sensible defaults for endpoint and pricing so first-time provers
 * don't have to think about it; they can update via setPrice / update*
 * later.
 *
 * @param {{ endpoint?: string, pricePerGibDayWei?: string, pricePerByteServedWei?: string, metadata?: string }} opts
 */
async function registerProver (opts = {}) {
  const network = provaConfig.getNetworkConfig()
  const c = network.contracts
  if (!c.proverRegistry) throw new Error('chain: prover registry not configured')
  const { signer } = await getProviderAndSigner()
  const reg = new ethers.Contract(c.proverRegistry, PROVER_REGISTRY_ABI, signer)
  const endpoint = opts.endpoint || 'https://localhost'
  // FEATURE_PDP (1) | FEATURE_HTTPS_SERVING (2) = 3
  const features = 3n
  // Pricing defaults are intentionally conservative; user can update
  // via setPrice once they know what they want to charge.
  //   1e15 wei/GiB-day  ~ 0.001 token/GiB-day
  //   1e9  wei/byte     ~ negligible default for retrieval
  const pricePerGibDay = opts.pricePerGibDayWei ? BigInt(opts.pricePerGibDayWei) : 10n ** 15n
  const pricePerByteServed = opts.pricePerByteServedWei ? BigInt(opts.pricePerByteServedWei) : 10n ** 9n
  const metadata = opts.metadata ?? '{}'
  const tx = await reg.register(
    endpoint,
    features,
    pricePerGibDay,
    pricePerByteServed,
    metadata
  )
  await tx.wait()
  return { txHash: tx.hash }
}

module.exports = {
  getStakeSnapshot,
  stake,
  requestUnstake,
  withdrawUnbonded,
  registerProver,
  resetProvider
}
