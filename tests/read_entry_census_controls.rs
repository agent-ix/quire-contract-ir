// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (C) 2026 Agent-IX

//! Deliberate real-entry bypass controls for the compiler invocation census.
//! The bypass tests pass their original reader oracle but MUST fail the
//! separate compiler census; this binary is outside the default test suite.

#[path = "it/support/qspec_read_proof.rs"]
mod qspec_read_proof;
use qspec_read_proof::read_and_account;
use quire_contract_model::{
    CheckedPackageEvidence, CheckedPackageReadLimits, CheckedPackageRefusalCode, CheckedPackageV2,
    CheckedPackageV2ReadResult,
};

fn measure_qspec_read_requests(read: impl FnOnce()) -> u64 {
    allocation_counter::measure(read).bytes_total
}
fn without_qspec_instrumentation<T>(operation: impl FnOnce() -> T) -> T {
    let mut result = None;
    allocation_counter::opt_out(|| {
        result = Some(std::panic::catch_unwind(std::panic::AssertUnwindSafe(
            operation,
        )));
    });
    match result.expect("instrumentation runs once") {
        Ok(value) => value,
        Err(panic) => std::panic::resume_unwind(panic),
    }
}
fn malformed(result: CheckedPackageV2ReadResult) {
    let CheckedPackageV2ReadResult::Refused(refusal) = result else {
        panic!("the original malformed-input oracle must refuse");
    };
    assert_eq!(refusal.code, CheckedPackageRefusalCode::MalformedWire);
}
fn unwrapped_shared_helper(evidence: &CheckedPackageEvidence) {
    malformed(CheckedPackageV2::read(
        b"not json",
        CheckedPackageReadLimits::bounded(),
        evidence,
    ));
}

#[test]
fn wrapped_function_item_has_no_unaccounted_entry() {
    let evidence = CheckedPackageEvidence::new();
    let entry = read_and_account;
    malformed(entry(
        b"not json",
        CheckedPackageReadLimits::bounded(),
        &evidence,
    ));
}

#[test]
fn unwrapped_type_alias_before_trusted_read_must_fail_census() {
    type Alias = CheckedPackageV2;
    let evidence = CheckedPackageEvidence::new();
    malformed(Alias::read(
        b"not json",
        CheckedPackageReadLimits::bounded(),
        &evidence,
    ));
    malformed(read_and_account(
        b"not json",
        CheckedPackageReadLimits::bounded(),
        &evidence,
    ));
}

#[test]
fn unwrapped_function_item_after_trusted_read_must_fail_census() {
    let evidence = CheckedPackageEvidence::new();
    malformed(read_and_account(
        b"not json",
        CheckedPackageReadLimits::bounded(),
        &evidence,
    ));
    let entry = CheckedPackageV2::read;
    malformed(entry(
        b"not json",
        CheckedPackageReadLimits::bounded(),
        &evidence,
    ));
}

#[test]
fn unwrapped_shared_helper_before_trusted_read_must_fail_census() {
    let evidence = CheckedPackageEvidence::new();
    unwrapped_shared_helper(&evidence);
    malformed(read_and_account(
        b"not json",
        CheckedPackageReadLimits::bounded(),
        &evidence,
    ));
}

#[test]
fn unwrapped_shared_helper_after_trusted_read_must_fail_census() {
    let evidence = CheckedPackageEvidence::new();
    malformed(read_and_account(
        b"not json",
        CheckedPackageReadLimits::bounded(),
        &evidence,
    ));
    unwrapped_shared_helper(&evidence);
}
