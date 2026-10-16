// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (C) 2026 Agent-IX

//! FR-052 neutral outcome and synthetic census tests.

use ix_trace_rs::trace;
use quire_contract_model::source_obligation::{
    build_source_obligation_census, SourceAssertionId, SourceAssertionProvenance,
    SourceAuthoredCriterionId, SourceContractAuthorId, SourceCorrespondenceAssertion,
    SourceCorrespondenceError, SourceCorrespondenceRejectionReason, SourceCriterionSourceBinding,
    SourceDeclarationHandle, SourceExpressionHandle, SourceInventoryCompleteness,
    SourceInventoryDisposition, SourceInventoryEnumerator, SourceInventoryIdentity,
    SourceInventoryIncompleteReason, SourceInventoryOutcome, SourceInventoryOutcomeError,
    SourceInventoryOwner, SourceInventoryRow, SourceInventoryScope, SourceMissingMappingReason,
    SourceObligationInput, SourceObligationSite, SourcePendingObligationId,
    SourcePendingObligationKind, SourceProductionFunctionId, SourceProductionModuleId,
    SourceRequirementOccurrenceId, SourceSemanticOccurrenceId, SourceTypeExhaustion,
    SourceTypeWorkDimension, SourceUnitHandle,
};
use quire_contract_model::{
    CheckedNodeId, CheckedOccurrence, CheckedOccurrenceRole, CheckedPackageIncomplete,
    CheckedPackageLimit, SourceDocumentId, SourceIdentity, SourceRevision,
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

fn criterion(id: &str) -> SourceAuthoredCriterionId {
    SourceAuthoredCriterionId::new("obligations", "spec/model/FR-052.md", id)
}

fn binding(selected: SourceIdentity) -> SourceCriterionSourceBinding {
    SourceCriterionSourceBinding::new(
        "selected-corpus",
        "traceability-model",
        "obligations",
        selected,
        SourceContractAuthorId::new("author"),
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

/// Trace: FR-052-AC-14, TC-452
#[trace("TC-452", "FR-052-AC-14")]
#[test]
fn quire_criteria_and_selection_binding_remain_typed_and_distinct() {
    let selected = source();
    let first = criterion("AC-1");
    let second = criterion("AC-2");
    let rows = vec![
        SourceInventoryRow::new(
            selected.clone(),
            SourceInventoryIdentity::Criterion(first.clone()),
        ),
        SourceInventoryRow::new(
            selected.clone(),
            SourceInventoryIdentity::Criterion(second.clone()),
        ),
    ];
    assert_ne!(first, second);
    assert_eq!(first.obligation_declaration(), "obligations");
    assert_eq!(first.scoped_document(), "spec/model/FR-052.md");
    assert_eq!(first.criterion_id(), "AC-1");
    assert_eq!(rows[0].owner(), SourceInventoryOwner::Quire);
    assert_eq!(rows[1].owner(), SourceInventoryOwner::Quire);
    assert_ne!(rows[0].identity(), rows[1].identity());

    let quire = SourceInventoryOutcome::from_enumerator(TestEnumerator::new(
        SourceInventoryOwner::Quire,
        rows.clone(),
    ))
    .expect("Quire outcome");
    let link = SourceCorrespondenceAssertion::new(
        SourceAssertionId::new("link-1"),
        selected.clone(),
        first.clone(),
        occurrence(),
        None,
        SourceAssertionProvenance {
            source: selected.clone(),
            author: SourceContractAuthorId::new("author"),
        },
    );
    let chosen = binding(selected.clone());
    let input = SourceObligationInput::new(
        selected.clone(),
        Some(chosen.clone()),
        Some(quire),
        None,
        None,
        None,
        vec![link],
    );
    assert_eq!(input.criterion_source_binding(), Some(&chosen));
    assert_eq!(chosen.quire_corpus(), "selected-corpus");
    assert_eq!(chosen.traceability_model(), "traceability-model");
    assert_eq!(chosen.obligation_scope(), "obligations");
    assert_eq!(chosen.source(), &selected);
    assert_eq!(chosen.author().as_str(), "author");
    assert_eq!(input.quire().expect("Quire report").items(), rows);
    assert_eq!(input.assertions()[0].criterion(), &first);
    assert_ne!(input.assertions()[0].criterion(), &second);
}

/// Trace: FR-052-AC-14, TC-452
#[trace("TC-452", "FR-052-AC-14")]
#[test]
fn quire_outcome_rejects_other_owner_scope_and_rows() {
    let selected = source();
    let row = SourceInventoryRow::new(
        selected.clone(),
        SourceInventoryIdentity::Criterion(criterion("AC-1")),
    );
    let outcome = SourceInventoryOutcome::from_enumerator(TestEnumerator::new(
        SourceInventoryOwner::Quire,
        vec![row.clone()],
    ))
    .expect("synthetic Quire outcome");
    assert_eq!(
        outcome.scope(),
        SourceInventoryScope::QuireSelectedAuthoredCriteria
    );
    assert_eq!(outcome.items(), &[row]);

    let mut wrong_scope = TestEnumerator::new(SourceInventoryOwner::Quire, vec![]);
    wrong_scope.scope = SourceInventoryScope::QslCheckedRequirementsAndPendingObligations;
    assert_eq!(
        SourceInventoryOutcome::from_enumerator(wrong_scope),
        Err(SourceInventoryOutcomeError::OwnerScopeMismatch)
    );
    let qsl_row = SourceInventoryRow::new(
        selected,
        SourceInventoryIdentity::RequirementRecord(SourceRequirementOccurrenceId {
            node: [1; 32],
            role: "expression".into(),
            ordinal: 0,
        }),
    );
    assert_eq!(
        SourceInventoryOutcome::from_enumerator(TestEnumerator::new(
            SourceInventoryOwner::Quire,
            vec![qsl_row],
        )),
        Err(SourceInventoryOutcomeError::RowOwnerMismatch)
    );
}

/// Trace: FR-052-AC-12, FR-052-AC-13
#[trace("FR-052-AC-12", "FR-052-AC-13")]
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
    let qsl_rows = pending_keys
        .into_iter()
        .map(|identity| {
            SourceInventoryRow::new(
                selected.clone(),
                SourceInventoryIdentity::PendingObligation(identity),
            )
        })
        .collect();
    let mut enumerator = TestEnumerator::new(SourceInventoryOwner::Qsl, qsl_rows);
    enumerator.stop = Some(exhaustion.clone());
    let report = SourceInventoryOutcome::from_enumerator(enumerator).expect("QSL outcome");
    assert_eq!(report.items().len(), 8);
    assert_eq!(
        report.completeness(),
        &SourceInventoryCompleteness::Incomplete(exhaustion)
    );
    assert_ne!(
        report.completeness(),
        &SourceInventoryCompleteness::Incomplete(unfinished)
    );
}

/// Trace: FR-052-AC-4
#[trace("FR-052-AC-4")]
#[test]
fn census_input_has_no_result_or_completeness_fields() {
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
            SourceInventoryOwner::Quire => SourceInventoryScope::QuireSelectedAuthoredCriteria,
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

/// Trace: FR-052-AC-12, FR-052-AC-13
#[trace("FR-052-AC-12", "FR-052-AC-13")]
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

    let rows = vec![row.clone(), module_row("src/another.rs")];
    let multiple = SourceInventoryOutcome::from_enumerator(TestEnumerator::new(
        SourceInventoryOwner::QuireCodeRs,
        rows.clone(),
    ))
    .expect("normal end after two emitted rows");
    assert_eq!(
        multiple.completeness(),
        &SourceInventoryCompleteness::Complete
    );
    assert_eq!(multiple.items(), rows.as_slice());
    assert_eq!(multiple.observed_count(), 2);

    let mut deleted = TestEnumerator::new(SourceInventoryOwner::QuireCodeRs, vec![row]);
    deleted.reported_count = Some(2);
    assert_eq!(
        SourceInventoryOutcome::from_enumerator(deleted),
        Err(SourceInventoryOutcomeError::ObservedCountMismatch {
            observed: 2,
            retained: 1,
        })
    );
}

/// Trace: FR-052-AC-12, FR-052-AC-13
#[trace("FR-052-AC-12", "FR-052-AC-13", "FR-052-AC-3")]
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

fn census_outcome(
    owner: SourceInventoryOwner,
    rows: Vec<SourceInventoryRow>,
) -> SourceInventoryOutcome {
    SourceInventoryOutcome::from_enumerator(TestEnumerator::new(owner, rows))
        .expect("synthetic outcome")
}

fn semantic(id: char) -> quire_contract_model::CheckedSemanticId {
    quire_contract_model::CheckedSemanticId {
        domain: "semantic/v1".into(),
        algorithm: "sha256".into(),
        digest: id.to_string().repeat(64).into(),
    }
}

fn census_input(assertions: Vec<SourceCorrespondenceAssertion>) -> SourceObligationInput {
    let selected = source();
    let occ = occurrence();
    let module = SourceProductionModuleId::new("src/lib.rs");
    let function = SourceProductionFunctionId::new(module.clone(), "check");
    let second_function = SourceProductionFunctionId::new(module.clone(), "verify");
    SourceObligationInput::new(
        selected.clone(),
        Some(binding(selected.clone())),
        Some(census_outcome(
            SourceInventoryOwner::Quire,
            vec![
                SourceInventoryRow::new(
                    selected.clone(),
                    SourceInventoryIdentity::Criterion(criterion("AC-1")),
                ),
                SourceInventoryRow::new(
                    selected.clone(),
                    SourceInventoryIdentity::Criterion(criterion("AC-1")),
                ),
                SourceInventoryRow::new(
                    selected.clone(),
                    SourceInventoryIdentity::Criterion(criterion("AC-2")),
                ),
            ],
        )),
        Some(census_outcome(
            SourceInventoryOwner::IrV2,
            vec![
                SourceInventoryRow::new(
                    selected.clone(),
                    SourceInventoryIdentity::SemanticNode {
                        identity: occ.node().clone(),
                        semantic: semantic('a'),
                    },
                ),
                SourceInventoryRow::new(
                    selected.clone(),
                    SourceInventoryIdentity::SemanticOccurrence(occ),
                ),
                SourceInventoryRow::new(
                    selected.clone(),
                    SourceInventoryIdentity::SemanticOccurrence(occurrence_at(1)),
                ),
                SourceInventoryRow::new(
                    selected.clone(),
                    SourceInventoryIdentity::SemanticOccurrence(occurrence_at(2)),
                ),
            ],
        )),
        Some(census_outcome(
            SourceInventoryOwner::Qsl,
            vec![
                SourceInventoryRow::new(
                    selected.clone(),
                    SourceInventoryIdentity::RequirementRecord(SourceRequirementOccurrenceId {
                        node: [1; 32],
                        role: "expression".into(),
                        ordinal: 0,
                    }),
                ),
                SourceInventoryRow::new(
                    selected.clone(),
                    SourceInventoryIdentity::PendingObligation(pending(
                        site(),
                        SourcePendingObligationKind::Presence,
                    )),
                ),
            ],
        )),
        Some(census_outcome(
            SourceInventoryOwner::QuireCodeRs,
            vec![
                SourceInventoryRow::new(
                    selected.clone(),
                    SourceInventoryIdentity::ProductionModule(module),
                ),
                SourceInventoryRow::new(
                    selected.clone(),
                    SourceInventoryIdentity::ProductionModule(SourceProductionModuleId::new(
                        "src/empty.rs",
                    )),
                ),
                SourceInventoryRow::new(
                    selected,
                    SourceInventoryIdentity::ProductionFunction(function),
                ),
                SourceInventoryRow::new(
                    source(),
                    SourceInventoryIdentity::ProductionFunction(second_function),
                ),
            ],
        )),
        assertions,
    )
}

fn occurrence_at(ordinal: u64) -> SourceSemanticOccurrenceId {
    SourceSemanticOccurrenceId::new(
        occurrence().node().clone(),
        CheckedOccurrence {
            role: CheckedOccurrenceRole::Expression,
            ordinal,
        },
    )
}

fn census_assertion(
    id: &str,
    criterion_id: &str,
    function: Option<SourceProductionFunctionId>,
) -> SourceCorrespondenceAssertion {
    SourceCorrespondenceAssertion::new(
        SourceAssertionId::new(id),
        source(),
        criterion(criterion_id),
        occurrence(),
        function,
        SourceAssertionProvenance {
            source: source(),
            author: SourceContractAuthorId::new("author"),
        },
    )
}

/// Trace: FR-052-AC-17, TC-452
#[trace("TC-452", "FR-052-AC-17", "FR-052-AC-3")]
#[test]
fn synthetic_census_has_hand_written_rows_and_dispositions() {
    let function =
        SourceProductionFunctionId::new(SourceProductionModuleId::new("src/lib.rs"), "check");
    let second =
        SourceProductionFunctionId::new(SourceProductionModuleId::new("src/lib.rs"), "verify");
    let mk = |id, ordinal, function| {
        SourceCorrespondenceAssertion::new(
            SourceAssertionId::new(id),
            source(),
            criterion("AC-1"),
            occurrence_at(ordinal),
            function,
            SourceAssertionProvenance {
                source: source(),
                author: SourceContractAuthorId::new("author"),
            },
        )
    };
    let census = build_source_obligation_census(census_input(vec![
        mk("a", 0, Some(function.clone())),
        mk("b", 1, Some(second.clone())),
        mk("c", 2, None),
        mk("d", 0, Some(second.clone())),
    ]))
    .expect("census");
    use SourceInventoryDisposition::{Mapped, ZeroMappings};
    assert_eq!(census.source(), &source());
    assert_eq!(
        census.criterion_source_binding().obligation_scope(),
        "obligations"
    );
    assert_eq!(
        census
            .inventory_rows()
            .iter()
            .map(|r| (r.row().identity().clone(), r.disposition()))
            .collect::<Vec<_>>(),
        vec![
            (
                SourceInventoryIdentity::Criterion(criterion("AC-1")),
                Mapped
            ),
            (
                SourceInventoryIdentity::Criterion(criterion("AC-2")),
                ZeroMappings(SourceMissingMappingReason::NoAuthoredMapping)
            ),
            (
                SourceInventoryIdentity::SemanticNode {
                    identity: occurrence().node().clone(),
                    semantic: semantic('a')
                },
                Mapped
            ),
            (
                SourceInventoryIdentity::SemanticOccurrence(occurrence()),
                Mapped
            ),
            (
                SourceInventoryIdentity::SemanticOccurrence(occurrence_at(1)),
                Mapped
            ),
            (
                SourceInventoryIdentity::SemanticOccurrence(occurrence_at(2)),
                ZeroMappings(SourceMissingMappingReason::ExplicitNoProductionFunction)
            ),
            (
                SourceInventoryIdentity::RequirementRecord(SourceRequirementOccurrenceId {
                    node: [1; 32],
                    role: "expression".into(),
                    ordinal: 0
                }),
                ZeroMappings(SourceMissingMappingReason::NoAuthoredMapping)
            ),
            (
                SourceInventoryIdentity::PendingObligation(pending(
                    site(),
                    SourcePendingObligationKind::Presence
                )),
                ZeroMappings(SourceMissingMappingReason::NoAuthoredMapping)
            ),
            (
                SourceInventoryIdentity::ProductionModule(SourceProductionModuleId::new(
                    "src/empty.rs"
                )),
                ZeroMappings(SourceMissingMappingReason::NoAuthoredMapping)
            ),
            (
                SourceInventoryIdentity::ProductionModule(SourceProductionModuleId::new(
                    "src/lib.rs"
                )),
                Mapped
            ),
            (
                SourceInventoryIdentity::ProductionFunction(function.clone()),
                Mapped
            ),
            (
                SourceInventoryIdentity::ProductionFunction(second.clone()),
                Mapped
            ),
        ]
    );
    assert!(census
        .inventory_rows()
        .iter()
        .all(|r| r.row().source() == &source()));
    assert_eq!(
        census
            .correspondence_rows()
            .iter()
            .map(|r| (
                r.criterion().clone(),
                r.occurrence().clone(),
                r.production_function().cloned(),
                r.provenance().clone()
            ))
            .collect::<Vec<_>>(),
        vec![
            (
                criterion("AC-1"),
                occurrence_at(0),
                Some(function),
                SourceAssertionProvenance {
                    source: source(),
                    author: SourceContractAuthorId::new("author")
                }
            ),
            (
                criterion("AC-1"),
                occurrence_at(0),
                Some(second.clone()),
                SourceAssertionProvenance {
                    source: source(),
                    author: SourceContractAuthorId::new("author")
                }
            ),
            (
                criterion("AC-1"),
                occurrence_at(1),
                Some(second),
                SourceAssertionProvenance {
                    source: source(),
                    author: SourceContractAuthorId::new("author")
                }
            ),
            (
                criterion("AC-1"),
                occurrence_at(2),
                None,
                SourceAssertionProvenance {
                    source: source(),
                    author: SourceContractAuthorId::new("author")
                }
            ),
        ]
    );
    assert!(census.rejected_assertions().is_empty());
}

/// Trace: FR-052-AC-15, TC-452
#[trace("TC-452", "FR-052-AC-15")]
#[test]
fn binding_refusals_are_typed() {
    let base = census_input(vec![]);
    let input = SourceObligationInput::new(
        source(),
        None,
        base.quire().cloned(),
        base.ir_v2().cloned(),
        base.qsl().cloned(),
        base.quire_code_rs().cloned(),
        vec![],
    );
    assert_eq!(
        build_source_obligation_census(input),
        Err(SourceCorrespondenceError::MissingCriterionSourceBinding)
    );
    let foreign = SourceIdentity::new(
        SourceDocumentId::new("other").unwrap(),
        SourceRevision::new(1).unwrap(),
    );
    let input = census_input(vec![]);
    let input = SourceObligationInput::new(
        source(),
        Some(binding(foreign.clone())),
        input.quire().cloned(),
        input.ir_v2().cloned(),
        input.qsl().cloned(),
        input.quire_code_rs().cloned(),
        vec![],
    );
    assert_eq!(
        build_source_obligation_census(input),
        Err(SourceCorrespondenceError::ForeignCriterionSourceBinding {
            expected: source(),
            observed: foreign
        })
    );
}

/// Trace: FR-052-AC-22, TC-452
#[trace("TC-452", "FR-052-AC-22")]
#[test]
fn assertion_conflicts_and_relation_dedup_are_deterministic() {
    let a = census_assertion("a", "AC-1", None);
    let b = census_assertion("b", "AC-1", None);
    let census =
        build_source_obligation_census(census_input(vec![a.clone(), b.clone(), a])).unwrap();
    assert_eq!(census.correspondence_rows().len(), 1);
    assert_eq!(
        census.inventory_rows()[2].disposition(),
        SourceInventoryDisposition::ZeroMappings(
            SourceMissingMappingReason::ExplicitNoProductionFunction
        )
    );
    assert_eq!(
        census.inventory_rows()[3].disposition(),
        SourceInventoryDisposition::ZeroMappings(
            SourceMissingMappingReason::ExplicitNoProductionFunction
        )
    );
    let b = SourceCorrespondenceAssertion::new(
        SourceAssertionId::new("b"),
        source(),
        criterion("AC-1"),
        occurrence(),
        None,
        SourceAssertionProvenance {
            source: source(),
            author: SourceContractAuthorId::new("other"),
        },
    );
    let census =
        build_source_obligation_census(census_input(vec![census_assertion("a", "AC-1", None), b]))
            .unwrap();
    assert!(census.correspondence_rows().is_empty());
    assert_eq!(
        census
            .rejected_assertions()
            .iter()
            .map(|r| r.reason())
            .collect::<Vec<_>>(),
        vec![SourceCorrespondenceRejectionReason::ConflictingProvenance; 2]
    );
}

fn with_positions(
    binding_value: Option<SourceCriterionSourceBinding>,
    quire: Option<SourceInventoryOutcome>,
    ir: Option<SourceInventoryOutcome>,
    qsl: Option<SourceInventoryOutcome>,
    code: Option<SourceInventoryOutcome>,
    assertions: Vec<SourceCorrespondenceAssertion>,
) -> SourceObligationInput {
    SourceObligationInput::new(source(), binding_value, quire, ir, qsl, code, assertions)
}

/// Trace: FR-052-AC-6, TC-452
#[trace("TC-452", "FR-052-AC-6")]
#[test]
fn every_owner_position_refuses_missing_and_typed_incomplete_outcomes() {
    let base = census_input(vec![]);
    let owners = [
        SourceInventoryOwner::Quire,
        SourceInventoryOwner::IrV2,
        SourceInventoryOwner::Qsl,
        SourceInventoryOwner::QuireCodeRs,
    ];
    for owner in owners {
        let positions = |replacement: Option<SourceInventoryOutcome>| {
            with_positions(
                Some(binding(source())),
                if owner == SourceInventoryOwner::Quire {
                    replacement.clone()
                } else {
                    base.quire().cloned()
                },
                if owner == SourceInventoryOwner::IrV2 {
                    replacement.clone()
                } else {
                    base.ir_v2().cloned()
                },
                if owner == SourceInventoryOwner::Qsl {
                    replacement.clone()
                } else {
                    base.qsl().cloned()
                },
                if owner == SourceInventoryOwner::QuireCodeRs {
                    replacement
                } else {
                    base.quire_code_rs().cloned()
                },
                vec![],
            )
        };
        assert_eq!(
            build_source_obligation_census(positions(None)),
            Err(SourceCorrespondenceError::MissingOwnerInventory { owner })
        );
        let causes = match owner {
            SourceInventoryOwner::Quire => {
                vec![SourceInventoryIncompleteReason::EarlyStop { owner }]
            }
            SourceInventoryOwner::IrV2 => vec![
                SourceInventoryIncompleteReason::EarlyStop { owner },
                SourceInventoryIncompleteReason::IrV2ReadLimit(CheckedPackageIncomplete {
                    limit_kind: CheckedPackageLimit::Nodes,
                    limit: 3,
                    consumed: 4,
                    path: Some(
                        quire_contract_model::JsonPointer::parse("/semantic_graph/nodes/3")
                            .unwrap(),
                    ),
                }),
            ],
            SourceInventoryOwner::Qsl => vec![
                SourceInventoryIncompleteReason::TypeWorkExhausted(SourceTypeExhaustion {
                    dimension: SourceTypeWorkDimension::Records,
                    limit: 4,
                    prior: 4,
                    requested: 1,
                    site: site(),
                }),
                SourceInventoryIncompleteReason::TypeAdmissionUnfinished {
                    declaration: SourceDeclarationHandle(7),
                },
            ],
            SourceInventoryOwner::QuireCodeRs => {
                vec![SourceInventoryIncompleteReason::ModuleUnavailable {
                    module: SourceProductionModuleId::new("src/empty.rs"),
                    cause: "unavailable".into(),
                }]
            }
        };
        for cause in causes {
            let mut enumerator = TestEnumerator::new(owner, vec![]);
            enumerator.stop = Some(cause.clone());
            let incomplete = SourceInventoryOutcome::from_enumerator(enumerator).unwrap();
            assert_eq!(
                build_source_obligation_census(positions(Some(incomplete))),
                Err(SourceCorrespondenceError::IncompleteOwnerInventory { owner, cause })
            );
        }
    }
    let empty_module = module_row("src/empty.rs");
    let complete = census_outcome(
        SourceInventoryOwner::QuireCodeRs,
        vec![empty_module.clone()],
    );
    let census = build_source_obligation_census(with_positions(
        Some(binding(source())),
        base.quire().cloned(),
        base.ir_v2().cloned(),
        base.qsl().cloned(),
        Some(complete),
        vec![],
    ))
    .unwrap();
    assert!(census.inventory_rows().iter().any(|row| {
        row.row() == &empty_module
            && row.disposition()
                == SourceInventoryDisposition::ZeroMappings(
                    SourceMissingMappingReason::NoAuthoredMapping,
                )
    }));
}

/// Trace: FR-052-AC-2, TC-452
#[trace("TC-452", "FR-052-AC-2")]
#[test]
fn synthetic_many_to_many_and_unmapped_inventory_survive_permutation_and_row_removal() {
    let first =
        SourceProductionFunctionId::new(SourceProductionModuleId::new("src/lib.rs"), "check");
    let second =
        SourceProductionFunctionId::new(SourceProductionModuleId::new("src/lib.rs"), "verify");
    let assertion = |id, ordinal, function| {
        SourceCorrespondenceAssertion::new(
            SourceAssertionId::new(id),
            source(),
            criterion("AC-1"),
            occurrence_at(ordinal),
            Some(function),
            SourceAssertionProvenance {
                source: source(),
                author: SourceContractAuthorId::new("author"),
            },
        )
    };
    let assertions = vec![
        assertion("a", 0, first.clone()),
        assertion("b", 0, second.clone()),
        assertion("c", 1, second.clone()),
    ];
    let base = census_input(vec![]);
    let build = |ir: SourceInventoryOutcome, statements: Vec<SourceCorrespondenceAssertion>| {
        build_source_obligation_census(with_positions(
            Some(binding(source())),
            base.quire().cloned(),
            Some(ir),
            base.qsl().cloned(),
            base.quire_code_rs().cloned(),
            statements,
        ))
        .unwrap()
    };
    let census = build(base.ir_v2().unwrap().clone(), assertions.clone());
    assert_eq!(census.inventory_rows().len(), 12);
    assert_eq!(
        census
            .inventory_rows()
            .iter()
            .map(|row| row.row().identity().clone())
            .collect::<Vec<_>>(),
        vec![
            SourceInventoryIdentity::Criterion(criterion("AC-1")),
            SourceInventoryIdentity::Criterion(criterion("AC-2")),
            SourceInventoryIdentity::SemanticNode {
                identity: occurrence().node().clone(),
                semantic: semantic('a')
            },
            SourceInventoryIdentity::SemanticOccurrence(occurrence_at(0)),
            SourceInventoryIdentity::SemanticOccurrence(occurrence_at(1)),
            SourceInventoryIdentity::SemanticOccurrence(occurrence_at(2)),
            SourceInventoryIdentity::RequirementRecord(SourceRequirementOccurrenceId {
                node: [1; 32],
                role: "expression".into(),
                ordinal: 0
            }),
            SourceInventoryIdentity::PendingObligation(pending(
                site(),
                SourcePendingObligationKind::Presence
            )),
            SourceInventoryIdentity::ProductionModule(SourceProductionModuleId::new(
                "src/empty.rs"
            )),
            SourceInventoryIdentity::ProductionModule(SourceProductionModuleId::new("src/lib.rs")),
            SourceInventoryIdentity::ProductionFunction(first.clone()),
            SourceInventoryIdentity::ProductionFunction(second.clone()),
        ]
    );
    assert_eq!(
        census
            .correspondence_rows()
            .iter()
            .map(|row| (
                row.criterion().clone(),
                row.occurrence().clone(),
                row.production_function().cloned(),
                row.provenance().clone(),
            ))
            .collect::<Vec<_>>(),
        vec![
            (
                criterion("AC-1"),
                occurrence_at(0),
                Some(first.clone()),
                SourceAssertionProvenance {
                    source: source(),
                    author: SourceContractAuthorId::new("author")
                }
            ),
            (
                criterion("AC-1"),
                occurrence_at(0),
                Some(second.clone()),
                SourceAssertionProvenance {
                    source: source(),
                    author: SourceContractAuthorId::new("author")
                }
            ),
            (
                criterion("AC-1"),
                occurrence_at(1),
                Some(second.clone()),
                SourceAssertionProvenance {
                    source: source(),
                    author: SourceContractAuthorId::new("author")
                }
            ),
        ]
    );
    let permute = |owner, value: &SourceInventoryOutcome| {
        census_outcome(owner, value.items().iter().cloned().rev().collect())
    };
    let mut reversed = assertions.clone();
    reversed.reverse();
    let reordered = build_source_obligation_census(with_positions(
        Some(binding(source())),
        Some(permute(SourceInventoryOwner::Quire, base.quire().unwrap())),
        Some(permute(SourceInventoryOwner::IrV2, base.ir_v2().unwrap())),
        Some(permute(SourceInventoryOwner::Qsl, base.qsl().unwrap())),
        Some(permute(
            SourceInventoryOwner::QuireCodeRs,
            base.quire_code_rs().unwrap(),
        )),
        reversed,
    ))
    .unwrap();
    assert_eq!(census, reordered);
    let reduced_rows = base
        .ir_v2()
        .unwrap()
        .items()
        .iter()
        .filter(|row| {
            row.identity() != &SourceInventoryIdentity::SemanticOccurrence(occurrence_at(2))
        })
        .cloned()
        .collect();
    let reduced = build(
        census_outcome(SourceInventoryOwner::IrV2, reduced_rows),
        assertions,
    );
    assert_ne!(census, reduced);
    assert!(!reduced
        .inventory_rows()
        .iter()
        .any(|row| row.row().identity()
            == &SourceInventoryIdentity::SemanticOccurrence(occurrence_at(2))));

    let sparse = build(
        base.ir_v2().unwrap().clone(),
        vec![assertion("only", 0, first)],
    );
    for identity in [
        SourceInventoryIdentity::Criterion(criterion("AC-2")),
        SourceInventoryIdentity::SemanticOccurrence(occurrence_at(1)),
        SourceInventoryIdentity::ProductionModule(SourceProductionModuleId::new("src/empty.rs")),
        SourceInventoryIdentity::ProductionFunction(second),
    ] {
        assert!(sparse
            .inventory_rows()
            .iter()
            .any(|row| row.row().identity() == &identity
                && row.disposition()
                    == SourceInventoryDisposition::ZeroMappings(
                        SourceMissingMappingReason::NoAuthoredMapping
                    )));
    }
}

fn incomplete(owner: SourceInventoryOwner) -> SourceInventoryOutcome {
    let mut e = TestEnumerator::new(owner, vec![]);
    e.stop = Some(SourceInventoryIncompleteReason::EarlyStop { owner });
    SourceInventoryOutcome::from_enumerator(e).unwrap()
}

fn incomplete_conflicting_quire() -> SourceInventoryOutcome {
    let row = SourceInventoryRow::new(
        source(),
        SourceInventoryIdentity::Criterion(SourceAuthoredCriterionId::new(
            "other-scope",
            "doc",
            "AC-0",
        )),
    );
    let mut e = TestEnumerator::new(SourceInventoryOwner::Quire, vec![row]);
    e.stop = Some(SourceInventoryIncompleteReason::EarlyStop {
        owner: SourceInventoryOwner::Quire,
    });
    SourceInventoryOutcome::from_enumerator(e).unwrap()
}

/// Trace: FR-052-AC-15, FR-052-AC-18, TC-452
#[trace("TC-452", "FR-052-AC-15", "FR-052-AC-18")]
#[test]
fn adjacent_census_refusals_follow_fixed_position_order() {
    let base = census_input(vec![]);
    let foreign = SourceIdentity::new(
        SourceDocumentId::new("other").unwrap(),
        SourceRevision::new(1).unwrap(),
    );
    let cases = vec![
        (
            Some(binding(foreign.clone())),
            Some(incomplete(SourceInventoryOwner::Quire)),
            base.ir_v2().cloned(),
            base.qsl().cloned(),
            base.quire_code_rs().cloned(),
            SourceCorrespondenceError::ForeignCriterionSourceBinding {
                expected: source(),
                observed: foreign,
            },
        ),
        (
            Some(binding(source())),
            Some(incomplete_conflicting_quire()),
            None,
            base.qsl().cloned(),
            base.quire_code_rs().cloned(),
            SourceCorrespondenceError::IncompleteOwnerInventory {
                owner: SourceInventoryOwner::Quire,
                cause: SourceInventoryIncompleteReason::EarlyStop {
                    owner: SourceInventoryOwner::Quire,
                },
            },
        ),
        (
            Some(binding(source())),
            base.quire().cloned(),
            None,
            None,
            base.quire_code_rs().cloned(),
            SourceCorrespondenceError::MissingOwnerInventory {
                owner: SourceInventoryOwner::IrV2,
            },
        ),
        (
            Some(binding(source())),
            base.quire().cloned(),
            base.ir_v2().cloned(),
            Some(incomplete(SourceInventoryOwner::Qsl)),
            None,
            SourceCorrespondenceError::IncompleteOwnerInventory {
                owner: SourceInventoryOwner::Qsl,
                cause: SourceInventoryIncompleteReason::EarlyStop {
                    owner: SourceInventoryOwner::Qsl,
                },
            },
        ),
    ];
    for (binding_value, quire, ir, qsl, code, expected) in cases {
        assert_eq!(
            build_source_obligation_census(with_positions(
                binding_value,
                quire,
                ir,
                qsl,
                code,
                vec![]
            )),
            Err(expected)
        );
    }
    let conflict = SourceInventoryRow::new(
        source(),
        SourceInventoryIdentity::Criterion(SourceAuthoredCriterionId::new(
            "other-scope",
            "doc",
            "AC-0",
        )),
    );
    let conflicting_quire = census_outcome(SourceInventoryOwner::Quire, vec![conflict]);
    assert_eq!(
        build_source_obligation_census(with_positions(
            Some(binding(source())),
            Some(conflicting_quire),
            None,
            base.qsl().cloned(),
            base.quire_code_rs().cloned(),
            vec![]
        )),
        Err(
            SourceCorrespondenceError::ConflictingCriterionSourceBinding {
                binding_scope: "obligations".into(),
                criterion: SourceAuthoredCriterionId::new("other-scope", "doc", "AC-0")
            }
        )
    );
}

/// Trace: FR-052-AC-20, TC-452
#[trace("TC-452", "FR-052-AC-20")]
#[test]
fn input_permutations_produce_equal_ordered_census() {
    let mk = |id, ordinal, function| {
        SourceCorrespondenceAssertion::new(
            SourceAssertionId::new(id),
            source(),
            criterion("AC-1"),
            occurrence_at(ordinal),
            function,
            SourceAssertionProvenance {
                source: source(),
                author: SourceContractAuthorId::new("author"),
            },
        )
    };
    let a = mk(
        "a",
        0,
        Some(SourceProductionFunctionId::new(
            SourceProductionModuleId::new("src/lib.rs"),
            "check",
        )),
    );
    let b = mk(
        "b",
        1,
        Some(SourceProductionFunctionId::new(
            SourceProductionModuleId::new("src/lib.rs"),
            "verify",
        )),
    );
    let c = mk("c", 2, None);
    let original =
        build_source_obligation_census(census_input(vec![a.clone(), b.clone(), c.clone()]))
            .unwrap();
    let base = census_input(vec![]);
    let reverse = |owner, outcome: &SourceInventoryOutcome| {
        census_outcome(owner, outcome.items().iter().cloned().rev().collect())
    };
    let permuted = build_source_obligation_census(with_positions(
        Some(binding(source())),
        Some(reverse(SourceInventoryOwner::Quire, base.quire().unwrap())),
        Some(reverse(SourceInventoryOwner::IrV2, base.ir_v2().unwrap())),
        Some(reverse(SourceInventoryOwner::Qsl, base.qsl().unwrap())),
        Some(reverse(
            SourceInventoryOwner::QuireCodeRs,
            base.quire_code_rs().unwrap(),
        )),
        vec![c, b, a],
    ))
    .unwrap();
    assert_eq!(original, permuted);
}

/// Trace: FR-052-AC-21, TC-452
#[trace("TC-452", "FR-052-AC-21")]
#[test]
fn owner_scope_and_source_errors_precede_incompleteness() {
    let base = census_input(vec![]);
    for (owner, expected) in [
        (
            SourceInventoryOwner::IrV2,
            SourceInventoryScope::IrV2SemanticGraphAndOccurrences,
        ),
        (
            SourceInventoryOwner::Qsl,
            SourceInventoryScope::QslCheckedRequirementsAndPendingObligations,
        ),
        (
            SourceInventoryOwner::QuireCodeRs,
            SourceInventoryScope::QuireCodeRsProductionModulesAndFunctions,
        ),
    ] {
        let wrong = Some(incomplete(SourceInventoryOwner::Quire));
        let (ir, qsl, code) = match owner {
            SourceInventoryOwner::IrV2 => {
                (wrong, base.qsl().cloned(), base.quire_code_rs().cloned())
            }
            SourceInventoryOwner::Qsl => {
                (base.ir_v2().cloned(), wrong, base.quire_code_rs().cloned())
            }
            SourceInventoryOwner::QuireCodeRs => {
                (base.ir_v2().cloned(), base.qsl().cloned(), wrong)
            }
            SourceInventoryOwner::Quire => unreachable!(),
        };
        assert_eq!(
            build_source_obligation_census(with_positions(
                Some(binding(source())),
                base.quire().cloned(),
                ir,
                qsl,
                code,
                vec![],
            )),
            Err(SourceCorrespondenceError::InventoryScopeMismatch {
                owner,
                expected,
                observed: SourceInventoryScope::QuireSelectedAuthoredCriteria,
            })
        );
    }
    assert_eq!(
        build_source_obligation_census(with_positions(
            Some(binding(source())),
            Some(incomplete(SourceInventoryOwner::Qsl)),
            base.ir_v2().cloned(),
            base.qsl().cloned(),
            base.quire_code_rs().cloned(),
            vec![]
        )),
        Err(SourceCorrespondenceError::InventoryScopeMismatch {
            owner: SourceInventoryOwner::Quire,
            expected: SourceInventoryScope::QuireSelectedAuthoredCriteria,
            observed: SourceInventoryScope::QslCheckedRequirementsAndPendingObligations
        })
    );
    let other = SourceIdentity::new(
        SourceDocumentId::new("other").unwrap(),
        SourceRevision::new(1).unwrap(),
    );
    let mut wrong_both = TestEnumerator::new(SourceInventoryOwner::Qsl, vec![]);
    wrong_both.source = other.clone();
    wrong_both.stop = Some(SourceInventoryIncompleteReason::EarlyStop {
        owner: SourceInventoryOwner::Qsl,
    });
    assert_eq!(
        build_source_obligation_census(with_positions(
            Some(binding(source())),
            Some(SourceInventoryOutcome::from_enumerator(wrong_both).unwrap()),
            base.ir_v2().cloned(),
            base.qsl().cloned(),
            base.quire_code_rs().cloned(),
            vec![]
        )),
        Err(SourceCorrespondenceError::InventoryScopeMismatch {
            owner: SourceInventoryOwner::Quire,
            expected: SourceInventoryScope::QuireSelectedAuthoredCriteria,
            observed: SourceInventoryScope::QslCheckedRequirementsAndPendingObligations,
        })
    );
    let mut e = TestEnumerator::new(SourceInventoryOwner::Quire, vec![]);
    e.source = other.clone();
    e.stop = Some(SourceInventoryIncompleteReason::EarlyStop {
        owner: SourceInventoryOwner::Quire,
    });
    let foreign = SourceInventoryOutcome::from_enumerator(e).unwrap();
    assert_eq!(
        build_source_obligation_census(with_positions(
            Some(binding(source())),
            Some(foreign),
            base.ir_v2().cloned(),
            base.qsl().cloned(),
            base.quire_code_rs().cloned(),
            vec![]
        )),
        Err(SourceCorrespondenceError::InventorySourceMismatch {
            owner: SourceInventoryOwner::Quire,
            expected: source(),
            observed: other
        })
    );
}

/// Trace: FR-052-AC-19, FR-052-AC-22, TC-452
#[trace("TC-452", "FR-052-AC-19", "FR-052-AC-22", "FR-052-AC-3")]
#[test]
fn rejected_assertions_collapse_duplicates_and_sort_with_reason_precedence() {
    let foreign_source = SourceIdentity::new(
        SourceDocumentId::new("other").unwrap(),
        SourceRevision::new(1).unwrap(),
    );
    let provenance_foreign = SourceCorrespondenceAssertion::new(
        SourceAssertionId::new("z"),
        source(),
        criterion("AC-1"),
        occurrence(),
        None,
        SourceAssertionProvenance {
            source: foreign_source,
            author: SourceContractAuthorId::new("author"),
        },
    );
    let both_dangling = SourceCorrespondenceAssertion::new(
        SourceAssertionId::new("b"),
        source(),
        criterion("missing"),
        occurrence_at(99),
        None,
        SourceAssertionProvenance {
            source: source(),
            author: SourceContractAuthorId::new("author"),
        },
    );
    let repeated = census_assertion("a", "missing", None);
    let conflict = SourceCorrespondenceAssertion::new(
        SourceAssertionId::new("d"),
        source(),
        criterion("AC-1"),
        occurrence(),
        None,
        SourceAssertionProvenance {
            source: source(),
            author: SourceContractAuthorId::new("other"),
        },
    );
    let census = build_source_obligation_census(census_input(vec![
        provenance_foreign,
        both_dangling,
        census_assertion("c", "AC-1", None),
        conflict,
        repeated.clone(),
        repeated,
    ]))
    .unwrap();
    assert_eq!(
        census
            .rejected_assertions()
            .iter()
            .map(|r| (r.assertion().id().as_str(), r.reason()))
            .collect::<Vec<_>>(),
        vec![
            ("a", SourceCorrespondenceRejectionReason::DanglingCriterion),
            ("b", SourceCorrespondenceRejectionReason::DanglingCriterion),
            (
                "c",
                SourceCorrespondenceRejectionReason::ConflictingProvenance
            ),
            (
                "d",
                SourceCorrespondenceRejectionReason::ConflictingProvenance
            ),
            ("z", SourceCorrespondenceRejectionReason::ForeignSource),
        ]
    );
}

/// Trace: FR-052-AC-19, TC-452
#[trace("TC-452", "FR-052-AC-19", "FR-052-AC-3")]
#[test]
fn assertion_reason_precedence_rejects_foreign_before_dangling_and_relation_before_provenance() {
    let other = SourceIdentity::new(
        SourceDocumentId::new("other").unwrap(),
        SourceRevision::new(1).unwrap(),
    );
    let foreign = SourceCorrespondenceAssertion::new(
        SourceAssertionId::new("foreign"),
        other.clone(),
        criterion("missing"),
        occurrence(),
        None,
        SourceAssertionProvenance {
            source: other,
            author: SourceContractAuthorId::new("author"),
        },
    );
    let first = census_assertion("same", "AC-1", None);
    let second = SourceCorrespondenceAssertion::new(
        SourceAssertionId::new("same"),
        source(),
        criterion("AC-2"),
        occurrence(),
        None,
        SourceAssertionProvenance {
            source: source(),
            author: SourceContractAuthorId::new("other"),
        },
    );
    let census =
        build_source_obligation_census(census_input(vec![foreign, first, second])).unwrap();
    assert_eq!(
        census
            .rejected_assertions()
            .iter()
            .map(|r| r.reason())
            .collect::<Vec<_>>(),
        vec![
            SourceCorrespondenceRejectionReason::ForeignSource,
            SourceCorrespondenceRejectionReason::ConflictingRelation,
            SourceCorrespondenceRejectionReason::ConflictingRelation
        ]
    );
    assert!(census.correspondence_rows().is_empty());
}

/// Trace: FR-052-AC-23, TC-452
#[trace("TC-452", "FR-052-AC-23", "FR-052-AC-3")]
#[test]
fn orphan_or_conflicting_endpoints_do_not_resolve() {
    let base = census_input(vec![]);
    let conflict_rows = vec![
        SourceInventoryRow::new(
            source(),
            SourceInventoryIdentity::SemanticNode {
                identity: occurrence().node().clone(),
                semantic: semantic('a'),
            },
        ),
        SourceInventoryRow::new(
            source(),
            SourceInventoryIdentity::SemanticNode {
                identity: occurrence().node().clone(),
                semantic: semantic('b'),
            },
        ),
        SourceInventoryRow::new(
            source(),
            SourceInventoryIdentity::SemanticOccurrence(occurrence()),
        ),
    ];
    let assertion = census_assertion("a", "AC-1", None);
    for (case, rows) in [
        conflict_rows,
        vec![SourceInventoryRow::new(
            source(),
            SourceInventoryIdentity::SemanticOccurrence(occurrence()),
        )],
    ]
    .into_iter()
    .enumerate()
    {
        let census = build_source_obligation_census(with_positions(
            Some(binding(source())),
            base.quire().cloned(),
            Some(census_outcome(SourceInventoryOwner::IrV2, rows)),
            base.qsl().cloned(),
            base.quire_code_rs().cloned(),
            vec![assertion.clone()],
        ))
        .unwrap();
        assert_eq!(
            census.rejected_assertions()[0].reason(),
            SourceCorrespondenceRejectionReason::DanglingOccurrence
        );
        assert!(census.correspondence_rows().is_empty());
        assert!(census.inventory_rows().iter().any(|r| matches!(
            r.row().identity(),
            SourceInventoryIdentity::SemanticOccurrence(_)
        )));
        if case == 0 {
            assert_eq!(
                census
                    .inventory_rows()
                    .iter()
                    .filter(|r| r.disposition() == SourceInventoryDisposition::ConflictingIdentity)
                    .count(),
                2
            );
        }
    }
    let function =
        SourceProductionFunctionId::new(SourceProductionModuleId::new("missing.rs"), "check");
    let census = build_source_obligation_census(with_positions(
        Some(binding(source())),
        base.quire().cloned(),
        base.ir_v2().cloned(),
        base.qsl().cloned(),
        Some(census_outcome(
            SourceInventoryOwner::QuireCodeRs,
            vec![SourceInventoryRow::new(
                source(),
                SourceInventoryIdentity::ProductionFunction(function.clone()),
            )],
        )),
        vec![census_assertion("a", "AC-1", Some(function))],
    ))
    .unwrap();
    assert_eq!(
        census.rejected_assertions()[0].reason(),
        SourceCorrespondenceRejectionReason::DanglingProductionSymbol
    );
    assert!(census.inventory_rows().iter().any(|r| matches!(
        r.row().identity(),
        SourceInventoryIdentity::ProductionFunction(_)
    )));
}

/// Trace: FR-052-AC-3, TC-452
#[trace("TC-452", "FR-052-AC-3")]
#[test]
fn absent_occurrence_and_same_id_provenance_conflict_never_enter_accepted_rows() {
    let absent = SourceCorrespondenceAssertion::new(
        SourceAssertionId::new("absent"),
        source(),
        criterion("AC-1"),
        occurrence_at(99),
        None,
        SourceAssertionProvenance {
            source: source(),
            author: SourceContractAuthorId::new("author"),
        },
    );
    let census = build_source_obligation_census(census_input(vec![absent])).unwrap();
    assert_eq!(census.rejected_assertions().len(), 1);
    assert_eq!(
        census.rejected_assertions()[0].reason(),
        SourceCorrespondenceRejectionReason::DanglingOccurrence
    );
    assert!(census.correspondence_rows().is_empty());

    let first = census_assertion("same", "AC-1", None);
    let second = SourceCorrespondenceAssertion::new(
        SourceAssertionId::new("same"),
        source(),
        criterion("AC-1"),
        occurrence(),
        None,
        SourceAssertionProvenance {
            source: source(),
            author: SourceContractAuthorId::new("other"),
        },
    );
    let census = build_source_obligation_census(census_input(vec![first, second])).unwrap();
    assert_eq!(census.rejected_assertions().len(), 2);
    assert!(census
        .rejected_assertions()
        .iter()
        .all(|row| row.reason() == SourceCorrespondenceRejectionReason::ConflictingProvenance));
    assert!(census.correspondence_rows().is_empty());
}

/// Trace: FR-052-AC-15, TC-452
#[trace("TC-452", "FR-052-AC-15", "FR-052-AC-3")]
#[test]
fn foreign_and_dangling_assertions_remain_separate_from_unmapped_inventory() {
    let other = SourceIdentity::new(
        SourceDocumentId::new("other").unwrap(),
        SourceRevision::new(1).unwrap(),
    );
    let foreign = SourceCorrespondenceAssertion::new(
        SourceAssertionId::new("foreign"),
        other.clone(),
        criterion("AC-1"),
        occurrence(),
        None,
        SourceAssertionProvenance {
            source: other,
            author: SourceContractAuthorId::new("author"),
        },
    );
    let dangling = census_assertion("missing-criterion", "AC-99", None);
    let missing_function =
        SourceProductionFunctionId::new(SourceProductionModuleId::new("src/lib.rs"), "absent");
    let missing_symbol = census_assertion("missing-symbol", "AC-1", Some(missing_function));
    let census =
        build_source_obligation_census(census_input(vec![foreign, dangling, missing_symbol]))
            .unwrap();
    assert!(census.correspondence_rows().is_empty());
    assert_eq!(
        census
            .rejected_assertions()
            .iter()
            .map(|r| r.reason())
            .collect::<Vec<_>>(),
        vec![
            SourceCorrespondenceRejectionReason::ForeignSource,
            SourceCorrespondenceRejectionReason::DanglingCriterion,
            SourceCorrespondenceRejectionReason::DanglingProductionSymbol,
        ]
    );
    assert_eq!(
        census.inventory_rows()[0].disposition(),
        SourceInventoryDisposition::ZeroMappings(SourceMissingMappingReason::NoAuthoredMapping)
    );
}
