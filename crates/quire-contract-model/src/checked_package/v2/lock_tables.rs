// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (C) 2026 Agent-IX

//! The lock stage's checks and tables, every allocation charged to the
//! caller's budget before it is made (FR-038 "Budget-charged strict read",
//! stage `Lock`).
//!
//! A repeated value is found on a sorted slice of `(text, position)` rows
//! that borrows the lock, requested at its exact capacity first; the read
//! builds no `BTreeMap` or `BTreeSet` here. The unit references are a sorted
//! slice of exact capacity, each owned name and node key requested before it
//! is built.
//!
//! Implements: FR-038 "Budget-charged strict read", FR-322.

use super::model_members::{
    arc_bytes, node_id_bytes, Budget, DomainModel, SortedMap, UnitRefs, DURATION_RESOLVED_PREFIX,
};
use super::{
    identity, member_pointer, refuse, CheckedDependencySelection, CheckedNodeId, CheckedNodeKind,
    CheckedNodeOwner, CheckedNodeTag, CheckedPackageRefusalCause, CheckedPackageRefusalCode,
    CheckedRefinementInternal, RawSemanticNodeV2, ScalarTypeForm, ValidationFailure, WorkMeter,
};
use crate::checked_package::common::{is_nonempty, node_pointer};
use ix_trace_rs::string_edge;
use std::sync::Arc;

const UNIT_PREFIX: &str = "ix://quire/unit/";

/// The earliest repeat among `items`: the position of the first occurrence
/// and of the earliest later position whose text equals an earlier one.
/// `table` is requested at the number of items first and sorted in place.
fn first_repeat<'a>(
    items: impl Iterator<Item = &'a str>,
    table: &mut Vec<(&'a str, usize)>,
) -> Option<(usize, usize)> {
    table.clear();
    table.extend(items.enumerate().map(|(position, text)| (text, position)));
    table.sort_unstable();
    let mut earliest: Option<(usize, usize)> = None;
    for (at, pair) in table.windows(2).enumerate() {
        let starts_run = at
            .checked_sub(1)
            .and_then(|before| table.get(before))
            .is_none_or(|before| before.0 != pair[0].0);
        if pair[0].0 == pair[1].0
            && starts_run
            && earliest.is_none_or(|(_, second)| pair[1].1 < second)
        {
            earliest = Some((pair[0].1, pair[1].1));
        }
    }
    earliest
}

/// Checks `lock.dependency_selections` (FR-322): nonempty identity,
/// uniqueness, then strictly ascending
/// UTF-8 byte order of `identity`. Each condition sweeps the whole array
/// before the next, so the refusal is decided by defect class and not by
/// array position. A repeated identity refuses `invalid_package` /
/// `conflicting-definition` at the repeating entry, the later of the two
/// entries FR-322 names as loci; an entry not strictly after its
/// predecessor refuses `invalid_package` / `invalid-value` at that entry.
/// Only the lock carries selections.
pub(super) fn validate_dependency_selections(
    selections: &[CheckedDependencySelection],
    meter: &mut WorkMeter,
) -> Result<(), ValidationFailure> {
    const FORM: &str = "dependency_selections";
    let at = |index: usize| member_pointer(&["lock", "dependency_selections"]).index(index);
    if let Some(index) = selections
        .iter()
        .position(|entry| !is_nonempty(&entry.identity))
    {
        return Err(refuse(
            CheckedPackageRefusalCode::MalformedWire,
            at(index).key("identity"),
        ));
    }
    let mut table = Budget::lock(meter, "dependency_selections").vec(selections.len(), FORM)?;
    if let Some((first, index)) = first_repeat(
        selections.iter().map(|entry| entry.identity.as_ref()),
        &mut table,
    ) {
        return Err(ValidationFailure::refused_because_at_two_paths(
            CheckedPackageRefusalCode::InvalidPackage,
            at(first),
            at(index),
            CheckedPackageRefusalCause::ConflictingDefinition,
        ));
    }
    if let Some(index) = selections
        .windows(2)
        .position(|pair| pair[0].identity >= pair[1].identity)
    {
        return Err(ValidationFailure::refused_because(
            CheckedPackageRefusalCode::InvalidPackage,
            at(index + 1),
            CheckedPackageRefusalCause::InvalidValue,
        ));
    }
    Ok(())
}

