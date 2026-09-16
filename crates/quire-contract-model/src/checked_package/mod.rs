//! QSpec I04 checked-package consumption.
//!
//! - [`v1`] is the frozen `quire.checked-package/v1` reader and lowering.
//! - [`v2`] is the distinct `quire.checked-package/v2` reader, nominal identity
//!   re-derivation and per-item lowering.
//! - [`dispatch`] parses untrusted bytes once and routes to exactly one
//!   version-specific decoder.
//! - [`migration`] re-links an admitted V1 package to an admitted V2 package.

mod common;
mod dispatch;
mod evidence;
mod migration;
mod v1;
mod v2;

pub use dispatch::*;
pub use evidence::*;
pub use migration::*;
pub use v1::*;
pub use v2::*;
