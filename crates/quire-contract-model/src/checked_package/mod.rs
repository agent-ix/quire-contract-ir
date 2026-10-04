//! QSpec I04 checked-package consumption.
//!
//! - [`shared`] holds the version-neutral limits, refusal/incomplete
//!   outcomes, and locked-artifact vocabulary the reader and its lowering
//!   pipeline both use.
//! - `terms` is the shared iterative walk over the terms of a body, on the
//!   `quire-walk` crate's heap stack: every walk of the reader over a term enters
//!   it in document pre-order and never recurses on the call stack, so the reader
//!   has no depth limit and no stack that follows the input.
//! - [`v2`] is the sole admitted `quire.checked-package/v2` reader, nominal
//!   identity re-derivation, and per-item lowering.
//! - [`dispatch`] parses untrusted bytes once, reads `contract_version`, and
//!   either admits the current contract or refuses any other version with a
//!   typed [`CheckedPackageRefusalCode::UnknownContractVersion`] code.

mod common;
mod dispatch;
mod evidence;
mod shared;
mod terms;
mod v2;

pub use dispatch::*;
pub use evidence::*;
pub use shared::*;
pub use v2::*;
