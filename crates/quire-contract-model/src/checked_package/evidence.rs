//! Authoritative evidence consulted by the version dispatcher and V2 reader.

use super::common::{digest_bytes, ArtifactDigests};
use super::shared::CheckedArtifactLocator;
use std::borrow::Cow;
use std::collections::{BTreeMap, BTreeSet};

/// Exact `(identity, version)` of a `sha256-jcs` domain package selected by a
/// V2 lock. Distinct from [`CheckedArtifactLocator`]: a domain package digest
/// is never looked up as, or substituted for, a raw byte digest.
#[derive(Clone, Debug, Eq, PartialEq, Ord, PartialOrd, Hash)]
pub struct CheckedDomainPackageLocator {
    /// Domain package identity.
    pub identity: Box<str>,
    /// Domain package version.
    pub version: Box<str>,
}

/// Raw-artifact digests, domain package digests and reader-supported
/// features for one read.
///
/// A digest is either computed here from supplied bytes or attested by a
/// verified digest store the caller owns. Staleness is always an exact
/// comparison of the package lock against this evidence.
#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct CheckedPackageEvidence {
    artifacts: BTreeMap<CheckedArtifactLocator, Box<str>>,
    domain_packages: BTreeMap<CheckedDomainPackageLocator, Box<str>>,
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

    /// Records the lowercase `sha256-jcs` digest of one domain package,
    /// attested by a verified store the caller owns.
    pub fn insert_domain_package_digest(
        &mut self,
        locator: CheckedDomainPackageLocator,
        digest: impl Into<Box<str>>,
    ) {
        self.domain_packages.insert(locator, digest.into());
    }

    pub(super) fn domain_package_digest(
        &self,
        locator: &CheckedDomainPackageLocator,
    ) -> Option<&str> {
        self.domain_packages.get(locator).map(AsRef::as_ref)
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
