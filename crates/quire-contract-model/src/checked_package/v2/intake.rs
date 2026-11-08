//! The closed-schema decode of the package document, without native recursion
//! following the nesting of a term.
//!
//! A node `body` and a diagnostic's `details` entry may nest as deep as the
//! strict parser's limit of 128. The outer typed decode retains handles into
//! the shared document and decodes admitted terms after the outer frame has
//! returned. A `NodeId.path` contains fixed-depth string or graph-index
//! segments, so a derived chain does not increase JSON nesting depth.
//!
//! The lossless-decode check (no member defaulted, nulled or dropped) compares
//! document nodes against their typed serialization one at a time, including
//! structural node identities.

use super::NativeJoinBudget;
use super::TermNode;
use super::{
    CheckedDeclaration, CheckedDiagnosticCause, CheckedDiagnosticCode, CheckedDiagnosticStage,
    CheckedDiagnosticV2, CheckedDiagnosticsV2, CheckedJson, CheckedNodeOwner,
    CheckedPackageEnvelope, CheckedPackageLockV2, RawCheckedPackageWireV2, RawSemanticGraphV2,
    RawSemanticNodeV2, ValidationFailure,
};
use crate::checked_package::common::{
    decode_closed_document_with_classifier, node_pointer, Step, Trail,
};
use crate::checked_package::shared::CheckedPackageRefusalCode;
use crate::checked_package::shared::{
    CheckedArtifactRef, CheckedCapability, CheckedNodeId, CheckedOccurrence, CheckedOperandEntry,
    CheckedSourceMapEntry, CheckedSourceRegion,
};
use quire_canonical::Document;
use serde::{Deserialize, Deserializer};
use std::sync::Arc;

/// Check a present position ordinal before semantic graph validation.
#[ix_trace_rs::string_edge]
fn key_ordinal_node(value: TermNode<'_>) -> bool {
    const MAX_SAFE: &str = "9007199254740991";
    if value
        .as_u64()
        .is_some_and(|number| number <= 9_007_199_254_740_991)
    {
        return true;
    }
    let Some(text) = value.as_str() else {
        return false;
    };
    !text.is_empty()
        && !text.starts_with('0')
        && text.bytes().all(|byte| byte.is_ascii_digit())
        && (text.len() > MAX_SAFE.len() || (text.len() == MAX_SAFE.len() && text > MAX_SAFE))
}

fn position_member_shape(body: TermNode<'_>) -> bool {
    if body.get("term").and_then(TermNode::as_str) == Some("application") {
        if let Some(member) = body
            .get("operation")
            .and_then(|operation| operation.get("member"))
        {
            if member.get("kind").and_then(TermNode::as_str) == Some("position") {
                if let Some(position) = member.get("position") {
                    if !key_ordinal_node(position) {
                        return false;
                    }
                }
            }
        }
    }
    body.items()
        .is_none_or(|items| items.into_iter().all(position_member_shape))
        && body.members().is_none_or(|members| {
            members
                .into_iter()
                .all(|(_, value)| position_member_shape(value))
        })
}

// The borrowed intake mirrors only the closed containers that contain an
// arbitrary JSON term. Those terms are decoded after the outer typed wire
// frame has returned, keeping their recursion off its call stack. Every other
// member is decoded by the same public type as the admitted wire.
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct BorrowedWire {
    contract_version: Box<str>,
    name: Box<str>,
    lock: CheckedPackageLockV2,
    semantic_graph: BorrowedGraph,
    #[serde(deserialize_with = "super::document_de::exact_vec")]
    source_map: Vec<CheckedSourceMapEntry>,
    #[serde(default, deserialize_with = "super::document_de::exact_optional_vec")]
    refinement_step_rows: Option<Vec<super::refinement::CheckedRefinementStepRowRegion>>,
    #[serde(default)]
    checked_term_data: Option<super::checked_term_data::CheckedTermDataV2>,
    #[serde(default, deserialize_with = "super::document_de::exact_optional_vec")]
    temporal_bindings: Option<Vec<super::temporal_binding::TemporalBinding>>,
    #[serde(default, deserialize_with = "super::document_de::exact_optional_vec")]
    temporal_clause_names: Option<Vec<super::TemporalClauseName>>,
    #[serde(default, deserialize_with = "super::document_de::exact_optional_vec")]
    schedulability_claims: Option<Vec<super::CheckedSchedulabilityClaimRow>>,
    #[serde(default, deserialize_with = "super::operation_effects::present_rows")]
    operation_effects: Option<Vec<super::CheckedOperationEffectRow>>,
    #[serde(deserialize_with = "super::document_de::exact_vec")]
    capability_report: Vec<CheckedCapability>,
    diagnostics: BorrowedDiagnostics,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct BorrowedGraph {
    graph_version: Box<str>,
    #[serde(deserialize_with = "super::document_de::exact_vec")]
    nodes: Vec<BorrowedNode>,
    #[serde(deserialize_with = "super::document_de::exact_rows")]
    operand_rows: Vec<Vec<CheckedOperandEntry>>,
}

// Keep native wire errors at the logical ID boundary. The refinement below
// then locates an invalid ordinal or path segment without exposing an
// implementation-dependent Serde member path. The public ID deserializer
// remains the one owner of the flat ID grammar.
struct WireNodeId(CheckedNodeId);

impl<'de> Deserialize<'de> for WireNodeId {
    fn deserialize<D: Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        // The document deserializer decodes the id whole, so a fault inside
        // it is reported at the id and not at a member of it.
        struct Held;
        impl<'de> serde::de::Visitor<'de> for Held {
            type Value = WireNodeId;

            fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                formatter.write_str("a structural node id")
            }

            fn visit_unit<E: serde::de::Error>(self) -> Result<WireNodeId, E> {
                super::document_de::take_node_id()
                    .map(WireNodeId)
                    .ok_or_else(|| E::custom("no node id was decoded"))
            }

            fn visit_newtype_struct<D2: Deserializer<'de>>(
                self,
                inner: D2,
            ) -> Result<WireNodeId, D2::Error> {
                CheckedNodeId::deserialize(inner).map(WireNodeId)
            }
        }
        deserializer.deserialize_newtype_struct(super::document_de::NODE_ID_TOKEN, Held)
    }
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct BorrowedNode {
    node_id: WireNodeId,
    schema_version: Box<str>,
    node_tag: Box<str>,
    semantic_form: Box<str>,
    semantic_type: WireNodeId,
    #[serde(deserialize_with = "super::document_de::exact_vec")]
    dependencies: Vec<WireNodeId>,
    #[serde(deserialize_with = "super::document_de::exact_vec")]
    occurrences: Vec<CheckedOccurrence>,
    #[serde(default)]
    recursion_group: Option<Box<str>>,
    #[serde(default, deserialize_with = "present_nullable")]
    declaration: Option<Option<CheckedDeclaration>>,
    #[serde(default)]
    owner: Option<CheckedNodeOwner>,
    #[serde(default)]
    owner_model: Option<WireNodeId>,
    body: CheckedJson,
}

