//! Exact I04 contract-version selection before any version-specific decode.

use super::common::{canonical_value, Stop};
use super::evidence::CheckedPackageEvidence;
use super::v1::{
    CheckedPackage, CheckedPackageIncomplete, CheckedPackageReadLimits, CheckedPackageRefusal,
    CheckedPackageRefusalCode, CHECKED_PACKAGE_V1,
};
use super::v2::{CheckedPackageV2, CHECKED_PACKAGE_V2};
use serde_json::Value;

/// The closed result of dispatching untrusted checked-package bytes.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum CheckedPackageDispatchResult {
    /// A `quire.checked-package/v1` package admitted by the frozen V1 reader.
    AdmittedV1(Box<CheckedPackage>),
    /// A `quire.checked-package/v2` package admitted by the V2 reader.
    AdmittedV2(Box<CheckedPackageV2>),
    /// The input is invalid for its selected version, or selects none.
    Refused(CheckedPackageRefusal),
    /// The input exceeded a caller limit before a conclusion.
    Incomplete(CheckedPackageIncomplete),
}

/// Parses bytes once, reads `contract_version`, and routes to exactly one
/// strict decoder. Neither decoder relabels or upgrades the other version.
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

fn dispatch(
    bytes: &[u8],
    limits: CheckedPackageReadLimits,
    evidence: &CheckedPackageEvidence,
) -> Result<CheckedPackageDispatchResult, Stop> {
    let value = canonical_value(bytes, limits)?;
    let version = match &value {
        Value::Object(members) => match members.get("contract_version") {
            Some(Value::String(version)) => version.clone(),
            _ => {
                return Err(Stop::refused(
                    CheckedPackageRefusalCode::MalformedWire,
                    "contract_version",
                ))
            }
        },
        _ => {
            return Err(Stop::refused(
                CheckedPackageRefusalCode::MalformedWire,
                "document",
            ))
        }
    };
    match version.as_str() {
        CHECKED_PACKAGE_V1 => CheckedPackage::admit_value(value, limits, evidence)
            .map(|package| CheckedPackageDispatchResult::AdmittedV1(Box::new(package))),
        CHECKED_PACKAGE_V2 => CheckedPackageV2::admit_value(value, limits, evidence)
            .map(|package| CheckedPackageDispatchResult::AdmittedV2(Box::new(package))),
        _ => Err(Stop::refused(
            CheckedPackageRefusalCode::UnknownContractVersion,
            "contract_version",
        )),
    }
}
