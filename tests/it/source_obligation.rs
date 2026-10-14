// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (C) 2026 Agent-IX

//! FR-052 / TC-452: the neutral source census keeps complete owner populations
//! and accepts only explicit, source-authored correspondence assertions.

use ix_trace_rs::trace;
use quire_contract_model::source_obligation::{
    build_source_obligation_census, SourceAssertionId, SourceAssertionProvenance,
    SourceContractAuthorId, SourceCorrespondenceAssertion, SourceCorrespondenceError,
    SourceCorrespondenceRejectionReason, SourceInventoryCompleteness, SourceInventoryDisposition,
    SourceInventoryEnvelope, SourceInventoryIdentity, SourceInventoryIncompleteReason,
    SourceInventoryOwner, SourceInventoryRow, SourceInventoryScope, SourceMissingMappingReason,
    SourceObligationInput, SourceProductionFunctionId, SourceProductionModuleId,
    SourceSemanticOccurrenceId,
};

const SOURCE_OBLIGATION_API: &str =
    include_str!("../../crates/quire-contract-model/src/source_obligation.rs");
use quire_contract_model::{
    CheckedNodeId, CheckedOccurrence, CheckedOccurrenceRole, CheckedSemanticId, ClauseId,
    ClauseRef, PackageId, RequirementId, RequirementRef, RequirementRevision, SourceDocumentId,
    SourceIdentity, SourceRevision,
};

fn source(revision: u64) -> SourceIdentity {
    SourceIdentity::new(
        SourceDocumentId::new("selected-source").expect("valid source document"),
        SourceRevision::new(revision).expect("valid source revision"),
    )
}

fn node(name: &str) -> CheckedNodeId {
    CheckedNodeId {
        domain: "quire.checked-semantic-node/v1".into(),
        digest: format!("{name:0<64}").into(),
    }
}

fn semantic(name: &str) -> CheckedSemanticId {
    CheckedSemanticId {
        domain: "quire.semantic-obligation/v1".into(),
        algorithm: "sha256".into(),
        digest: format!("{name:0<64}").into(),
    }
}

fn occurrence(node: CheckedNodeId, ordinal: u64) -> SourceSemanticOccurrenceId {
    SourceSemanticOccurrenceId::new(
        node,
        CheckedOccurrence {
            role: CheckedOccurrenceRole::Expression,
            ordinal,
        },
    )
}

fn criterion(name: &str) -> ClauseRef {
    let package = PackageId::new("example.contract").expect("valid package");
    let requirement = RequirementRef::new(
        package,
        RequirementId::new("REQ_source_obligation").expect("valid requirement"),
        RequirementRevision::new(1).expect("valid requirement revision"),
    );
    ClauseRef::new(requirement, ClauseId::new(name).expect("valid criterion"))
}

fn inventory_item(
    source: &SourceIdentity,
    identity: SourceInventoryIdentity,
) -> SourceInventoryRow {
    SourceInventoryRow::new(source.clone(), identity)
}

fn make_envelope(
    owner: SourceInventoryOwner,
    source: &SourceIdentity,
    rows: Vec<SourceInventoryRow>,
) -> SourceInventoryEnvelope {
    let scope = match owner {
        SourceInventoryOwner::IrV2 => SourceInventoryScope::IrV2SemanticGraphAndOccurrences,
        SourceInventoryOwner::Qsl => {
            SourceInventoryScope::QslCheckedRequirementsAndPendingObligations
        }
        SourceInventoryOwner::QuireCodeRs => {
            SourceInventoryScope::QuireCodeRsProductionModulesAndFunctions
        }
    };
    SourceInventoryEnvelope::new(
        owner,
        source.clone(),
        scope,
        rows.len(),
        SourceInventoryCompleteness::Complete,
        rows,
    )
}

