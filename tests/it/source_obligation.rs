// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (C) 2026 Agent-IX

//! FR-052 provisional input tests. TC-452's complete owner oracle is unrun.

use ix_trace_rs::trace;
use quire_contract_model::source_obligation::{
    SourceAssertionId, SourceAssertionProvenance, SourceContractAuthorId,
    SourceCorrespondenceAssertion, SourceDeclarationHandle, SourceExpressionHandle,
    SourceInventoryCompleteness, SourceInventoryEnumerator, SourceInventoryEnvelope,
    SourceInventoryIdentity, SourceInventoryIncompleteReason, SourceInventoryOutcome,
    SourceInventoryOutcomeError, SourceInventoryOwner, SourceInventoryRow, SourceInventoryScope,
    SourceObligationInput, SourceObligationSite, SourcePendingObligationId,
    SourcePendingObligationKind, SourceProductionFunctionId, SourceProductionModuleId,
    SourceRequirementOccurrenceId, SourceSemanticOccurrenceId, SourceTypeExhaustion,
    SourceTypeWorkDimension, SourceUnitHandle,
};
use quire_contract_model::{
    CheckedNodeId, CheckedOccurrence, CheckedOccurrenceRole, CheckedPackageIncomplete,
    CheckedPackageLimit, ClauseId, ClauseRef, PackageId, RequirementId, RequirementRef,
    RequirementRevision, SourceDocumentId, SourceIdentity, SourceRevision,
};

fn source() -> SourceIdentity {
    SourceIdentity::new(
        SourceDocumentId::new("selected-source").expect("valid source document"),
        SourceRevision::new(1).expect("valid source revision"),
    )
}

fn site() -> SourceObligationSite {
    SourceObligationSite {
        declaration: SourceDeclarationHandle(1),
        unit: SourceUnitHandle(2),
        expression: Some(SourceExpressionHandle(3)),
        start: 10,
        end: 20,
    }
}

fn pending(
    site: SourceObligationSite,
    kind: SourcePendingObligationKind,
) -> SourcePendingObligationId {
    SourcePendingObligationId { site, kind }
}

fn criterion() -> ClauseRef {
    ClauseRef::new(
        RequirementRef::new(
            PackageId::new("example.contract").expect("valid package"),
            RequirementId::new("REQ_source_obligation").expect("valid requirement"),
            RequirementRevision::new(1).expect("valid revision"),
        ),
        ClauseId::new("AC_1").expect("valid criterion"),
    )
}

fn occurrence() -> SourceSemanticOccurrenceId {
    SourceSemanticOccurrenceId::new(
        CheckedNodeId {
            domain: "quire.checked-semantic-node/v1".into(),
            digest: "a".repeat(64).into(),
        },
        CheckedOccurrence {
            role: CheckedOccurrenceRole::Expression,
            ordinal: 0,
        },
    )
}

/// Trace: FR-052-AC-8
#[trace("FR-052-AC-8")]
#[test]
fn provisional_input_retains_owner_rows_and_authored_assertions_without_a_census() {
    let selected = source();
    let qsl_identity = SourceInventoryIdentity::RequirementRecord(SourceRequirementOccurrenceId {
        node: [1; 32],
        role: "expression".into(),
        ordinal: 0,
    });
    let qsl_row = SourceInventoryRow::new(selected.clone(), qsl_identity.clone());
    let qsl = SourceInventoryEnvelope::new(
        SourceInventoryOwner::Qsl,
        selected.clone(),
        SourceInventoryScope::QslCheckedRequirementsAndPendingObligations,
        vec![qsl_row],
        None,
    );
    let function =
        SourceProductionFunctionId::new(SourceProductionModuleId::new("src/lib.rs"), "check");
    let assertion = SourceCorrespondenceAssertion::new(
        SourceAssertionId::new("link-1"),
        selected.clone(),
        criterion(),
        occurrence(),
        Some(function.clone()),
        SourceAssertionProvenance {
            source: selected.clone(),
            author: SourceContractAuthorId::new("author"),
        },
    );
    let input =
        SourceObligationInput::new(selected.clone(), None, Some(qsl), None, vec![assertion]);
    assert_eq!(input.source(), &selected);
    assert!(input.ir_v2().is_none());
    assert!(input.quire_code_rs().is_none());
    let report = input.qsl().expect("QSL report was supplied");
    assert_eq!(report.owner(), SourceInventoryOwner::Qsl);
    assert_eq!(report.source(), &selected);
    assert_eq!(
        report.scope(),
        SourceInventoryScope::QslCheckedRequirementsAndPendingObligations
    );
    assert!(report.stop_reason().is_none());
    assert_eq!(report.items().len(), 1);
    assert_eq!(report.items()[0].owner(), SourceInventoryOwner::Qsl);
    assert_eq!(report.items()[0].source(), &selected);
    assert_eq!(report.items()[0].identity(), &qsl_identity);
    assert_eq!(input.assertions().len(), 1);
    assert_eq!(input.assertions()[0].production_function(), Some(&function));
}