/// Checks `lock.required_features`: every feature nonempty and none
/// repeated. The refusal is at the earliest feature that is empty or repeats
/// an earlier one.
pub(super) fn validate_required_features(
    features: &[Box<str>],
    meter: &mut WorkMeter,
) -> Result<(), ValidationFailure> {
    let mut table =
        Budget::lock(meter, "required_features").vec(features.len(), "required_features")?;
    let empty = features.iter().position(|feature| !is_nonempty(feature));
    let repeated = first_repeat(features.iter().map(AsRef::as_ref), &mut table);
    let earliest = match (empty, repeated) {
        (Some(empty), Some((_, repeat))) => Some(empty.min(repeat)),
        (empty, repeated) => empty.or(repeated.map(|(_, repeat)| repeat)),
    };
    match earliest {
        Some(index) => Err(refuse(
            CheckedPackageRefusalCode::MalformedWire,
            member_pointer(&["lock", "required_features"]).index(index),
        )),
        None => Ok(()),
    }
}

/// Enforce FR-322's canonical refinement internal lock rows and spellings.
pub(super) fn validate_refinement_internals(
    rows: Option<&[CheckedRefinementInternal]>,
    meter: &mut WorkMeter,
) -> Result<(), ValidationFailure> {
    const FORM: &str = "refinement_internals";
    let Some(rows) = rows else { return Ok(()) };
    let at = |index: usize| member_pointer(&["lock", "refinement_internals"]).index(index);
    if rows.is_empty() {
        return Err(refuse(
            CheckedPackageRefusalCode::MalformedWire,
            member_pointer(&["lock", "refinement_internals"]),
        ));
    }
    let widest = rows.iter().map(|row| row.internal.len()).max().unwrap_or(0);
    let mut budget = Budget::lock(meter, "refinement_internals");
    let mut row_table = budget.vec(rows.len(), FORM)?;
    let mut class_table = budget.vec(widest, FORM)?;
    let repeated_row = first_repeat(
        rows.iter().map(|row| row.refinement.as_ref()),
        &mut row_table,
    );
    for (index, row) in rows.iter().enumerate() {
        if !is_nonempty(&row.refinement) || row.internal.is_empty() {
            return Err(refuse(CheckedPackageRefusalCode::MalformedWire, at(index)));
        }
        if let Some((first, _)) = repeated_row.filter(|&(_, second)| second == index) {
            return Err(ValidationFailure::refused_because_at_two_paths(
                CheckedPackageRefusalCode::InvalidPackage,
                at(first).key("refinement"),
                at(index).key("refinement"),
                CheckedPackageRefusalCause::ConflictingDefinition,
            ));
        }
        let repeated_class = first_repeat(row.internal.iter().map(AsRef::as_ref), &mut class_table);
        for (class_index, class) in row.internal.iter().enumerate() {
            let class_at = || at(index).key("internal").index(class_index);
            if !is_nonempty(class) {
                return Err(refuse(CheckedPackageRefusalCode::MalformedWire, class_at()));
            }
            if let Some((first, _)) = repeated_class.filter(|&(_, second)| second == class_index) {
                return Err(ValidationFailure::refused_because_at_two_paths(
                    CheckedPackageRefusalCode::InvalidPackage,
                    at(index).key("internal").index(first),
                    class_at(),
                    CheckedPackageRefusalCause::ConflictingDefinition,
                ));
            }
            if class_index > 0 && row.internal[class_index - 1] >= *class {
                return Err(ValidationFailure::refused_because(
                    CheckedPackageRefusalCode::InvalidPackage,
                    class_at(),
                    CheckedPackageRefusalCause::InvalidValue,
                ));
            }
        }
        if index > 0 && rows[index - 1].refinement >= row.refinement {
            return Err(ValidationFailure::refused_because(
                CheckedPackageRefusalCode::InvalidPackage,
                at(index).key("refinement"),
                CheckedPackageRefusalCause::InvalidValue,
            ));
        }
    }
    Ok(())
}