struct Fixture {
    source: SourceIdentity,
    ir: SourceInventoryEnvelope,
    qsl: SourceInventoryEnvelope,
    code: SourceInventoryEnvelope,
    assertions: Vec<SourceCorrespondenceAssertion>,
    c1: ClauseRef,
    c2: ClauseRef,
    o1: SourceSemanticOccurrenceId,
    o2: SourceSemanticOccurrenceId,
    f1: SourceProductionFunctionId,
    f2: SourceProductionFunctionId,
}

fn fixture() -> Fixture {
    let source = source(1);
    let n1 = node("a");
    let n2 = node("b");
    let n3 = node("c");
    let s1 = semantic("1");
    let s2 = semantic("2");
    let s3 = semantic("3");
    let o1 = occurrence(n1.clone(), 0);
    let o2 = occurrence(n2.clone(), 0);
    let o3 = occurrence(n3.clone(), 0);
    let c1 = criterion("AC_1");
    let c2 = criterion("AC_2");
    let empty_module = SourceProductionModuleId::new("src/empty.rs");
    let populated_module = SourceProductionModuleId::new("src/lib.rs");
    let f1 = SourceProductionFunctionId::new(populated_module.clone(), "run");
    let f2 = SourceProductionFunctionId::new(populated_module.clone(), "check");
    let ir = make_envelope(
        SourceInventoryOwner::IrV2,
        &source,
        vec![
            inventory_item(
                &source,
                SourceInventoryIdentity::SemanticNode {
                    identity: n1.clone(),
                    semantic: s1.clone(),
                },
            ),
            inventory_item(
                &source,
                SourceInventoryIdentity::SemanticNode {
                    identity: n2.clone(),
                    semantic: s2.clone(),
                },
            ),
            inventory_item(
                &source,
                SourceInventoryIdentity::SemanticNode {
                    identity: n3.clone(),
                    semantic: s3.clone(),
                },
            ),
            inventory_item(
                &source,
                SourceInventoryIdentity::SemanticOccurrence(o1.clone()),
            ),
            inventory_item(
                &source,
                SourceInventoryIdentity::SemanticOccurrence(o2.clone()),
            ),
            inventory_item(
                &source,
                SourceInventoryIdentity::SemanticOccurrence(o3.clone()),
            ),
        ],
    );
    let qsl = make_envelope(
        SourceInventoryOwner::Qsl,
        &source,
        vec![
            inventory_item(&source, SourceInventoryIdentity::Criterion(c1.clone())),
            inventory_item(&source, SourceInventoryIdentity::Criterion(c2.clone())),
            inventory_item(
                &source,
                SourceInventoryIdentity::RequirementRecord(c1.requirement().clone()),
            ),
            inventory_item(&source, SourceInventoryIdentity::PendingObligation(s1)),
            inventory_item(&source, SourceInventoryIdentity::PendingObligation(s2)),
            inventory_item(&source, SourceInventoryIdentity::PendingObligation(s3)),
        ],
    );
    let code = make_envelope(
        SourceInventoryOwner::QuireCodeRs,
        &source,
        vec![
            inventory_item(
                &source,
                SourceInventoryIdentity::ProductionModule(empty_module.clone()),
            ),
            inventory_item(
                &source,
                SourceInventoryIdentity::ProductionModule(populated_module.clone()),
            ),
            inventory_item(
                &source,
                SourceInventoryIdentity::ProductionFunction(f1.clone()),
            ),
            inventory_item(
                &source,
                SourceInventoryIdentity::ProductionFunction(f2.clone()),
            ),
        ],
    );
    let provenance = || SourceAssertionProvenance {
        source: source.clone(),
        author: SourceContractAuthorId::new("contract-author"),
    };
    let assertions = vec![
        SourceCorrespondenceAssertion::new(
            SourceAssertionId::new("map-1"),
            source.clone(),
            c1.clone(),
            o1.clone(),
            Some(f1.clone()),
            provenance(),
        ),
        SourceCorrespondenceAssertion::new(
            SourceAssertionId::new("map-2"),
            source.clone(),
            c1.clone(),
            o2.clone(),
            None,
            provenance(),
        ),
    ];
    Fixture {
        source,
        ir,
        qsl,
        code,
        assertions,
        c1,
        c2,
        o1,
        o2,
        f1,
        f2,
    }
}

