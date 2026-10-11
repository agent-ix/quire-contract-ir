//! Exact I04 contract-version refusal on the direct V2 byte reader.

use super::evidence::CheckedPackageEvidence;
use super::shared::{CheckedPackageIncomplete, CheckedPackageReadLimits, CheckedPackageRefusal};
use super::v2::{CheckedPackageV2, CheckedPackageV2ReadResult};

/// The closed result of dispatching untrusted checked-package bytes.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum CheckedPackageDispatchResult {
    /// A `quire.checked-package/v2` package admitted by the V2 reader.
    AdmittedV2(Box<CheckedPackageV2>),
    /// The input is invalid for its selected version, or selects none.
    Refused(CheckedPackageRefusal),
    /// The input exceeded a caller limit before a conclusion.
    Incomplete(CheckedPackageIncomplete),
}

/// Reads `contract_version` from borrowed source bytes after strict syntax and
/// canonical checks, then either admits the current contract or refuses any
/// other version with a typed
/// [`super::shared::CheckedPackageRefusalCode::UnknownContractVersion`] code. This is a
/// refusal control, not a compatibility layer: it never relabels or widens
/// the admitted contract.
pub fn read_checked_package(
    bytes: &[u8],
    limits: CheckedPackageReadLimits,
    evidence: &CheckedPackageEvidence,
) -> CheckedPackageDispatchResult {
    match CheckedPackageV2::read(bytes, limits, evidence) {
        CheckedPackageV2ReadResult::Admitted(package) => {
            CheckedPackageDispatchResult::AdmittedV2(package)
        }
        CheckedPackageV2ReadResult::Refused(refusal) => {
            CheckedPackageDispatchResult::Refused(refusal)
        }
        CheckedPackageV2ReadResult::Incomplete(incomplete) => {
            CheckedPackageDispatchResult::Incomplete(incomplete)
        }
    }
}
