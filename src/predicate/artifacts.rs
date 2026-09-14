//! Deterministic TL signal/proposition artifacts and projection identity.

use std::collections::BTreeMap;

use serde::{Deserialize, Serialize};
use serde_json::json;
use tl_syntax::{
    OwnedSignalDeclaration, PropositionBinding, PropositionEntry, PropositionId,
    PropositionMapDocument, SignalCatalogDocument, SignalDomain, SignalId, SyntaxArtifactLimits,
};

use crate::bridge::{canonical, BridgeDigest, BridgeError, BridgeErrorCode, BridgeLimits};

use super::{
    admission::TargetSelection,
    decision::PredicateProjectionDecision,
    definition::{PredicateDefinition, PredicateRef},
    MAP_ARTIFACT_PROFILE, PROJECTION_REF_PROFILE, SIGNAL_ARTIFACT_PROFILE,
};

/// One deterministic predicate/proposition/signal correspondence.
#[derive(Clone, Debug, Eq, Ord, PartialEq, PartialOrd, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct PredicateCorrespondence {
    predicate_ref: PredicateRef,
    proposition_id: u32,
    proposition_name: String,
    signal_id: u32,
    signal_name: String,
}

impl PredicateCorrespondence {
    /// Returns the checked predicate identity.
    #[must_use]
    pub const fn predicate_ref(&self) -> PredicateRef {
        self.predicate_ref
    }

    /// Returns the zero-based TL proposition identity.
    #[must_use]
    pub const fn proposition_id(&self) -> u32 {
        self.proposition_id
    }

    /// Returns the generated proposition name.
    #[must_use]
    pub fn proposition_name(&self) -> &str {
        &self.proposition_name
    }

    /// Returns the zero-based signal identity.
    #[must_use]
    pub const fn signal_id(&self) -> u32 {
        self.signal_id
    }

    /// Returns the generated Boolean signal name.
    #[must_use]
    pub fn signal_name(&self) -> &str {
        &self.signal_name
    }
}

/// Complete projection emitted by [`super::project`].
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct PredicateProjection {
    validated: ValidatedPredicateProjection,
}

impl PredicateProjection {
    /// Returns the constructor-private validated projection for valuation.
    #[must_use]
    pub const fn validated(&self) -> &ValidatedPredicateProjection {
        &self.validated
    }

    /// Returns the canonical projection decision.
    #[must_use]
    pub const fn decision(&self) -> &PredicateProjectionDecision {
        &self.validated.decision
    }

    /// Returns exact canonical signal-catalog bytes.
    #[must_use]
    pub fn signal_catalog_bytes(&self) -> &[u8] {
        &self.validated.signal_catalog_bytes
    }

    /// Returns exact canonical proposition-map bytes.
    #[must_use]
    pub fn proposition_map_bytes(&self) -> &[u8] {
        &self.validated.proposition_map_bytes
    }
}

/// Projection admitted by construction or strict re-derivation.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ValidatedPredicateProjection {
    pub(crate) decision: PredicateProjectionDecision,
    pub(crate) signal_catalog_bytes: Vec<u8>,
    pub(crate) proposition_map_bytes: Vec<u8>,
    pub(crate) signal_catalog: SignalCatalogDocument,
    pub(crate) proposition_map: PropositionMapDocument,
    pub(crate) definitions: BTreeMap<PredicateRef, PredicateDefinition>,
}

impl ValidatedPredicateProjection {
    /// Returns the admitted projection decision.
    #[must_use]
    pub const fn decision(&self) -> &PredicateProjectionDecision {
        &self.decision
    }

    /// Returns deterministic correspondences in proposition order.
    #[must_use]
    pub fn correspondences(&self) -> &[PredicateCorrespondence] {
        self.decision.correspondences()
    }

    /// Returns exact canonical signal-catalog bytes.
    #[must_use]
    pub fn signal_catalog_bytes(&self) -> &[u8] {
        &self.signal_catalog_bytes
    }

    /// Returns exact canonical proposition-map bytes.
    #[must_use]
    pub fn proposition_map_bytes(&self) -> &[u8] {
        &self.proposition_map_bytes
    }

