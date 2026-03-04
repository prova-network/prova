// cli_wallet.rs — CLI wallet integration for Prova SDK
//
// Implements SDK-003: CLI wallet integration (import keys, sign offline)
//   - Import keys from hex seed, passphrase, or encrypted keystore JSON
//   - Export keys to encrypted keystore format
//   - Offline transaction signing (no network required)
//   - Transaction serialization for later broadcast
//   - Keystore file management (list, import, export, delete)
//   - Address derivation and display

use prova_chain::types::Address;
use sha2::{Sha256, Digest};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

// ── CLI Wallet Errors ──────────────────────────────────────────

#[derive(Debug, PartialEq)]
pub enum WalletError {
    InvalidHexSeed(String),
    InvalidKeystoreJson(String),
    DecryptionFailed,
    KeyNotFound(String),
    DuplicateKey(String),
    InvalidPassword,
    SerializationError(String),
    SigningError(String),
    InvalidTransaction(String),
}

impl std::fmt::Display for WalletError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::InvalidHexSeed(s) => write!(f, "invalid hex seed: {s}"),
            Self::InvalidKeystoreJson(s) => write!(f, "invalid keystore JSON: {s}"),
            Self::DecryptionFailed => write!(f, "decryption failed"),
            Self::KeyNotFound(a) => write!(f, "key not found: {a}"),
            Self::DuplicateKey(a) => write!(f, "duplicate key: {a}"),
            Self::InvalidPassword => write!(f, "invalid password"),
            Self::SerializationError(s) => write!(f, "serialization error: {s}"),
            Self::SigningError(s) => write!(f, "signing error: {s}"),
            Self::InvalidTransaction(s) => write!(f, "invalid transaction: {s}"),
        }
    }
}

// ── Key types (simplified Ed25519-like, matching wallet.rs patterns) ────

#[derive(Clone)]
pub struct SecretKey(pub [u8; 32]);

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct PublicKey(pub [u8; 32]);

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Signature(pub [u8; 64]);

impl SecretKey {
    pub fn from_hex(hex: &str) -> Result<Self, WalletError> {
        let hex = hex.strip_prefix("0x").unwrap_or(hex);
        if hex.len() != 64 {
            return Err(WalletError::InvalidHexSeed(
                format!("expected 64 hex chars, got {}", hex.len()),
            ));
        }
        let mut bytes = [0u8; 32];
        for i in 0..32 {
            bytes[i] = u8::from_str_radix(&hex[i * 2..i * 2 + 2], 16)
                .map_err(|e| WalletError::InvalidHexSeed(e.to_string()))?;
        }
        Ok(Self(bytes))
    }

    pub fn from_passphrase(passphrase: &str) -> Self {
        let hash = Sha256::digest(passphrase.as_bytes());
        let mut bytes = [0u8; 32];
        bytes.copy_from_slice(&hash);
        Self(bytes)
    }

    pub fn public_key(&self) -> PublicKey {
        // SHA-256 one-way derivation (matches simplified scheme in wallet.rs)
        let mut hasher = sha2::Sha512::new();
        hasher.update(&self.0);
        let result = hasher.finalize();
        let mut pk = [0u8; 32];
        pk.copy_from_slice(&result[..32]);
        pk[0] &= 248;
        pk[31] &= 127;
        pk[31] |= 64;
        PublicKey(pk)
    }

    pub fn sign(&self, message: &[u8]) -> Signature {
        use sha2::Sha512;
        let mut hasher = Sha512::new();
        hasher.update(&self.0);
        let expanded = hasher.finalize();

        let mut hasher_r = Sha512::new();
        hasher_r.update(&expanded[32..64]);
        hasher_r.update(message);
        let r = hasher_r.finalize();

        let pk = self.public_key();
        let mut hasher_s = Sha512::new();
        hasher_s.update(&pk.0);
        hasher_s.update(message);
        hasher_s.update(&r[..32]);
        let s = hasher_s.finalize();

        let mut sig = [0u8; 64];
        sig[..32].copy_from_slice(&r[..32]);
        sig[32..].copy_from_slice(&s[..32]);
        Signature(sig)
    }

    pub fn to_hex(&self) -> String {
        self.0.iter().map(|b| format!("{b:02x}")).collect()
    }
}

