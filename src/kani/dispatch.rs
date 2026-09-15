//! Shared dispatch vocabulary; semantic family modules register independently.

use std::collections::BTreeMap;

use serde::{Deserialize, Serialize};
use thiserror::Error;

/// A non-overlapping semantic family owned by one delivery lane.
#[derive(Clone, Copy, Debug, Eq, Ord, PartialEq, PartialOrd, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum SemanticFamily {
    /// Checked partial operations and bounded integer arithmetic.
    DefinednessArithmetic,
    /// Identity-bearing finite objects, references, and graph operations.
    ObjectsReferencesGraphs,
    /// Ordered duplicate-preserving collections and bounded queries.
    CollectionsQueries,
}

/// A versioned module registration in the common dispatch index.
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct ModuleDescriptor {
    /// Unique module identity.
    pub module_id: String,
    /// Semantic family owned exclusively by this module.
    pub family: SemanticFamily,
    /// Exact ABI revision consumed and emitted by this module.
    pub abi_revision: String,
    /// Construct identities this module is eligible to lower.
    pub constructs: Vec<String>,
}

/// Failure to build or query a non-overlapping dispatch index.
#[derive(Clone, Debug, Eq, PartialEq, Error)]
pub enum DispatchError {
    /// A descriptor lacks a required identity.
    #[error("dispatch descriptor is missing `{0}`")]
    MissingField(&'static str),
    /// A construct or family is owned more than once.
    #[error("dispatch ownership conflicts for `{0}`")]
    Conflict(String),
    /// No registered module owns a requested construct.
    #[error("no bounded-Kani module owns `{0}`")]
    Unowned(String),
}

/// Immutable routing authority shared by the independently implemented lanes.
#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct DispatchIndex {
    by_construct: BTreeMap<String, ModuleDescriptor>,
    by_family: BTreeMap<SemanticFamily, ModuleDescriptor>,
}

impl DispatchIndex {
    /// Builds a non-overlapping index from independently declared module descriptors.
    pub fn new(modules: Vec<ModuleDescriptor>) -> Result<Self, DispatchError> {
        let mut index = Self::default();
        for module in modules {
            if module.module_id.trim().is_empty() {
                return Err(DispatchError::MissingField("module_id"));
            }
            if module.abi_revision.trim().is_empty() {
                return Err(DispatchError::MissingField("abi_revision"));
            }
            if module.constructs.is_empty() {
                return Err(DispatchError::MissingField("constructs"));
            }
            if index.by_family.contains_key(&module.family) {
                return Err(DispatchError::Conflict(format!(
                    "family:{:?}",
                    module.family
                )));
            }
            for construct in &module.constructs {
                if construct.trim().is_empty() || index.by_construct.contains_key(construct) {
                    return Err(DispatchError::Conflict(format!("construct:{construct}")));
                }
            }
            for construct in &module.constructs {
                index.by_construct.insert(construct.clone(), module.clone());
            }
            index.by_family.insert(module.family, module);
        }
        Ok(index)
    }

    /// Resolves a construct without falling through to another semantic family.
    pub fn resolve(&self, construct: &str) -> Result<&ModuleDescriptor, DispatchError> {
        self.by_construct
            .get(construct)
            .ok_or_else(|| DispatchError::Unowned(construct.to_owned()))
    }
}