    /// Returns the owner-admitted predicate definition for one correspondence.
    #[must_use]
    pub fn definition(&self, predicate_ref: PredicateRef) -> Option<&PredicateDefinition> {
        self.definitions.get(&predicate_ref)
    }

    pub(crate) fn correspondence(
        &self,
        predicate_ref: PredicateRef,
    ) -> Option<&PredicateCorrespondence> {
        self.correspondences()
            .binary_search_by_key(&predicate_ref, PredicateCorrespondence::predicate_ref)
            .ok()
            .and_then(|index| self.correspondences().get(index))
    }
}

pub(crate) struct BuiltArtifacts {
    pub(crate) correspondences: Vec<PredicateCorrespondence>,
    pub(crate) signal_catalog: SignalCatalogDocument,
    pub(crate) signal_catalog_bytes: Vec<u8>,
    pub(crate) signal_catalog_ref: BridgeDigest,
    pub(crate) proposition_map: PropositionMapDocument,
    pub(crate) proposition_map_bytes: Vec<u8>,
    pub(crate) proposition_map_ref: BridgeDigest,
    pub(crate) projection_ref: BridgeDigest,
}

pub(crate) fn build(
    definitions: &BTreeMap<PredicateRef, PredicateDefinition>,
    target: &TargetSelection,
    limits: BridgeLimits,
) -> Result<BuiltArtifacts, BridgeError> {
    let limits = limits.effective();
    reserve(definitions.len().saturating_mul(384), limits)?;
    let mut correspondences = Vec::new();
    let mut signals = Vec::new();
    let mut bindings = Vec::new();
    let mut propositions = Vec::new();
    correspondences
        .try_reserve_exact(definitions.len())
        .map_err(|_| resource("correspondence allocation failed"))?;
    signals
        .try_reserve_exact(definitions.len())
        .map_err(|_| resource("signal allocation failed"))?;
    bindings
        .try_reserve_exact(definitions.len())
        .map_err(|_| resource("binding allocation failed"))?;
    propositions
        .try_reserve_exact(definitions.len())
        .map_err(|_| resource("proposition allocation failed"))?;

    for (ordinal, predicate_ref) in definitions.keys().copied().enumerate() {
        let id = u32::try_from(ordinal).map_err(|_| resource("predicate identity overflow"))?;
        let name = format!("quire-predicate/{predicate_ref}");
        correspondences.push(PredicateCorrespondence {
            predicate_ref,
            proposition_id: id,
            proposition_name: name.clone(),
            signal_id: id,
            signal_name: name.clone(),
        });
        signals.push(OwnedSignalDeclaration::new(
            SignalId(id),
            name.clone(),
            SignalDomain::Boolean,
        ));
        bindings.push(PropositionBinding::new(PropositionId(id), SignalId(id)));
        propositions.push(PropositionEntry {
            id: PropositionId(id),
            name,
        });
    }

    let signal_catalog = SignalCatalogDocument::new(signals, bindings)
        .map_err(|_| invalid("generated signal catalog was rejected", "signal_catalog"))?;
    let signal_catalog_bytes = signal_catalog
        .canonical_json_bytes()
        .map_err(|_| resource("signal catalog serialization failed"))?;
    let syntax_limits = syntax_limits(limits);
    let signal_catalog =
        SignalCatalogDocument::from_json_bytes(&signal_catalog_bytes, syntax_limits).map_err(
            |_| {
                invalid(
                    "public signal-catalog reader rejected output",
                    "signal_catalog",
                )
            },
        )?;
    let proposition_map = PropositionMapDocument::new(propositions)
        .map_err(|_| invalid("generated proposition map was rejected", "proposition_map"))?;
    let proposition_map_bytes = proposition_map
        .canonical_json_bytes()
        .map_err(|_| resource("proposition map serialization failed"))?;
    let proposition_map =
        PropositionMapDocument::from_json_bytes(&proposition_map_bytes, syntax_limits).map_err(
            |_| {
                invalid(
                    "public proposition-map reader rejected output",
                    "proposition_map",
                )
            },
        )?;
    validate_join(&signal_catalog, &proposition_map, &correspondences)?;

    let signal_catalog_ref = BridgeDigest::domain(SIGNAL_ARTIFACT_PROFILE, &signal_catalog_bytes);
    let proposition_map_ref = BridgeDigest::domain(MAP_ARTIFACT_PROFILE, &proposition_map_bytes);
    let tuple = json!({
        "bridge_profile": super::PROFILE,
        "correspondences": correspondences,
        "native_contract": target.native(),
        "proposition_map_ref": proposition_map_ref,
        "proposition_map_target": target.proposition_map(),
        "signal_catalog_ref": signal_catalog_ref,
        "signal_catalog_target": target.signal_catalog(),
    });
    let tuple_bytes = canonical::encode(&tuple, limits)?;
    let projection_ref = BridgeDigest::domain(PROJECTION_REF_PROFILE, &tuple_bytes);
    Ok(BuiltArtifacts {
        correspondences,
        signal_catalog,
        signal_catalog_bytes,
        signal_catalog_ref,
        proposition_map,
        proposition_map_bytes,
        proposition_map_ref,
        projection_ref,
    })
}

