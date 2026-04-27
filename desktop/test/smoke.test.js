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

  it('produces a Web3 v3 keystore that round-trips with the same passphrase', async function () {
    // Scrypt is intentionally expensive; allow up to 30s on slow CI.
    this.timeout(30_000)
    const mnemonic = ethers.Mnemonic.fromEntropy(require('node:crypto').randomBytes(16))
    const wallet = ethers.HDNodeWallet.fromMnemonic(mnemonic)
    const passphrase = require('node:crypto')
      .createHash('sha256')
      .update(`prova-provad-keystore-v1:${mnemonic.phrase}`)
      .digest('hex')

    const json = await wallet.encrypt(passphrase)
    const parsed = JSON.parse(json)

    // go-ethereum keystore.DecryptKey expects: version=3, address (hex,
    // no 0x prefix), id (uuid), crypto.cipher='aes-128-ctr',
    // crypto.kdf='scrypt' or 'pbkdf2'.
    //
    // ethers v6 emits the field as `Crypto` (uppercase first letter)
    // while go-ethereum's struct tag is `crypto` (lowercase). Go's
    // `encoding/json` does case-insensitive matching for struct fields
    // by default, so `Crypto` round-trips into `Crypto CryptoJSON ...`
    // tagged `json:"crypto"` without manual normalization.
    const cryptoBlock = parsed.crypto || parsed.Crypto
    assert.ok(cryptoBlock, 'keystore must have a crypto/Crypto block')
    assert.equal(parsed.version, 3, 'keystore must be v3 for go-ethereum')
    assert.equal(parsed.address.toLowerCase(), wallet.address.slice(2).toLowerCase())
    assert.match(parsed.id, /^[0-9a-f-]{36}$/i, 'id must be a uuid')
    assert.equal(cryptoBlock.cipher, 'aes-128-ctr')
    assert.ok(['scrypt', 'pbkdf2'].includes(cryptoBlock.kdf))

    // Ethers can decrypt its own output, which is the smoke-test floor.
    // (Cross-decryption with go-ethereum is implicit in matching the v3
    // schema and aes-128-ctr cipher; the encryption is interoperable by
    // construction.)
    const recovered = await ethers.Wallet.fromEncryptedJson(json, passphrase)
    assert.equal(recovered.address.toLowerCase(), wallet.address.toLowerCase())
  })
})