fn build(
    f: Fixture,
) -> Result<
    quire_contract_model::source_obligation::SourceObligationCensus,
    SourceCorrespondenceError,
> {
    build_source_obligation_census(SourceObligationInput::new(
        f.source,
        Some(f.ir),
        Some(f.qsl),
        Some(f.code),
        f.assertions,
    ))
}

/// Trace: FR-052-AC-1, FR-052-AC-2, FR-052-AC-4, FR-052-AC-5, FR-052-AC-7
#[trace(
    "TC-452",
    "FR-052-AC-1",
    "FR-052-AC-2",
    "FR-052-AC-4",
    "FR-052-AC-5",
    "FR-052-AC-7"
)]
#[test]
fn tc_452_external_consumer_borrows_complete_census_and_preserves_explicit_rows() {
    let f = fixture();
    let census = build(f).expect("complete three-owner fixture");
    let census_borrow: &quire_contract_model::source_obligation::SourceObligationCensus = &census;
    let row_borrow: &[_] = census_borrow.inventory_rows();
    assert_eq!(row_borrow.len(), census.inventory_rows().len());
    assert_eq!(census.inventory_rows().len(), 16);
    assert_eq!(census.correspondence_rows().len(), 2);
    assert!(census.rejected_assertions().is_empty());
    let explicit_absence = census
        .correspondence_rows()
        .iter()
        .find(|row| row.occurrence() == &occurrence(node("b"), 0))
        .expect("explicit no-function relation");
    assert!(explicit_absence.production_function().is_none());
    assert_eq!(explicit_absence.criterion(), &criterion("AC_1"));
    assert!(census.inventory_rows().iter().any(|row| row.identity()
        == &SourceInventoryIdentity::ProductionModule(SourceProductionModuleId::new(
            "src/empty.rs"
        ))
        && row.disposition()
            == &SourceInventoryDisposition::ZeroMappings(
                SourceMissingMappingReason::NoAuthoredMapping
            )));
    assert!(census.inventory_rows().iter().any(|row| row.identity()
        == &SourceInventoryIdentity::SemanticOccurrence(occurrence(node("b"), 0))
        && row.disposition()
            == &SourceInventoryDisposition::ZeroMappings(
                SourceMissingMappingReason::ExplicitNoProductionFunction
            )));
    assert!(census.inventory_rows().iter().any(|row| row.identity()
        == &SourceInventoryIdentity::Criterion(criterion("AC_2"))
        && row.disposition()
            == &SourceInventoryDisposition::ZeroMappings(
                SourceMissingMappingReason::NoAuthoredMapping
            )));
    assert!(census.inventory_rows().iter().any(|row| row.identity()
        == &SourceInventoryIdentity::SemanticOccurrence(occurrence(node("c"), 0))
        && row.disposition()
            == &SourceInventoryDisposition::ZeroMappings(
                SourceMissingMappingReason::NoAuthoredMapping
            )));
    assert!(census.inventory_rows().iter().any(|row| row.identity()
        == &SourceInventoryIdentity::ProductionFunction(SourceProductionFunctionId::new(
            SourceProductionModuleId::new("src/lib.rs"),
            "check"
        ))
        && row.disposition()
            == &SourceInventoryDisposition::ZeroMappings(
                SourceMissingMappingReason::NoAuthoredMapping
            )));
}