impl PublicKey {
    pub fn address(&self) -> Address {
        let hash = Sha256::digest(&self.0);
        let mut addr = [0u8; 20];
        addr.copy_from_slice(&hash[12..32]);
        Address(addr)
    }

    pub fn verify(&self, message: &[u8], signature: &Signature) -> bool {
        use sha2::Sha512;
        let mut hasher = Sha512::new();
        hasher.update(&self.0);
        hasher.update(message);
        hasher.update(&signature.0[..32]);
        let s = hasher.finalize();
        s[..32] == signature.0[32..64]
    }

    pub fn to_hex(&self) -> String {
        self.0.iter().map(|b| format!("{b:02x}")).collect()
    }
}

impl Signature {
    pub fn to_hex(&self) -> String {
        self.0.iter().map(|b| format!("{b:02x}")).collect()
    }

    pub fn from_hex(hex: &str) -> Result<Self, WalletError> {
        let hex = hex.strip_prefix("0x").unwrap_or(hex);
        if hex.len() != 128 {
            return Err(WalletError::SigningError("signature must be 128 hex chars".into()));
        }
        let mut bytes = [0u8; 64];
        for i in 0..64 {
            bytes[i] = u8::from_str_radix(&hex[i * 2..i * 2 + 2], 16)
                .map_err(|e| WalletError::SigningError(e.to_string()))?;
        }
        Ok(Self(bytes))
    }
}

// ── Encrypted Keystore (JSON format) ───────────────────────────────────

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct KeystoreEntry {
    pub version: u32,
    pub address: String,
    pub ciphertext: Vec<u8>,
    pub salt: [u8; 16],
    pub label: Option<String>,
}

impl KeystoreEntry {
    pub fn encrypt(secret: &SecretKey, password: &str, label: Option<String>) -> Self {
        let addr = secret.public_key().address();
        let salt = {
            let hash = Sha256::digest(&addr.0);
            let mut s = [0u8; 16];
            s.copy_from_slice(&hash[..16]);
            s
        };
        let mask = Self::derive_mask(password, &salt);
        let ciphertext: Vec<u8> = secret.0.iter().zip(mask.iter()).map(|(a, b)| a ^ b).collect();
        KeystoreEntry {
            version: 1,
            address: format!("{addr}"),
            ciphertext,
            salt,
            label,
        }
    }

    pub fn decrypt(&self, password: &str) -> Result<SecretKey, WalletError> {
        let mask = Self::derive_mask(password, &self.salt);
        if self.ciphertext.len() != 32 {
            return Err(WalletError::DecryptionFailed);
        }
        let mut bytes = [0u8; 32];
        for (i, (c, m)) in self.ciphertext.iter().zip(mask.iter()).enumerate() {
            bytes[i] = c ^ m;
        }
        let sk = SecretKey(bytes);
        // Verify decrypted key produces the stored address
        let derived_addr = format!("{}", sk.public_key().address());
        if derived_addr != self.address {
            return Err(WalletError::DecryptionFailed);
        }
        Ok(sk)
    }

    fn derive_mask(password: &str, salt: &[u8; 16]) -> [u8; 32] {
        let mut hasher = Sha256::new();
        hasher.update(password.as_bytes());
        hasher.update(salt);
        let result = hasher.finalize();
        let mut mask = [0u8; 32];
        mask.copy_from_slice(&result);
        mask
    }

    pub fn to_json(&self) -> Result<String, WalletError> {
        serde_json::to_string_pretty(self)
            .map_err(|e| WalletError::SerializationError(e.to_string()))
    }

    pub fn from_json(json: &str) -> Result<Self, WalletError> {
        serde_json::from_str(json)
            .map_err(|e| WalletError::InvalidKeystoreJson(e.to_string()))
    }
}

// ── Offline Transaction ────────────────────────────────────────────────

/// A transaction that can be constructed and signed offline.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OfflineTransaction {
    pub from: Address,
    pub nonce: u64,
    pub to: Address,
    pub value: u128,
    pub data: Vec<u8>,
    pub gas_limit: u64,
    pub max_fee_per_gas: u128,
    pub max_priority_fee: u128,
    pub chain_id: u64,
}

