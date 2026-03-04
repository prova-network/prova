//! Encryption key management (NODE-032)
//!
//! Per-inference ephemeral key derivation and lifecycle management for
//! confidential inference. Each inference gets a unique ephemeral keypair
//! derived from a provider master secret + inference-specific context,
//! ensuring forward secrecy: compromising one inference key reveals nothing
//! about others.
//!
//! Design:
//! - Master secret stored encrypted at rest (XChaCha20-Poly1305, passphrase-derived KEK)
//! - Ephemeral keys derived via HKDF-SHA256(master, context) where context = (model_id || epoch || nonce)
//! - Keys are cached in memory during active inference, then zeroized on finalization
//! - Blinding factors derived from same HKDF chain for commit-reveal integration

use std::collections::HashMap;
use std::fmt;

pub type Hash = [u8; 32];
pub type Nonce = [u8; 24];

// ── Constants ───────────────────────────────────────────────────────

/// HKDF info prefix for ephemeral encryption keys.
const HKDF_INFO_ENCRYPT: &[u8] = b"prova-confidential-encrypt-v1";
/// HKDF info prefix for blinding factors.
const HKDF_INFO_BLIND: &[u8] = b"prova-confidential-blind-v1";
/// Max cached ephemeral keys before forced eviction.
const MAX_CACHED_KEYS: usize = 1024;
/// Minimum passphrase length for master key encryption.
const MIN_PASSPHRASE_LEN: usize = 12;

// ── Error Types ─────────────────────────────────────────────────────

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum KeyError {
    /// Master secret not initialized.
    NotInitialized,
    /// Passphrase too short.
    PassphraseTooShort { min: usize, got: usize },
    /// Decryption failed (wrong passphrase or corrupted data).
    DecryptionFailed,
    /// Key not found in cache.
    KeyNotFound(InferenceId),
    /// Cache full and eviction failed.
    CacheFull,
    /// Invalid key material (all zeros, etc.).
    InvalidKeyMaterial,
    /// Derivation context is empty.
    EmptyContext,
}

impl fmt::Display for KeyError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::NotInitialized => write!(f, "master secret not initialized"),
            Self::PassphraseTooShort { min, got } =>
                write!(f, "passphrase too short: need {min}, got {got}"),
            Self::DecryptionFailed => write!(f, "decryption failed"),
            Self::KeyNotFound(id) => write!(f, "ephemeral key not found: {id:?}"),
            Self::CacheFull => write!(f, "ephemeral key cache full"),
            Self::InvalidKeyMaterial => write!(f, "invalid key material"),
            Self::EmptyContext => write!(f, "derivation context is empty"),
        }
    }
}

// ── Identifier Types ────────────────────────────────────────────────

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct InferenceId(pub u64);

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ModelId(pub String);

pub type Epoch = u64;

// ── Key Material ────────────────────────────────────────────────────

/// An ephemeral key derived for a single inference.
#[derive(Clone)]
pub struct EphemeralKey {
    /// 256-bit encryption key.
    pub encrypt_key: [u8; 32],
    /// 256-bit blinding factor for commit-reveal.
    pub blinding_factor: [u8; 32],
    /// The inference this key is bound to.
    pub inference_id: InferenceId,
    /// Whether this key has been used (revealed or finalized).
    pub consumed: bool,
}

impl fmt::Debug for EphemeralKey {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        // Never print key material in debug output
        f.debug_struct("EphemeralKey")
            .field("inference_id", &self.inference_id)
            .field("consumed", &self.consumed)
            .field("encrypt_key", &"[REDACTED]")
            .field("blinding_factor", &"[REDACTED]")
            .finish()
    }
}

impl Drop for EphemeralKey {
    fn drop(&mut self) {
        // Zeroize key material on drop
        self.encrypt_key = [0u8; 32];
        self.blinding_factor = [0u8; 32];
    }
}

/// Encrypted master secret stored at rest.
#[derive(Debug, Clone)]
pub struct EncryptedMasterSecret {
    /// XChaCha20-Poly1305 ciphertext.
    pub ciphertext: Vec<u8>,
    /// 24-byte nonce.
    pub nonce: Nonce,
    /// Argon2id salt for passphrase → KEK derivation.
    pub salt: [u8; 16],
    /// Argon2id time cost.
    pub argon_time_cost: u32,
    /// Argon2id memory cost (KiB).
    pub argon_mem_cost: u32,
}

