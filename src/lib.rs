//! Versioned bounded-Kani interface for the Quire contract intermediate representation.
//!
//! The semantic model is owned by the separate `quire_contract_model` crate.
//!
//! A model item is available through the model crate:
//!
//! ```
//! use quire_contract_model::SchemaVersion;
//! let _ = SchemaVersion::V1_1;
//! ```
//!
//! The root crate does not export that item (FR-019-AC-5, TC-058):
//!
//! ```compile_fail,E0432
//! use quire_contract_ir::SchemaVersion;
//! ```

#![forbid(unsafe_code)]

pub mod kani;