pub(crate) fn finish_projection(
    decision: PredicateProjectionDecision,
    built: BuiltArtifacts,
    definitions: BTreeMap<PredicateRef, PredicateDefinition>,
) -> PredicateProjection {
    PredicateProjection {
        validated: ValidatedPredicateProjection {
            decision,
            signal_catalog_bytes: built.signal_catalog_bytes,
            proposition_map_bytes: built.proposition_map_bytes,
            signal_catalog: built.signal_catalog,
            proposition_map: built.proposition_map,
            definitions,
        },
    }
}

pub(crate) fn validate_join(
    catalog: &SignalCatalogDocument,
    map: &PropositionMapDocument,
    correspondences: &[PredicateCorrespondence],
) -> Result<(), BridgeError> {
    let catalog = catalog
        .validate()
        .map_err(|_| invalid("signal catalog does not validate", "signal_catalog"))?;
    if catalog.signal_count() != correspondences.len()
        || catalog.bindings().len() != correspondences.len()
        || map.propositions().len() != correspondences.len()
    {
        return Err(invalid(
            "catalog, map, and correspondence populations differ",
            "proposition_map",
        ));
    }
    for (index, correspondence) in correspondences.iter().enumerate() {
        let signal = catalog
            .signals()
            .nth(index)
            .ok_or_else(|| invalid("signal is missing", "signal_catalog.signals"))?;
        let binding = catalog
            .bindings()
            .get(index)
            .ok_or_else(|| invalid("binding is missing", "signal_catalog.bindings"))?;
        let proposition = map
            .propositions()
            .get(index)
            .ok_or_else(|| invalid("proposition is missing", "proposition_map.propositions"))?;
        if signal.id().0 != correspondence.signal_id
            || signal.name() != correspondence.signal_name
            || signal.domain() != SignalDomain::Boolean
            || binding.proposition().0 != correspondence.proposition_id
            || binding.signal().0 != correspondence.signal_id
            || proposition.id.0 != correspondence.proposition_id
            || proposition.name != correspondence.proposition_name
        {
            return Err(invalid(
                "catalog/map correspondence is not a Boolean bijection",
                "proposition_map",
            ));
        }
    }
    Ok(())
}

pub(crate) fn syntax_limits(limits: BridgeLimits) -> SyntaxArtifactLimits {
    let owner = SyntaxArtifactLimits::OWNER_MAXIMA;
    SyntaxArtifactLimits {
        document_bytes: limits.document_bytes.min(owner.document_bytes),
        json_depth: limits.json_depth.min(owner.json_depth),
        string_bytes: limits.string_bytes.min(owner.string_bytes),
        formula_nodes: owner.formula_nodes,
        formula_depth: owner.formula_depth,
        signals: limits.predicates.min(owner.signals),
        bindings: limits.predicates.min(owner.bindings),
        propositions: limits.predicates.min(owner.propositions),
        work: limits.visited_work.min(owner.work),
    }
}

