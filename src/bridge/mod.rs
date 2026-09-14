//! Shared foundations for owner-consuming Contract IR bridges.

pub(crate) mod canonical;
mod contract;
mod diagnostic;
mod identity;
mod limits;

pub use contract::ContractSelection;
pub use diagnostic::{BridgeError, BridgeErrorCode};
pub use identity::{BridgeDigest, BridgeDigestParseError};
pub use limits::BridgeLimits;