/// Trace: FR-052-AC-8
#[trace("FR-052-AC-8")]
#[test]
fn native_qsl_identity_fields_and_stop_causes_remain_distinct() {
    let selected = source();
    let base = SourceRequirementOccurrenceId {
        node: [1; 32],
        role: "expression".into(),
        ordinal: 0,
    };
    let mut different_node = base.clone();
    different_node.node = [2; 32];
    let mut different_role = base.clone();
    different_role.role = "claim".into();
    let mut different_ordinal = base.clone();
    different_ordinal.ordinal = 1;
    let requirement_keys = [base, different_node, different_role, different_ordinal];
    assert_eq!(
        requirement_keys
            .iter()
            .collect::<std::collections::BTreeSet<_>>()
            .len(),
        4
    );

    let kind = SourcePendingObligationKind::PredicateTotal {
        target: SourceDeclarationHandle(7),
    };
    let base = pending(site(), kind);
    let mut changed_declaration = base;
    changed_declaration.site.declaration = SourceDeclarationHandle(4);
    let mut changed_unit = base;
    changed_unit.site.unit = SourceUnitHandle(5);
    let mut changed_expression = base;
    changed_expression.site.expression = None;
    let mut changed_start = base;
    changed_start.site.start = 11;
    let mut changed_end = base;
    changed_end.site.end = 21;
    let pending_keys = [
        base,
        changed_declaration,
        changed_unit,
        changed_expression,
        changed_start,
        changed_end,
        pending(
            site(),
            SourcePendingObligationKind::PredicateTotal {
                target: SourceDeclarationHandle(8),
            },
        ),
        pending(site(), SourcePendingObligationKind::Presence),
    ];
    assert_eq!(
        pending_keys
            .iter()
            .collect::<std::collections::BTreeSet<_>>()
            .len(),
        8
    );

    let exhaustion = SourceInventoryIncompleteReason::TypeWorkExhausted(SourceTypeExhaustion {
        dimension: SourceTypeWorkDimension::Records,
        limit: 4,
        prior: 4,
        requested: 1,
        site: site(),
    });
    let unfinished = SourceInventoryIncompleteReason::TypeAdmissionUnfinished {
        declaration: SourceDeclarationHandle(1),
    };
    let report = SourceInventoryEnvelope::new(
        SourceInventoryOwner::Qsl,
        selected.clone(),
        SourceInventoryScope::QslCheckedRequirementsAndPendingObligations,
        pending_keys
            .into_iter()
            .map(|identity| {
                SourceInventoryRow::new(
                    selected.clone(),
                    SourceInventoryIdentity::PendingObligation(identity),
                )
            })
            .collect(),
        Some(exhaustion.clone()),
    );
    assert_eq!(report.items().len(), 8);
    assert_eq!(report.stop_reason(), Some(&exhaustion));
    assert_ne!(report.stop_reason(), Some(&unfinished));
}

