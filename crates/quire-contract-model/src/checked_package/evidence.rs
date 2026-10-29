//! Authoritative evidence consulted by the version dispatcher and V2 reader.

use super::shared::CheckedArtifactRef;
use super::v2::CheckedPackageV2;
use serde::{Deserialize, Serialize};
use std::collections::{btree_map::Entry, BTreeMap, BTreeSet};
use std::sync::Arc;
use thiserror::Error;

/// A repeated evidence declaration conflicts with the retained first value.
#[derive(Clone, Debug, Eq, Error, PartialEq)]
pub enum CheckedPackageEvidenceInsertionError {
    /// Different extension definition under an occupied exact reference.
    #[error("conflicting extension definition under {authority}/{identity}")]
    ExtensionDefinitionConflict {
        /// Definition authority.
        authority: Box<str>,
        /// Definition identity.
        identity: Box<str>,
    },
    /// Different document bytes were supplied under an occupied exact key.
    #[error("conflicting domain document under {identity}")]
    DomainDocumentConflict {
        /// The exact supplied domain package identity.
        identity: Box<str>,
    },
    /// Different admitted content was supplied under an occupied exact identity.
    #[error("conflicting dependency package under {identity}")]
    DependencyPackageConflict {
        /// The exact supplied library identity.
        identity: Box<str>,
    },
}

/// A caller-validated FR-467 extension definition. The reader uses the typed
/// schema and retains the grammar schema for exact evidence equality.
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct CheckedExtensionDefinition {
    /// The validated grammar schema.
    pub grammar_schema: CheckedExtensionGrammarSchema,
    /// The validated typed-node schema.
    pub typed_node_schema: CheckedExtensionTypedNodeSchema,
}

/// Grammar productions of a validated FR-467 extension definition.
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct CheckedExtensionGrammarSchema {
    /// One or more declaration productions.
    pub productions: Vec<CheckedExtensionProduction>,
}

/// One source declaration keyword and its ordered clause names.
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct CheckedExtensionProduction {
    /// Leading keyword.
    pub keyword: Box<str>,
    /// Clause names in source order.
    pub clauses: Vec<Box<str>>,
}

/// Typed node entries of a validated extension definition.
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct CheckedExtensionTypedNodeSchema {
    /// One entry per extension production.
    pub nodes: Vec<CheckedExtensionNodeSchema>,
}

/// One extension node kind and its ordered clauses.
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct CheckedExtensionNodeSchema {
    /// Leading source keyword.
    pub keyword: Box<str>,
    /// Checked node kind name.
    pub node_kind: Box<str>,
    /// Ordered clause types.
    pub clauses: Vec<CheckedExtensionClauseSchema>,
}

/// One clause of a validated extension production.
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct CheckedExtensionClauseSchema {
    /// Clause name.
    pub name: Box<str>,
    /// Core type-ref or extension node kind.
    #[serde(rename = "type")]
    pub type_ref: Box<str>,
}

/// Domain package documents, dependency packages and reader-supported
/// features for one read.
///
/// A domain package is supplied as its document bytes, keyed by the domain
/// package identity a `model_root` owner names; one document is held per
/// identity (FR-154 admission, FR-322 "Model-owned members" step 1).
///
/// A library dependency is supplied as its already admitted
/// [`CheckedPackageV2`] under the identity its `import` names (FR-322
/// `dependency_selections`); the reader binds it to the selection entry of
/// that identity and never reads a dependency's bytes itself.
#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct CheckedPackageEvidence {
    domain_packages: BTreeMap<Box<str>, Box<[u8]>>,
    dependency_packages: BTreeMap<Box<str>, Arc<CheckedPackageV2>>,
    extension_definitions: BTreeMap<CheckedArtifactRef, CheckedExtensionDefinition>,
    supported_features: BTreeSet<Box<str>>,
}

impl CheckedPackageEvidence {
    /// Supplies a caller-validated extension definition under its exact ref.
    /// Equal repeats retain the first definition; conflicting repeats fail.
    pub fn insert_extension_definition(
        &mut self,
        reference: CheckedArtifactRef,
        definition: CheckedExtensionDefinition,
    ) -> Result<(), CheckedPackageEvidenceInsertionError> {
        match self.extension_definitions.entry(reference) {
            Entry::Vacant(entry) => {
                entry.insert(definition);
                Ok(())
            }
            Entry::Occupied(entry) if entry.get() == &definition => Ok(()),
            Entry::Occupied(entry) => Err(
                CheckedPackageEvidenceInsertionError::ExtensionDefinitionConflict {
                    authority: entry.key().authority.clone(),
                    identity: entry.key().identity.clone(),
                },
            ),
        }
    }