impl OfflineTransaction {
    /// Compute the signing hash (SHA-256 of canonical serialization).
    pub fn signing_hash(&self) -> [u8; 32] {
        let mut hasher = Sha256::new();
        hasher.update(&self.chain_id.to_le_bytes());
        hasher.update(&self.nonce.to_le_bytes());
        hasher.update(&self.from.0);
        hasher.update(&self.to.0);
        hasher.update(&self.value.to_le_bytes());
        hasher.update(&(self.data.len() as u32).to_le_bytes());
        hasher.update(&self.data);
        hasher.update(&self.gas_limit.to_le_bytes());
        hasher.update(&self.max_fee_per_gas.to_le_bytes());
        hasher.update(&self.max_priority_fee.to_le_bytes());
        let result = hasher.finalize();
        let mut hash = [0u8; 32];
        hash.copy_from_slice(&result);
        hash
    }

    /// Sign offline — returns a signed envelope ready for broadcast.
    pub fn sign(&self, secret: &SecretKey) -> Result<SignedTransaction, WalletError> {
        let pk = secret.public_key();
        if pk.address() != self.from {
            return Err(WalletError::SigningError(
                "secret key does not match 'from' address".into(),
            ));
        }
        let hash = self.signing_hash();
        let sig = secret.sign(&hash);
        Ok(SignedTransaction {
            tx: self.clone(),
            signature: sig,
            signer: pk,
        })
    }
}

/// A signed transaction envelope — serializable for later broadcast.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SignedTransaction {
    pub tx: OfflineTransaction,
    #[serde(with = "sig_serde")]
    pub signature: Signature,
    pub signer: PublicKey,
}

mod sig_serde {
    use super::Signature;
    use serde::{self, Deserializer, Serializer, Deserialize};

    pub fn serialize<S: Serializer>(sig: &Signature, s: S) -> Result<S::Ok, S::Error> {
        s.serialize_str(&sig.to_hex())
    }

    pub fn deserialize<'de, D: Deserializer<'de>>(d: D) -> Result<Signature, D::Error> {
        let hex = String::deserialize(d)?;
        Signature::from_hex(&hex).map_err(serde::de::Error::custom)
    }
}

impl SignedTransaction {
    /// Verify the signature is valid for the embedded transaction.
    pub fn verify(&self) -> bool {
        let hash = self.tx.signing_hash();
        self.signer.verify(&hash, &self.signature)
            && self.signer.address() == self.tx.from
    }

    /// Serialize to JSON for file storage or broadcast submission.
    pub fn to_json(&self) -> Result<String, WalletError> {
        serde_json::to_string_pretty(self)
            .map_err(|e| WalletError::SerializationError(e.to_string()))
    }

    /// Deserialize from JSON.
    pub fn from_json(json: &str) -> Result<Self, WalletError> {
        serde_json::from_str(json)
            .map_err(|e| WalletError::InvalidTransaction(e.to_string()))
    }
}

// ── CLI Keystore Manager ───────────────────────────────────────────────

/// Manages a collection of encrypted keys (in-memory, backed by JSON files).
pub struct KeystoreManager {
    entries: HashMap<String, KeystoreEntry>,
}

impl KeystoreManager {
    pub fn new() -> Self {
        Self {
            entries: HashMap::new(),
        }
    }

    /// Import a key from hex seed, encrypt with password.
    pub fn import_hex(
        &mut self,
        hex_seed: &str,
        password: &str,
        label: Option<String>,
    ) -> Result<String, WalletError> {
        let sk = SecretKey::from_hex(hex_seed)?;
        let addr = format!("{}", sk.public_key().address());
        if self.entries.contains_key(&addr) {
            return Err(WalletError::DuplicateKey(addr));
        }
        let entry = KeystoreEntry::encrypt(&sk, password, label);
        self.entries.insert(addr.clone(), entry);
        Ok(addr)
    }

    /// Import a key from passphrase.
    pub fn import_passphrase(
        &mut self,
        passphrase: &str,
        password: &str,
        label: Option<String>,
    ) -> Result<String, WalletError> {
        let sk = SecretKey::from_passphrase(passphrase);
        let addr = format!("{}", sk.public_key().address());
        if self.entries.contains_key(&addr) {
            return Err(WalletError::DuplicateKey(addr));
        }
        let entry = KeystoreEntry::encrypt(&sk, password, label);
        self.entries.insert(addr.clone(), entry);
        Ok(addr)
    }

