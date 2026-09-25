//! Exact I04 contract-version refusal before any decode.

use super::common::{canonical_value, ValidationFailure};
use super::evidence::CheckedPackageEvidence;
use super::shared::{
    CheckedPackageIncomplete, CheckedPackageReadLimits, CheckedPackageRefusal,
    CheckedPackageRefusalCode, JsonPointer,
};
use super::v2::{CheckedPackageV2, CHECKED_PACKAGE_V2};
use serde_json::Value;

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

/// Parses bytes once, reads `contract_version` once, and either admits the
/// current contract or refuses any other version with a typed
/// [`CheckedPackageRefusalCode::UnknownContractVersion`] code. This is a
/// refusal control, not a compatibility layer: it never relabels or widens
/// the admitted contract.
pub fn read_checked_package(
    bytes: &[u8],
    limits: CheckedPackageReadLimits,
    evidence: &CheckedPackageEvidence,
) -> CheckedPackageDispatchResult {
    match dispatch(bytes, limits, evidence) {
        Ok(result) => result,
        Err(stop) => stop.into_result(
            CheckedPackageDispatchResult::Refused,
            CheckedPackageDispatchResult::Incomplete,
        ),
    }
}

// string-edge: the reader's version dispatch: the one read of `contract_version`.
fn dispatch(
    bytes: &[u8],
    limits: CheckedPackageReadLimits,
    evidence: &CheckedPackageEvidence,
) -> Result<CheckedPackageDispatchResult, ValidationFailure> {
    let value = canonical_value(bytes, limits)?;
    let version = match &value {
        Value::Object(members) => match members.get("contract_version") {
            Some(Value::String(version)) => version.clone(),
            // Present with the wrong kind: the member is the value at fault.
            Some(_) => {
                return Err(ValidationFailure::refused(
                    CheckedPackageRefusalCode::MalformedWire,
                    JsonPointer::root().key("contract_version"),
                ))
            }
            // Absent: the document lacks it.
            None => {
                return Err(ValidationFailure::refused(
                    CheckedPackageRefusalCode::MalformedWire,
                    JsonPointer::root(),
                ))
            }
        },
        _ => {
            return Err(ValidationFailure::refused(
                CheckedPackageRefusalCode::MalformedWire,
                JsonPointer::root(),
            ))
        }
    };
    match version.as_str() {
        CHECKED_PACKAGE_V2 => CheckedPackageV2::admit_value(value, limits, evidence)
            .map(|package| CheckedPackageDispatchResult::AdmittedV2(Box::new(package))),
        _ => Err(ValidationFailure::unknown_contract_version(&version)),
    }
}
