//! Authoritative evidence consulted by the version dispatcher and V2 reader.

use super::common::{digest_bytes, ArtifactDigests};
use super::v1::CheckedArtifactLocator;
use std::borrow::Cow;
use std::collections::{BTreeMap, BTreeSet};

/// Raw-artifact digests and reader-supported features for one read.
///
/// A digest is either computed here from supplied bytes or attested by a
/// verified digest store the caller owns. Staleness is always an exact
/// comparison of the package lock against this evidence.
#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct CheckedPackageEvidence {
    artifacts: BTreeMap<CheckedArtifactLocator, Box<str>>,
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