/// The selected domain packages: one per `model_root` node owner, in
/// ascending identity order. Each is the node position of its root. Two roots
/// naming one identity refuse as an invalid graph at the later root's owner.
/// The table is requested at the number of roots first, at the first node.
pub(super) fn selected_model_roots<'a>(
    nodes: &'a [RawSemanticNodeV2],
    meter: &mut WorkMeter,
) -> Result<Vec<(&'a str, usize)>, ValidationFailure> {
    fn owner(node: &RawSemanticNodeV2) -> Option<&str> {
        match &node.owner {
            Some(CheckedNodeOwner::ModelRoot { identity }) => Some(identity.as_ref()),
            _ => None,
        }
    }
    let count = nodes.iter().filter(|node| owner(node).is_some()).count();
    let mut roots: Vec<(&str, usize)> =
        Budget::lock_node(meter, 0, true).vec(count, "model_roots")?;
    roots.extend(
        nodes
            .iter()
            .enumerate()
            .filter_map(|(position, node)| Some((owner(node)?, position))),
    );
    roots.sort_unstable();
    let mut later: Option<usize> = None;
    for (at, pair) in roots.windows(2).enumerate() {
        let starts_run = at
            .checked_sub(1)
            .and_then(|before| roots.get(before))
            .is_none_or(|before| before.0 != pair[0].0);
        if pair[0].0 == pair[1].0 && starts_run && later.is_none_or(|second| pair[1].1 < second) {
            later = Some(pair[1].1);
        }
    }
    if let Some(position) = later {
        return Err(refuse(
            CheckedPackageRefusalCode::InvalidSemanticGraph,
            node_pointer(position).key("owner"),
        ));
    }
    Ok(roots)
}

/// The spelling of a unit declaration's reference name, without building it.
enum UnitName<'a> {
    Source(&'a [Box<str>]),
    Model { identity: &'a str, unit: &'a str },
}

impl UnitName<'_> {
    fn len(&self) -> usize {
        UNIT_PREFIX.len()
            + match self {
                Self::Source(parts) => {
                    parts.iter().map(|part| part.len()).sum::<usize>()
                        + parts.len().saturating_sub(1)
                }
                Self::Model { identity, unit } => identity.len() + 1 + unit.len(),
            }
    }

    /// Writes the name into `out`, which holds `self.len()` bytes of room.
    fn write(&self, out: &mut String) {
        out.push_str(UNIT_PREFIX);
        match self {
            Self::Source(parts) => {
                for (at, part) in parts.iter().enumerate() {
                    if at > 0 {
                        out.push('/');
                    }
                    out.push_str(part);
                }
            }
            Self::Model { identity, unit } => {
                out.push_str(identity);
                out.push('/');
                out.push_str(unit);
            }
        }
    }
}

/// The reference name a unit declaration node is a candidate under.
fn unit_name<B>(node: &super::CheckedSemanticNodeV2<B>) -> Option<UnitName<'_>> {
    if CheckedNodeTag::from_wire(&node.node_tag)
        .and_then(|tag| CheckedNodeKind::decode(tag, &node.semantic_form))
        != Some(CheckedNodeKind::ScalarType(ScalarTypeForm::Unit))
    {
        return None;
    }
    match (&node.owner, &node.declaration) {
        (Some(CheckedNodeOwner::Source { .. }), Some(declaration)) => {
            Some(UnitName::Source(&declaration.qualified_name))
        }
        (
            Some(CheckedNodeOwner::Model {
                identity,
                node: owner_node,
            }),
            None,
        ) => {
            let unit = owner_node
                .strip_prefix("ix://")?
                .strip_prefix(identity.as_ref())?
                .strip_prefix("/unit/")?;
            Some(UnitName::Model {
                identity: identity.as_ref(),
                unit,
            })
        }
        _ => None,
    }
}

/// Requests `bytes` at `PackageRead(Lock, position)`, form `unit_refs`.
fn request(meter: &mut WorkMeter, position: usize, bytes: usize) -> Result<(), ValidationFailure> {
    Budget::lock_node(meter, position, false).request(bytes, "unit_refs")
}

