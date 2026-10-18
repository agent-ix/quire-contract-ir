// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (C) 2026 Agent-IX

use ix_trace_rs::trace;
use quire_contract_model::source_obligation::{
    build_source_obligation_census, SourceAssertionId, SourceAssertionProvenance,
    SourceAuthoredCriterionId, SourceCensusInventoryRow, SourceContractAuthorId,
    SourceCorrespondenceAssertion, SourceCorrespondenceRejection, SourceCriterionSourceBinding,
    SourceInventoryEnumerator, SourceInventoryIncompleteReason, SourceInventoryOutcome,
    SourceInventoryOwner, SourceInventoryRow, SourceInventoryScope, SourceObligationCensus,
    SourceObligationInput, SourceObligationRow, SourceSemanticOccurrenceId,
};
use quire_contract_model::{
    CheckedNodeId, CheckedOccurrence, CheckedOccurrenceRole, SourceDocumentId,
    SourceIdentity, SourceRevision,
};

struct Rows {
    source: SourceIdentity,
    owner: SourceInventoryOwner,
    scope: SourceInventoryScope,
    rows: std::vec::IntoIter<SourceInventoryRow>,
    count: usize,
}

impl SourceInventoryEnumerator for Rows {
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
        Ok(self.rows.next())
    }
    fn observed_count(&self) -> usize {
        self.count
    }
}

fn outcome(
    source: &SourceIdentity,
    owner: SourceInventoryOwner,
    rows: Vec<SourceInventoryRow>,
) -> SourceInventoryOutcome {
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
    let count = rows.len();
    SourceInventoryOutcome::from_enumerator(Rows {
        source: source.clone(),
        owner,
        scope,
        rows: rows.into_iter(),
        count,
    })
    .expect("synthetic outcome")
}

#[trace("TC-452", "FR-052-AC-1", "FR-052-AC-5")]
#[test]
fn default_feature_consumer_borrows_public_census_and_rows() {
    let source = SourceIdentity::new(
        SourceDocumentId::new("selected").unwrap(),
        SourceRevision::new(1).unwrap(),
    );
    let criterion = SourceAuthoredCriterionId::new("obligations", "FR-052.md", "AC-1");
    let node = CheckedNodeId {
        domain: "quire.checked-semantic-node/v1".into(),
        digest: "a".repeat(64).into(),
    };
    let occurrence = SourceSemanticOccurrenceId::new(
        node.clone(),
        CheckedOccurrence {
            role: CheckedOccurrenceRole::Expression,
            ordinal: 0,
        },
    );
    let row = |identity| SourceInventoryRow::new(source.clone(), identity);
    use quire_contract_model::source_obligation::SourceInventoryIdentity;
    let quire = outcome(
        &source,
        SourceInventoryOwner::Quire,
        vec![row(SourceInventoryIdentity::Criterion(criterion.clone()))],
    );
    let ir = outcome(
        &source,
        SourceInventoryOwner::IrV2,
        vec![
            row(SourceInventoryIdentity::SemanticNode {
                identity: node,
                semantic_type: CheckedNodeId {
                    domain: "quire.checked-semantic-node/v1".into(),
                    digest: "b".repeat(64).into(),
                },
            }),
            row(SourceInventoryIdentity::SemanticOccurrence(
                occurrence.clone(),
            )),
        ],
    );
    let qsl = outcome(&source, SourceInventoryOwner::Qsl, vec![]);
    let code = outcome(&source, SourceInventoryOwner::QuireCodeRs, vec![]);
    let provenance = SourceAssertionProvenance {
        source: source.clone(),
        author: SourceContractAuthorId::new("author"),
    };
    let valid = SourceCorrespondenceAssertion::new(
        SourceAssertionId::new("valid"),
        source.clone(),
        criterion.clone(),
        occurrence.clone(),
        None,
        provenance.clone(),
    );
    let rejected = SourceCorrespondenceAssertion::new(
        SourceAssertionId::new("dangling"),
        source.clone(),
        SourceAuthoredCriterionId::new("obligations", "FR-052.md", "missing"),
        occurrence,
        None,
        provenance,
    );
    let input: SourceObligationInput = SourceObligationInput::new(
        source.clone(),
        Some(SourceCriterionSourceBinding::new(
            "corpus",
            "model",
            "obligations",
            source.clone(),
            SourceContractAuthorId::new("author"),
        )),
        Some(quire),
        Some(ir),
        Some(qsl),
        Some(code),
        vec![valid, rejected],
    );
    let census: SourceObligationCensus =
        build_source_obligation_census(input).expect("synthetic census");
    let inventory: &SourceCensusInventoryRow = &census.inventory_rows()[0];
    let accepted: &SourceObligationRow = &census.correspondence_rows()[0];
    let rejection: &SourceCorrespondenceRejection = &census.rejected_assertions()[0];
    assert_eq!(census.source(), &source);
    assert_eq!(
        inventory.row().identity(),
        &SourceInventoryIdentity::Criterion(criterion.clone())
    );
    assert_eq!(accepted.criterion(), &criterion);
    assert_eq!(rejection.assertion().id().as_str(), "dangling");
}
