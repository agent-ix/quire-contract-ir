// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (C) 2026 Agent-IX

//! FR-052 provisional input tests. TC-452's complete owner oracle is unrun.

use ix_trace_rs::trace;
use quire_contract_model::source_obligation::{
    SourceAssertionId, SourceAssertionProvenance, SourceContractAuthorId,
    SourceCorrespondenceAssertion, SourceDeclarationHandle, SourceExpressionHandle,
    SourceInventoryEnvelope, SourceInventoryIdentity, SourceInventoryIncompleteReason,
    SourceInventoryOwner, SourceInventoryRow, SourceInventoryScope, SourceObligationInput,
    SourceObligationSite, SourcePendingObligationId, SourcePendingObligationKind,
    SourceProductionFunctionId, SourceProductionModuleId, SourceRequirementOccurrenceId,
    SourceSemanticOccurrenceId, SourceTypeExhaustion, SourceTypeWorkDimension, SourceUnitHandle,
};
use quire_contract_model::{
    CheckedNodeId, CheckedOccurrence, CheckedOccurrenceRole, ClauseId, ClauseRef, PackageId,
    RequirementId, RequirementRef, RequirementRevision, SourceDocumentId, SourceIdentity,
    SourceRevision,
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