/// Candidate unit references for model selection admission. The input byte
/// limit bounds this one graph walk and its index before graph work is charged.
/// The graph stage subsequently validates each node, name and key before the
/// package admits.
///
/// Every name and node key is requested at `PackageRead(Lock, node)` before it
/// is built, and the sorted table at its exact capacity first.
#[string_edge]
pub(super) fn checked_unit_refs<B>(
    nodes: &[super::CheckedSemanticNodeV2<B>],
    terms: &[Option<super::CheckedTerm>],
    meter: &mut WorkMeter,
) -> Result<UnitRefs, ValidationFailure> {
    let count = nodes
        .iter()
        .filter(|node| unit_name(node).is_some())
        .count();
    let mut names: Vec<(Box<str>, usize)> =
        Budget::lock_node(meter, 0, false).vec(count, "unit_refs")?;
    for (position, node) in nodes.iter().enumerate() {
        let Some(name) = unit_name(node) else {
            continue;
        };
        let len = name.len();
        request(meter, position, len)?;
        let mut text = String::with_capacity(len);
        name.write(&mut text);
        names.push((text.into_boxed_str(), position));
    }
    // The first node holding a name keeps it.
    names.sort_unstable();
    let mut kept = 0_usize;
    for at in 0..names.len() {
        let repeated = kept
            .checked_sub(1)
            .and_then(|last| names.get(last))
            .zip(names.get(at))
            .is_some_and(|(last, this)| last.0 == this.0);
        if !repeated {
            names.swap(kept, at);
            kept += 1;
        }
    }
    names.truncate(kept);
    // A Duration member can use its owner's s only when that unit has the
    // exact one-second Time type. Keep the candidate's presence in the table
    // even when invalid: an invalid selected s must not fall back to source s.
    let candidates = names
        .iter()
        .filter(|(name, _)| name.ends_with("/s"))
        .count();
    let mut entries: Vec<(Box<str>, CheckedNodeId)> =
        Budget::lock_node(meter, 0, false).vec(names.len() + candidates, "unit_refs")?;
    for (name, position) in &mut names {
        let node_id = &nodes[*position].node_id;
        request(meter, *position, node_id_bytes(node_id))?;
        entries.push((std::mem::take(name), node_id.clone()));
    }
    // `names` keeps each entry's node position beside it.
    for (at, (_, position)) in names.iter().enumerate() {
        let Some(name) = entries.get(at).map(|entry| entry.0.as_ref()) else {
            continue;
        };
        if !name.ends_with("/s") {
            continue;
        }
        if let Some(unit_id) = identity::duration_unit_key(nodes, terms, name, meter)? {
            let len = DURATION_RESOLVED_PREFIX.len() + name.len();
            request(meter, *position, len)?;
            let mut key = String::with_capacity(len);
            key.push_str(DURATION_RESOLVED_PREFIX);
            key.push_str(name);
            entries.push((key.into_boxed_str(), unit_id));
        }
    }
    // The resolved keys cannot repeat a unit name or each other.
    entries.sort_unstable_by(|left, right| left.0.cmp(&right.0));
    request(meter, 0, arc_bytes::<SortedMap<Box<str>, CheckedNodeId>>())?;
    Ok(Arc::new(SortedMap::from_sorted(entries)))
}

/// The empty unit table of a package that selects no model, requested at the
/// first node.
pub(super) fn no_unit_refs(meter: &mut WorkMeter) -> Result<UnitRefs, ValidationFailure> {
    request(meter, 0, arc_bytes::<SortedMap<Box<str>, CheckedNodeId>>())?;
    Ok(Arc::new(SortedMap::new()))
}

