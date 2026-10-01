//! Authoritative evidence consulted by the version dispatcher and V2 reader.

use super::v2::CheckedPackageV2;
use std::collections::{BTreeMap, BTreeSet};
use std::sync::Arc;

/// Domain package documents, dependency packages and reader-supported
/// features for one read.
///
/// A domain package is supplied as its document bytes, keyed by the
/// `sha256-jcs` digest the caller supplies them under; the reader recomputes
/// that digest itself (FR-154 admission, FR-322 "Model-owned members"
/// step 1). The lock's raw-artifact digests are not checked against any
/// evidence: the package's `package_id` is the content identity.
///
/// A library dependency is supplied as its already admitted
/// [`CheckedPackageV2`] under the identity its `import` names (FR-322
/// `dependency_selections`); the reader binds it to the selection entry of
/// that identity and never reads a dependency's bytes itself.
#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct CheckedPackageEvidence {
    domain_packages: BTreeMap<Box<str>, Box<[u8]>>,
    dependency_packages: BTreeMap<Box<str>, Arc<CheckedPackageV2>>,
    supported_features: BTreeSet<Box<str>>,
}

impl CheckedPackageEvidence {
    /// Starts with no documents, no dependency packages and no supported features.
    pub fn new() -> Self {
        Self::default()
    }

    /// Supplies one domain package document's bytes under the lowercase
    /// `sha256-jcs` digest a `model_selections` row names. The reader
    /// recomputes the digest from the bytes, so a document supplied under
    /// another document's digest is refused, not trusted.
    pub fn insert_domain_package_document(
        &mut self,
        digest: impl Into<Box<str>>,
        document: impl Into<Box<[u8]>>,
    ) {
        self.domain_packages.insert(digest.into(), document.into());
    }

    /// Supplies one selected dependency's admitted package under the library
    /// `identity` its import names. One package is held per identity, as
    /// `dependency_selections` holds one entry per identity; a second call
    /// for an identity replaces the first.
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
    ) {
        self.dependency_packages
            .insert(identity.into(), package.into());
    }

    /// The admitted package supplied for a library identity.
    pub(super) fn dependency_package(&self, identity: &str) -> Option<&CheckedPackageV2> {
        self.dependency_packages.get(identity).map(AsRef::as_ref)
    }

    pub(super) fn domain_package_document(&self, digest: &str) -> Option<&[u8]> {
        self.domain_packages.get(digest).map(AsRef::as_ref)
    }

    /// Declares one required feature this reader implements.
    pub fn support_feature(&mut self, feature: impl Into<Box<str>>) {
        self.supported_features.insert(feature.into());
    }

    pub(super) fn supports(&self, feature: &str) -> bool {
        self.supported_features.contains(feature)
    }
}
