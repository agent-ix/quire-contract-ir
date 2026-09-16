//! Typed V1-to-V2 re-link: the closed `MigrationOutcome` of QSpec
//! `migration-correspondence.schema.json`.
//!
//! The V1 source is consumed only as an admitted package reference. Nothing
//! here converts, relabels or fills V1 bytes: the V2 target must already be an
//! independently admitted package rebuilt from exact reconstruction inputs.

use super::v1::{
    CheckedArtifactRef, CheckedNodeId, CheckedPackage, CheckedPackageLock, CheckedSelection,
    CheckedSemanticId, CHECKED_PACKAGE_V1,
};
use super::v2::{CheckedPackageV2, CHECKED_PACKAGE_V2};
use serde::{Deserialize, Serialize};
use std::collections::BTreeSet;

/// Correspondence receipt version.
pub const MIGRATION_CORRESPONDENCE_V1: &str = "quire.checked-package-migration-correspondence/v1";

/// Caller-supplied reconstruction inputs; any member may be absent.
#[derive(Clone, Debug, Default, Eq, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct MigrationReconstructionRequest {
    /// Raw source documents.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub sources: Option<Vec<CheckedArtifactRef>>,
    /// Selected edition.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub edition: Option<CheckedSelection>,
    /// Selected profiles.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub profile_selections: Option<Vec<CheckedSelection>>,
    /// Selected definitions.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub definition_selections: Option<Vec<CheckedArtifactRef>>,
    /// Selected models.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub model_selections: Option<Vec<CheckedArtifactRef>>,
    /// Dependency selections.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub dependency_selections: Option<Vec<CheckedSelection>>,
}

/// A migration request: the claimed target key and the reconstruction inputs.
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct CheckedPackageMigrationRequest {
    /// The V2 package key the caller claims the target has.
    pub target_package_id: CheckedSemanticId,
    /// Exact reconstruction inputs; absent is byte-only reconstruction.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub reconstruction_inputs: Option<MigrationReconstructionRequest>,
}

/// Exact reconstruction inputs recorded in a relinked correspondence.
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct MigrationReconstructionInputs {
    /// Raw source documents.
    pub sources: Vec<CheckedArtifactRef>,
    /// Selected edition.
    pub edition: CheckedSelection,
    /// Selected profiles.
    pub profile_selections: Vec<CheckedSelection>,
    /// Selected definitions.
    pub definition_selections: Vec<CheckedArtifactRef>,
    /// Selected models.
    pub model_selections: Vec<CheckedArtifactRef>,
    /// Dependency selections.
    pub dependency_selections: Vec<CheckedSelection>,
}

/// Why one V1 node re-links to its V2 counterpart.
#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub enum CorrespondenceBasis {
    /// Source and target node digests are equal.
    #[serde(rename = "relinked-identical")]
    RelinkedIdentical,
    /// The target node carries a re-derived key.
    #[serde(rename = "relinked-rekeyed")]
    RelinkedRekeyed,
}

/// One ordered V1-node to V2-node correspondence.
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct NodeCorrespondence {
    /// Graph position shared by both nodes.
    pub ordinal: u64,
    /// V1 node key.
    pub source_node_id: CheckedNodeId,
    /// V2 node key.
    pub target_node_id: CheckedNodeId,
    /// Re-link basis.
    pub basis: CorrespondenceBasis,
}

/// Typed V1-to-V2 re-link receipt.
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct CheckedPackageMigrationCorrespondence {
    /// `quire.checked-package-migration-correspondence/v1`.
    pub version: Box<str>,
    /// `quire.checked-package/v1`.
    pub source_contract_version: Box<str>,
    /// `quire.checked-package/v2`.
    pub target_contract_version: Box<str>,
    /// V1 package key.
    pub source_package_id: CheckedSemanticId,
    /// V2 package key.
    pub target_package_id: CheckedSemanticId,
    /// Exact reconstruction inputs.
    pub reconstruction_inputs: MigrationReconstructionInputs,
    /// One ordered row per V1 node.
    pub node_correspondences: Vec<NodeCorrespondence>,
}

/// Closed migration refusal code.
#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum MigrationRefusalCode {
    /// A required reconstruction input is absent.
    MigrationInputMissing,
    /// A reconstruction input differs from the locked evidence.
    MigrationInputStale,
    /// A reconstruction input names one artifact more than once.
    MigrationInputAmbiguous,
    /// The target is not the package rebuilt from these inputs.
    MigrationTargetIncompatible,
    /// Only V1 bytes were supplied; no semantic reconstruction is possible.
    MigrationByteOnlyReconstruction,
}