/// Trace: FR-052-AC-4, FR-052-AC-8
#[trace("FR-052-AC-4", "FR-052-AC-8")]
#[test]
fn provisional_input_has_no_result_or_completeness_fields() {
    let source = include_str!("../../crates/quire-contract-model/src/source_obligation.rs");
    let input = source
        .split("pub struct SourceObligationInput {")
        .nth(1)
        .expect("input declaration")
        .split('}')
        .next()
        .expect("input fields");
    for forbidden in [
        "result:",
        "method:",
        "profile:",
        "artifact:",
        "attempt:",
        "run:",
        "completeness:",
        "item_count:",
    ] {
        assert!(
            !input.contains(forbidden),
            "forbidden input field {forbidden}"
        );
    }
    let envelope = source
        .split("pub struct SourceInventoryEnvelope {")
        .nth(1)
        .expect("envelope declaration")
        .split('}')
        .next()
        .expect("envelope fields");
    assert!(!envelope.contains("completeness:"));
    assert!(!envelope.contains("item_count:"));
    assert!(!source.contains("pub struct SourceObligationCensus"));
    assert!(!source.contains("pub fn build_source_obligation_census"));
}

// These synthetic enumerators exercise the neutral retention boundary. They
// do not stand in for QSL or quire-code-rs owner-output conformance.
struct TestEnumerator {
    source: SourceIdentity,
    owner: SourceInventoryOwner,
    scope: SourceInventoryScope,
    rows: std::vec::IntoIter<SourceInventoryRow>,
    observed: usize,
    reported_count: Option<usize>,
    stop: Option<SourceInventoryIncompleteReason>,
}

impl TestEnumerator {
    fn new(owner: SourceInventoryOwner, rows: Vec<SourceInventoryRow>) -> Self {
        let scope = match owner {
            SourceInventoryOwner::IrV2 => SourceInventoryScope::IrV2SemanticGraphAndOccurrences,
            SourceInventoryOwner::Qsl => {
                SourceInventoryScope::QslCheckedRequirementsAndPendingObligations
            }
            SourceInventoryOwner::QuireCodeRs => {
                SourceInventoryScope::QuireCodeRsProductionModulesAndFunctions
            }
        };
        Self {
            source: source(),
            owner,
            scope,
            rows: rows.into_iter(),
            observed: 0,
            reported_count: None,
            stop: None,
        }
    }
}

impl SourceInventoryEnumerator for TestEnumerator {
    fn source(&self) -> &SourceIdentity {
        &self.source
    }
    fn owner(&self) -> SourceInventoryOwner {
        self.owner
    }
    fn scope(&self) -> SourceInventoryScope {
        self.scope
    }
    fn next_row(&mut self) -> Result<Option<SourceInventoryRow>, SourceInventoryIncompleteReason> {
        if let Some(row) = self.rows.next() {
            self.observed += 1;
            Ok(Some(row))
        } else if let Some(cause) = self.stop.take() {
            Err(cause)
        } else {
            Ok(None)
        }
    }
    fn observed_count(&self) -> usize {
        self.reported_count.unwrap_or(self.observed)
    }
}

fn module_row(name: &str) -> SourceInventoryRow {
    SourceInventoryRow::new(
        source(),
        SourceInventoryIdentity::ProductionModule(SourceProductionModuleId::new(name)),
    )
}

/// Trace: FR-052-AC-11
#[trace("FR-052-AC-11")]
#[test]
fn terminal_boundary_distinguishes_empty_complete_partial_and_invalid_count() {
    let empty = SourceInventoryOutcome::from_enumerator(TestEnumerator::new(
        SourceInventoryOwner::QuireCodeRs,
        vec![],
    ))
    .expect("normal empty end");
    assert_eq!(empty.completeness(), &SourceInventoryCompleteness::Complete);
    assert_eq!(empty.observed_count(), 0);
    let row = module_row("src/empty.rs");
    let complete = SourceInventoryOutcome::from_enumerator(TestEnumerator::new(
        SourceInventoryOwner::QuireCodeRs,
        vec![row.clone()],
    ))
    .expect("module without functions");
    assert_eq!(complete.items(), std::slice::from_ref(&row));
    assert_eq!(complete.observed_count(), 1);

    let mut deleted = TestEnumerator::new(SourceInventoryOwner::QuireCodeRs, vec![row]);
    deleted.reported_count = Some(2);
    assert_eq!(
        SourceInventoryOutcome::from_enumerator(deleted),
        Err(SourceInventoryOutcomeError::ObservedCountMismatch {
            observed: 2,
            retained: 1,
        })
    );
    let provisional = SourceInventoryEnvelope::new(
        SourceInventoryOwner::QuireCodeRs,
        source(),
        SourceInventoryScope::QuireCodeRsProductionModulesAndFunctions,
        vec![],
        None,
    );
    assert!(provisional.stop_reason().is_none());
    assert!(provisional.items().is_empty());
}