/// Gives each selected model the unit table with its own one-second Duration
/// units resolved. A table without any is left shared as it is.
pub(super) fn resolve_model_durations(
    unit_refs: &UnitRefs,
    models: &mut [DomainModel],
    nodes: &[RawSemanticNodeV2],
    meter: &mut WorkMeter,
) -> Result<(), ValidationFailure> {
    if models.is_empty() {
        return Ok(());
    }
    let widest = models
        .iter()
        .map(|model| model.identity.len())
        .max()
        .unwrap_or(0);
    request(meter, 0, UNIT_PREFIX.len() + widest + "/s".len())?;
    let mut name = String::with_capacity(UNIT_PREFIX.len() + widest + "/s".len());
    let mut extras: Vec<(Box<str>, CheckedNodeId)> =
        Budget::lock_node(meter, 0, false).vec(models.len(), "unit_refs")?;
    for (position, model) in models.iter().enumerate() {
        name.clear();
        name.push_str(UNIT_PREFIX);
        name.push_str(&model.identity);
        name.push_str("/s");
        if !unit_refs.contains_key(name.as_str()) {
            continue;
        }
        if let Some(unit_id) = identity::model_duration_unit_key(nodes, model, meter)? {
            let len = DURATION_RESOLVED_PREFIX.len() + name.len();
            request(meter, position, len)?;
            let mut key = String::with_capacity(len);
            key.push_str(DURATION_RESOLVED_PREFIX);
            key.push_str(&name);
            extras.push((key.into_boxed_str(), unit_id));
        }
    }
    merge_durations(unit_refs, extras, models, meter)
}