    /// Import from encrypted keystore JSON.
    pub fn import_json(&mut self, json: &str) -> Result<String, WalletError> {
        let entry = KeystoreEntry::from_json(json)?;
        let addr = entry.address.clone();
        if self.entries.contains_key(&addr) {
            return Err(WalletError::DuplicateKey(addr));
        }
        self.entries.insert(addr.clone(), entry);
        Ok(addr)
    }

    /// Export a key as encrypted JSON.
    pub fn export_json(&self, address: &str) -> Result<String, WalletError> {
        let entry = self.entries.get(address)
            .ok_or_else(|| WalletError::KeyNotFound(address.into()))?;
        entry.to_json()
    }

    /// List all addresses with optional labels.
    pub fn list(&self) -> Vec<(String, Option<String>)> {
        let mut out: Vec<_> = self.entries.iter()
            .map(|(addr, e)| (addr.clone(), e.label.clone()))
            .collect();
        out.sort_by(|a, b| a.0.cmp(&b.0));
        out
    }

    /// Delete a key by address.
    pub fn delete(&mut self, address: &str) -> Result<(), WalletError> {
        self.entries.remove(address)
            .map(|_| ())
            .ok_or_else(|| WalletError::KeyNotFound(address.into()))
    }

    /// Sign an offline transaction using a stored key.
    pub fn sign_transaction(
        &self,
        address: &str,
        password: &str,
        tx: &OfflineTransaction,
    ) -> Result<SignedTransaction, WalletError> {
        let entry = self.entries.get(address)
            .ok_or_else(|| WalletError::KeyNotFound(address.into()))?;
        let sk = entry.decrypt(password)?;
        tx.sign(&sk)
    }

    /// Get the number of stored keys.
    pub fn len(&self) -> usize {
        self.entries.len()
    }

    pub fn is_empty(&self) -> bool {
        self.entries.is_empty()
    }

    /// Serialize entire keystore to JSON (for file persistence).
    pub fn save_to_json(&self) -> Result<String, WalletError> {
        let entries: Vec<&KeystoreEntry> = self.entries.values().collect();
        serde_json::to_string_pretty(&entries)
            .map_err(|e| WalletError::SerializationError(e.to_string()))
    }

    /// Load keystore from JSON array.
    pub fn load_from_json(json: &str) -> Result<Self, WalletError> {
        let entries: Vec<KeystoreEntry> = serde_json::from_str(json)
            .map_err(|e| WalletError::InvalidKeystoreJson(e.to_string()))?;
        let mut mgr = Self::new();
        for entry in entries {
            mgr.entries.insert(entry.address.clone(), entry);
        }
        Ok(mgr)
    }
}