/// Trace: FR-052-AC-4
#[trace("TC-452", "FR-052-AC-4")]
#[test]
fn tc_452_input_signature_contains_only_source_inventory_and_assertion_data() {
    let f = fixture();
    let _input = SourceObligationInput::new(
        f.source,
        Some(f.ir),
        Some(f.qsl),
        Some(f.code),
        f.assertions,
    );
    let input = SOURCE_OBLIGATION_API
        .split_once("pub struct SourceObligationInput {")
        .expect("public input declaration")
        .1
        .split_once("\n}")
        .expect("input declaration end")
        .0;
    for field in ["source:", "ir_v2:", "qsl:", "quire_code_rs:", "assertions:"] {
        assert!(input.contains(field), "missing typed input field {field}");
    }
    for forbidden in [
        "method:",
        "profile:",
        "result:",
        "artifact:",
        "attempt:",
        "run:",
    ] {
        assert!(
            !input.contains(forbidden),
            "forbidden evidence-attempt field {forbidden}"
        );
    }
}

/// Trace: FR-052-AC-2, FR-052-AC-3
#[trace("TC-452", "FR-052-AC-2", "FR-052-AC-3")]
#[test]
fn tc_452_deduplicates_conflicts_and_rejects_foreign_or_dangling_assertions() {
    let mut f = fixture();
    // Exact inventory repetition collapses; a repeated native node identity with
    // a different semantic identity remains visible as a conflict.
    let mut ir_items = f.ir.items().to_vec();
    ir_items.push(ir_items[0].clone());
    let duplicate_node = match ir_items[0].identity() {
        SourceInventoryIdentity::SemanticNode { identity, .. } => identity.clone(),
        other => panic!("expected semantic node row, found {other:?}"),
    };
    ir_items.push(inventory_item(
        &f.source,
        SourceInventoryIdentity::SemanticNode {
            identity: duplicate_node,
            semantic: semantic("different"),
        },
    ));
    f.ir = make_envelope(SourceInventoryOwner::IrV2, &f.source, ir_items);

    let provenance = |author: &str| SourceAssertionProvenance {
        source: f.source.clone(),
        author: SourceContractAuthorId::new(author),
    };
    let duplicate_assertion = f.assertions[0].clone();
    f.assertions.push(duplicate_assertion);
    let foreign_function =
        SourceProductionFunctionId::new(SourceProductionModuleId::new("foreign.rs"), "foreign_fn");
    let mut bad = vec![
        // Foreign source.
        SourceCorrespondenceAssertion::new(
            SourceAssertionId::new("foreign"),
            source(2),
            f.c1.clone(),
            f.o1.clone(),
            Some(f.f1.clone()),
            provenance("a"),
        ),
        // Dangling criterion.
        SourceCorrespondenceAssertion::new(
            SourceAssertionId::new("dangling-c"),
            f.source.clone(),
            criterion("MISSING"),
            f.o1.clone(),
            Some(f.f1.clone()),
            provenance("a"),
        ),
        // Dangling occurrence.
        SourceCorrespondenceAssertion::new(
            SourceAssertionId::new("dangling-o"),
            f.source.clone(),
            f.c1.clone(),
            occurrence(node("missing"), 0),
            Some(f.f1.clone()),
            provenance("a"),
        ),
        // Dangling function.
        SourceCorrespondenceAssertion::new(
            SourceAssertionId::new("dangling-f"),
            f.source.clone(),
            f.c1.clone(),
            f.o2.clone(),
            Some(SourceProductionFunctionId::new(
                SourceProductionModuleId::new("missing.rs"),
                "f",
            )),
            provenance("a"),
        ),
        // A foreign inventory row cannot satisfy a selected-source endpoint.
        SourceCorrespondenceAssertion::new(
            SourceAssertionId::new("foreign-endpoint"),
            f.source.clone(),
            f.c1.clone(),
            f.o2.clone(),
            Some(foreign_function.clone()),
            provenance("a"),
        ),
    ];
    // Same identity, different relation facts.
    bad.push(SourceCorrespondenceAssertion::new(
        SourceAssertionId::new("conflict-r"),
        f.source.clone(),
        f.c1.clone(),
        occurrence(node("c"), 0),
        Some(f.f1.clone()),
        provenance("a"),
    ));
    bad.push(SourceCorrespondenceAssertion::new(
        SourceAssertionId::new("conflict-r"),
        f.source.clone(),
        f.c2.clone(),
        occurrence(node("c"), 0),
        Some(f.f1.clone()),
        provenance("a"),
    ));
    // Same identity and relation, different source-contract provenance.
    bad.push(SourceCorrespondenceAssertion::new(
        SourceAssertionId::new("conflict-p"),
        f.source.clone(),
        f.c2.clone(),
        f.o1.clone(),
        Some(f.f2.clone()),
        provenance("a"),
    ));
    bad.push(SourceCorrespondenceAssertion::new(
        SourceAssertionId::new("conflict-p"),
        f.source.clone(),
        f.c2.clone(),
        f.o1.clone(),
        Some(f.f2.clone()),
        provenance("b"),
    ));
    f.assertions.extend(bad);
    let mut code_items = f.code.items().to_vec();
    code_items.push(inventory_item(
        &source(2),
        SourceInventoryIdentity::ProductionModule(SourceProductionModuleId::new("foreign.rs")),
    ));
    code_items.push(inventory_item(
        &source(2),
        SourceInventoryIdentity::ProductionFunction(foreign_function),
    ));
    f.code = make_envelope(SourceInventoryOwner::QuireCodeRs, &f.source, code_items);

    let census = build(f).expect("complete envelopes retain foreign inventory as a row");
    assert_eq!(
        census.correspondence_rows().len(),
        1,
        "exact duplicate assertion must collapse"
    );
    assert_eq!(
        census.inventory_rows().len(),
        19,
        "exact inventory duplicates collapse while conflicting facts and foreign rows remain visible"
    );
    assert!(!census
        .correspondence_rows()
        .iter()
        .any(|row| row.occurrence() == &occurrence(node("c"), 0)));
    assert!(!census
        .correspondence_rows()
        .iter()
        .any(|row| row.criterion() == &criterion("AC_2")
            && row.occurrence() == &occurrence(node("a"), 0)
            && row.production_function()
                == Some(&SourceProductionFunctionId::new(
                    SourceProductionModuleId::new("src/lib.rs"),
                    "check"
                ))));
    let reasons = census
        .rejected_assertions()
        .iter()
        .map(|entry| entry.reason())
        .collect::<Vec<_>>();
    for reason in [
        SourceCorrespondenceRejectionReason::ForeignSource,
        SourceCorrespondenceRejectionReason::DanglingCriterion,
        SourceCorrespondenceRejectionReason::DanglingOccurrence,
        SourceCorrespondenceRejectionReason::DanglingProductionSymbol,
        SourceCorrespondenceRejectionReason::ConflictingRelation,
        SourceCorrespondenceRejectionReason::ConflictingProvenance,
    ] {
        assert!(
            reasons.contains(&reason),
            "missing typed rejection {reason:?}: {reasons:?}"
        );
    }
    assert!(census
        .rejected_assertions()
        .iter()
        .any(
            |entry| entry.assertion().id().as_str() == "foreign-endpoint"
                && entry.reason() == SourceCorrespondenceRejectionReason::DanglingProductionSymbol
        ));
    assert!(census
        .inventory_rows()
        .iter()
        .any(|row| row.disposition() == &SourceInventoryDisposition::ForeignInventory));
    assert!(census.inventory_rows().iter().filter(|row| matches!(row.identity(), SourceInventoryIdentity::SemanticNode { identity, .. } if identity == &node("a")))
        .all(|row| row.disposition() == &SourceInventoryDisposition::ConflictingIdentity));
    assert!(census
        .correspondence_rows()
        .iter()
        .all(|row| row.criterion() != &criterion("MISSING")));
}