/// Gives each model the unit table with `extras`, its resolved Duration
/// units, added; every entry copied and the new table requested first. No
/// extras leave the shared table as it is.
fn merge_durations(
    unit_refs: &UnitRefs,
    extras: Vec<(Box<str>, CheckedNodeId)>,
    models: &mut [DomainModel],
    meter: &mut WorkMeter,
) -> Result<(), ValidationFailure> {
    if extras.is_empty() {
        return Ok(());
    }
    let mut budget = Budget::lock_node(meter, 0, false);
    let mut resolved =
        SortedMap::with_capacity(&mut budget, unit_refs.len() + extras.len(), "unit_refs")?;
    for (key, node_id) in unit_refs.iter() {
        budget.request(key.len() + node_id_bytes(node_id), "unit_refs")?;
        resolved.insert(&mut budget, key.clone(), node_id.clone(), "unit_refs")?;
    }
    for (key, node_id) in extras {
        resolved.insert(&mut budget, key, node_id, "unit_refs")?;
    }
    budget.request(
        arc_bytes::<SortedMap<Box<str>, CheckedNodeId>>(),
        "unit_refs",
    )?;
    let resolved = Arc::new(resolved);
    for model in models {
        model.unit_refs = resolved.clone();
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::checked_package::v2::PackageReadStage;
    use crate::{NativeJoinBudget, NativeJoinLocusSource};
    use ix_trace_rs::trace;
    use serde_json::json;

    /// What one run did under `allowance` bytes of read storage.
    struct Run {
        ok: bool,
        consumed: u64,
        allocated: u64,
        locus: Option<crate::NativeJoinLocus>,
    }

    fn run_under<T>(
        allowance: u64,
        input: T,
        body: impl FnOnce(T, &mut WorkMeter) -> Result<(), ValidationFailure>,
    ) -> Run {
        let mut budget =
            NativeJoinBudget::new(u64::MAX, u64::MAX).with_read_storage_limit(allowance);
        let mut ok = false;
        let measured = allocation_counter::measure(|| {
            let mut meter = WorkMeter::over(&mut budget);
            ok = body(input, &mut meter).is_ok();
        });
        Run {
            ok,
            consumed: budget.read_storage_consumed(),
            allocated: measured.bytes_total,
            locus: budget.first_unpaid_locus(),
        }
    }

    /// Every byte `body` allocates is a byte it requested, the exact allowance
    /// admits it, one byte short refuses in the Lock stage, and no shorter
    /// allowance allocates past what it was granted.
    fn assert_lock_exact<T>(
        prepare: impl Fn() -> T,
        body: impl Fn(T, &mut WorkMeter) -> Result<(), ValidationFailure>,
    ) {
        let unlimited = run_under(u64::MAX, prepare(), &body);
        assert!(unlimited.ok, "the run succeeds without a limit");
        assert!(unlimited.consumed > 0, "the run requests storage");
        assert_eq!(
            unlimited.allocated, unlimited.consumed,
            "every byte allocated is a byte requested"
        );
        assert!(run_under(unlimited.consumed, prepare(), &body).ok);
        let short = run_under(unlimited.consumed - 1, prepare(), &body);
        assert!(!short.ok, "one byte short refuses");
        assert!(matches!(
            short.locus.map(|locus| locus.source),
            Some(NativeJoinLocusSource::PackageRead(
                PackageReadStage::Lock,
                _
            ))
        ));
        let denial = allocation_counter::measure(|| {
            drop(node_pointer(99_999).key("owner"));
            drop(member_pointer(&["lock", "refinement_internals"]).index(99_999));
        })
        .bytes_total;
        for allowance in 0..unlimited.consumed {
            let refused = run_under(allowance, prepare(), &body);
            assert!(!refused.ok, "allowance {allowance} is short");
            assert!(
                refused.allocated <= allowance + denial,
                "allowance {allowance}: allocated {}",
                refused.allocated
            );
        }
    }

    /// The tables that find a repeated lock value are requested before they
    /// are made, at their exact size, in the Lock stage.
    #[trace("TC-472", "FR-038-AC-502")]
    #[test]
    fn tc_472_the_lock_value_tables_allocate_exactly_what_they_request() {
        let selections: Vec<CheckedDependencySelection> = ["a", "b", "c", "d", "e"]
            .iter()
            .map(|identity| CheckedDependencySelection {
                identity: (*identity).into(),
            })
            .collect();
        assert_lock_exact(
            || (),
            |(), meter| validate_dependency_selections(&selections, meter),
        );
        let features: Vec<Box<str>> = ["x", "y", "z"].iter().map(|name| (*name).into()).collect();
        assert_lock_exact(
            || (),
            |(), meter| validate_required_features(&features, meter),
        );
        let rows: Vec<CheckedRefinementInternal> = [("A", ["a", "b", "c"]), ("B", ["d", "e", "f"])]
            .iter()
            .map(|(refinement, internal)| CheckedRefinementInternal {
                refinement: (*refinement).into(),
                internal: internal.iter().map(|name| (*name).into()).collect(),
            })
            .collect();
        assert_lock_exact(
            || (),
            |(), meter| validate_refinement_internals(Some(&rows), meter),
        );
    }

    fn node(
        digit: usize,
        tag: &str,
        form: &str,
        owner: serde_json::Value,
        declaration: serde_json::Value,
    ) -> RawSemanticNodeV2 {
        let id =
            json!({"package": "example", "path": [tag, form, format!("n{digit}")], "ordinal": 0});
        let mut value = json!({
            "node_id": id,
            "schema_version": "quire.checked-semantic-graph/v2",
            "node_tag": tag,
            "semantic_form": form,
            "semantic_type": id,
            "dependencies": [],
            "occurrences": [],
            "body": {},
        });
        if !owner.is_null() {
            value["owner"] = owner;
        }
        if !declaration.is_null() {
            value["declaration"] = declaration;
        }
        serde_json::from_value(value).expect("test node")
    }

    fn source_unit(digit: usize, name: &[&str]) -> RawSemanticNodeV2 {
        node(
            digit,
            "scalar_type",
            "unit",
            json!({"kind": "source", "authority": "a", "identity": "b"}),
            json!({"qualified_name": name}),
        )
    }

    fn model_unit(digit: usize, identity: &str, unit: &str) -> RawSemanticNodeV2 {
        node(
            digit,
            "scalar_type",
            "unit",
            json!({"kind": "model", "identity": identity, "node": format!("ix://{identity}/unit/{unit}")}),
            serde_json::Value::Null,
        )
    }

    fn model_root(digit: usize, identity: &str) -> RawSemanticNodeV2 {
        node(
            digit,
            "model",
            "model_root",
            json!({"kind": "model_root", "identity": identity}),
            serde_json::Value::Null,
        )
    }

    /// The model roots and the unit references are requested before they are
    /// made: one name per unit, its node key, the sorted table and its `Arc`.
    #[trace("TC-472", "FR-038-AC-502")]
    #[test]
    fn tc_472_the_model_roots_and_unit_references_allocate_exactly_what_they_request() {
        let nodes = vec![
            model_root(0, "acme/orders"),
            source_unit(1, &["quire", "m"]),
            model_unit(2, "acme/orders", "km"),
            source_unit(3, &["quire", "m"]),
            model_root(4, "acme/billing"),
            source_unit(5, &["quire", "kg"]),
        ];
        assert_lock_exact(
            || (),
            |(), meter| selected_model_roots(&nodes, meter).map(drop),
        );
        assert_lock_exact(
            || (),
            |(), meter| checked_unit_refs(&nodes, meter).map(drop),
        );
        let roots = selected_model_roots(&nodes, &mut WorkMeter::new(u64::MAX)).expect("roots");
        assert_eq!(
            roots,
            vec![("acme/billing", 4), ("acme/orders", 0)],
            "roots in identity order"
        );
        let refs = checked_unit_refs(&nodes, &mut WorkMeter::new(u64::MAX)).expect("refs");
        assert_eq!(
            refs.keys().map(AsRef::as_ref).collect::<Vec<&str>>(),
            [
                "ix://quire/unit/acme/orders/km",
                "ix://quire/unit/quire/kg",
                "ix://quire/unit/quire/m",
            ],
        );
        assert_eq!(
            refs.get("ix://quire/unit/quire/m"),
            Some(&nodes[1].node_id),
            "the first node holding a name keeps it"
        );
    }

    #[trace("TC-472", "FR-038-AC-502")]
    #[test]
    fn tc_472_a_repeated_root_refuses_at_the_later_owner() {
        let nodes = vec![
            model_root(0, "b"),
            model_root(1, "a"),
            model_root(2, "b"),
            model_root(3, "a"),
        ];
        let Err(ValidationFailure::Refused(refusal)) =
            selected_model_roots(&nodes, &mut WorkMeter::new(u64::MAX))
        else {
            panic!("a repeated root refuses");
        };
        assert_eq!(
            refusal.loci[0].path.as_ref().map(ToString::to_string),
            Some("/semantic_graph/nodes/2/owner".to_owned())
        );
    }

    /// The merge that gives each model its resolved Duration units is
    /// requested before it is made. Resolving a unit (`identity.rs`'s
    /// `model_duration_unit_key`, group D) allocates uncharged text, a
    /// `BTreeSet` and exact ratios, so this covers the charged merge over
    /// supplied units and the charged scratch of a table that needs no
    /// resolution, and not that call.
    #[trace("TC-472", "FR-038-AC-502")]
    #[test]
    fn tc_472_the_duration_unit_merge_allocates_exactly_what_it_requests() {
        use crate::checked_package::v2::model_members::tests as model_tests;
        let nodes = vec![
            source_unit(0, &["quire", "m"]),
            source_unit(1, &["quire", "kg"]),
        ];
        let refs = checked_unit_refs(&nodes, &mut WorkMeter::new(u64::MAX)).expect("refs");
        let model =
            model_tests::read_with_unit_refs(&model_tests::document(vec![]), &refs).expect("model");
        assert_lock_exact(
            || {
                let extras: Vec<(Box<str>, CheckedNodeId)> = vec![
                    (
                        format!("{DURATION_RESOLVED_PREFIX}ix://quire/unit/quire/s").into(),
                        nodes[0].node_id.clone(),
                    ),
                    (
                        format!("{DURATION_RESOLVED_PREFIX}ix://quire/unit/acme/s").into(),
                        nodes[1].node_id.clone(),
                    ),
                ];
                (extras, vec![model.clone(), model.clone()])
            },
            |(extras, mut models), meter| merge_durations(&refs, extras, &mut models, meter),
        );
        let (extras, mut models) = (
            vec![(
                Box::<str>::from(format!("{DURATION_RESOLVED_PREFIX}ix://quire/unit/quire/s")),
                nodes[0].node_id.clone(),
            )],
            vec![model.clone()],
        );
        merge_durations(&refs, extras, &mut models, &mut WorkMeter::new(u64::MAX)).expect("merged");
        assert_eq!(models[0].unit_refs.len(), refs.len() + 1);
        assert!(models[0]
            .unit_refs
            .contains_key("ix://quire/resolved-duration/ix://quire/unit/quire/s"));
        // No unit named `s`: the scratch and the empty extras are all it asks.
        assert_lock_exact(
            || vec![model.clone()],
            |mut models, meter| resolve_model_durations(&refs, &mut models, &nodes, meter),
        );
    }
}
