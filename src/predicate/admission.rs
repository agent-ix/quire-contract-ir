//! Exact contract admission and checked-predicate population projection.

use std::collections::BTreeMap;

use quire_spec_language::protocol_artifact::checked_predicate::ValidatedCheckedPredicate;

use crate::bridge::{BridgeDigest, BridgeErrorCode, BridgeLimits, ContractSelection};

use super::{
    artifacts::{build, finish_projection},
    decision::{
        projection_kind, sort_causes, PredicateCause, PredicateCauseCode, PredicateDecision,
        PredicateProjectionDecision,
    },
    definition::PredicateDefinition,
    PredicateProjection,
};

/// Exact native and TL owner selections used by one projection.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct TargetSelection {
    native: ContractSelection,
    signal_catalog: ContractSelection,
    proposition_map: ContractSelection,
}

impl TargetSelection {
    /// Constructs an explicit selection for admission.
    #[must_use]
    pub const fn new(
        native: ContractSelection,
        signal_catalog: ContractSelection,
        proposition_map: ContractSelection,
    ) -> Self {
        Self {
            native,
            signal_catalog,
            proposition_map,
        }
    }

    /// Returns the exact merged owner selections compiled into this bridge.
    #[must_use]
    pub fn current() -> Self {
        Self::new(
            ContractSelection::new(
                "quire.checked-predicate/v1",
                "0.2.0",
                "agent-ix/quire-spec-language",
                "f1700a9264d6d3bcdd07e0f77b70f3dae9ed4c07",
                BridgeDigest::raw(
                    quire_spec_language::protocol_artifact::checked_predicate::SCHEMA_BYTES,
                ),
            ),
            ContractSelection::new(
                "tl-syntax.signal-catalog/v1",
                "0.1.0",
                "agent-ix/tl-syntax",
                "4a5614193d21e5ae99950ae683b04ba0ec931358",
                BridgeDigest::raw(tl_syntax::SIGNAL_CATALOG_V1_SCHEMA_BYTES),
            ),
            ContractSelection::new(
                "tl-syntax.proposition-map/v1",
                "0.1.0",
                "agent-ix/tl-syntax",
                "4a5614193d21e5ae99950ae683b04ba0ec931358",
                BridgeDigest::raw(tl_syntax::PROPOSITION_MAP_V1_SCHEMA_BYTES),
            ),
        )
    }

    /// Returns the QSL checked-predicate selection.
    #[must_use]
    pub const fn native(&self) -> &ContractSelection {
        &self.native
    }

    /// Returns the TL signal-catalog selection.
    #[must_use]
    pub const fn signal_catalog(&self) -> &ContractSelection {
        &self.signal_catalog
    }

    /// Returns the TL proposition-map selection.
    #[must_use]
    pub const fn proposition_map(&self) -> &ContractSelection {
        &self.proposition_map
    }
}