    pub(super) fn extension_definition(
        &self,
        reference: &CheckedArtifactRef,
    ) -> Option<&CheckedExtensionDefinition> {
        self.extension_definitions.get(reference)
    }
    /// Starts with no documents, no dependency packages and no supported features.
    pub fn new() -> Self {
        Self::default()
    }

    /// Supplies one domain package document's bytes under the domain package
    /// identity a `model_root` owner names. One document is held per
    /// identity. The reader checks that the document's own `package.identity`
    /// equals the identity it was supplied under.
    ///
    /// An occupied exact key retains its first bytes: an equal raw-byte repeat
    /// succeeds, and different bytes return a keyed conflict without changing
    /// any evidence. Insertion performs no document admission or normalization.
    pub fn insert_domain_package_document(
        &mut self,
        identity: impl Into<Box<str>>,
        document: impl Into<Box<[u8]>>,
    ) -> Result<(), CheckedPackageEvidenceInsertionError> {
        let document = document.into();
        match self.domain_packages.entry(identity.into()) {
            Entry::Vacant(entry) => {
                entry.insert(document);
                Ok(())
            }
            Entry::Occupied(entry) => {
                if entry.get() == &document {
                    Ok(())
                } else {
                    Err(
                        CheckedPackageEvidenceInsertionError::DomainDocumentConflict {
                            identity: entry.key().clone(),
                        },
                    )
                }
            }
        }
    }

    /// Supplies one selected dependency's admitted package under the library
    /// `identity` its import names. One package is held per identity, as
    /// `dependency_selections` holds one entry per identity. An equal
    /// admitted-content repeat succeeds and retains the first package object;
    /// different content returns a keyed conflict without changing any evidence.
    /// Equality uses [`CheckedPackageV2::eq`], including retained wire metadata,
    /// rather than package identity or `Arc` address alone.
    ///
    /// `identity` is only the caller's claim. The reader finds the package by
    /// `identity` (an identity no entry names is never looked up, and an
    /// entry with no package under its identity refuses `missing_import`) and
    /// binds the package by its `package_id` content digest: a
    /// `CheckedPackageV2` can only be built by the reader's own `read`, so
    /// that digest was recomputed from its identity preimage. The package is
    /// shared, not cloned, when passed as an `Arc`.
    pub fn insert_dependency_package(
        &mut self,
        identity: impl Into<Box<str>>,
        package: impl Into<Arc<CheckedPackageV2>>,
    ) -> Result<(), CheckedPackageEvidenceInsertionError> {
        let package = package.into();
        match self.dependency_packages.entry(identity.into()) {
            Entry::Vacant(entry) => {
                entry.insert(package);
                Ok(())
            }
            Entry::Occupied(entry) => {
                if entry.get().as_ref() == package.as_ref() {
                    Ok(())
                } else {
                    Err(
                        CheckedPackageEvidenceInsertionError::DependencyPackageConflict {
                            identity: entry.key().clone(),
                        },
                    )
                }
            }
        }
    }

    /// The admitted package supplied for a library identity.
    pub(super) fn dependency_package(&self, identity: &str) -> Option<&CheckedPackageV2> {
        self.dependency_packages.get(identity).map(AsRef::as_ref)
    }

    pub(super) fn retained_dependency(&self, identity: &str) -> Option<Arc<CheckedPackageV2>> {
        self.dependency_packages.get(identity).cloned()
    }

    pub(super) fn domain_package_document(&self, identity: &str) -> Option<&[u8]> {
        self.domain_packages.get(identity).map(AsRef::as_ref)
    }

    /// Declares one exact required feature this reader implements.
    /// Repeating the same string is idempotent; distinct strings remain distinct.
    pub fn support_feature(&mut self, feature: impl Into<Box<str>>) {
        self.supported_features.insert(feature.into());
    }

    pub(super) fn supports(&self, feature: &str) -> bool {
        self.supported_features.contains(feature)
    }
}