// ── HKDF (simplified, production would use ring/hkdf crate) ─────────

fn hkdf_sha256(ikm: &[u8], salt: &[u8], info: &[u8]) -> [u8; 32] {
    // HKDF-Extract: PRK = HMAC-SHA256(salt, IKM)
    let prk = hmac_sha256(salt, ikm);
    // HKDF-Expand: OKM = HMAC-SHA256(PRK, info || 0x01)
    let mut expand_input = Vec::with_capacity(info.len() + 1);
    expand_input.extend_from_slice(info);
    expand_input.push(0x01);
    hmac_sha256(&prk, &expand_input)
}

fn hmac_sha256(key: &[u8], data: &[u8]) -> [u8; 32] {
    // Simplified HMAC-SHA256 using our simple_sha256
    let mut ipad = [0x36u8; 64];
    let mut opad = [0x5cu8; 64];

    let key_block = if key.len() <= 64 {
        let mut kb = [0u8; 64];
        kb[..key.len()].copy_from_slice(key);
        kb
    } else {
        let h = simple_sha256(key);
        let mut kb = [0u8; 64];
        kb[..32].copy_from_slice(&h);
        kb
    };

    for i in 0..64 {
        ipad[i] ^= key_block[i];
        opad[i] ^= key_block[i];
    }

    let mut inner = Vec::with_capacity(64 + data.len());
    inner.extend_from_slice(&ipad);
    inner.extend_from_slice(data);
    let inner_hash = simple_sha256(&inner);

    let mut outer = Vec::with_capacity(64 + 32);
    outer.extend_from_slice(&opad);
    outer.extend_from_slice(&inner_hash);
    simple_sha256(&outer)
}

fn simple_sha256(data: &[u8]) -> [u8; 32] {
    use std::collections::hash_map::DefaultHasher;
    use std::hash::{Hash, Hasher};
    // For deterministic testing: double-hash with different seeds
    // Production would use sha2 crate
    let mut result = [0u8; 32];
    for chunk in 0..4 {
        let mut h = DefaultHasher::new();
        chunk.hash(&mut h);
        data.hash(&mut h);
        let v = h.finish();
        let offset = chunk * 8;
        result[offset..offset + 8].copy_from_slice(&v.to_le_bytes());
    }
    result
}

// ── XChaCha20-Poly1305 (simplified for scaffold) ───────────────────

fn xchacha_encrypt(key: &[u8; 32], nonce: &Nonce, plaintext: &[u8]) -> Vec<u8> {
    // Simplified: XOR with HKDF-derived stream + append MAC tag
    let stream_key = hkdf_sha256(key, nonce, b"xchacha-stream");
    let mut ciphertext = Vec::with_capacity(plaintext.len() + 16);
    for (i, &byte) in plaintext.iter().enumerate() {
        ciphertext.push(byte ^ stream_key[i % 32]);
    }
    // Append 16-byte MAC tag
    let tag = hkdf_sha256(key, &ciphertext, b"xchacha-tag");
    ciphertext.extend_from_slice(&tag[..16]);
    ciphertext
}

fn xchacha_decrypt(key: &[u8; 32], nonce: &Nonce, ciphertext: &[u8]) -> Result<Vec<u8>, KeyError> {
    if ciphertext.len() < 16 {
        return Err(KeyError::DecryptionFailed);
    }
    let (ct, tag) = ciphertext.split_at(ciphertext.len() - 16);
    // Verify MAC
    let expected_tag = hkdf_sha256(key, ct, b"xchacha-tag");
    if tag != &expected_tag[..16] {
        return Err(KeyError::DecryptionFailed);
    }
    // Decrypt
    let stream_key = hkdf_sha256(key, nonce, b"xchacha-stream");
    let mut plaintext = Vec::with_capacity(ct.len());
    for (i, &byte) in ct.iter().enumerate() {
        plaintext.push(byte ^ stream_key[i % 32]);
    }
    Ok(plaintext)
}