// The outer option records member presence; the inner option records JSON null.
fn present_nullable<'de, D, T>(deserializer: D) -> Result<Option<Option<T>>, D::Error>
where
    D: Deserializer<'de>,
    T: Deserialize<'de>,
{
    Option::<T>::deserialize(deserializer).map(Some)
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct BorrowedDiagnostics {
    catalog: CheckedArtifactRef,
    #[serde(deserialize_with = "super::document_de::exact_vec")]
    entries: Vec<BorrowedDiagnostic>,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct BorrowedDiagnostic {
    stage: CheckedDiagnosticStage,
    code: CheckedDiagnosticCode,
    cause_tag: CheckedDiagnosticCause,
    #[serde(deserialize_with = "super::document_de::exact_vec")]
    details: Vec<CheckedJson>,
    #[serde(deserialize_with = "super::document_de::exact_vec")]
    loci: Vec<CheckedSourceRegion>,
}

#[ix_trace_rs::string_edge]
pub(super) fn decode_borrowed_wire(
    document: &Arc<Document>,
    budget: &mut NativeJoinBudget,
) -> Result<(RawCheckedPackageWireV2, Vec<CheckedJson>), ValidationFailure> {
    let wire: BorrowedWire =
        decode_closed_document_with_classifier(document, budget, |code, steps, meter| {
            classify_wire_fault(document, code, steps, meter)
        })?;
    if wire
        .schedulability_claims
        .as_ref()
        .is_some_and(Vec::is_empty)
    {
        return Err(malformed(
            super::stage_storage::Path::root().key("schedulability_claims"),
            &mut super::WorkMeter::over(budget),
        ));
    }
    let graph = wire.semantic_graph;
    let diagnostics = wire.diagnostics;
    let mut nodes = super::read_charge::with_capacity(
        budget,
        super::read_charge::Site::new(
            super::PackageReadStage::WireDecode,
            0,
            "semantic_graph.nodes",
        ),
        graph.nodes.len(),
    )?;
    // The node bodies as handles into the retained document; validators read
    // these directly through TermNode.
    let mut bodies = super::read_charge::with_capacity(
        budget,
        super::read_charge::Site::new(
            super::PackageReadStage::WireDecode,
            0,
            "semantic_graph.nodes",
        ),
        graph.nodes.len(),
    )?;
    for (position, node) in graph.nodes.into_iter().enumerate() {
        if node.node_tag.as_ref() == "scalar_type"
            && node.semantic_form.as_ref() == "dimension_equivalence"
            && node.declaration.is_some()
        {
            return Err(ValidationFailure::refused(
                CheckedPackageRefusalCode::MalformedWire,
                node_pointer(position),
            ));
        }
        bodies.push(node.body.clone());
        let body = node.body;
        if !position_member_shape(TermNode::of(&body)) {
            return Err(ValidationFailure::refused(
                CheckedPackageRefusalCode::MalformedWire,
                node_pointer(position),
            ));
        }
        let mut dependencies = super::read_charge::with_capacity(
            budget,
            super::read_charge::Site::new(
                super::PackageReadStage::WireDecode,
                position,
                "dependencies",
            ),
            node.dependencies.len(),
        )?;
        dependencies.extend(node.dependencies.into_iter().map(|id| id.0));
        nodes.push(RawSemanticNodeV2 {
            node_id: node.node_id.0,
            schema_version: node.schema_version,
            node_tag: node.node_tag,
            semantic_form: node.semantic_form,
            semantic_type: node.semantic_type.0,
            dependencies,
            occurrences: node.occurrences,
            recursion_group: node.recursion_group,
            declaration: node.declaration.flatten(),
            owner: node.owner,
            owner_model: node.owner_model.map(|id| id.0),
            body,
        });
    }
    let mut diagnostic_entries = super::read_charge::with_capacity(
        budget,
        super::read_charge::Site::new(
            super::PackageReadStage::WireDecode,
            0,
            "diagnostics.entries",
        ),
        diagnostics.entries.len(),
    )?;
    for entry in diagnostics.entries {
        diagnostic_entries.push(CheckedDiagnosticV2 {
            stage: entry.stage,
            code: entry.code,
            cause_tag: entry.cause_tag,
            details: entry.details,
            loci: entry.loci,
        });
    }
    let envelope = CheckedPackageEnvelope {
        contract_version: wire.contract_version,
        name: wire.name,
        lock: wire.lock,
        semantic_graph: RawSemanticGraphV2 {
            graph_version: graph.graph_version,
            nodes,
            operand_rows: graph.operand_rows,
        },
        source_map: wire.source_map,
        refinement_step_rows: wire.refinement_step_rows,
        checked_term_data: wire.checked_term_data,
        temporal_bindings: wire.temporal_bindings,
        temporal_clause_names: wire.temporal_clause_names,
        schedulability_claims: wire.schedulability_claims,
        operation_effects: wire.operation_effects,
        capability_report: wire.capability_report,
        diagnostics: CheckedDiagnosticsV2 {
            catalog: diagnostics.catalog,
            entries: diagnostic_entries,
        },
    };
    Ok((envelope, bodies))
}

/// Select the existing intake-specific code and position before paying for
/// the one final diagnostic. The decoder supplies at most TRACKED_DEPTH steps;
/// the longest refinement appends the two steps `path` and its bad segment.
#[ix_trace_rs::string_edge]
fn classify_wire_fault(
    document: &Document,
    mut code: CheckedPackageRefusalCode,
    original: &[Step<'_>],
    meter: &mut super::WorkMeter,
) -> ValidationFailure {
    use CheckedPackageRefusalCode::{MalformedWire, UnknownMember};
    const EXTRA: usize = 2;
    let mut steps = [Step::Index(0); super::document_de::TRACKED_DEPTH + EXTRA];
    steps[..original.len()].copy_from_slice(original);
    let mut length = original.len();
    let index = original
        .iter()
        .filter_map(|step| match step {
            Step::Index(index) => Some(*index),
            _ => None,
        })
        .next_back()
        .unwrap_or(0);
    if matches!(
        original,
        [
            Step::Key("semantic_graph"),
            Step::Key("operand_rows"),
            Step::Index(_),
            Step::Index(_),
            ..
        ]
    ) {
        code = MalformedWire;
        length = 4;
    }
    // Owner shape classification precedes diagnostic reservation, including
    // the final shortened pointer. Other namespaces never build a node prefix.
    let owner_shape = code == UnknownMember
        && matches!(
            original,
            [
                Step::Key("semantic_graph"),
                Step::Key("nodes"),
                Step::Index(_),
                Step::Key("owner" | "owner_model"),
                ..
            ]
        );
    if owner_shape {
        code = MalformedWire;
        length = 4;
    }
    if code == UnknownMember {
        match original {
            [Step::Key("temporal_clause_names" | "schedulability_claims"), Step::Index(_), ..] => {
                code = MalformedWire;
                length = 2;
            }
            [Step::Key(key), ..] if key.starts_with("checked_term_data") => {
                if matches!(
                    original,
                    [
                        Step::Key("checked_term_data"),
                        Step::Key("probabilistic_claims"),
                        Step::Index(_),
                        Step::Key(_)
                    ]
                ) {
                    length = 3;
                }
                code = MalformedWire;
            }
            [Step::Key("operation_effects"), ..] => code = MalformedWire,
            _ => {}
        }
    }
    if code == MalformedWire && !owner_shape {
        let at = &steps[..length];
        if let Some((first, second)) = invalid_node_id_suffix(document, at) {
            steps[length] = first;
            length += 1;
            if let Some(second) = second {
                steps[length] = second;
                length += 1;
            }
        } else if let Some(field) = invalid_clock_field(document, at) {
            steps[length] = Step::Key(field);
            length += 1;
        } else if missing_probabilistic_occurrence(document, at) {
            steps[length] = Step::Key("occurrence");
            length += 1;
        } else if let Some(parent) = operation_effect_schema_parent_length(at) {
            length = parent;
        }
    }
    super::stage_storage::refuse(
        meter,
        &super::stage_storage::Site::new(
            super::PackageReadStage::WireDecode,
            index,
            "wire",
            super::stage_storage::Path::root(),
        ),
        super::stage_storage::Refusal::new(code, Trail::Base(&steps[..length])),
    )
}

fn operation_effect_schema_parent_length(at: &[Step<'_>]) -> Option<usize> {
    match at {
        [Step::Key("operation_effects"), Step::Index(_), Step::Key(member)]
            if !matches!(
                *member,
                "document" | "declaring" | "operation" | "effect" | "provenance"
            ) =>
        {
            Some(2)
        }
        [Step::Key("operation_effects"), Step::Index(_), Step::Key("effect"), Step::Key("modifies"), Step::Index(_), Step::Key("kind")] => {
            Some(5)
        }
        [Step::Key("operation_effects"), Step::Index(_), Step::Key("effect"), Step::Key(member)]
            if !matches!(
                *member,
                "modifies" | "creates" | "deletes" | "deletes_scoped"
            ) =>
        {
            Some(3)
        }
        _ => None,
    }
}

fn missing_probabilistic_occurrence(document: &Document, at: &[Step<'_>]) -> bool {
    matches!(
        at,
        [
            Step::Key("checked_term_data"),
            Step::Key("probabilistic_claims"),
            Step::Index(_)
        ]
    ) && term_at_steps(document, at)
        .is_some_and(|row| row.get("declaration").is_some() && row.get("occurrence").is_none())
}

#[ix_trace_rs::string_edge]
fn invalid_clock_field(document: &Document, at: &[Step<'_>]) -> Option<&'static str> {
    if !matches!(at.first(), Some(Step::Key("temporal_bindings")))
        || !matches!(at.last(), Some(Step::Key("clock")))
        || at.len() < 2
    {
        return None;
    }
    let clock = term_at_steps(document, at)?;
    let field = match clock.get("kind")?.as_str()? {
        "event_position" => "sequence_authority",
        "timestamped_event" => "timestamp_unit",
        "fixed_sample" => "unit",
        _ => return None,
    };
    clock
        .get(field)
        .filter(|value| value.as_str().is_none())
        .map(|_| field)
}

/// V2 path objects are closed graph-local indices; nested IDs are refused at
/// the segment itself, before semantic-graph admission.
#[ix_trace_rs::string_edge]
fn invalid_node_id_suffix(
    document: &Document,
    at: &[Step<'_>],
) -> Option<(Step<'static>, Option<Step<'static>>)> {
    const MAX_SAFE: u64 = 9_007_199_254_740_991;
    let id = term_at_steps(document, at)?;
    if id.get("package").is_none() || id.get("path").is_none() {
        return None;
    }
    if id
        .get("ordinal")
        .is_some_and(|ordinal| ordinal.as_u64().is_none_or(|value| value > MAX_SAFE))
    {
        return Some((Step::Key("ordinal"), None));
    }
    for (index, segment) in id.get("path")?.items()?.enumerate() {
        if segment.as_str().is_some()
            || (segment.exact_members(&["node"])
                && segment
                    .get("node")
                    .and_then(TermNode::as_u64)
                    .is_some_and(|value| value <= MAX_SAFE))
        {
            continue;
        }
        return Some((Step::Key("path"), Some(Step::Index(index))));
    }
    None
}

fn term_at_steps<'d>(document: &'d Document, steps: &[Step<'_>]) -> Option<TermNode<'d>> {
    let mut current = TermNode::from_ref(document.root());
    for step in steps {
        current = match *step {
            Step::Key(key) => current.get(key)?,
            Step::Index(index) => current.items()?.nth(index)?,
        };
    }
    Some(current)
}

/// `malformed_wire` at `at`.
fn malformed(
    at: super::stage_storage::Path<'static>,
    meter: &mut super::WorkMeter,
) -> ValidationFailure {
    super::stage_storage::refuse(
        meter,
        &super::stage_storage::Site::new(
            super::PackageReadStage::WireDecode,
            at.steps()
                .iter()
                .filter_map(|step| match step {
                    crate::checked_package::common::Step::Index(index) => Some(*index),
                    _ => None,
                })
                .next_back()
                .unwrap_or(0),
            "wire",
            at,
        ),
        super::stage_storage::Refusal::new(CheckedPackageRefusalCode::MalformedWire, at),
    )
}

/// Check the lossless closed decode against the retained source document.
/// The comparator follows document nodes as the typed serializer visits them;
/// it does not reparse a source slice or build a second JSON tree.
#[ix_trace_rs::string_edge]
pub(super) fn check_lossless_source(
    document: &Document,
    wire: &RawCheckedPackageWireV2,
    budget: &mut NativeJoinBudget,
) -> Result<(), ValidationFailure> {
    let mut meter = super::WorkMeter::over(budget);
    let meter = &mut meter;
    let root = TermNode::from_ref(document.root());
    let root_at = super::stage_storage::Path::root();
    compare_root(
        root,
        "capability_report",
        &wire.capability_report,
        &[],
        meter,
    )?;
    compare_root(root, "contract_version", &wire.contract_version, &[], meter)?;
    compare_root(root, "diagnostics", &wire.diagnostics, &["entries"], meter)?;
    compare_root(root, "lock", &wire.lock, &[], meter)?;
    compare_root(root, "name", &wire.name, &[], meter)?;
    compare_root(
        root,
        "semantic_graph",
        &wire.semantic_graph,
        &["nodes"],
        meter,
    )?;

    let nodes_at = root_at.key("semantic_graph").key("nodes");
    let nodes = root
        .get("semantic_graph")
        .and_then(|graph| graph.get("nodes"))
        .and_then(TermNode::items)
        .ok_or_else(|| malformed(nodes_at, meter))?;
    if nodes.len() != wire.semantic_graph.nodes.len() {
        return Err(malformed(nodes_at, meter));
    }
    for (index, (original, node)) in nodes.zip(&wire.semantic_graph.nodes).enumerate() {
        let at = root_at.key("semantic_graph").key("nodes").index(index);
        super::lossless::compare(
            original,
            node,
            at,
            &[
                "body",
                "node_id",
                "semantic_type",
                "dependencies",
                "owner_model",
            ],
            meter,
        )?;
        for key in ["node_id", "semantic_type"] {
            let locus = at.key(key);
            let source = original.get(key).ok_or_else(|| malformed(locus, meter))?;
            let typed = if key == "node_id" {
                &node.node_id
            } else {
                &node.semantic_type
            };
            super::lossless::compare(source, typed, locus, &[], meter)?;
        }
        let dependencies_at = at.key("dependencies");
        let dependencies = original
            .get("dependencies")
            .and_then(TermNode::items)
            .ok_or_else(|| malformed(dependencies_at, meter))?;
        if dependencies.len() != node.dependencies.len() {
            return Err(malformed(dependencies_at, meter));
        }
        for (index, (source, id)) in dependencies.zip(&node.dependencies).enumerate() {
            super::lossless::compare(source, id, at.key("dependencies").index(index), &[], meter)?;
        }
        match (&node.owner_model, original.get("owner_model")) {
            (Some(id), Some(source)) => {
                super::lossless::compare(source, id, at.key("owner_model"), &[], meter)?
            }
            (None, None) => {}
            _ => return Err(malformed(at.key("owner_model"), meter)),
        }
    }

    compare_row_collection(root, "source_map", &wire.source_map, &[], meter)?;
    if let Some(rows) = &wire.refinement_step_rows {
        compare_row_collection(root, "refinement_step_rows", rows, &[], meter)?;
    } else if root.get("refinement_step_rows").is_some() {
        return Err(malformed(root_at.key("refinement_step_rows"), meter));
    }
    if let Some(data) = &wire.checked_term_data {
        compare_root(root, "checked_term_data", data, &[], meter)?;
    } else if root.get("checked_term_data").is_some() {
        return Err(malformed(root_at.key("checked_term_data"), meter));
    }
    compare_optional_rows(
        root,
        "temporal_bindings",
        wire.temporal_bindings.as_deref(),
        meter,
    )?;
    compare_optional_rows(
        root,
        "temporal_clause_names",
        wire.temporal_clause_names.as_deref(),
        meter,
    )?;
    compare_optional_rows(
        root,
        "schedulability_claims",
        wire.schedulability_claims.as_deref(),
        meter,
    )?;
    let entries = root
        .get("diagnostics")
        .and_then(|value| value.get("entries"))
        .and_then(TermNode::items)
        .ok_or_else(|| malformed(root_at.key("diagnostics").key("entries"), meter))?;
    compare_terms(
        root_at.key("diagnostics").key("entries"),
        entries,
        &wire.diagnostics.entries,
        &["details"],
        meter,
    )
}

fn compare_row_collection<T: serde::Serialize>(
    root: TermNode<'_>,
    key: &'static str,
    typed: &[T],
    skipped: &'static [&'static str],
    meter: &mut super::WorkMeter,
) -> Result<(), ValidationFailure> {
    let at = super::stage_storage::Path::root().key(key);
    let rows = root
        .get(key)
        .and_then(TermNode::items)
        .ok_or_else(|| malformed(at, meter))?;
    compare_terms(at, rows, typed, skipped, meter)
}

fn compare_root<T: serde::Serialize>(
    root: TermNode<'_>,
    key: &'static str,
    typed: &T,
    skipped: &'static [&'static str],
    meter: &mut super::WorkMeter,
) -> Result<(), ValidationFailure> {
    let at = super::stage_storage::Path::root().key(key);
    let raw = root.get(key).ok_or_else(|| malformed(at, meter))?;
    super::lossless::compare(raw, typed, at, skipped, meter)
}

fn compare_optional_rows<T: serde::Serialize>(
    root: TermNode<'_>,
    key: &'static str,
    typed: Option<&[T]>,
    meter: &mut super::WorkMeter,
) -> Result<(), ValidationFailure> {
    let Some(typed) = typed else {
        return if root.get(key).is_some() {
            Err(malformed(
                super::stage_storage::Path::root().key(key),
                meter,
            ))
        } else {
            Ok(())
        };
    };
    if typed.is_empty() {
        return Err(malformed(
            super::stage_storage::Path::root().key(key),
            meter,
        ));
    }
    compare_row_collection(root, key, typed, &[], meter)
}

fn compare_terms<T: serde::Serialize>(
    at: super::stage_storage::Path<'static>,
    terms: super::TermItems<'_>,
    typed: &[T],
    skipped: &'static [&'static str],
    meter: &mut super::WorkMeter,
) -> Result<(), ValidationFailure> {
    if terms.len() != typed.len() {
        return Err(malformed(at, meter));
    }
    for (index, (source, value)) in terms.zip(typed).enumerate() {
        super::lossless::compare(source, value, at.index(index), skipped, meter)?;
    }
    Ok(())
}

#[cfg(test)]
mod ordinal_locus_tests {
    use super::*;

    #[test]
    fn source_lookup_skips_deep_sibling_without_deserializing_it() {
        let deep = format!("{}0{}", "[".repeat(120), "]".repeat(120));
        let bytes = format!(
            "{{\"semantic_graph\":{{\"nodes\":[{{\"body\":{deep},\"node_id\":{{\"package\":\"p\",\"path\":[\"x\"],\"ordinal\":0}}}}]}}}}"
        );
        let document = quire_canonical::read(bytes.as_bytes(), u64::MAX).expect("document");
        let pointer = [
            Step::Key("semantic_graph"),
            Step::Key("nodes"),
            Step::Index(0),
            Step::Key("node_id"),
            Step::Key("ordinal"),
        ];
        assert_eq!(
            term_at_steps(&document, &pointer).and_then(TermNode::as_u64),
            Some(0)
        );
    }

    #[test]
    fn source_lookup_handles_escaped_member_and_string_contents() {
        let bytes = br#"{"a/b":{"~name":["quoted \" { [ }",{"node":3}]}}"#;
        let document = quire_canonical::read(bytes, u64::MAX).expect("document");
        let pointer = [
            Step::Key("a/b"),
            Step::Key("~name"),
            Step::Index(1),
            Step::Key("node"),
        ];
        assert_eq!(
            term_at_steps(&document, &pointer).and_then(TermNode::as_u64),
            Some(3)
        );
    }

    #[test]
    fn malformed_node_id_members_keep_their_exact_pointer() {
        let node = [
            Step::Key("semantic_graph"),
            Step::Key("nodes"),
            Step::Index(1),
            Step::Key("node_id"),
        ];
        for ordinal in ["9007199254740992", "1.5", "-1", "\"one\""] {
            let bytes = format!(
                "{{\"semantic_graph\":{{\"nodes\":[null,{{\"node_id\":{{\"package\":\"p\",\"path\":[\"x\"],\"ordinal\":{ordinal}}}}}]}}}}"
            );
            let document = quire_canonical::read(bytes.as_bytes(), u64::MAX).expect("document");
            assert_eq!(
                invalid_node_id_suffix(&document, &node),
                Some((Step::Key("ordinal"), None))
            );
        }

        let nested = br#"{"semantic_graph":{"nodes":[null,{"node_id":{"package":"p","path":[{"package":"p","path":["x"],"ordinal":1.5}],"ordinal":0}}]}}"#;
        let document = quire_canonical::read(nested, u64::MAX).expect("document");
        assert_eq!(
            invalid_node_id_suffix(&document, &node),
            Some((Step::Key("path"), Some(Step::Index(0))))
        );

        let dependency = br#"{"semantic_graph":{"nodes":[{"dependencies":[{"package":"p","path":["x"],"ordinal":"one"}]}]}}"#;
        let reference = [
            Step::Key("semantic_graph"),
            Step::Key("nodes"),
            Step::Index(0),
            Step::Key("dependencies"),
            Step::Index(0),
        ];
        let document = quire_canonical::read(dependency, u64::MAX).expect("document");
        assert_eq!(
            invalid_node_id_suffix(&document, &reference),
            Some((Step::Key("ordinal"), None))
        );
    }
}

#[cfg(test)]
mod lossless_budget_tests {
    use super::*;
    use crate::checked_package::v2::monitor_plan::model_document_test_fixture as fixture;

    /// The actual intake comparison borrows healthy paths, and pays its sole
    /// malformed-field diagnostic before requesting any owned pointer bytes.
    #[ix_trace_rs::trace("TC-472", "FR-038-AC-511")]
    #[test]
    fn lossless_intake_borrows_healthy_paths_and_prepays_first_fault() {
        let value = fixture::v2_all_families();
        let bytes = fixture::canonical(&value);
        let document = Arc::new(quire_canonical::read(&bytes, u64::MAX).unwrap());
        let mut decode_budget =
            NativeJoinBudget::new(u64::MAX, 0).with_read_storage_limit(u64::MAX);
        let (mut wire, _bodies) = decode_borrowed_wire(&document, &mut decode_budget).unwrap();
        let run = |wire: &RawCheckedPackageWireV2, limit| {
            let mut budget = NativeJoinBudget::new(u64::MAX, 0).with_read_storage_limit(limit);
            let mut result = None;
            let measured = allocation_counter::measure(|| {
                result = Some(check_lossless_source(&document, wire, &mut budget));
            });
            (
                result.unwrap(),
                measured.bytes_total,
                budget.read_storage_consumed(),
            )
        };
        let (healthy, allocated, charged) = run(&wire, 0);
        healthy.expect("healthy original document agrees with its typed wire");
        assert_eq!((allocated, charged), (0, 0));
        wire.name = format!("{}-different", wire.name).into_boxed_str();
        let (failure, allocated, required) = run(&wire, u64::MAX);
        let ValidationFailure::Refused(refusal) = failure.unwrap_err() else {
            panic!("the first changed name is malformed wire");
        };
        assert_eq!(refusal.code, CheckedPackageRefusalCode::MalformedWire);
        assert_eq!(refusal.primary_path().unwrap().as_str(), "/name");
        assert_eq!(allocated, required);
        assert_eq!(
            required as usize,
            std::mem::size_of::<crate::checked_package::shared::CheckedPackageRefusal>()
                + std::mem::size_of::<crate::checked_package::shared::CheckedPackageRefusalLocation>(
                )
                + "/name".len()
        );
        let (exact, allocated, charged) = run(&wire, required);
        assert!(matches!(exact, Err(ValidationFailure::Refused(_))));
        assert_eq!((allocated, charged), (required, required));
        for limit in [0, required - 1] {
            let (short, allocated, charged) = run(&wire, limit);
            assert!(matches!(short, Err(ValidationFailure::Budget(_))));
            assert_eq!((allocated, charged), (0, 0));
        }
    }

    #[ix_trace_rs::trace("TC-472", "FR-038-AC-511")]
    #[test]
    fn lossless_public_reader_preserves_dropped_null_first_fault() {
        use crate::checked_package::shared::CheckedPackageReadLimits;
        use crate::checked_package::v2::{CheckedPackageV2, CheckedPackageV2BudgetedReadResult};
        let mut value = fixture::v2_all_families();
        let evidence = fixture::evidence_for(&value);
        let nodes = value["semantic_graph"]["nodes"].as_array_mut().unwrap();
        let (index, node) = nodes
            .iter_mut()
            .enumerate()
            .find(|(_, node)| node.get("owner").is_none() && node.get("recursion_group").is_none())
            .expect("the complete fixture includes an unowned node");
        node["owner"] = serde_json::Value::Null;
        node["recursion_group"] = serde_json::Value::Null;
        let bytes = fixture::canonical(&value);
        let limits = CheckedPackageReadLimits {
            bytes: 1 << 30,
            nodes: 1 << 24,
            edges: 1 << 26,
            occurrences: 1 << 26,
            diagnostics: 1 << 24,
            work: u64::MAX,
        };
        let mut budget = NativeJoinBudget::new(u64::MAX, 0).with_read_storage_limit(u64::MAX);
        let result = CheckedPackageV2::read_with_budget(&bytes, limits, &evidence, &mut budget);
        let CheckedPackageV2BudgetedReadResult::Refused(refusal) = result else {
            panic!("the original null owner is not silently dropped: {result:?}");
        };
        assert_eq!(refusal.code, CheckedPackageRefusalCode::MalformedWire);
        assert_eq!(
            refusal.primary_path().unwrap().as_str(),
            format!("/semantic_graph/nodes/{index}/owner")
        );
    }
}

#[cfg(test)]
mod schema_policy_tests {
    use super::*;
    use CheckedPackageRefusalCode::{MalformedWire, UnknownMember};

    fn verify(
        document: &Document,
        code: CheckedPackageRefusalCode,
        steps: &[Step<'_>],
        expected_code: CheckedPackageRefusalCode,
        expected_path: &str,
    ) {
        let run = |limit| {
            let mut budget = NativeJoinBudget::new(u64::MAX, 0).with_read_storage_limit(limit);
            let mut result = None;
            let measured = allocation_counter::measure(|| {
                result = Some(classify_wire_fault(
                    document,
                    code,
                    steps,
                    &mut super::super::WorkMeter::over(&mut budget),
                ));
            });
            (result.unwrap(), measured.bytes_total, budget)
        };
        let (failure, allocated, budget) = run(u64::MAX);
        let ValidationFailure::Refused(refusal) = failure else {
            panic!("schema refusal")
        };
        assert_eq!(refusal.code, expected_code);
        assert_eq!(refusal.primary_path().unwrap().as_str(), expected_path);
        let required = budget.read_storage_consumed();
        assert_eq!(allocated, required);
        assert_eq!(
            required as usize,
            std::mem::size_of::<crate::checked_package::shared::CheckedPackageRefusal>()
                + std::mem::size_of::<crate::checked_package::shared::CheckedPackageRefusalLocation>(
                )
                + expected_path.len()
        );
        let (exact, allocated, budget) = run(required);
        assert!(matches!(exact, ValidationFailure::Refused(_)));
        assert_eq!(allocated, required);
        assert_eq!(budget.read_storage_consumed(), required);
        for limit in [0, required - 1] {
            let (short, allocated, budget) = run(limit);
            assert!(matches!(short, ValidationFailure::Budget(_)));
            assert_eq!((allocated, budget.read_storage_consumed()), (0, 0));
            assert_eq!(
                budget.first_unpaid_read_storage(),
                Some(u128::from(required))
            );
        }
    }

    #[ix_trace_rs::trace("TC-472", "FR-038-AC-511")]
    #[test]
    fn schema_policy_prepays_only_the_final_classified_path() {
        let empty = quire_canonical::read(b"{}", u64::MAX).unwrap();
        for (code, steps, expected_code, expected_path) in [
            (
                UnknownMember,
                vec![
                    Step::Key("temporal_clause_names"),
                    Step::Index(3),
                    Step::Key("unknown"),
                ],
                MalformedWire,
                "/temporal_clause_names/3",
            ),
            (
                UnknownMember,
                vec![
                    Step::Key("schedulability_claims"),
                    Step::Index(2),
                    Step::Key("unknown"),
                ],
                MalformedWire,
                "/schedulability_claims/2",
            ),
            (
                UnknownMember,
                vec![
                    Step::Key("checked_term_data"),
                    Step::Key("probabilistic_claims"),
                    Step::Index(0),
                    Step::Key("/~é"),
                ],
                MalformedWire,
                "/checked_term_data/probabilistic_claims/0",
            ),
            (
                UnknownMember,
                vec![
                    Step::Key("operation_effects"),
                    Step::Index(0),
                    Step::Key("unknown"),
                ],
                MalformedWire,
                "/operation_effects/0",
            ),
            (
                MalformedWire,
                vec![
                    Step::Key("operation_effects"),
                    Step::Index(0),
                    Step::Key("effect"),
                    Step::Key("modifies"),
                    Step::Index(4),
                    Step::Key("kind"),
                ],
                MalformedWire,
                "/operation_effects/0/effect/modifies/4",
            ),
            (
                MalformedWire,
                vec![
                    Step::Key("operation_effects"),
                    Step::Index(0),
                    Step::Key("effect"),
                    Step::Key("unknown"),
                ],
                MalformedWire,
                "/operation_effects/0/effect",
            ),
            (
                UnknownMember,
                vec![Step::Key("/~é")],
                UnknownMember,
                "/~1~0é",
            ),
        ] {
            verify(&empty, code, &steps, expected_code, expected_path);
        }
        let id =
            quire_canonical::read(br#"{"package":"p","path":[false],"ordinal":1.5}"#, u64::MAX)
                .unwrap();
        verify(&id, MalformedWire, &[], MalformedWire, "/ordinal");
        let clock = quire_canonical::read(br#"{"temporal_bindings":[{"clock":{"kind":"event_position","sequence_authority":false}}]}"#, u64::MAX).unwrap();
        verify(
            &clock,
            MalformedWire,
            &[
                Step::Key("temporal_bindings"),
                Step::Index(0),
                Step::Key("clock"),
            ],
            MalformedWire,
            "/temporal_bindings/0/clock/sequence_authority",
        );
        let row = quire_canonical::read(
            br#"{"checked_term_data":{"probabilistic_claims":[{"declaration":null}]}}"#,
            u64::MAX,
        )
        .unwrap();
        verify(
            &row,
            MalformedWire,
            &[
                Step::Key("checked_term_data"),
                Step::Key("probabilistic_claims"),
                Step::Index(0),
            ],
            MalformedWire,
            "/checked_term_data/probabilistic_claims/0/occurrence",
        );
    }

    #[ix_trace_rs::trace("TC-472", "FR-038-AC-510", "FR-038-AC-511")]
    #[test]
    fn owner_schema_classification_pays_only_the_final_owner_path() {
        let document = quire_canonical::read(br#"{"semantic_graph":{"nodes":[{"owner":{"package":"p","path":[false],"ordinal":0}}]}}"#, u64::MAX).unwrap();
        for member in ["owner", "owner_model"] {
            let steps = [
                Step::Key("semantic_graph"),
                Step::Key("nodes"),
                Step::Index(0),
                Step::Key(member),
                Step::Key("extra"),
            ];
            let expected = format!("/semantic_graph/nodes/0/{member}");
            verify(&document, UnknownMember, &steps, MalformedWire, &expected);
            let mut budget = NativeJoinBudget::new(u64::MAX, 0);
            assert!(matches!(
                classify_wire_fault(
                    &document,
                    UnknownMember,
                    &steps,
                    &mut super::super::WorkMeter::over(&mut budget)
                ),
                ValidationFailure::Budget(_)
            ));
            assert_eq!(
                budget.take_pending(),
                Some(super::super::CheckedPackagePendingOutcome::Refused(
                    MalformedWire
                ))
            );
        }
        for member in ["digest_domain", "revision"] {
            verify(
                &document,
                UnknownMember,
                &[
                    Step::Key("lock"),
                    Step::Key("sources"),
                    Step::Index(0),
                    Step::Key(member),
                ],
                UnknownMember,
                &format!("/lock/sources/0/{member}"),
            );
        }
    }

    #[ix_trace_rs::trace("TC-472", "FR-038-AC-511")]
    #[test]
    fn schema_policy_maximum_snapshot_plus_two_steps_and_deep_siblings_are_exact() {
        let depth = super::super::document_de::TRACKED_DEPTH;
        let mut bytes = r#"{"package":"p","path":[false],"ordinal":0}"#.to_owned();
        for _ in 0..depth {
            bytes = format!(r#"{{"a/~é":{bytes}}}"#);
        }
        let document = quire_canonical::read(bytes.as_bytes(), u64::MAX).unwrap();
        let steps = vec![Step::Key("a/~é"); depth];
        let expected = format!("{}/path/0", "/a~1~0é".repeat(depth));
        verify(&document, MalformedWire, &steps, MalformedWire, &expected);
        let deep = format!("{}false{}", "[".repeat(120), "]".repeat(120));
        let document = quire_canonical::read(deep.as_bytes(), u64::MAX).unwrap();
        let steps = vec![Step::Index(0); depth];
        verify(
            &document,
            MalformedWire,
            &steps,
            MalformedWire,
            &"/0".repeat(depth),
        );
    }

    #[ix_trace_rs::trace("TC-472", "FR-038-AC-511")]
    #[test]
    fn schema_intake_actual_decode_pays_final_temporal_row_diagnostic_once() {
        use crate::checked_package::v2::monitor_plan::model_document_test_fixture as fixture;
        let mut wire = fixture::v2_all_families();
        wire["temporal_clause_names"] = serde_json::json!([{"unknown":false}]);
        let bytes = fixture::canonical(&wire);
        let document = Arc::new(quire_canonical::read(&bytes, u64::MAX).unwrap());
        let mut budget = NativeJoinBudget::new(u64::MAX, 0).with_read_storage_limit(u64::MAX);
        let mut result = None;
        let measured = allocation_counter::measure(|| {
            result = Some(decode_borrowed_wire(&document, &mut budget));
        });
        let ValidationFailure::Refused(refusal) = result.unwrap().unwrap_err() else {
            panic!("temporal row schema refusal")
        };
        assert_eq!(refusal.code, MalformedWire);
        assert_eq!(
            refusal.primary_path().unwrap().as_str(),
            "/temporal_clause_names/0"
        );
        assert_eq!(measured.bytes_total, budget.read_storage_consumed());
    }

    #[ix_trace_rs::trace("TC-048", "FR-038-AC-53", "TC-472", "FR-038-AC-511")]
    #[test]
    fn lock_source_unknown_member_public_reader_pays_every_request() {
        use crate::checked_package::shared::CheckedPackageReadLimits;
        use crate::checked_package::v2::monitor_plan::model_document_test_fixture as fixture;
        use crate::checked_package::v2::{CheckedPackageV2, CheckedPackageV2BudgetedReadResult};
        let base = fixture::v2_nominal();
        for (member, value) in [
            ("digest_domain", serde_json::json!("quire.source.bytes/v1")),
            (
                "revision",
                serde_json::json!({"namespace":"quire-draft","value":"1"}),
            ),
        ] {
            let mut wire = base.clone();
            wire["lock"]["sources"][0][member] = value;
            let evidence = fixture::evidence_for(&wire);
            let bytes = fixture::canonical(&wire);
            let run = |budget: &mut NativeJoinBudget| {
                let mut result = None;
                let allocated = allocation_counter::measure(|| {
                    result = Some(CheckedPackageV2::read_with_budget(
                        &bytes,
                        CheckedPackageReadLimits::bounded(),
                        &evidence,
                        budget,
                    ));
                })
                .bytes_total;
                (result.unwrap(), allocated)
            };
            let mut ample = NativeJoinBudget::new(u64::MAX, 0).with_read_storage_limit(u64::MAX);
            let (result, allocated) = run(&mut ample);
            let CheckedPackageV2BudgetedReadResult::Refused(refusal) = &result else {
                panic!("unknown source member")
            };
            assert_eq!(refusal.code, UnknownMember);
            assert_eq!(refusal.cause, None);
            assert_eq!(
                refusal.primary_path().unwrap().as_str(),
                format!("/lock/sources/0/{member}")
            );
            let required = ample.read_storage_consumed();
            assert_eq!(allocated, required);
            let mut exact = NativeJoinBudget::new(u64::MAX, 0).with_read_storage_limit(required);
            assert_eq!(run(&mut exact), (result.clone(), required));
            for limit in [0, required - 1] {
                let mut short = NativeJoinBudget::new(u64::MAX, 0).with_read_storage_limit(limit);
                let (result, allocated) = run(&mut short);
                assert!(matches!(
                    result,
                    CheckedPackageV2BudgetedReadResult::Exhausted(_)
                ));
                assert_eq!(allocated, short.read_storage_consumed());
            }
            let mut repeated =
                NativeJoinBudget::new(u64::MAX, 0).with_read_storage_limit(2 * required);
            assert_eq!(run(&mut repeated), (result.clone(), required));
            assert_eq!(run(&mut repeated), (result, required));
            assert_eq!(repeated.read_storage_consumed(), 2 * required);
            assert_eq!(repeated.consumed().work, 2 * ample.consumed().work);
        }
    }

    /// Paired identical public reads cancel unrelated earlier reader storage:
    /// only the final classified diagnostic fits in the first allowance.
    #[ix_trace_rs::trace("TC-472", "FR-038-AC-511")]
    #[test]
    fn schema_public_reader_final_diagnostic_is_paid_once_or_exhausted() {
        use crate::checked_package::shared::CheckedPackageReadLimits;
        use crate::checked_package::v2::monitor_plan::model_document_test_fixture as fixture;
        use crate::checked_package::v2::{
            CheckedPackagePendingOutcome, CheckedPackageV2, CheckedPackageV2BudgetedReadResult,
        };
        let mut wire = fixture::v2_all_families();
        let evidence = fixture::evidence_for(&wire);
        wire["temporal_clause_names"] = serde_json::json!([{"unknown":false}]);
        let bytes = fixture::canonical(&wire);
        let limits = CheckedPackageReadLimits {
            bytes: 1 << 30,
            nodes: 1 << 24,
            edges: 1 << 26,
            occurrences: 1 << 26,
            diagnostics: 1 << 24,
            work: u64::MAX,
        };
        let run = |storage| {
            let mut budget = NativeJoinBudget::new(u64::MAX, 0).with_read_storage_limit(storage);
            let mut result = None;
            let measured = allocation_counter::measure(|| {
                result = Some(CheckedPackageV2::read_with_budget(
                    &bytes,
                    limits,
                    &evidence,
                    &mut budget,
                ));
            });
            (result.unwrap(), measured.bytes_total, budget)
        };
        let (unlimited, _, budget) = run(u64::MAX);
        let CheckedPackageV2BudgetedReadResult::Refused(refusal) = unlimited else {
            panic!("classified temporal refusal")
        };
        assert_eq!(refusal.code, MalformedWire);
        assert_eq!(
            refusal.primary_path().unwrap().as_str(),
            "/temporal_clause_names/0"
        );
        let required = budget.read_storage_consumed();
        let diagnostic = (std::mem::size_of::<crate::checked_package::shared::CheckedPackageRefusal>(
        ) + std::mem::size_of::<
            crate::checked_package::shared::CheckedPackageRefusalLocation,
        >() + "/temporal_clause_names/0".len()) as u64;
        let (exact, exact_allocated, exact_budget) = run(required);
        assert!(matches!(
            exact,
            CheckedPackageV2BudgetedReadResult::Refused(_)
        ));
        assert_eq!(exact_budget.read_storage_consumed(), required);
        let (short, short_allocated, short_budget) = run(required - 1);
        let CheckedPackageV2BudgetedReadResult::Exhausted(exhausted) = short else {
            panic!("the final schema diagnostic cannot fit")
        };
        assert_eq!(
            exhausted.pending,
            Some(CheckedPackagePendingOutcome::Refused(MalformedWire))
        );
        assert_eq!(
            short_budget.first_unpaid_read_storage(),
            Some(u128::from(diagnostic))
        );
        assert_eq!(exact_allocated - short_allocated, diagnostic);
        assert_eq!(
            exact_budget.read_storage_consumed() - short_budget.read_storage_consumed(),
            diagnostic
        );
    }
}
