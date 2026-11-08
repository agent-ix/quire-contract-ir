// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (C) 2026 Agent-IX

//! Compiler root only: every input stays opaque and caller-owned. This adapter
//! returns the production reader's real typed outcome without a fixture,
//! alternate decoder, observer, or result projection.

use quire_contract_model::{
    CheckedPackageEvidence, CheckedPackageReadLimits, CheckedPackageV2,
    CheckedPackageV2BudgetedReadResult, NativeJoinBudget,
};

#[inline(never)]
pub fn static_reader_root(
    bytes: &[u8],
    limits: CheckedPackageReadLimits,
    evidence: &CheckedPackageEvidence,
    budget: &mut NativeJoinBudget,
) -> CheckedPackageV2BudgetedReadResult {
    CheckedPackageV2::read_with_budget(bytes, limits, evidence, budget)
}