fn derive_kek(passphrase: &str, salt: &[u8; 16]) -> [u8; 32] {
    // Simplified Argon2id (production would use argon2 crate)
    let mut input = Vec::new();
    input.extend_from_slice(passphrase.as_bytes());
    input.extend_from_slice(salt);
    // Multiple rounds for key stretching simulation
    let mut key = simple_sha256(&input);
    for _ in 0..1000 {
        key = simple_sha256(&key);
    }
    key
}

// ── Key Manager ─────────────────────────────────────────────────────

/// Manages master secret and ephemeral key derivation/caching.
#[derive(Debug)]
pub struct KeyManager {
    /// Decrypted master secret (only in memory while unlocked).
    master_secret: Option<[u8; 32]>,
    /// Encrypted master secret for persistence.
    encrypted_master: Option<EncryptedMasterSecret>,
    /// Cache of active ephemeral keys.
    cache: HashMap<InferenceId, EphemeralKey>,
    /// Counter for unique nonce generation.
    derivation_counter: u64,
}

impl KeyManager {
    /// Create a new key manager (locked, no master secret).
    pub fn new() -> Self {
        Self {
            master_secret: None,
            encrypted_master: None,
            cache: HashMap::new(),
            derivation_counter: 0,
        }
    }

    /// Generate a new master secret and encrypt it with the given passphrase.
    pub fn initialize(&mut self, passphrase: &str) -> Result<(), KeyError> {
        if passphrase.len() < MIN_PASSPHRASE_LEN {
            return Err(KeyError::PassphraseTooShort {
                min: MIN_PASSPHRASE_LEN,
                got: passphrase.len(),
            });
        }

        // Generate master secret from passphrase + entropy simulation
        let mut entropy = Vec::new();
        entropy.extend_from_slice(passphrase.as_bytes());
        entropy.extend_from_slice(&self.derivation_counter.to_le_bytes());
        entropy.extend_from_slice(b"prova-master-init");
        let master = simple_sha256(&entropy);

        // Validate key material
        if master == [0u8; 32] {
            return Err(KeyError::InvalidKeyMaterial);
        }

        // Encrypt with passphrase-derived KEK
        let salt: [u8; 16] = {
            let h = simple_sha256(b"prova-salt-derive");
            let mut s = [0u8; 16];
            s.copy_from_slice(&h[..16]);
            s
        };
        let nonce: Nonce = {
            let h = simple_sha256(b"prova-nonce-derive");
            let mut n = [0u8; 24];
            n.copy_from_slice(&h[..24]);
            n
        };

        let kek = derive_kek(passphrase, &salt);
        let ciphertext = xchacha_encrypt(&kek, &nonce, &master);

        self.encrypted_master = Some(EncryptedMasterSecret {
            ciphertext,
            nonce,
            salt,
            argon_time_cost: 3,
            argon_mem_cost: 65536,
        });
        self.master_secret = Some(master);
        Ok(())
    }

    /// Unlock the master secret using a passphrase.
    pub fn unlock(&mut self, passphrase: &str) -> Result<(), KeyError> {
        let encrypted = self.encrypted_master.as_ref()
            .ok_or(KeyError::NotInitialized)?;

        let kek = derive_kek(passphrase, &encrypted.salt);
        let plaintext = xchacha_decrypt(&kek, &encrypted.nonce, &encrypted.ciphertext)?;

        if plaintext.len() != 32 {
            return Err(KeyError::DecryptionFailed);
        }

        let mut master = [0u8; 32];
        master.copy_from_slice(&plaintext);

        if master == [0u8; 32] {
            return Err(KeyError::InvalidKeyMaterial);
        }

        self.master_secret = Some(master);
        Ok(())
    }

    /// Lock the key manager, zeroizing the master secret from memory.
    pub fn lock(&mut self) {
        if let Some(ref mut secret) = self.master_secret {
            *secret = [0u8; 32];
        }
        self.master_secret = None;
        // Also clear all cached ephemeral keys
        self.cache.clear();
    }

    /// Whether the manager is unlocked (master secret in memory).
    pub fn is_unlocked(&self) -> bool {
        self.master_secret.is_some()
    }

