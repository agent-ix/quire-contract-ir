//! Compatibility bridge for the Quire contract intermediate representation.
//!
//! The stable semantic substrate lives in [`quire_contract_model`]. This
//! package re-exports that API so existing `quire_contract_ir` imports keep
//! their source and type identity while owner integrations remain downstream
//! of the cycle-free model package.

#![forbid(unsafe_code)]

pub mod bridge;
pub mod ecosystem_model;
pub mod predicate;
pub mod temporal;

pub use quire_contract_model::*;