/// Trace: FR-052-AC-2, FR-052-AC-6, FR-052-AC-7
#[trace("TC-452", "FR-052-AC-2", "FR-052-AC-6", "FR-052-AC-7")]
#[test]
fn tc_452_missing_incomplete_scope_source_and_count_never_return_a_census() {
    let base = fixture();
    for missing in 0..3 {
        let f = fixture();
        let result = build_source_obligation_census(SourceObligationInput::new(
            f.source,
            (missing != 0).then_some(f.ir),
            (missing != 1).then_some(f.qsl),
            (missing != 2).then_some(f.code),
            f.assertions,
        ));
        let expected = [
            SourceInventoryOwner::IrV2,
            SourceInventoryOwner::Qsl,
            SourceInventoryOwner::QuireCodeRs,
        ][missing];
        assert!(
            matches!(result, Err(SourceCorrespondenceError::MissingOwnerInventory { owner }) if owner == expected)
        );
    }
    let owners = [
        SourceInventoryOwner::IrV2,
        SourceInventoryOwner::Qsl,
        SourceInventoryOwner::QuireCodeRs,
    ];
    for missing_owner in owners {
        let mut f = fixture();
        let reason = match missing_owner {
            SourceInventoryOwner::IrV2 => SourceInventoryIncompleteReason::EarlyStop {
                owner: missing_owner,
            },
            SourceInventoryOwner::Qsl => SourceInventoryIncompleteReason::RequestFailed {
                owner: missing_owner,
                cause: "checked RequirementRecord request failed".into(),
            },
            SourceInventoryOwner::QuireCodeRs => {
                SourceInventoryIncompleteReason::ModuleUnavailable {
                    module: SourceProductionModuleId::new("src/lib.rs"),
                    cause: "selected module parse failed".into(),
                }
            }
        };
        let envelope = match missing_owner {
            SourceInventoryOwner::IrV2 => &mut f.ir,
            SourceInventoryOwner::Qsl => &mut f.qsl,
            SourceInventoryOwner::QuireCodeRs => &mut f.code,
        };
        *envelope = SourceInventoryEnvelope::new(
            missing_owner,
            f.source.clone(),
            envelope.scope(),
            envelope.item_count(),
            SourceInventoryCompleteness::Incomplete(reason.clone()),
            envelope.items().to_vec(),
        );
        let result = build(f);
        assert!(
            matches!(result, Err(SourceCorrespondenceError::IncompleteOwnerInventory { owner, cause }) if owner == missing_owner && cause == reason)
        );
    }
    // Empty function subinventory is a complete owner result and preserves its module.
    let census = build(base).expect("empty module is not an incomplete owner result");
    assert!(census.inventory_rows().iter().any(|row| matches!(row.identity(), SourceInventoryIdentity::ProductionModule(id) if id.as_str() == "src/empty.rs")));

    let empty_source = source(3);
    let empty_ir = SourceInventoryEnvelope::new(
        SourceInventoryOwner::IrV2,
        empty_source.clone(),
        SourceInventoryScope::IrV2SemanticGraphAndOccurrences,
        0,
        SourceInventoryCompleteness::Complete,
        Vec::new(),
    );
    let empty_qsl = SourceInventoryEnvelope::new(
        SourceInventoryOwner::Qsl,
        empty_source.clone(),
        SourceInventoryScope::QslCheckedRequirementsAndPendingObligations,
        0,
        SourceInventoryCompleteness::Complete,
        Vec::new(),
    );
    let empty_code = SourceInventoryEnvelope::new(
        SourceInventoryOwner::QuireCodeRs,
        empty_source.clone(),
        SourceInventoryScope::QuireCodeRsProductionModulesAndFunctions,
        0,
        SourceInventoryCompleteness::Complete,
        Vec::new(),
    );
    let empty_census = build_source_obligation_census(SourceObligationInput::new(
        empty_source,
        Some(empty_ir),
        Some(empty_qsl),
        Some(empty_code),
        Vec::new(),
    ))
    .expect("a complete owner scope may be empty");
    assert!(empty_census.inventory_rows().is_empty());

    let mut f = fixture();
    f.ir = SourceInventoryEnvelope::new(
        SourceInventoryOwner::IrV2,
        f.source.clone(),
        SourceInventoryScope::QslCheckedRequirementsAndPendingObligations,
        f.ir.item_count(),
        SourceInventoryCompleteness::Complete,
        f.ir.items().to_vec(),
    );
    assert!(matches!(
        build(f),
        Err(SourceCorrespondenceError::InventoryScopeMismatch {
            owner: SourceInventoryOwner::IrV2,
            ..
        })
    ));

    let mut f = fixture();
    f.ir = SourceInventoryEnvelope::new(
        SourceInventoryOwner::Qsl,
        f.source.clone(),
        SourceInventoryScope::QslCheckedRequirementsAndPendingObligations,
        f.ir.item_count(),
        SourceInventoryCompleteness::Complete,
        f.ir.items().to_vec(),
    );
    assert!(matches!(
        build(f),
        Err(SourceCorrespondenceError::InventoryScopeMismatch {
            owner: SourceInventoryOwner::IrV2,
            observed: SourceInventoryScope::QslCheckedRequirementsAndPendingObligations,
            ..
        })
    ));

    let mut f = fixture();
    f.qsl = SourceInventoryEnvelope::new(
        SourceInventoryOwner::Qsl,
        source(2),
        SourceInventoryScope::QslCheckedRequirementsAndPendingObligations,
        f.qsl.item_count(),
        SourceInventoryCompleteness::Complete,
        f.qsl.items().to_vec(),
    );
    assert!(matches!(
        build(f),
        Err(SourceCorrespondenceError::InventorySourceMismatch {
            owner: SourceInventoryOwner::Qsl,
            ..
        })
    ));

    let mut f = fixture();
    f.code = SourceInventoryEnvelope::new(
        SourceInventoryOwner::QuireCodeRs,
        f.source.clone(),
        SourceInventoryScope::QuireCodeRsProductionModulesAndFunctions,
        f.code.item_count() + 1,
        SourceInventoryCompleteness::Complete,
        f.code.items().to_vec(),
    );
    assert!(matches!(
        build(f),
        Err(SourceCorrespondenceError::InventoryItemCountMismatch {
            owner: SourceInventoryOwner::QuireCodeRs,
            ..
        })
    ));
}