/// The input a refusal names.
#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum MigrationInput {
    /// The whole reconstruction input record.
    ReconstructionInputs,
    /// `sources`.
    Sources,
    /// `edition`.
    Edition,
    /// `profile_selections`.
    ProfileSelections,
    /// `definition_selections`.
    DefinitionSelections,
    /// `model_selections`.
    ModelSelections,
    /// `dependency_selections`.
    DependencySelections,
    /// `target_package_id`.
    TargetPackageId,
}

/// The subject of a migration refusal.
#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
pub struct MigrationRefusalSubject {
    /// Named input.
    pub input: MigrationInput,
    /// Artifact authority and identity, when one artifact is the subject.
    #[serde(flatten, skip_serializing_if = "Option::is_none")]
    pub artifact: Option<MigrationArtifactSubject>,
}

/// An artifact named by authority and identity.
#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
pub struct MigrationArtifactSubject {
    /// Artifact authority.
    pub authority: Box<str>,
    /// Artifact identity.
    pub identity: Box<str>,
}

/// The closed V1-to-V2 migration outcome.
#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
#[serde(tag = "outcome", rename_all = "snake_case")]
pub enum MigrationOutcome {
    /// Every node re-linked under the recorded correspondence.
    Relinked {
        /// The receipt.
        correspondence: Box<CheckedPackageMigrationCorrespondence>,
    },
    /// The first failing check, in the normative order.
    Refused {
        /// Refusal code.
        code: MigrationRefusalCode,
        /// Refusal subject.
        subject: MigrationRefusalSubject,
    },
}

fn refused(code: MigrationRefusalCode, input: MigrationInput) -> MigrationOutcome {
    MigrationOutcome::Refused {
        code,
        subject: MigrationRefusalSubject {
            input,
            artifact: None,
        },
    }
}

fn refused_artifact(
    code: MigrationRefusalCode,
    input: MigrationInput,
    artifact: &CheckedArtifactRef,
) -> MigrationOutcome {
    MigrationOutcome::Refused {
        code,
        subject: MigrationRefusalSubject {
            input,
            artifact: Some(MigrationArtifactSubject {
                authority: artifact.authority.clone(),
                identity: artifact.identity.clone(),
            }),
        },
    }
}

fn same_artifact(left: &CheckedArtifactRef, right: &CheckedArtifactRef) -> bool {
    left.authority == right.authority && left.identity == right.identity
}

/// The first artifact whose authority, identity and (for models) export
/// repeat an earlier entry of the same list.
fn first_duplicate(artifacts: &[CheckedArtifactRef]) -> Option<&CheckedArtifactRef> {
    let mut seen = BTreeSet::new();
    artifacts
        .iter()
        .find(|artifact| !seen.insert((&artifact.authority, &artifact.identity, &artifact.export)))
}

/// Source documents are a set keyed by authority and identity: order carries
/// no meaning, so two duplicate-free lists match when they hold equal members.
fn same_sources(left: &[CheckedArtifactRef], right: &[CheckedArtifactRef]) -> bool {
    left.len() == right.len() && left.iter().all(|source| right.contains(source))
}