    /// Derive an ephemeral key for a specific inference.
    pub fn derive_ephemeral(
        &mut self,
        inference_id: InferenceId,
        model_id: &ModelId,
        epoch: Epoch,
    ) -> Result<&EphemeralKey, KeyError> {
        let master = self.master_secret.ok_or(KeyError::NotInitialized)?;

        if self.cache.len() >= MAX_CACHED_KEYS && !self.cache.contains_key(&inference_id) {
            // Evict consumed keys first
            let consumed: Vec<InferenceId> = self.cache.iter()
                .filter(|(_, k)| k.consumed)
                .map(|(id, _)| *id)
                .collect();
            for id in consumed {
                self.cache.remove(&id);
            }
            if self.cache.len() >= MAX_CACHED_KEYS {
                return Err(KeyError::CacheFull);
            }
        }

        if !self.cache.contains_key(&inference_id) {
            // Build derivation context: model_id || epoch || inference_id
            let mut context = Vec::new();
            context.extend_from_slice(model_id.0.as_bytes());
            context.extend_from_slice(&epoch.to_le_bytes());
            context.extend_from_slice(&inference_id.0.to_le_bytes());

            if context.is_empty() {
                return Err(KeyError::EmptyContext);
            }

            // Derive encryption key
            let encrypt_key = hkdf_sha256(&master, &context, HKDF_INFO_ENCRYPT);
            // Derive blinding factor
            let blinding_factor = hkdf_sha256(&master, &context, HKDF_INFO_BLIND);

            if encrypt_key == [0u8; 32] || blinding_factor == [0u8; 32] {
                return Err(KeyError::InvalidKeyMaterial);
            }

            self.derivation_counter += 1;

            self.cache.insert(inference_id, EphemeralKey {
                encrypt_key,
                blinding_factor,
                inference_id,
                consumed: false,
            });
        }

        Ok(self.cache.get(&inference_id).unwrap())
    }

    /// Get a cached ephemeral key (does not derive).
    pub fn get_ephemeral(&self, inference_id: &InferenceId) -> Result<&EphemeralKey, KeyError> {
        self.cache.get(inference_id)
            .ok_or(KeyError::KeyNotFound(*inference_id))
    }

    /// Mark an ephemeral key as consumed (after reveal or finalization).
    pub fn consume_key(&mut self, inference_id: &InferenceId) -> Result<(), KeyError> {
        let key = self.cache.get_mut(inference_id)
            .ok_or(KeyError::KeyNotFound(*inference_id))?;
        key.consumed = true;
        Ok(())
    }

    /// Remove a specific ephemeral key from cache (zeroized on drop).
    pub fn evict_key(&mut self, inference_id: &InferenceId) -> Result<(), KeyError> {
        self.cache.remove(inference_id)
            .map(|_| ())
            .ok_or(KeyError::KeyNotFound(*inference_id))
    }

    /// Evict all consumed keys from cache.
    pub fn evict_consumed(&mut self) -> usize {
        let before = self.cache.len();
        self.cache.retain(|_, k| !k.consumed);
        before - self.cache.len()
    }

    /// Number of cached ephemeral keys.
    pub fn cache_size(&self) -> usize {
        self.cache.len()
    }

    /// Number of derivations performed.
    pub fn derivation_count(&self) -> u64 {
        self.derivation_counter
    }

    /// Encrypt plaintext activations with an inference's ephemeral key.
    pub fn encrypt_activations(
        &self,
        inference_id: &InferenceId,
        plaintext: &[u8],
    ) -> Result<Vec<u8>, KeyError> {
        let key = self.get_ephemeral(inference_id)?;
        // Use inference_id as part of nonce for uniqueness
        let mut nonce = [0u8; 24];
        nonce[..8].copy_from_slice(&inference_id.0.to_le_bytes());
        nonce[8..16].copy_from_slice(&simple_sha256(&key.encrypt_key)[..8]);
        Ok(xchacha_encrypt(&key.encrypt_key, &nonce, plaintext))
    }

    /// Decrypt activations with an inference's ephemeral key.
    pub fn decrypt_activations(
        &self,
        inference_id: &InferenceId,
        ciphertext: &[u8],
    ) -> Result<Vec<u8>, KeyError> {
        let key = self.get_ephemeral(inference_id)?;
        let mut nonce = [0u8; 24];
        nonce[..8].copy_from_slice(&inference_id.0.to_le_bytes());
        nonce[8..16].copy_from_slice(&simple_sha256(&key.encrypt_key)[..8]);
        xchacha_decrypt(&key.encrypt_key, &nonce, ciphertext)
    }

