//! Model Registry — on-chain model manifest storage.
//!
//! Models are registered with their per-layer weight hashes,
//! architecture group, and metadata. This enables verifiers to
//! confirm they're running the exact same model.

use crate::types::*;
use std::collections::HashMap;

/// Per-layer weight hash entry.
#[derive(Debug, Clone)]
pub struct LayerWeightHash {
    pub layer_index: u32,
    pub weight_hash: Hash,
}

/// Registered model manifest.
#[derive(Debug, Clone)]
pub struct ModelManifest {
    /// Unique model identifier (hash of manifest).
    pub model_id: ModelId,
    /// Human-readable name.
    pub name: String,
    /// Number of layers.
    pub layer_count: u32,
    /// Per-layer weight hashes for verification.
    pub layer_hashes: Vec<LayerWeightHash>,
    /// Supported architecture groups.
    pub arch_groups: Vec<ArchGroup>,
    /// Who registered this model.
    pub registrar: Address,
    /// Registration epoch.
    pub registered_at: Epoch,
}

/// The on-chain model registry.
#[derive(Debug, Default)]
pub struct ModelRegistry {
    models: HashMap<ModelId, ModelManifest>,
}

impl ModelRegistry {
    pub fn new() -> Self {
        Self::default()
    }

    /// Register a new model. Returns error if model ID already exists.
    pub fn register(&mut self, manifest: ModelManifest) -> Result<ModelId, RegistryError> {
        if self.models.contains_key(&manifest.model_id) {
            return Err(RegistryError::AlreadyRegistered(manifest.model_id));
        }

        if manifest.layer_hashes.len() != manifest.layer_count as usize {
            return Err(RegistryError::LayerCountMismatch {
                declared: manifest.layer_count,
                provided: manifest.layer_hashes.len() as u32,
            });
        }

        if manifest.arch_groups.is_empty() {
            return Err(RegistryError::NoArchGroups);
        }

        let id = manifest.model_id;
        self.models.insert(id, manifest);
        Ok(id)
    }

    /// Look up a model by ID.
    pub fn get(&self, model_id: &ModelId) -> Option<&ModelManifest> {
        self.models.get(model_id)
    }

    /// Check if a model supports a given architecture group.
    pub fn supports_arch(&self, model_id: &ModelId, arch: &ArchGroup) -> bool {
        self.models
            .get(model_id)
            .map(|m| m.arch_groups.contains(arch))
            .unwrap_or(false)
    }

    /// Total registered models.
    pub fn model_count(&self) -> usize {
        self.models.len()
    }
}

#[derive(Debug)]
pub enum RegistryError {
    AlreadyRegistered(ModelId),
    LayerCountMismatch { declared: u32, provided: u32 },
    NoArchGroups,
}

impl fmt::Display for RegistryError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::AlreadyRegistered(id) => write!(f, "model {:?} already registered", id),
            Self::LayerCountMismatch { declared, provided } => {
                write!(
                    f,
                    "declared {declared} layers but provided {provided} hashes"
                )
            }
            Self::NoArchGroups => write!(f, "must specify at least one architecture group"),
        }
    }
}

use std::fmt;
impl std::error::Error for RegistryError {}

#[cfg(test)]
mod tests {
    use super::*;

    fn test_manifest(name: &str, layers: u32) -> ModelManifest {
        let model_id = ModelId({
            let mut h = [0u8; 32];
            h[0] = name.as_bytes()[0];
            h
        });

        let layer_hashes = (0..layers)
            .map(|i| LayerWeightHash {
                layer_index: i,
                weight_hash: {
                    let mut h = [0u8; 32];
                    h[0] = i as u8;
                    h
                },
            })
            .collect();

        ModelManifest {
            model_id,
            name: name.to_string(),
            layer_count: layers,
            layer_hashes,
            arch_groups: vec![ArchGroup::new("nvidia-sm89-int8")],
            registrar: Address::test(1),
            registered_at: 100,
        }
    }

    #[test]
    fn test_register_model() {
        let mut reg = ModelRegistry::new();
        let manifest = test_manifest("tinyllama", 22);
        let id = reg.register(manifest).unwrap();
        assert_eq!(reg.model_count(), 1);
        assert!(reg.get(&id).is_some());
    }

    #[test]
    fn test_duplicate_registration() {
        let mut reg = ModelRegistry::new();
        let m1 = test_manifest("tinyllama", 22);
        let m2 = test_manifest("tinyllama", 22); // same ID
        reg.register(m1).unwrap();
        assert!(reg.register(m2).is_err());
    }

    #[test]
    fn test_arch_group_check() {
        let mut reg = ModelRegistry::new();
        let manifest = test_manifest("tinyllama", 22);
        let id = reg.register(manifest).unwrap();
        assert!(reg.supports_arch(&id, &ArchGroup::new("nvidia-sm89-int8")));
        assert!(!reg.supports_arch(&id, &ArchGroup::new("nvidia-sm90-int8")));
    }

    #[test]
    fn test_layer_count_mismatch() {
        let mut reg = ModelRegistry::new();
        let mut manifest = test_manifest("bad", 10);
        manifest.layer_hashes.pop(); // Now 9 hashes for 10 declared
        assert!(reg.register(manifest).is_err());
    }
}