// ── Tests ──────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    fn test_seed() -> [u8; 32] {
        let mut s = [0u8; 32];
        for i in 0..32 { s[i] = i as u8 + 1; }
        s
    }

    #[test]
    fn test_import_hex_and_derive_address() {
        let sk = SecretKey(test_seed());
        let hex = sk.to_hex();
        let sk2 = SecretKey::from_hex(&hex).unwrap();
        assert_eq!(sk.public_key().address(), sk2.public_key().address());
    }

    #[test]
    fn test_import_hex_with_0x_prefix() {
        let sk = SecretKey(test_seed());
        let hex = format!("0x{}", sk.to_hex());
        let sk2 = SecretKey::from_hex(&hex).unwrap();
        assert_eq!(sk.public_key().address(), sk2.public_key().address());
    }

    #[test]
    fn test_invalid_hex_seed() {
        assert!(SecretKey::from_hex("tooshort").is_err());
        assert!(SecretKey::from_hex("zzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzz").is_err());
    }

    #[test]
    fn test_passphrase_deterministic() {
        let sk1 = SecretKey::from_passphrase("my secret wallet");
        let sk2 = SecretKey::from_passphrase("my secret wallet");
        assert_eq!(sk1.public_key(), sk2.public_key());
    }

    #[test]
    fn test_sign_and_verify() {
        let sk = SecretKey(test_seed());
        let pk = sk.public_key();
        let msg = b"hello prova";
        let sig = sk.sign(msg);
        assert!(pk.verify(msg, &sig));
        assert!(!pk.verify(b"wrong message", &sig));
    }

    #[test]
    fn test_keystore_encrypt_decrypt() {
        let sk = SecretKey(test_seed());
        let entry = KeystoreEntry::encrypt(&sk, "password123", Some("test key".into()));
        let recovered = entry.decrypt("password123").unwrap();
        assert_eq!(sk.public_key().address(), recovered.public_key().address());
    }

    #[test]
    fn test_keystore_wrong_password_fails() {
        let sk = SecretKey(test_seed());
        let entry = KeystoreEntry::encrypt(&sk, "correct", None);
        assert!(entry.decrypt("wrong").is_err());
    }

    #[test]
    fn test_keystore_json_roundtrip() {
        let sk = SecretKey(test_seed());
        let entry = KeystoreEntry::encrypt(&sk, "pw", Some("label".into()));
        let json = entry.to_json().unwrap();
        let entry2 = KeystoreEntry::from_json(&json).unwrap();
        assert_eq!(entry.address, entry2.address);
        assert_eq!(entry.ciphertext, entry2.ciphertext);
        let recovered = entry2.decrypt("pw").unwrap();
        assert_eq!(sk.public_key().address(), recovered.public_key().address());
    }

    #[test]
    fn test_offline_tx_sign_and_verify() {
        let sk = SecretKey(test_seed());
        let addr = sk.public_key().address();
        let tx = OfflineTransaction {
            from: addr,
            nonce: 0,
            to: Address([0xAA; 20]),
            value: 1_000_000,
            data: vec![0x01, 0x02],
            gas_limit: 21000,
            max_fee_per_gas: 100,
            max_priority_fee: 10,
            chain_id: 1,
        };
        let signed = tx.sign(&sk).unwrap();
        assert!(signed.verify());
    }

    #[test]
    fn test_offline_tx_wrong_key_rejected() {
        let sk = SecretKey(test_seed());
        let wrong_addr = Address([0xFF; 20]);
        let tx = OfflineTransaction {
            from: wrong_addr,
            nonce: 0,
            to: Address([0xAA; 20]),
            value: 0,
            data: vec![],
            gas_limit: 21000,
            max_fee_per_gas: 100,
            max_priority_fee: 10,
            chain_id: 1,
        };
        assert!(tx.sign(&sk).is_err());
    }

    #[test]
    fn test_signed_tx_json_roundtrip() {
        let sk = SecretKey(test_seed());
        let tx = OfflineTransaction {
            from: sk.public_key().address(),
            nonce: 42,
            to: Address([0xBB; 20]),
            value: 999,
            data: vec![0xDE, 0xAD],
            gas_limit: 50000,
            max_fee_per_gas: 200,
            max_priority_fee: 20,
            chain_id: 7,
        };
        let signed = tx.sign(&sk).unwrap();
        let json = signed.to_json().unwrap();
        let recovered = SignedTransaction::from_json(&json).unwrap();
        assert!(recovered.verify());
        assert_eq!(recovered.tx.nonce, 42);
        assert_eq!(recovered.tx.chain_id, 7);
    }

    #[test]
    fn test_manager_import_and_list() {
        let mut mgr = KeystoreManager::new();
        let sk = SecretKey(test_seed());
        let addr = mgr.import_hex(&sk.to_hex(), "pw", Some("main".into())).unwrap();
        let list = mgr.list();
        assert_eq!(list.len(), 1);
        assert_eq!(list[0].0, addr);
        assert_eq!(list[0].1, Some("main".into()));
    }

    #[test]
    fn test_manager_duplicate_rejected() {
        let mut mgr = KeystoreManager::new();
        let sk = SecretKey(test_seed());
        mgr.import_hex(&sk.to_hex(), "pw", None).unwrap();
        assert!(mgr.import_hex(&sk.to_hex(), "pw2", None).is_err());
    }

    #[test]
    fn test_manager_delete() {
        let mut mgr = KeystoreManager::new();
        let addr = mgr.import_passphrase("test", "pw", None).unwrap();
        assert_eq!(mgr.len(), 1);
        mgr.delete(&addr).unwrap();
        assert!(mgr.is_empty());
        assert!(mgr.delete(&addr).is_err());
    }

    #[test]
    fn test_manager_sign_transaction() {
        let mut mgr = KeystoreManager::new();
        let sk = SecretKey(test_seed());
        let addr = mgr.import_hex(&sk.to_hex(), "pw", None).unwrap();

        let tx = OfflineTransaction {
            from: sk.public_key().address(),
            nonce: 1,
            to: Address([0xCC; 20]),
            value: 500,
            data: vec![],
            gas_limit: 21000,
            max_fee_per_gas: 50,
            max_priority_fee: 5,
            chain_id: 1,
        };
        let signed = mgr.sign_transaction(&addr, "pw", &tx).unwrap();
        assert!(signed.verify());
    }

    #[test]
    fn test_manager_sign_wrong_password() {
        let mut mgr = KeystoreManager::new();
        let sk = SecretKey(test_seed());
        let addr = mgr.import_hex(&sk.to_hex(), "correct", None).unwrap();

        let tx = OfflineTransaction {
            from: sk.public_key().address(),
            nonce: 0,
            to: Address([0xDD; 20]),
            value: 0,
            data: vec![],
            gas_limit: 21000,
            max_fee_per_gas: 50,
            max_priority_fee: 5,
            chain_id: 1,
        };
        assert!(mgr.sign_transaction(&addr, "wrong", &tx).is_err());
    }

    #[test]
    fn test_manager_export_json() {
        let mut mgr = KeystoreManager::new();
        let addr = mgr.import_passphrase("wallet1", "pw", Some("export-test".into())).unwrap();
        let json = mgr.export_json(&addr).unwrap();
        assert!(json.contains("export-test"));
        assert!(json.contains(&addr));
    }

    #[test]
    fn test_manager_save_load_roundtrip() {
        let mut mgr = KeystoreManager::new();
        mgr.import_passphrase("key1", "pw", Some("first".into())).unwrap();
        mgr.import_passphrase("key2", "pw", Some("second".into())).unwrap();

        let json = mgr.save_to_json().unwrap();
        let mgr2 = KeystoreManager::load_from_json(&json).unwrap();
        assert_eq!(mgr2.len(), 2);
    }

    #[test]
    fn test_manager_import_from_keystore_json() {
        let sk = SecretKey(test_seed());
        let entry = KeystoreEntry::encrypt(&sk, "pw", Some("imported".into()));
        let json = entry.to_json().unwrap();

        let mut mgr = KeystoreManager::new();
        let addr = mgr.import_json(&json).unwrap();
        assert_eq!(addr, entry.address);
    }

    #[test]
    fn test_signature_hex_roundtrip() {
        let sk = SecretKey(test_seed());
        let sig = sk.sign(b"test");
        let hex = sig.to_hex();
        let sig2 = Signature::from_hex(&hex).unwrap();
        assert_eq!(sig.0, sig2.0);
    }

    #[test]
    fn test_signing_hash_deterministic() {
        let sk = SecretKey(test_seed());
        let tx = OfflineTransaction {
            from: sk.public_key().address(),
            nonce: 0,
            to: Address([0; 20]),
            value: 0,
            data: vec![],
            gas_limit: 21000,
            max_fee_per_gas: 0,
            max_priority_fee: 0,
            chain_id: 1,
        };
        assert_eq!(tx.signing_hash(), tx.signing_hash());
    }

    #[test]
    fn test_different_chain_id_different_hash() {
        let sk = SecretKey(test_seed());
        let addr = sk.public_key().address();
        let tx1 = OfflineTransaction {
            from: addr, nonce: 0, to: Address([0; 20]),
            value: 0, data: vec![], gas_limit: 21000,
            max_fee_per_gas: 0, max_priority_fee: 0, chain_id: 1,
        };
        let tx2 = OfflineTransaction {
            from: addr, nonce: 0, to: Address([0; 20]),
            value: 0, data: vec![], gas_limit: 21000,
            max_fee_per_gas: 0, max_priority_fee: 0, chain_id: 2,
        };
        assert_ne!(tx1.signing_hash(), tx2.signing_hash());
    }
}
