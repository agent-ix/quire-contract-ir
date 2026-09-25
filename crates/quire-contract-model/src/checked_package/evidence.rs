//! Authoritative evidence consulted by the version dispatcher and V2 reader.

use super::common::{digest_bytes, ArtifactDigests};
use super::shared::CheckedArtifactLocator;
use std::borrow::Cow;
use std::collections::{BTreeMap, BTreeSet};

/// Raw-artifact digests, domain package documents and reader-supported
/// features for one read.
///
/// A raw-artifact digest is either computed here from supplied bytes or
/// attested by a verified digest store the caller owns. A domain package is
/// supplied as its document bytes, keyed by the `sha256-jcs` digest the
/// caller supplies them under; the reader recomputes that digest itself
/// (FR-154 admission, FR-322 "Model-owned members" step 1). Staleness is
/// always an exact comparison of the package lock against this evidence.
#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct CheckedPackageEvidence {
    artifacts: BTreeMap<CheckedArtifactLocator, Box<str>>,
    domain_packages: BTreeMap<Box<str>, Box<[u8]>>,
    supported_features: BTreeSet<Box<str>>,
}

impl CheckedPackageEvidence {
    /// Starts with no artifacts and no supported features.
    pub fn new() -> Self {
        Self::default()
    }

    /// Records the SHA-256 of immutable bytes under their complete locator.
    pub fn insert_artifact_bytes(&mut self, locator: CheckedArtifactLocator, bytes: &[u8]) {
        self.artifacts
            .insert(locator, digest_bytes(bytes).into_boxed_str());
    }

    /// Records a lowercase SHA-256 digest attested by a verified digest store.
    pub fn insert_artifact_digest(
        &mut self,
        locator: CheckedArtifactLocator,
        digest: impl Into<Box<str>>,
    ) {
        self.artifacts.insert(locator, digest.into());
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

impl ArtifactDigests for CheckedPackageEvidence {
    fn artifact_digest(&self, locator: &CheckedArtifactLocator) -> Option<Cow<'_, str>> {
        self.artifacts
            .get(locator)
            .map(|digest| Cow::Borrowed(digest.as_ref()))
    }
}