/// Re-links an admitted V1 package to an admitted V2 package.
///
/// Checks run in the normative order byte-only, missing, ambiguous, stale,
/// target-incompatible; only the first failing check is reported. Sources are
/// compared as a set in both the stale and the target check; selections are
/// ordered, as they are in the package identity preimage.
pub fn migrate_checked_package(
    source: &CheckedPackage,
    target: &CheckedPackageV2,
    request: &CheckedPackageMigrationRequest,
) -> MigrationOutcome {
    use MigrationInput as Input;
    use MigrationRefusalCode as Code;

    let Some(draft) = &request.reconstruction_inputs else {
        return refused(
            Code::MigrationByteOnlyReconstruction,
            Input::ReconstructionInputs,
        );
    };
    let lock = source.lock();

    // Missing.
    let Some(sources) = draft.sources.as_ref().filter(|sources| !sources.is_empty()) else {
        return refused(Code::MigrationInputMissing, Input::Sources);
    };
    if let Some(absent) = lock
        .sources
        .iter()
        .find(|locked| !sources.iter().any(|source| same_artifact(source, locked)))
    {
        return refused_artifact(Code::MigrationInputMissing, Input::Sources, absent);
    }
    let Some(edition) = &draft.edition else {
        return refused(Code::MigrationInputMissing, Input::Edition);
    };
    let Some(profile_selections) = &draft.profile_selections else {
        return refused(Code::MigrationInputMissing, Input::ProfileSelections);
    };
    let Some(definition_selections) = &draft.definition_selections else {
        return refused(Code::MigrationInputMissing, Input::DefinitionSelections);
    };
    let Some(model_selections) = &draft.model_selections else {
        return refused(Code::MigrationInputMissing, Input::ModelSelections);
    };
    let Some(dependency_selections) = &draft.dependency_selections else {
        return refused(Code::MigrationInputMissing, Input::DependencySelections);
    };

    // Ambiguous: one artifact named twice within one artifact-list input.
    let artifact_lists: [(&[CheckedArtifactRef], Input); 3] = [
        (sources, Input::Sources),
        (definition_selections, Input::DefinitionSelections),
        (model_selections, Input::ModelSelections),
    ];
    for (artifacts, input) in artifact_lists {
        if let Some(duplicate) = first_duplicate(artifacts) {
            return refused_artifact(Code::MigrationInputAmbiguous, input, duplicate);
        }
    }

    // Stale.
    if let Some(stale) = sources.iter().find(|source| !lock.sources.contains(source)) {
        return refused_artifact(Code::MigrationInputStale, Input::Sources, stale);
    }
    let checks: [(bool, Input); 5] = [
        (*edition == lock.edition, Input::Edition),
        (
            *profile_selections == lock.profile_selections,
            Input::ProfileSelections,
        ),
        (
            *definition_selections == lock.definition_selections,
            Input::DefinitionSelections,
        ),
        (
            *model_selections == lock.model_selections,
            Input::ModelSelections,
        ),
        (
            *dependency_selections == lock.dependency_selections,
            Input::DependencySelections,
        ),
    ];
    if let Some((_, input)) = checks.iter().find(|(current, _)| !current) {
        return refused(Code::MigrationInputStale, *input);
    }

    // Target.
    let inputs = MigrationReconstructionInputs {
        sources: sources.clone(),
        edition: edition.clone(),
        profile_selections: profile_selections.clone(),
        definition_selections: definition_selections.clone(),
        model_selections: model_selections.clone(),
        dependency_selections: dependency_selections.clone(),
    };
    let Some(node_correspondences) = relink_nodes(source, target, lock, &inputs, request) else {
        return refused(Code::MigrationTargetIncompatible, Input::TargetPackageId);
    };
    MigrationOutcome::Relinked {
        correspondence: Box::new(CheckedPackageMigrationCorrespondence {
            version: MIGRATION_CORRESPONDENCE_V1.into(),
            source_contract_version: CHECKED_PACKAGE_V1.into(),
            target_contract_version: CHECKED_PACKAGE_V2.into(),
            source_package_id: source.package_id().clone(),
            target_package_id: target.package_id().clone(),
            reconstruction_inputs: inputs,
            node_correspondences,
        }),
    }
}

/// Returns the ordered correspondence when the target is exactly the package
/// rebuilt from `inputs`, or `None` when it is incompatible.
fn relink_nodes(
    source: &CheckedPackage,
    target: &CheckedPackageV2,
    source_lock: &CheckedPackageLock,
    inputs: &MigrationReconstructionInputs,
    request: &CheckedPackageMigrationRequest,
) -> Option<Vec<NodeCorrespondence>> {
    let target_lock = target.lock();
    let compatible = request.target_package_id == *target.package_id()
        && same_sources(&target_lock.sources, &inputs.sources)
        && target_lock.edition == inputs.edition
        && target_lock.profile_selections == inputs.profile_selections
        && target_lock.definition_selections == inputs.definition_selections
        && target_lock.model_selections == inputs.model_selections
        && target_lock.dependency_selections == inputs.dependency_selections
        && target_lock.required_features == source_lock.required_features;
    let source_nodes = &source.graph().nodes;
    let target_nodes = &target.graph().nodes;
    if !compatible || source_nodes.len() != target_nodes.len() {
        return None;
    }
    source_nodes
        .iter()
        .zip(target_nodes)
        .enumerate()
        .map(|(ordinal, (from, to))| {
            (from.node_tag == to.node_tag && from.semantic_form == to.semantic_form).then(|| {
                NodeCorrespondence {
                    ordinal: u64::try_from(ordinal).unwrap_or(u64::MAX),
                    source_node_id: from.node_id.clone(),
                    target_node_id: to.node_id.clone(),
                    basis: if from.node_id == to.node_id {
                        CorrespondenceBasis::RelinkedIdentical
                    } else {
                        CorrespondenceBasis::RelinkedRekeyed
                    },
                }
            })
        })
        .collect()
}
