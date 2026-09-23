//! Authoritative evidence consulted by the version dispatcher and V2 reader.

use super::common::{digest_bytes, is_digest, ArtifactDigests};
use super::shared::CheckedArtifactLocator;
use std::borrow::Cow;
use std::collections::{btree_map::Entry, BTreeMap, BTreeSet};

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

/// Why an attestation was not recorded.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum EvidenceRefusal {
    /// The digest is not 64 lowercase hexadecimal characters.
    MalformedDigest,
    /// The locator already holds a different digest. Evidence attests at
    /// most one digest per locator, so the order attestations arrive in can
    /// never decide a staleness verdict.
    ConflictingAttestation,
}

/// Raw-artifact digests, domain package digests and reader-supported
/// features for one read.
///
/// A digest is either computed here from supplied bytes or attested by a
/// verified digest store the caller owns. Staleness is always an exact
/// comparison of the package lock against this evidence. Each locator holds
/// at most one digest: a second attestation of the same digest is accepted,
/// and a different one is refused rather than replacing the first.
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
    pub fn insert_artifact_bytes(
        &mut self,
        locator: CheckedArtifactLocator,
        bytes: &[u8],
    ) -> Result<(), EvidenceRefusal> {
        attest(&mut self.artifacts, locator, digest_bytes(bytes))
    }

    /// Records a lowercase SHA-256 digest attested by a verified digest store.
    pub fn insert_artifact_digest(
        &mut self,
        locator: CheckedArtifactLocator,
        digest: impl Into<Box<str>>,
    ) -> Result<(), EvidenceRefusal> {
        attest(&mut self.artifacts, locator, digest)
    }

    /// Records the lowercase `sha256-jcs` digest of one domain package,
    /// attested by a verified store the caller owns.
    pub fn insert_domain_package_digest(
        &mut self,
        locator: CheckedDomainPackageLocator,
        digest: impl Into<Box<str>>,
    ) -> Result<(), EvidenceRefusal> {
        attest(&mut self.domain_packages, locator, digest)
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

/// Records one attestation: a malformed digest or a different digest for an
/// already-attested locator is refused, and the same digest again is a no-op.
fn attest<L: Ord>(
    attested: &mut BTreeMap<L, Box<str>>,
    locator: L,
    digest: impl Into<Box<str>>,
) -> Result<(), EvidenceRefusal> {
    let digest = digest.into();
    if !is_digest(&digest) {
        return Err(EvidenceRefusal::MalformedDigest);
    }
    match attested.entry(locator) {
        Entry::Vacant(slot) => {
            slot.insert(digest);
            Ok(())
        }
        Entry::Occupied(held) if *held.get() == digest => Ok(()),
        Entry::Occupied(_) => Err(EvidenceRefusal::ConflictingAttestation),
    }
}

impl ArtifactDigests for CheckedPackageEvidence {
    fn artifact_digest(&self, locator: &CheckedArtifactLocator) -> Option<Cow<'_, str>> {
        self.artifacts
            .get(locator)
            .map(|digest| Cow::Borrowed(digest.as_ref()))
    }
}
