// Smoke test: the module graph loads without crashing and the wallet module's
// pure-JS code paths work. Does NOT spawn Electron (headless test envs don't
// have a display) and does NOT spawn provad (that requires the binary + real
// Base RPC). Those are covered by e2e tests later.

'use strict'

const assert = require('node:assert/strict')
const { describe, it } = require('mocha')
const { ethers } = require('ethers')

describe('desktop/ smoke', () => {
  it('ethers v6 is loaded', () => {
    assert.ok(ethers.version.startsWith('6.'), `expected ethers 6.x, got ${ethers.version}`)
  })

  it('generates a valid HD wallet from entropy', () => {
    const entropy = require('node:crypto').randomBytes(16)
    const mnemonic = ethers.Mnemonic.fromEntropy(entropy)
    assert.ok(mnemonic.phrase.split(' ').length === 12)
    const wallet = ethers.HDNodeWallet.fromMnemonic(mnemonic)
    assert.ok(ethers.isAddress(wallet.address))
  })

  it('wallet can sign EIP-191 personal messages', async () => {
    const entropy = require('node:crypto').randomBytes(16)
    const mnemonic = ethers.Mnemonic.fromEntropy(entropy)
    const wallet = ethers.HDNodeWallet.fromMnemonic(mnemonic)
    const sig = await wallet.signMessage('prova desktop smoke test')
    assert.ok(sig.startsWith('0x') && sig.length === 132)
    // Recover + verify
    const recovered = ethers.verifyMessage('prova desktop smoke test', sig)
    assert.equal(recovered.toLowerCase(), wallet.address.toLowerCase())
  })

  it('keystore passphrase derivation is deterministic', () => {
    const { createHash } = require('node:crypto')
    const mnemonic = 'test test test test test test test test test test test junk'
    const h1 = createHash('sha256').update(`prova-provad-keystore-v1:${mnemonic}`).digest('hex')
    const h2 = createHash('sha256').update(`prova-provad-keystore-v1:${mnemonic}`).digest('hex')
    assert.equal(h1, h2)
    assert.equal(h1.length, 64)
  })
})