/// Trace: FR-052-AC-10, FR-052-AC-11
#[trace("FR-052-AC-10", "FR-052-AC-11")]
#[test]
fn terminal_boundary_preserves_typed_stops_and_rejects_mismatched_rows() {
    let exhaustion = SourceInventoryIncompleteReason::TypeWorkExhausted(SourceTypeExhaustion {
        dimension: SourceTypeWorkDimension::Records,
        limit: 1,
        prior: 1,
        requested: 1,
        site: site(),
    });
    let qsl_row = SourceInventoryRow::new(
        source(),
        SourceInventoryIdentity::PendingObligation(pending(
            site(),
            SourcePendingObligationKind::Presence,
        )),
    );
    for cause in [
        exhaustion,
        SourceInventoryIncompleteReason::TypeAdmissionUnfinished {
            declaration: SourceDeclarationHandle(7),
        },
    ] {
        let mut enumerator = TestEnumerator::new(SourceInventoryOwner::Qsl, vec![qsl_row.clone()]);
        enumerator.stop = Some(cause.clone());
        let outcome = SourceInventoryOutcome::from_enumerator(enumerator).expect("typed stop");
        assert_eq!(outcome.items(), std::slice::from_ref(&qsl_row));
        assert_eq!(
            outcome.completeness(),
            &SourceInventoryCompleteness::Incomplete(cause)
        );
    }
    let limit = CheckedPackageIncomplete {
        limit_kind: CheckedPackageLimit::Nodes,
        limit: 3,
        consumed: 4,
        path: Some(
            quire_contract_model::JsonPointer::parse("/semantic_graph/nodes/3").expect("pointer"),
        ),
    };
    let mut ir = TestEnumerator::new(SourceInventoryOwner::IrV2, vec![]);
    ir.stop = Some(SourceInventoryIncompleteReason::IrV2ReadLimit(
        limit.clone(),
    ));
    assert_eq!(
        SourceInventoryOutcome::from_enumerator(ir)
            .expect("IR limit")
            .completeness(),
        &SourceInventoryCompleteness::Incomplete(SourceInventoryIncompleteReason::IrV2ReadLimit(
            limit
        ))
    );
    assert_eq!(
        SourceInventoryOutcome::from_enumerator(TestEnumerator::new(
            SourceInventoryOwner::IrV2,
            vec![qsl_row],
        )),
        Err(SourceInventoryOutcomeError::RowOwnerMismatch)
    );
    let mut wrong_scope = TestEnumerator::new(SourceInventoryOwner::Qsl, vec![]);
    wrong_scope.scope = SourceInventoryScope::IrV2SemanticGraphAndOccurrences;
    assert_eq!(
        SourceInventoryOutcome::from_enumerator(wrong_scope),
        Err(SourceInventoryOutcomeError::OwnerScopeMismatch)
    );
    let wrong_source = TestEnumerator::new(
        SourceInventoryOwner::QuireCodeRs,
        vec![SourceInventoryRow::new(
            SourceIdentity::new(
                SourceDocumentId::new("another-source").expect("document"),
                SourceRevision::new(1).expect("revision"),
            ),
            SourceInventoryIdentity::ProductionModule(SourceProductionModuleId::new("src/lib.rs")),
        )],
    );
    assert_eq!(
        SourceInventoryOutcome::from_enumerator(wrong_source),
        Err(SourceInventoryOutcomeError::RowSourceMismatch)
    );
    let mut wrong_stop = TestEnumerator::new(SourceInventoryOwner::IrV2, vec![]);
    wrong_stop.stop = Some(SourceInventoryIncompleteReason::TypeAdmissionUnfinished {
        declaration: SourceDeclarationHandle(7),
    });
    assert_eq!(
        SourceInventoryOutcome::from_enumerator(wrong_stop),
        Err(SourceInventoryOutcomeError::StopOwnerMismatch)
    );
}