/// Trace: FR-052-AC-2, FR-052-AC-7
#[trace("TC-452", "FR-052-AC-2", "FR-052-AC-7")]
#[test]
fn tc_452_input_permutation_preserves_ordered_census_and_inventory_removal_changes_it() {
    let first_fixture = fixture();
    let first = build(first_fixture).expect("complete fixture");
    let mut second_fixture = fixture();
    let mut ir = second_fixture.ir.items().to_vec();
    ir.reverse();
    let mut qsl = second_fixture.qsl.items().to_vec();
    qsl.reverse();
    let mut code = second_fixture.code.items().to_vec();
    code.reverse();
    second_fixture.ir = make_envelope(SourceInventoryOwner::IrV2, &second_fixture.source, ir);
    second_fixture.qsl = make_envelope(SourceInventoryOwner::Qsl, &second_fixture.source, qsl);
    second_fixture.code = make_envelope(
        SourceInventoryOwner::QuireCodeRs,
        &second_fixture.source,
        code,
    );
    second_fixture.assertions.reverse();
    let duplicate_assertion = second_fixture.assertions[0].clone();
    second_fixture.assertions.push(duplicate_assertion);
    let second = build(second_fixture).expect("reordered complete fixture");
    assert_eq!(first, second);

    let mut changed = fixture();
    let mut rows = changed.ir.items().to_vec();
    rows.retain(|row| {
        row.identity() != &SourceInventoryIdentity::SemanticOccurrence(changed.o2.clone())
    });
    changed.ir = make_envelope(SourceInventoryOwner::IrV2, &changed.source, rows);
    let changed = build(changed).expect("still complete after authoritative inventory changed");
    assert_ne!(first, changed);
    assert!(changed
        .rejected_assertions()
        .iter()
        .any(|entry| entry.reason() == SourceCorrespondenceRejectionReason::DanglingOccurrence));
}
