// SPDX-License-Identifier: MIT
// Copyright (c) 2026 Prova Network contributors.
//
// Local wallet for the Prova Desktop app. Generates or loads a BIP-39
// seed, derives an Ethereum address on the Base network, and stores
// the mnemonic in the OS keychain (libsecret/Keychain/CredentialManager)
// via `keytar`. Falls back to electron-store (AES-encrypted) if keytar
// fails (rare, usually CI/headless Linux).
//
// DESIGN NOTES
//
//   - Single account per install (index 0). We don't expose HD derivation
//     to the UI because it adds complexity without solving a real user
//     need at this tier. A power user wanting a custom key can import
//     their own mnemonic via the settings panel.
//
//   - The wallet never holds real assets beyond a small gas float. Its
//     job is signing: deal proposals (for client mode) or staking /
//     proof-submission transactions (for prover mode). For meaningful
//     PROVA stake, users should use a hardware wallet paired via
//     WalletConnect — that path is tracked separately.
//
//   - Passphrase: we derive a deterministic passphrase for the provad
//     keystore from the mnemonic, so provad can decrypt its on-disk
//     keystore without the desktop app also having to hold the raw
//     private key. The provad keystore is a standard geth-format
//     encrypted JSON.

'use strict'

const electronLog = require('electron-log')
const keytar = require('keytar')
const Store = require('electron-store')
const { ethers } = require('ethers')
const { randomBytes, createHash } = require('node:crypto')

const log = electronLog.scope('wallet')

// Keytar identifiers. `service` is the app-wide namespace; `account` is
// per-key role. If a user reinstalls, these keys persist in the OS keychain
// and the app will reuse them unless they explicitly reset.
const SERVICE = 'network.prova.desktop'
const ACCOUNT_MNEMONIC = 'mnemonic'

// Fallback store when keytar is unavailable (e.g., headless Linux CI).
// electron-store auto-encrypts when `encryptionKey` is provided.
const fallbackStore = new Store({
  name: 'wallet-fallback',
  // Not a real secret — just raises the bar above "plain JSON on disk".
  // If keytar works, this store is never touched.
  encryptionKey: 'prova-desktop-wallet-v1'
})

/** @typedef {import('./typings').Context} Context */

/** @type {ethers.HDNodeWallet | null} */
let walletInstance = null

/** @type {Context | null} */
let ctx = null

/**
 * Initialise the wallet. Loads an existing mnemonic from the keychain, or
 * generates a new 12-word BIP-39 mnemonic on first run. Idempotent.
 *
 * @param {Context} _ctx
 */
async function setup (_ctx) {
  ctx = _ctx

  /** @type {string | null} */
  let mnemonic = await loadMnemonic()
  let isNew = false

  if (!mnemonic) {
    const entropy = randomBytes(16) // 128 bits → 12-word mnemonic
    mnemonic = ethers.Mnemonic.fromEntropy(entropy).phrase
    await saveMnemonic(mnemonic)
    isNew = true
    log.info('Created new seed phrase')
  } else {
    log.info('Using existing seed phrase')
  }

  walletInstance = ethers.HDNodeWallet.fromPhrase(mnemonic)
  log.info(`Wallet address: ${walletInstance.address}`)

  if (ctx && typeof ctx.setWalletAddress === 'function') {
    ctx.setWalletAddress(walletInstance.address)
  }

  return { isNew, address: walletInstance.address }
}

/**
 * Replace the current mnemonic with a user-supplied one (for import flows).
 * Validates the phrase before persisting. Throws on invalid input.
 *
 * @param {string} phrase
 */
async function importMnemonic (phrase) {
  const trimmed = phrase.trim().toLowerCase()
  // Throws if invalid; we let the exception propagate to the UI layer.
  const mnemonic = ethers.Mnemonic.fromPhrase(trimmed)
  walletInstance = ethers.HDNodeWallet.fromMnemonic(mnemonic)
  await saveMnemonic(trimmed)
  log.info('Imported seed phrase; address %s', walletInstance.address)
  if (ctx && typeof ctx.setWalletAddress === 'function') {
    ctx.setWalletAddress(walletInstance.address)
  }
  return walletInstance.address
}