/// Projects constructor-private QSL checked leaves into exact TL owner artifacts.
pub fn project(
    predicates: &[ValidatedCheckedPredicate],
    target: TargetSelection,
    limits: BridgeLimits,
) -> Result<PredicateProjection, PredicateDecision> {
    let limits = limits.effective();
    let mut causes = selection_causes(&target);
    if predicates.is_empty() || predicates.len() > limits.predicates {
        causes.push(PredicateCause::assigned(
            PredicateCauseCode::PopulationInvalid,
            predicates.len().to_string(),
            None,
        ));
    }
    if !causes.is_empty() {
        return Err(rejection(causes, limits));
    }

    let input_work = predicates.iter().fold(0usize, |work, predicate| {
        work.saturating_add(predicate.document().bytes().len())
    });
    if input_work > limits.visited_work {
        return Err(PredicateDecision::Operation(
            crate::bridge::BridgeError::new(
                BridgeErrorCode::PredicateProjectionResourceExhausted,
                "combined predicate input exceeds the visited-work ceiling",
                "predicates",
            ),
        ));
    }

    let mut definitions = BTreeMap::new();
    if predicates.len().saturating_mul(512) > limits.allocation_bytes {
        return Err(PredicateDecision::Operation(
            crate::bridge::BridgeError::new(
                BridgeErrorCode::PredicateProjectionResourceExhausted,
                "predicate definition reservation exceeds caller allocation ceiling",
                "predicates",
            ),
        ));
    }
    for checked in predicates {
        if let Err(error) = crate::bridge::canonical::preflight(checked.document().bytes(), limits)
        {
            return Err(PredicateDecision::Operation(error));
        }
        let definition = PredicateDefinition::from_checked(checked, target.native(), limits)
            .map_err(PredicateDecision::Operation)?;
        if !matches!(
            definition.parent_kind(),
            "predicate" | "state" | "temporal" | "protocol"
        ) {
            causes.push(PredicateCause::assigned(
                PredicateCauseCode::ConstructUnsupported,
                definition.owner_document_identity(),
                Some(definition.parent_kind().to_owned()),
            ));
            continue;
        }
        let predicate_ref = definition.predicate_ref();
        match definitions.entry(predicate_ref) {
            std::collections::btree_map::Entry::Vacant(entry) => {
                entry.insert(definition);
            }
            std::collections::btree_map::Entry::Occupied(entry)
                if entry.get().canonical_tuple() == definition.canonical_tuple() =>
            {
                causes.push(PredicateCause::assigned(
                    PredicateCauseCode::PopulationInvalid,
                    predicate_ref.to_string(),
                    None,
                ));
            }
            std::collections::btree_map::Entry::Occupied(_) => {
                causes.push(PredicateCause::assigned(
                    PredicateCauseCode::IdentityConflict,
                    predicate_ref.to_string(),
                    None,
                ));
            }
        }
    }
    if !causes.is_empty() {
        return Err(rejection(causes, limits));
    }

    let built = match build(&definitions, &target, limits) {
        Ok(value) => value,
        Err(error) if error.code() == BridgeErrorCode::PredicateProjectionResourceExhausted => {
            return Err(PredicateDecision::Operation(error));
        }
        Err(error) => {
            let code = match error.path() {
                path if path.starts_with("signal_catalog") => PredicateCauseCode::CatalogRejected,
                path if path.starts_with("proposition_map") => PredicateCauseCode::MapRejected,
                _ => return Err(PredicateDecision::Operation(error)),
            };
            return Err(rejection(
                vec![PredicateCause::assigned(code, "", None)],
                limits,
            ));
        }
    };
    let projection_work = definitions
        .values()
        .fold(input_work, |work, definition| {
            work.saturating_add(definition.canonical_tuple().len())
        })
        .saturating_add(built.signal_catalog_bytes.len())
        .saturating_add(built.proposition_map_bytes.len());
    if projection_work > limits.visited_work {
        return Err(PredicateDecision::Operation(
            crate::bridge::BridgeError::new(
                BridgeErrorCode::PredicateProjectionResourceExhausted,
                "combined projection work exceeds the visited-work ceiling",
                "projection",
            ),
        ));
    }
    let mut decision = PredicateProjectionDecision {
        kind: super::PredicateProjectionKind::Admitted,
        bridge_profile: Some(super::PROFILE.to_owned()),
        native_contract: Some(target.native.clone()),
        signal_contract: Some(target.signal_catalog.clone()),
        map_contract: Some(target.proposition_map.clone()),
        correspondences: built.correspondences.clone(),
        signal_catalog_ref: Some(built.signal_catalog_ref),
        proposition_map_ref: Some(built.proposition_map_ref),
        projection_ref: Some(built.projection_ref),
        causes: Vec::new(),
        bytes: Vec::new(),
    };
    decision
        .encode(limits)
        .map_err(PredicateDecision::Operation)?;
    if projection_work.saturating_add(decision.bytes().len()) > limits.visited_work {
        return Err(PredicateDecision::Operation(
            crate::bridge::BridgeError::new(
                BridgeErrorCode::PredicateProjectionResourceExhausted,
                "projection decision exceeds the remaining visited-work ceiling",
                "projection_decision",
            ),
        ));
    }
    Ok(finish_projection(decision, built, definitions))
}

fn selection_causes(target: &TargetSelection) -> Vec<PredicateCause> {
    let current = TargetSelection::current();
    let mut causes = selection_cause(
        &target.native,
        &current.native,
        PredicateCauseCode::NativeContractUnavailable,
        PredicateCauseCode::NativeProfileUnsupported,
        PredicateCauseCode::NativeContractConflict,
    )
    .into_iter()
    .collect::<Vec<_>>();
    for (selected, expected) in [
        (&target.signal_catalog, &current.signal_catalog),
        (&target.proposition_map, &current.proposition_map),
    ] {
        if let Some(cause) = selection_cause(
            selected,
            expected,
            PredicateCauseCode::TargetContractUnavailable,
            PredicateCauseCode::TargetProfileUnsupported,
            PredicateCauseCode::TargetContractConflict,
        ) {
            causes.push(cause);
        }
    }
    sort_causes(&mut causes);
    causes
}

fn selection_cause(
    selected: &ContractSelection,
    expected: &ContractSelection,
    unavailable: PredicateCauseCode,
    unsupported: PredicateCauseCode,
    conflict: PredicateCauseCode,
) -> Option<PredicateCause> {
    let code = if !selected.structurally_valid() {
        unavailable
    } else if selected.contract() == expected.contract()
        && selected.package_version() == expected.package_version()
        && selected.repository() == expected.repository()
        && selected.revision() == expected.revision()
        && selected.schema_digest() != expected.schema_digest()
    {
        conflict
    } else if selected != expected {
        unsupported
    } else {
        return None;
    };
    Some(PredicateCause::assigned(
        code,
        selected.contract(),
        (!selected.revision().is_empty()).then(|| selected.revision().to_owned()),
    ))
}

fn rejection(mut causes: Vec<PredicateCause>, limits: BridgeLimits) -> PredicateDecision {
    sort_causes(&mut causes);
    if causes.len() > limits.causes {
        return PredicateDecision::Operation(crate::bridge::BridgeError::new(
            BridgeErrorCode::PredicateProjectionResourceExhausted,
            "projection cause ceiling exceeded",
            "causes",
        ));
    }
    let mut decision = PredicateProjectionDecision {
        kind: projection_kind(&causes),
        bridge_profile: None,
        native_contract: None,
        signal_contract: None,
        map_contract: None,
        correspondences: Vec::new(),
        signal_catalog_ref: None,
        proposition_map_ref: None,
        projection_ref: None,
        causes,
        bytes: Vec::new(),
    };
    match decision.encode(limits) {
        Ok(()) => PredicateDecision::Semantic(Box::new(decision)),
        Err(error) => PredicateDecision::Operation(error),
    }
}