    /// Compute blinding hash: H(plaintext_root || blinding_factor) for commit-reveal.
    pub fn compute_blinding_hash(
        &self,
        inference_id: &InferenceId,
        plaintext_root: &Hash,
    ) -> Result<Hash, KeyError> {
        let key = self.get_ephemeral(inference_id)?;
        let mut preimage = Vec::with_capacity(64);
        preimage.extend_from_slice(plaintext_root);
        preimage.extend_from_slice(&key.blinding_factor);
        Ok(simple_sha256(&preimage))
    }
}

impl Default for KeyManager {
    fn default() -> Self {
        Self::new()
    }
}

// ── Tests ───────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    fn test_passphrase() -> &'static str {
        "my-secure-passphrase-123"
    }

    fn setup_unlocked() -> KeyManager {
        let mut km = KeyManager::new();
        km.initialize(test_passphrase()).unwrap();
        km
    }

    #[test]
    fn test_initialize_and_unlock() {
        let mut km = KeyManager::new();
        assert!(!km.is_unlocked());
        km.initialize(test_passphrase()).unwrap();
        assert!(km.is_unlocked());

        // Lock and re-unlock
        km.lock();
        assert!(!km.is_unlocked());
        km.unlock(test_passphrase()).unwrap();
        assert!(km.is_unlocked());
    }

    #[test]
    fn test_passphrase_too_short() {
        let mut km = KeyManager::new();
        let err = km.initialize("short").unwrap_err();
        assert_eq!(err, KeyError::PassphraseTooShort { min: 12, got: 5 });
    }

    #[test]
    fn test_wrong_passphrase_fails() {
        let mut km = KeyManager::new();
        km.initialize(test_passphrase()).unwrap();
        km.lock();
        let err = km.unlock("wrong-passphrase!").unwrap_err();
        assert_eq!(err, KeyError::DecryptionFailed);
    }

    #[test]
    fn test_unlock_without_init() {
        let mut km = KeyManager::new();
        let err = km.unlock("doesnt-matter-passphrase").unwrap_err();
        assert_eq!(err, KeyError::NotInitialized);
    }

    #[test]
    fn test_derive_ephemeral_key() {
        let mut km = setup_unlocked();
        let id = InferenceId(42);
        let model = ModelId("llama-7b".to_string());

        let key = km.derive_ephemeral(id, &model, 100).unwrap();
        assert_eq!(key.inference_id, id);
        assert!(!key.consumed);
        assert_ne!(key.encrypt_key, [0u8; 32]);
        assert_ne!(key.blinding_factor, [0u8; 32]);
        // Encrypt key and blinding factor should be different
        assert_ne!(key.encrypt_key, key.blinding_factor);
    }

    #[test]
    fn test_deterministic_derivation() {
        let mut km1 = setup_unlocked();
        let mut km2 = KeyManager::new();
        km2.initialize(test_passphrase()).unwrap();

        let id = InferenceId(99);
        let model = ModelId("gpt-neo".to_string());

        let k1 = km1.derive_ephemeral(id, &model, 50).unwrap();
        let k2 = km2.derive_ephemeral(id, &model, 50).unwrap();

        assert_eq!(k1.encrypt_key, k2.encrypt_key);
        assert_eq!(k1.blinding_factor, k2.blinding_factor);
    }

    #[test]
    fn test_different_inferences_get_different_keys() {
        let mut km = setup_unlocked();
        let model = ModelId("llama-7b".to_string());

        let k1 = km.derive_ephemeral(InferenceId(1), &model, 100).unwrap();
        let ek1 = k1.encrypt_key;
        let k2 = km.derive_ephemeral(InferenceId(2), &model, 100).unwrap();
        let ek2 = k2.encrypt_key;

        assert_ne!(ek1, ek2);
    }

    #[test]
    fn test_different_epochs_get_different_keys() {
        let mut km = setup_unlocked();
        let model = ModelId("llama-7b".to_string());

        let k1 = km.derive_ephemeral(InferenceId(1), &model, 100).unwrap();
        let ek1 = k1.encrypt_key;

        // New manager to avoid cache hit
        let mut km2 = setup_unlocked();
        let k2 = km2.derive_ephemeral(InferenceId(1), &model, 200).unwrap();
        let ek2 = k2.encrypt_key;

        assert_ne!(ek1, ek2);
    }

    #[test]
    fn test_derive_requires_unlock() {
        let km_init = setup_unlocked();
        let mut km = KeyManager::new();
        km.encrypted_master = km_init.encrypted_master.clone();
        // Not unlocked
        let err = km.derive_ephemeral(InferenceId(1), &ModelId("x".into()), 0).unwrap_err();
        assert_eq!(err, KeyError::NotInitialized);
    }

    #[test]
    fn test_consume_and_evict() {
        let mut km = setup_unlocked();
        let model = ModelId("test".to_string());
        let id = InferenceId(10);

        km.derive_ephemeral(id, &model, 1).unwrap();
        assert_eq!(km.cache_size(), 1);

        km.consume_key(&id).unwrap();
        assert!(km.get_ephemeral(&id).unwrap().consumed);

        let evicted = km.evict_consumed();
        assert_eq!(evicted, 1);
        assert_eq!(km.cache_size(), 0);
    }

    #[test]
    fn test_evict_specific_key() {
        let mut km = setup_unlocked();
        let model = ModelId("test".to_string());
        let id = InferenceId(10);

        km.derive_ephemeral(id, &model, 1).unwrap();
        km.evict_key(&id).unwrap();
        assert_eq!(km.cache_size(), 0);

        let err = km.evict_key(&id).unwrap_err();
        assert_eq!(err, KeyError::KeyNotFound(id));
    }

    #[test]
    fn test_encrypt_decrypt_activations() {
        let mut km = setup_unlocked();
        let id = InferenceId(7);
        let model = ModelId("llama-7b".to_string());

        km.derive_ephemeral(id, &model, 50).unwrap();

        let plaintext = b"activation data for layer 12";
        let ciphertext = km.encrypt_activations(&id, plaintext).unwrap();
        assert_ne!(&ciphertext[..plaintext.len()], plaintext);

        let decrypted = km.decrypt_activations(&id, &ciphertext).unwrap();
        assert_eq!(&decrypted, plaintext);
    }

    #[test]
    fn test_blinding_hash() {
        let mut km = setup_unlocked();
        let id = InferenceId(5);
        let model = ModelId("test".to_string());

        km.derive_ephemeral(id, &model, 1).unwrap();

        let root = [0xABu8; 32];
        let hash1 = km.compute_blinding_hash(&id, &root).unwrap();
        let hash2 = km.compute_blinding_hash(&id, &root).unwrap();

        // Deterministic
        assert_eq!(hash1, hash2);
        // Different from root
        assert_ne!(hash1, root);
    }

    #[test]
    fn test_lock_clears_cache() {
        let mut km = setup_unlocked();
        let model = ModelId("test".to_string());

        km.derive_ephemeral(InferenceId(1), &model, 1).unwrap();
        km.derive_ephemeral(InferenceId(2), &model, 1).unwrap();
        assert_eq!(km.cache_size(), 2);

        km.lock();
        assert_eq!(km.cache_size(), 0);
        assert!(!km.is_unlocked());
    }

    #[test]
    fn test_debug_redacts_keys() {
        let mut km = setup_unlocked();
        let id = InferenceId(1);
        let model = ModelId("test".to_string());

        km.derive_ephemeral(id, &model, 1).unwrap();
        let key = km.get_ephemeral(&id).unwrap();
        let debug = format!("{:?}", key);

        assert!(debug.contains("REDACTED"));
        assert!(!debug.contains(&format!("{:?}", key.encrypt_key)));
    }

    #[test]
    fn test_derivation_counter() {
        let mut km = setup_unlocked();
        let model = ModelId("test".to_string());

        assert_eq!(km.derivation_count(), 0);
        km.derive_ephemeral(InferenceId(1), &model, 1).unwrap();
        assert_eq!(km.derivation_count(), 1);
        // Re-deriving cached key doesn't increment
        km.derive_ephemeral(InferenceId(1), &model, 1).unwrap();
        assert_eq!(km.derivation_count(), 1);
        km.derive_ephemeral(InferenceId(2), &model, 1).unwrap();
        assert_eq!(km.derivation_count(), 2);
    }
}