fn reserve(bytes: usize, limits: BridgeLimits) -> Result<(), BridgeError> {
    if bytes > limits.allocation_bytes {
        Err(resource(
            "artifact reservation exceeds caller allocation ceiling",
        ))
    } else {
        Ok(())
    }
}

fn invalid(message: &'static str, path: &'static str) -> BridgeError {
    BridgeError::new(
        BridgeErrorCode::InvalidNativePredicateProjection,
        message,
        path,
    )
}

fn resource(message: &'static str) -> BridgeError {
    BridgeError::new(
        BridgeErrorCode::PredicateProjectionResourceExhausted,
        message,
        "projection",
    )
}

#[cfg(test)]
mod tests {
    use ix_trace_rs::trace;

    use super::*;

    fn predicate_ref() -> PredicateRef {
        PredicateRef::parse("1111111111111111111111111111111111111111111111111111111111111111")
            .expect("fixture predicate digest")
    }

    fn correspondence() -> PredicateCorrespondence {
        PredicateCorrespondence {
            predicate_ref: predicate_ref(),
            proposition_id: 0,
            proposition_name: format!("quire-predicate/{}", predicate_ref()),
            signal_id: 0,
            signal_name: format!("quire-predicate/{}", predicate_ref()),
        }
    }

    fn catalog(
        signal_id: u32,
        name: &str,
        domain: SignalDomain,
        proposition_id: u32,
    ) -> SignalCatalogDocument {
        SignalCatalogDocument::new(
            vec![OwnedSignalDeclaration::new(
                SignalId(signal_id),
                name.to_owned(),
                domain,
            )],
            vec![PropositionBinding::new(
                PropositionId(proposition_id),
                SignalId(signal_id),
            )],
        )
        .expect("valid catalog fixture")
    }

    fn map(proposition_id: u32, name: &str) -> PropositionMapDocument {
        PropositionMapDocument::new(vec![PropositionEntry {
            id: PropositionId(proposition_id),
            name: name.to_owned(),
        }])
        .expect("valid proposition-map fixture")
    }

    #[trace("TC-038", "FR-025-AC-1", "FR-025-AC-6", "FR-025-AC-8")]
    #[test]
    fn tc_038_join_rejects_every_population_id_name_domain_and_binding_disagreement() {
        let expected = correspondence();
        let name = expected.signal_name().to_owned();
        let valid_catalog = catalog(0, &name, SignalDomain::Boolean, 0);
        let valid_map = map(0, &name);
        assert!(validate_join(&valid_catalog, &valid_map, std::slice::from_ref(&expected)).is_ok());

        assert!(validate_join(&valid_catalog, &valid_map, &[]).is_err());
        assert!(validate_join(
            &catalog(1, &name, SignalDomain::Boolean, 0),
            &valid_map,
            std::slice::from_ref(&expected),
        )
        .is_err());
        assert!(validate_join(
            &catalog(0, "quire-predicate/wrong", SignalDomain::Boolean, 0),
            &valid_map,
            std::slice::from_ref(&expected),
        )
        .is_err());
        assert!(SignalCatalogDocument::new(
            vec![OwnedSignalDeclaration::new(
                SignalId(0),
                name.clone(),
                SignalDomain::Integer(
                    tl_syntax::IntegerSignalDomain::new(0, 1).expect("valid integer signal domain"),
                ),
            )],
            vec![PropositionBinding::new(PropositionId(0), SignalId(0))],
        )
        .is_err());
        assert!(validate_join(
            &catalog(0, &name, SignalDomain::Boolean, 1),
            &valid_map,
            std::slice::from_ref(&expected),
        )
        .is_err());
        assert!(validate_join(
            &valid_catalog,
            &map(1, &name),
            std::slice::from_ref(&expected),
        )
        .is_err());
        assert!(validate_join(
            &valid_catalog,
            &map(0, "quire-predicate/wrong"),
            std::slice::from_ref(&expected),
        )
        .is_err());
    }
}