/**
 * Returns the wallet's Ethereum address (checksummed).
 * Throws if `setup()` has not been called.
 *
 * @returns {Promise<string>}
 */
async function getAddress () {
  return getWallet().address
}

/**
 * Sign a message with the wallet's private key. Used for challenge-response
 * auth against prover registry, deal proposal signing, etc. Uses the
 * EIP-191 personal-sign format.
 *
 * @param {string} message
 * @returns {Promise<string>} 0x-prefixed signature
 */
async function signMessage (message) {
  return getWallet().signMessage(message)
}

/**
 * Derive a deterministic passphrase for the provad keystore.
 * The value is a SHA-256 of the mnemonic + a fixed purpose string; this
 * means: (a) the desktop app can always regenerate it from the stored
 * mnemonic without a second secret, (b) it's unique per install, (c) it's
 * not the raw mnemonic so if provad's keystore file leaks, the mnemonic
 * is still safe.
 *
 * @returns {Promise<string>} 64-char hex string
 */
async function getKeystorePassphrase () {
  getWallet() // ensure wallet is initialized before reading the keystore
  const mnemonic = await loadMnemonic()
  if (!mnemonic) throw new Error('wallet: no mnemonic available for passphrase derivation')
  return createHash('sha256')
    .update(`prova-provad-keystore-v1:${mnemonic}`)
    .digest('hex')
}

/**
 * Export the private key as a hex string. Gated behind an explicit UI
 * confirmation in the renderer; main-process exposure is minimal.
 *
 * @returns {Promise<string>} 0x-prefixed 32-byte hex
 */
async function exportPrivateKey () {
  return getWallet().privateKey
}

/**
 * Export the raw mnemonic (for user backup flows only; also gated by UI).
 *
 * @returns {Promise<string>}
 */
async function exportMnemonic () {
  const m = await loadMnemonic()
  if (!m) throw new Error('wallet: no mnemonic to export')
  return m
}

// ─── internals ───────────────────────────────────────────────────────

/**
 * Returns the initialized wallet instance, throwing if `setup()` has not
 * yet completed. Used in place of an `assertReady()` predicate so the
 * caller gets a non-null value back through the type system.
 *
 * @returns {ethers.HDNodeWallet}
 */
function getWallet () {
  if (!walletInstance) {
    throw new Error('wallet: not initialized; call setup() first')
  }
  return walletInstance
}

function assertReady () {
  if (!walletInstance) {
    throw new Error('wallet: setup() not called')
  }
}

/**
 * @returns {Promise<string | null>}
 */
async function loadMnemonic () {
  try {
    const m = await keytar.getPassword(SERVICE, ACCOUNT_MNEMONIC)
    if (m) return m
  } catch (/** @type {unknown} */ err) {
    const msg = err instanceof Error ? err.message : String(err)
    log.warn('keytar load failed, trying fallback store:', msg)
  }
  const stored = fallbackStore.get('mnemonic')
  return typeof stored === 'string' && stored.length > 0 ? stored : null
}

/**
 * @param {string} mnemonic
 */
async function saveMnemonic (mnemonic) {
  try {
    await keytar.setPassword(SERVICE, ACCOUNT_MNEMONIC, mnemonic)
    // Clear fallback copy if it was ever written
    fallbackStore.delete('mnemonic')
    return
  } catch (/** @type {unknown} */ err) {
    const msg = err instanceof Error ? err.message : String(err)
    log.warn('keytar save failed, using fallback store:', msg)
    fallbackStore.set('mnemonic', mnemonic)
  }
}

module.exports = {
  setup,
  importMnemonic,
  getAddress,
  signMessage,
  getKeystorePassphrase,
  exportPrivateKey,
  exportMnemonic,
  // Back-compat shims for anything in the codebase still calling the old API.
  getDestinationWalletAddress: getAddress
}
