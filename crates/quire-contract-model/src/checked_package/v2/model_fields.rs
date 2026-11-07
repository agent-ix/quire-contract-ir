//! FR-038 "Typed accessor for a model object type's fields" (IR-628): the
//! field table the reader builds once per object type at admission, the
//! tables, declaration index and relationship records an admitted
//! [`CheckedPackageV2`] retains, and the accessor
//! [`CheckedPackageV2::model_object_fields`] that reads the field tables (the
//! relationship records are read by `CheckedPackageV2::relationship_end`).
//! [`CheckedPackageV2::model_record_fields`] reads the same retained tables
//! for selected model record declarations without projecting Exact types.
//!
//! A model declaration node's body is `aggregate{[]}` and carries no member,
//! so the field set of a model object type is in the selected domain package
//! document ([`DomainModel`]). The table of one object type holds its exposed
//! effective fields and its hidden entries, as FR-322 step 3
//! ([`DomainModel::resolve`]) defines them, and is built from the tables of
//! the type's declared supertypes, each declaration read once. Declared
//! supertypes that form a cycle share one table per strongly connected
//! component, over the owners the reached-set rule of
//! `DomainModel::ancestors` gives: the component and every type reachable from
//! it, a member of a cycle being among its own ancestors.

use super::model_members::{
    arc_bytes, arc_str_bytes, check_declaration_node, field_decl_bytes, member_name, units, Budget,
    CollectionKind, DeclarationForm, DomainModel, FieldBearingTypeDecl, FieldDecl, MemberType,
    ModelOperationClass, NativeTypeRef, Owner, RelationshipDecl, SelectedTypeRef,
    SelectedVariantCase, SortedMap, SortedSet,
};
use super::{CheckedNodeId, CheckedPackageV2, CheckedTerm, PackageReadStage, WorkMeter};
use crate::checked_package::common::ValidationFailure;
use std::collections::{BTreeMap, BTreeSet};
use std::fmt;
use std::sync::Arc;

/// One exposed field of a table: the declaration and the member type FR-322
/// step 4 derives for it, derived once.
#[derive(Debug, Eq, PartialEq)]
pub(super) struct FieldEntry {
    /// The field's IR node identity, `<owner>/<name>`.
    identity: Arc<str>,
    pub(super) decl: FieldDecl,
    /// Selected declaration owner retained while building the table.
    declaring_owner: Box<str>,
    /// `DomainModel::field_type` of the declaration.
    member_type: Option<MemberType>,
}

/// One effective field and the identities whose inherited position it holds.
#[derive(Clone, Debug, Eq, PartialEq)]
struct OrderedField {
    entry: Arc<FieldEntry>,
    origins: SortedSet<Arc<str>>,
}

impl FieldEntry {
    fn name(&self) -> &str {
        member_name(&self.identity)
    }
}

/// The table of one object type, or of one strongly connected component of
/// object types: the exposed effective fields and the hidden entries of
/// `DomainModel::resolve` for `MemberKind::Field`.
#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub(super) struct FieldTable {
    /// Ascending by identity, one entry per identity.
    exposed: Vec<Arc<FieldEntry>>,
    /// The same exposed entries in inherited source order, then own declaration order.
    ordered_exposed: Vec<OrderedField>,
    /// Ascending, without repeats: every identity a redefinition hides, a
    /// redefined target and a redefiner hidden by a more derived one alike.
    hidden: Vec<Arc<str>>,
    /// The smallest name that two exposed fields share.
    ambiguous: Option<Box<str>>,
}

impl FieldTable {
    /// The work one field resolution charges: one unit per exposed or hidden
    /// entry.
    pub(super) fn entries(&self) -> usize {
        self.exposed.len().saturating_add(self.hidden.len())
    }

    /// The exposed field named `name`.
    pub(super) fn select(&self, name: &str) -> Selected<'_> {
        let mut named = self
            .exposed
            .iter()
            .filter(|entry| entry.name() == name)
            .map(|entry| &entry.decl);
        match (named.next(), named.next()) {
            (None, _) => Selected::Absent,
            (Some(field), None) => Selected::One(field),
            (Some(_), Some(_)) => Selected::Ambiguous,
        }
    }

    /// The effective record field types in inherited source order.
    pub(super) fn binder_types(&self, package: &str) -> Option<RecordBinderTypes> {
        if self.ambiguous.is_some() {
            return Some(RecordBinderTypes::Ambiguous);
        }
        let types = self
            .ordered_exposed
            .iter()
            .map(|entry| entry.entry.member_type.as_ref()?.node_id(package))
            .collect::<Option<Vec<_>>>()?;
        Some(RecordBinderTypes::Known(types))
    }
}

/// One borrowed effective field of an admitted selected record.
pub(super) struct SelectedRecordField<'a> {
    pub(super) identity: &'a str,
    pub(super) name: &'a str,
    pub(super) member_type: Option<&'a MemberType>,
    pub(super) optional: bool,
}

/// A record's checked binder types, or an inherited same-name ambiguity.
pub(super) enum RecordBinderTypes {
    Known(Vec<CheckedNodeId>),
    Ambiguous,
}

/// The exposed fields of a table that bear one name.
pub(super) enum Selected<'t> {
    Absent,
    One(&'t FieldDecl),
    Ambiguous,
}

/// The field tables of one domain package document: one table per strongly
/// connected component of its field-bearing types, and the component of each type.
#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub(super) struct FieldTables {
    by_type: SortedMap<Box<str>, usize>,
    components: Vec<FieldTable>,
}

impl FieldTables {
    /// The table of the object type (or systems interface) `node`.
    pub(super) fn of(&self, node: &str) -> Option<&FieldTable> {
        self.components.get(*self.by_type.get(node)?)
    }

    pub(super) fn exposed_names(&self, node: &str) -> Option<BTreeSet<Box<str>>> {
        Some(
            self.of(node)?
                .exposed
                .iter()
                .map(|field| Box::from(field.name()))
                .collect(),
        )
    }
}

/// Builds the field tables of every document, in lock order of the rows, each
/// charged to the `work` limit at its row. The last act of the lock stage:
/// every row's step 1 has already admitted.
pub(super) fn build_field_tables(
    models: &mut [DomainModel],
    meter: &mut WorkMeter,
) -> Result<(), ValidationFailure> {
    for (position, model) in models.iter_mut().enumerate() {
        let mut budget =
            Budget::new(meter, model.root, model.bytes).at(PackageReadStage::Lock, position);
        let tables = tables_of(model, &mut budget)?;
        model.fields = tables;
    }
    Ok(())
}

/// The `form` every request and work charge of the field tables is made at.
const F: &str = "field_tables";

/// A strongly connected component of the supertype graph.
struct Component<'m> {
    members: Vec<(&'m str, &'m FieldBearingTypeDecl)>,
    /// More than one member, or a member that extends itself.
    cyclic: bool,
}

fn tables_of(
    model: &DomainModel,
    budget: &mut Budget<'_, '_>,
) -> Result<FieldTables, ValidationFailure> {
    let found = components(model, budget)?;
    let nodes = model.object_types.len() + model.record_types.len();
    let mut tables = FieldTables {
        by_type: SortedMap::with_capacity(budget, nodes, F)?,
        components: budget.vec(found.len(), F)?,
    };
    for component in &found {
        let table = build_component(model, component, &tables, budget)?;
        let index = tables.components.len();
        tables.components.push(table);
        for (node, _) in &component.members {
            let key = budget.text(node, F)?;
            tables.by_type.insert(budget, key, index, F)?;
        }
    }
    Ok(tables)
}

/// The strongly connected components of the supertype graph of `model`'s
/// objects and records, a component after every component of its supertypes
/// (Tarjan's algorithm over an explicit stack). Every table the search keeps
/// is requested at the capacity the search can need: the number of nodes.
fn components<'m>(
    model: &'m DomainModel,
    budget: &mut Budget<'_, '_>,
) -> Result<Vec<Component<'m>>, ValidationFailure> {
    let nodes = model.object_types.len() + model.record_types.len();
    let mut search = Tarjan {
        model,
        visits: SortedMap::with_capacity(budget, nodes, F)?,
        stack: budget.vec(nodes, F)?,
        open: budget.vec(nodes, F)?,
        found: budget.vec(nodes, F)?,
    };
    for root in model
        .object_types
        .keys()
        .chain(model.record_types.keys())
        .map(AsRef::as_ref)
    {
        if !search.visits.contains_key(root) {
            search.visit(root, budget)?;
        }
    }
    Ok(search.found)
}

/// Tarjan's search state: a node's discovery index, its lowest reachable
/// index, and whether it is still on the component stack.
struct Visit {
    index: usize,
    low: usize,
    on_stack: bool,
}

/// An open node with the position of the next supertype it follows.
type Open<'m> = (&'m str, usize);

struct Tarjan<'m> {
    model: &'m DomainModel,
    visits: SortedMap<&'m str, Visit>,
    stack: Vec<&'m str>,
    open: Vec<Open<'m>>,
    found: Vec<Component<'m>>,
}

impl<'m> Tarjan<'m> {
    /// The next declared supertype of `node` at or after `from` that the
    /// document declares as a field-bearing type, and the position after it;
    /// any other has no table and no members.
    fn next_supertype(&self, node: &str, from: usize) -> Option<(&'m str, usize)> {
        let model = self.model;
        let declared = model.field_owner(node)?;
        declared
            .supertypes
            .iter()
            .enumerate()
            .skip(from)
            .find(|(_, supertype)| model.field_owner(supertype).is_some())
            .map(|(at, supertype)| (supertype.as_ref(), at + 1))
    }

    fn enter(
        &mut self,
        node: &'m str,
        budget: &mut Budget<'_, '_>,
    ) -> Result<(), ValidationFailure> {
        let index = self.visits.len();
        self.visits.insert(
            budget,
            node,
            Visit {
                index,
                low: index,
                on_stack: true,
            },
            F,
        )?;
        self.stack.push(node);
        self.open.push((node, 0));
        Ok(())
    }

    fn lower(&mut self, node: &str, to: usize) {
        if let Some(visit) = self.visits.get_mut(node) {
            visit.low = visit.low.min(to);
        }
    }

    /// Searches from `root` over an explicit stack, so a chain of any length
    /// needs no call stack.
    fn visit(
        &mut self,
        root: &'m str,
        budget: &mut Budget<'_, '_>,
    ) -> Result<(), ValidationFailure> {
        self.enter(root, budget)?;
        while let Some(&(node, from)) = self.open.last() {
            match self.next_supertype(node, from) {
                Some((next, after)) => {
                    if let Some(top) = self.open.last_mut() {
                        top.1 = after;
                    }
                    self.follow(node, next, budget)?;
                }
                None => {
                    self.open.pop();
                    let parent = self.open.last().map(|(parent, _)| *parent);
                    self.leave(node, parent, budget)?;
                }
            }
        }
        Ok(())
    }

    fn follow(
        &mut self,
        node: &str,
        next: &'m str,
        budget: &mut Budget<'_, '_>,
    ) -> Result<(), ValidationFailure> {
        match self.visits.get(next) {
            None => self.enter(next, budget)?,
            Some(seen) if seen.on_stack => {
                let index = seen.index;
                self.lower(node, index);
            }
            Some(_) => {}
        }
        Ok(())
    }

    /// Closes `node`; it roots a component when no node below it reached an
    /// earlier one.
    fn leave(
        &mut self,
        node: &'m str,
        parent: Option<&str>,
        budget: &mut Budget<'_, '_>,
    ) -> Result<(), ValidationFailure> {
        let Some(&Visit { index, low, .. }) = self.visits.get(node) else {
            return Ok(());
        };
        if let Some(parent) = parent {
            self.lower(parent, low);
        }
        if low != index {
            return Ok(());
        }
        // The component is the stack above `node`.
        let size = self
            .stack
            .iter()
            .rposition(|member| *member == node)
            .map_or(0, |at| self.stack.len() - at);
        let mut members = budget.vec(size, F)?;
        while let Some(member) = self.stack.pop() {
            if let Some(visit) = self.visits.get_mut(member) {
                visit.on_stack = false;
            }
            if let Some((key, declared)) = self
                .model
                .object_types
                .get_key_value(member)
                .or_else(|| self.model.record_types.get_key_value(member))
            {
                members.push((key.as_ref(), declared));
            }
            if member == node {
                break;
            }
        }
        members.sort_unstable_by(|a, b| a.0.cmp(b.0));
        let cyclic = members.len() > 1
            || members.iter().any(|(member, declared)| {
                declared
                    .supertypes
                    .iter()
                    .any(|supertype| supertype.as_ref() == *member)
            });
        self.found.push(Component { members, cyclic });
        Ok(())
    }
}

/// Where a slot's origins are held: in the table it was copied from, or among
/// the origins built for this component's own fields.
#[derive(Clone, Copy)]
enum Origins<'t> {
    Shared(&'t [Arc<str>]),
    Own(usize),
}

/// The inherited redefiners of `target` among `rows`, which are sorted by target.
fn redefined<'a, 't>(
    rows: &'a [(&'a str, &'a Arc<str>)],
    target: &'t str,
) -> impl Iterator<Item = &'a (&'a str, &'a Arc<str>)> + use<'a, 't> {
    let start = rows.partition_point(|row| row.0 < target);
    rows.get(start..)
        .unwrap_or_default()
        .iter()
        .take_while(move |row| row.0 == target)
}

/// The origins `origins` names.
fn origins_in<'a>(origins: Origins<'a>, own: &'a [Vec<Arc<str>>]) -> &'a [Arc<str>] {
    match origins {
        Origins::Shared(origins) => origins,
        Origins::Own(index) => own.get(index).map_or(&[], Vec::as_slice),
    }
}

/// One effective field in inherited source order, then own declaration order.
struct Slot<'t> {
    entry: &'t Arc<FieldEntry>,
    origins: Origins<'t>,
}

fn name(shared: &Arc<str>) -> &str {
    shared
}

/// A name with the shared handle that spells it, when there is one.
type Named<'t> = (&'t str, Option<&'t Arc<str>>);

/// Sorts `names` by name, a handle before none, and drops repeats; returns
/// the bytes the names without a handle take as shared text.
fn distinct(names: &mut Vec<Named<'_>>) -> usize {
    names.sort_unstable_by_key(|(name, arc)| (*name, arc.is_none()));
    names.dedup_by(|later, earlier| later.0 == earlier.0);
    names
        .iter()
        .filter(|(_, arc)| arc.is_none())
        .map(|(name, _)| arc_str_bytes(name.len()))
        .sum()
}

/// The shared names `names` spell, in order, those without a handle made
/// from their text (the request for them is `fresh`).
fn spell(
    names: &[Named<'_>],
    fresh: usize,
    budget: &mut Budget<'_, '_>,
) -> Result<Vec<Arc<str>>, ValidationFailure> {
    budget.request(fresh, F)?;
    let mut spelled = budget.vec(names.len(), F)?;
    for (name, arc) in names {
        spelled.push(arc.map_or_else(|| Arc::from(*name), Arc::clone));
    }
    Ok(spelled)
}

/// The table of one component, from its members' own fields and the tables of
/// the supertypes outside it, charged: one unit per own field and per
/// declared supertype edge of each member, and one per exposed or hidden entry
/// copied from the table of a supertype outside the component. Every
/// container is requested at its exact capacity before it is made, and an
/// entry's text, declaration and member type before they are copied.
fn build_component(
    model: &DomainModel,
    component: &Component<'_>,
    built: &FieldTables,
    budget: &mut Budget<'_, '_>,
) -> Result<FieldTable, ValidationFailure> {
    #[cfg(test)]
    probe::built();
    let own_units = component
        .members
        .iter()
        .fold(0_usize, |total, (_, declared)| {
            total
                .saturating_add(declared.fields.len())
                .saturating_add(declared.supertypes.len())
        });
    budget.charge_form(units(own_units), F)?;
    let (mut inherited_slots, mut inherited_hidden, mut own_fields) = (0_usize, 0_usize, 0_usize);
    for (_, declared) in &component.members {
        own_fields += declared.fields.len();
        for supertype in &declared.supertypes {
            // A supertype of the component itself has no table yet.
            let Some(table) = built.of(supertype) else {
                continue;
            };
            budget.charge_form(units(table.entries()), F)?;
            inherited_slots += table.ordered_exposed.len();
            inherited_hidden += table.hidden.len();
        }
    }
    // The hidden entries of the supertypes' tables.
    let mut hidden0: Vec<&Arc<str>> = budget.vec(inherited_hidden, F)?;
    for (_, declared) in &component.members {
        for supertype in &declared.supertypes {
            if let Some(table) = built.of(supertype) {
                hidden0.extend(table.hidden.iter());
            }
        }
    }
    hidden0.sort_unstable_by(|a, b| name(a).cmp(name(b)));
    hidden0.dedup_by(|later, earlier| name(later) == name(earlier));

    // The declarations of this component's own fields, one entry per
    // identity: a later declaration replaces an earlier one and the entry
    // keeps the position of the first.
    let mut declared: Vec<(&str, usize, &str, &FieldDecl)> = budget.vec(own_fields, F)?;
    for (owner, members) in &component.members {
        for decl in &members.fields {
            declared.push((decl.identity.as_ref(), declared.len(), owner, decl));
        }
    }
    declared.sort_unstable_by(|a, b| a.0.cmp(b.0).then(a.1.cmp(&b.1)));
    let distinct_own = declared
        .iter()
        .enumerate()
        .filter(|(at, row)| *at == 0 || declared.get(at - 1).is_none_or(|prior| prior.0 != row.0))
        .count();
    // Own entries in identity order, each with the position of its first
    // declaration.
    let mut own: Vec<(usize, Arc<FieldEntry>)> = budget.vec(distinct_own, F)?;
    let mut at = 0;
    while let Some(&(identity, first, _, _)) = declared.get(at) {
        let group = declared
            .get(at..)
            .map_or(0, |rest| rest.partition_point(|row| row.0 == identity));
        let &(_, _, owner, decl) = declared.get(at + group - 1).ok_or_else(|| {
            ValidationFailure::refused(
                crate::checked_package::shared::CheckedPackageRefusalCode::InvalidSemanticGraph,
                budget.row(),
            )
        })?;
        budget.request(
            arc_str_bytes(identity.len())
                .saturating_add(owner.len())
                .saturating_add(field_decl_bytes(decl))
                .saturating_add(arc_bytes::<FieldEntry>()),
            F,
        )?;
        let member_type = model.field_type_charged(decl, budget, F)?;
        own.push((
            first,
            Arc::new(FieldEntry {
                identity: Arc::from(identity),
                declaring_owner: Box::from(owner),
                member_type,
                decl: decl.clone(),
            }),
        ));
        at += group;
    }
    drop(declared);
    // Own entries in declaration order.
    let mut own_order: Vec<usize> = budget.vec(own.len(), F)?;
    own_order.extend(0..own.len());
    own_order.sort_unstable_by_key(|&at| own.get(at).map_or(usize::MAX, |entry| entry.0));

    // The effective fields of the supertypes' tables, in order, then the own
    // ones.
    let mut slots: Vec<Slot<'_>> = budget.vec(inherited_slots + own.len(), F)?;
    for (_, members) in &component.members {
        for supertype in &members.supertypes {
            if let Some(table) = built.of(supertype) {
                slots.extend(table.ordered_exposed.iter().map(|slot| Slot {
                    entry: &slot.entry,
                    origins: Origins::Shared(slot.origins.as_slice()),
                }));
            }
        }
    }
    // A diamond reaches one entry twice; a field one supertype exposes and
    // another hides is hidden. The first of each identity stays.
    let mut inherited: Vec<(&str, usize, &Arc<FieldEntry>)> = budget.vec(slots.len(), F)?;
    inherited.extend(
        slots
            .iter()
            .enumerate()
            .map(|(at, slot)| (slot.entry.identity.as_ref(), at, slot.entry)),
    );
    inherited.sort_unstable_by(|a, b| a.0.cmp(b.0).then(a.1.cmp(&b.1)));
    inherited.dedup_by(|later, earlier| later.0 == earlier.0);
    inherited.retain(|(identity, _, _)| {
        hidden0
            .binary_search_by(|hidden| name(hidden).cmp(identity))
            .is_err()
    });
    drop(hidden0);

    // FR-322 step 3: a redefinition hides its target; a redefiner of a target
    // that a redefiner in a more derived owner also redefines is hidden, and
    // every owner here is more derived than every owner copied. A member of a
    // cycle is its own ancestor, so each of its redefiners hides itself.
    let redefiners = inherited
        .iter()
        .filter(|(_, _, entry)| entry.decl.redefines.is_some())
        .count();
    let mut by_target: Vec<(&str, &Arc<str>)> = budget.vec(redefiners, F)?;
    for (_, _, entry) in &inherited {
        if let Some(target) = entry.decl.redefines.as_deref() {
            by_target.push((target, &entry.identity));
        }
    }
    by_target.sort_unstable_by(|a, b| a.0.cmp(b.0));
    let mut hiding = 0_usize;
    for (_, entry) in &own {
        if let Some(target) = entry.decl.redefines.as_deref() {
            hiding += 1 + usize::from(component.cyclic) + redefined(&by_target, target).count();
        }
    }
    let mut hidden_names: Vec<Named<'_>> = budget.vec(hiding, F)?;
    for (_, entry) in &own {
        let Some(target) = entry.decl.redefines.as_deref() else {
            continue;
        };
        hidden_names.push((target, None));
        if component.cyclic {
            hidden_names.push((entry.identity.as_ref(), Some(&entry.identity)));
        }
        hidden_names.extend(redefined(&by_target, target).map(|(_, id)| (id.as_ref(), Some(*id))));
    }
    // The hidden entries the supertypes' tables carry are part of the result.
    let mut hidden_all: Vec<Named<'_>> = budget.vec(hidden_names.len() + inherited_hidden, F)?;
    for (_, members) in &component.members {
        for supertype in &members.supertypes {
            if let Some(table) = built.of(supertype) {
                hidden_all.extend(table.hidden.iter().map(|id| (id.as_ref(), Some(id))));
            }
        }
    }
    hidden_all.extend(hidden_names.iter().copied());
    drop(hidden_names);
    let fresh = distinct(&mut hidden_all);
    let hidden = spell(&hidden_all, fresh, budget)?;
    drop(hidden_all);
    let is_hidden = |identity: &str| {
        hidden
            .binary_search_by(|hidden| name(hidden).cmp(identity))
            .is_ok()
    };

    // Each own field's slot: its identity, the member it redefines, and the
    // origins of every slot already placed that holds that member.
    let mut own_origins: Vec<Vec<Arc<str>>> = budget.vec(own.len(), F)?;
    for &at in &own_order {
        let Some((_, entry)) = own.get(at) else {
            continue;
        };
        let target = entry.decl.redefines.as_deref();
        let holds = |slot: &Slot<'_>| {
            target.is_some_and(|target| {
                origins_in(slot.origins, &own_origins)
                    .binary_search_by(|origin| origin.as_ref().cmp(target))
                    .is_ok()
            })
        };
        let mut count = 1 + usize::from(target.is_some());
        for slot in &slots {
            if holds(slot) {
                count += origins_in(slot.origins, &own_origins).len();
            }
        }
        let origins = {
            let mut names: Vec<Named<'_>> = budget.vec(count, F)?;
            names.push((entry.identity.as_ref(), Some(&entry.identity)));
            if let Some(target) = target {
                names.push((target, None));
            }
            for slot in &slots {
                if holds(slot) {
                    names.extend(
                        origins_in(slot.origins, &own_origins)
                            .iter()
                            .map(|id| (id.as_ref(), Some(id))),
                    );
                }
            }
            let fresh = distinct(&mut names);
            spell(&names, fresh, budget)?
        };
        let index = own_origins.len();
        own_origins.push(origins);
        slots.push(Slot {
            entry,
            origins: Origins::Own(index),
        });
    }

    // The exposed fields: inherited and own, one per identity (an inherited
    // one wins), those not hidden.
    let mut merged: Vec<(&str, u8, &Arc<FieldEntry>)> =
        budget.vec(inherited.len() + own.len(), F)?;
    merged.extend(
        inherited
            .iter()
            .map(|(identity, _, entry)| (*identity, 0, *entry)),
    );
    merged.extend(
        own.iter()
            .map(|(_, entry)| (entry.identity.as_ref(), 1, entry)),
    );
    drop(inherited);
    merged.sort_unstable_by(|a, b| a.0.cmp(b.0).then(a.1.cmp(&b.1)));
    merged.dedup_by(|later, earlier| later.0 == earlier.0);
    merged.retain(|(identity, _, _)| !is_hidden(identity));
    let mut exposed: Vec<Arc<FieldEntry>> = budget.vec(merged.len(), F)?;
    exposed.extend(merged.iter().map(|(_, _, entry)| Arc::clone(entry)));
    drop(merged);

    // Where each identity and each origin first appears among the slots.
    let mut by_identity: Vec<(&str, usize)> = budget.vec(slots.len(), F)?;
    by_identity.extend(
        slots
            .iter()
            .enumerate()
            .map(|(at, slot)| (slot.entry.identity.as_ref(), at)),
    );
    by_identity.sort_unstable();
    let origin_count: usize = slots
        .iter()
        .map(|slot| origins_in(slot.origins, &own_origins).len())
        .sum();
    let mut by_origin: Vec<(&str, usize)> = budget.vec(origin_count, F)?;
    for (at, slot) in slots.iter().enumerate() {
        by_origin.extend(
            origins_in(slot.origins, &own_origins)
                .iter()
                .map(|origin| (origin.as_ref(), at)),
        );
    }
    by_origin.sort_unstable();
    by_origin.dedup_by(|later, earlier| later.0 == earlier.0);

    let mut ordered_exposed: Vec<OrderedField> = budget.vec(exposed.len(), F)?;
    let mut keys: Vec<(&str, usize, usize)> = budget.vec(exposed.len(), F)?;
    for entry in &exposed {
        let identity = entry.identity.as_ref();
        let start = by_identity.partition_point(|row| row.0 < identity);
        let group = by_identity
            .get(start..)
            .unwrap_or_default()
            .iter()
            .take_while(|row| row.0 == identity);
        let mut count = 0;
        let mut declared_position = usize::MAX;
        let mut inherited_position = usize::MAX;
        for &(_, at) in group.clone() {
            declared_position = declared_position.min(at);
            if let Some(slot) = slots.get(at) {
                count += origins_in(slot.origins, &own_origins).len();
            }
        }
        let origins = {
            let mut names: Vec<Named<'_>> = budget.vec(count.max(1), F)?;
            if count == 0 {
                names.push((identity, Some(&entry.identity)));
            }
            for &(_, at) in group {
                let Some(slot) = slots.get(at) else {
                    continue;
                };
                for origin in origins_in(slot.origins, &own_origins) {
                    names.push((origin.as_ref(), Some(origin)));
                    if let Ok(found) = by_origin.binary_search_by(|row| row.0.cmp(origin.as_ref()))
                    {
                        if let Some(&(_, first)) = by_origin.get(found) {
                            inherited_position = inherited_position.min(first);
                        }
                    }
                }
            }
            let fresh = distinct(&mut names);
            spell(&names, fresh, budget)?
        };
        keys.push((identity, inherited_position, declared_position));
        ordered_exposed.push(OrderedField {
            entry: Arc::clone(entry),
            origins: SortedSet::from_sorted(origins),
        });
    }
    let key = |slot: &OrderedField| {
        keys.binary_search_by(|row| row.0.cmp(slot.entry.identity.as_ref()))
            .ok()
            .and_then(|at| keys.get(at))
            .map_or((usize::MAX, usize::MAX), |row| (row.1, row.2))
    };
    ordered_exposed.sort_unstable_by(|a, b| {
        key(a)
            .cmp(&key(b))
            .then_with(|| a.entry.identity.cmp(&b.entry.identity))
    });
    drop((keys, by_identity, by_origin));
    drop(own_origins);

    // The smallest name two exposed fields share.
    let mut names: Vec<&str> = budget.vec(exposed.len(), F)?;
    names.extend(exposed.iter().map(|entry| entry.name()));
    names.sort_unstable();
    let ambiguous = names
        .windows(2)
        .find(|pair| pair.first() == pair.get(1))
        .and_then(|pair| pair.first().copied())
        .map(|name| budget.text(name, F))
        .transpose()?;
    Ok(FieldTable {
        exposed,
        ordered_exposed,
        hidden,
        ambiguous,
    })
}

/// What a selected declaration's model declaration node key names.
#[derive(Clone, Debug, Eq, PartialEq)]
pub(super) struct DeclaredNode {
    pub(super) form: DeclarationForm,
    /// The index of the declaring document among the selected packages.
    pub(super) selection: usize,
    /// The declaration's IR node identity.
    pub(super) node: Box<str>,
}

/// An owned index from each selected declaration's model declaration node key
/// to its selection row, form and node identity.
#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub(super) struct DeclarationIndex(pub(super) BTreeMap<CheckedNodeId, DeclaredNode>);

/// What an admitted [`CheckedPackageV2`] retains of its selected documents:
/// the declaration index, each document's field tables, the selected
/// operation parameter types needed for clause-context reads, and each
/// selected relationship's [`RetainedRelationship`] record.
/// These are derived caches and do not take part in package equality.
/// The package separately retains complete admitted documents as content.
#[derive(Clone, Debug, Default)]
pub(super) struct RetainedModels {
    index: DeclarationIndex,
    original_declarations: BTreeMap<(usize, Box<str>), Option<CheckedNodeId>>,
    by_identity: BTreeMap<Box<str>, usize>,
    object_declarations: BTreeMap<usize, BTreeMap<Box<str>, DeclaringNode>>,
    /// Admitted selected documents, including their field tables, for
    /// catalog binder resolution only when a refinement requests it.
    models: Vec<DomainModel>,
    operation_types: BTreeMap<(usize, Box<str>, Box<str>), OperationTypes>,
    relationships: BTreeMap<(usize, Box<str>), RetainedRelationship>,
}

/// The selected document's complete retained body for one admitted variant declaration.
#[derive(Clone, Copy, Debug)]
pub struct CheckedModelVariantBody<'a> {
    document: &'a str,
    cases: &'a SortedMap<Box<str>, SelectedVariantCase>,
}

impl<'a> CheckedModelVariantBody<'a> {
    /// The identity of the selected domain package document.
    pub fn selected_document(&self) -> &'a str {
        self.document
    }

    /// `union` when any case has a payload, otherwise `enum`.
    pub fn family(&self) -> &'static str {
        if self.cases.values().any(|case| case.payload.is_some()) {
            "union"
        } else {
            "enum"
        }
    }

    /// Every retained case, in ascending UTF-8 name order.
    pub fn cases(&self) -> impl ExactSizeIterator<Item = CheckedModelVariantCase<'a>> + 'a {
        self.cases
            .iter()
            .map(|(name, case)| CheckedModelVariantCase {
                name,
                identity: &case.identity,
                position: case.position,
                payload: case.payload.as_ref().map(CheckedVariantPayloadType::from),
            })
    }
}

/// One case of an admitted selected variant declaration.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct CheckedModelVariantCase<'a> {
    name: &'a str,
    identity: &'a str,
    position: usize,
    payload: Option<CheckedVariantPayloadType<'a>>,
}

impl<'a> CheckedModelVariantCase<'a> {
    /// The exact case name used for ascending UTF-8 ordering.
    pub fn name(&self) -> &'a str {
        self.name
    }

    /// The original identity of this case in the selected document.
    pub fn identity(&self) -> &'a str {
        self.identity
    }

    /// The case's authored zero-based position in the selected document.
    pub fn position(&self) -> usize {
        self.position
    }

    /// Zero for a nullary case, one for a case with a payload.
    pub fn arity(&self) -> usize {
        usize::from(self.payload.is_some())
    }

    /// The alias-resolved type at position zero, if this case has a payload.
    pub fn payload_type(&self) -> Option<CheckedVariantPayloadType<'a>> {
        self.payload
    }
}

/// The admitted, alias-resolved form of a selected variant payload type.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum CheckedVariantPayloadType<'a> {
    /// A native type named by its full `ix://quire/native/` reference.
    Native(&'static str),
    /// A selected unit's exact identity.
    Unit(&'a str),
    /// A selected declared type's exact identity.
    Declared(&'a str),
}

impl<'a> From<&'a SelectedTypeRef> for CheckedVariantPayloadType<'a> {
    fn from(value: &'a SelectedTypeRef) -> Self {
        match value {
            SelectedTypeRef::Native(native) => Self::Native(native_type_reference(native)),
            SelectedTypeRef::Unit(unit) => Self::Unit(unit),
            SelectedTypeRef::Declared(declared) => Self::Declared(declared),
            SelectedTypeRef::Map(..) => {
                unreachable!("selected variant admission refuses map payloads")
            }
        }
    }
}

fn native_type_reference(native: &NativeTypeRef) -> &'static str {
    match native {
        NativeTypeRef::Boolean => "ix://quire/native/Boolean",
        NativeTypeRef::Integer => "ix://quire/native/Integer",
        NativeTypeRef::Rational => "ix://quire/native/Rational",
        NativeTypeRef::Decimal => "ix://quire/native/Decimal",
        NativeTypeRef::Float32 => "ix://quire/native/Float32",
        NativeTypeRef::Float64 => "ix://quire/native/Float64",
        NativeTypeRef::Text => "ix://quire/native/Text",
        NativeTypeRef::String => "ix://quire/native/String",
        NativeTypeRef::Uuid => "ix://quire/native/UUID",
        NativeTypeRef::Timestamp => "ix://quire/native/Timestamp",
        NativeTypeRef::Bytes => "ix://quire/native/Bytes",
        NativeTypeRef::JsonObject => "ix://quire/native/JsonObject",
        NativeTypeRef::Duration => "ix://quire/native/Duration",
        NativeTypeRef::Clock => "ix://quire/native/Clock",
    }
}

/// A selected owner has either one genuine declaration node or several.
#[derive(Clone, Debug)]
enum DeclaringNode {
    One(CheckedNodeId),
    Ambiguous,
}

/// The relationship record admission consults, kept for the accessor: both
/// ends (role, type reference, multiplicity) and the direction. The reaches
/// operation supplies its destination type ID separately.
#[derive(Clone, Debug)]
pub(super) struct RetainedRelationship {
    pub(super) declaration: RelationshipDecl,
}

/// The selected model signature already checked against a clause by FR-341.
#[derive(Clone, Debug)]
pub(super) struct OperationTypes {
    pub(super) has_result: bool,
    pub(super) parameters: Vec<Option<CheckedMemberType>>,
}

impl RetainedModels {
    /// Admission-retained owner of a selected model declaration key.
    pub(super) fn owner_by_key(&self, key: &CheckedNodeId) -> Option<Owner<'_>> {
        let declared = self.index.0.get(key)?;
        Some(Owner {
            package: self.models.get(declared.selection)?,
            form: declared.form,
            node: &declared.node,
            selection: declared.selection,
        })
    }

    /// All selected declaration joins in structural identity order.
    pub(super) fn selected_declarations(
        &self,
    ) -> impl Iterator<Item = (&CheckedNodeId, &DeclaredNode)> {
        self.index.0.iter()
    }

    /// Find one admitted declaration without enumerating unrelated selected models.
    pub(super) fn selected_declaration_entry(&self, key: &CheckedNodeId) -> Option<&DeclaredNode> {
        self.index.0.get(key)
    }

    /// Find a selected model dependency by admitted original identity.
    pub(super) fn selected_declaration_by_original(
        &self,
        selection: usize,
        original: &str,
    ) -> Option<&CheckedNodeId> {
        self.original_declarations
            .get(&(selection, Box::from(original)))?
            .as_ref()
    }

    /// The complete effective record fields in selected declaration order.
    pub(super) fn selected_record_fields(
        &self,
        record: &CheckedNodeId,
    ) -> Option<(
        Option<&str>,
        impl ExactSizeIterator<Item = SelectedRecordField<'_>>,
    )> {
        let declared = self.index.0.get(record)?;
        if declared.form != DeclarationForm::RecordValueType {
            return None;
        }
        let table = self
            .models
            .get(declared.selection)?
            .fields
            .of(&declared.node)?;
        let fields = table
            .ordered_exposed
            .iter()
            .map(|field| SelectedRecordField {
                identity: &field.entry.identity,
                name: field.entry.name(),
                member_type: field.entry.member_type.as_ref(),
                optional: field.entry.decl.optional,
            });
        Some((table.ambiguous.as_deref(), fields))
    }

    /// Complete effective object fields in selected declaration order.
    pub(super) fn selected_object_fields(
        &self,
        object: &CheckedNodeId,
    ) -> Option<(
        Option<&str>,
        impl ExactSizeIterator<Item = SelectedRecordField<'_>>,
    )> {
        let declared = self.index.0.get(object)?;
        if declared.form != DeclarationForm::ObjectType {
            return None;
        }
        let table = self
            .models
            .get(declared.selection)?
            .fields
            .of(&declared.node)?;
        let fields = table
            .ordered_exposed
            .iter()
            .map(|field| SelectedRecordField {
                identity: &field.entry.identity,
                name: field.entry.name(),
                member_type: field.entry.member_type.as_ref(),
                optional: field.entry.decl.optional,
            });
        Some((table.ambiguous.as_deref(), fields))
    }

    pub(super) fn selected_declaration(
        &self,
        key: &CheckedNodeId,
    ) -> Option<(&str, &str, DeclarationForm)> {
        let declared = self.index.0.get(key)?;
        let model = self.models.get(declared.selection)?;
        Some((&model.identity, &declared.node, declared.form))
    }

    /// Selected documents retained by this already admitted package.
    pub(super) fn selected_models(&self) -> &[DomainModel] {
        &self.models
    }

    /// Look up one admitted selected document without visiting other selections.
    pub(super) fn selected_model(&self, identity: &str) -> Option<&DomainModel> {
        self.models.get(*self.by_identity.get(identity)?)
    }

    /// Effective operation classes of the selected model document named by its identity.
    pub(super) fn operation_classes(
        &self,
        identity: &str,
        meter: &mut WorkMeter,
    ) -> Result<Vec<ModelOperationClass>, ValidationFailure> {
        let mut classes = Vec::new();
        for (id, declared) in &self.index.0 {
            let Some(model) = self.models.get(declared.selection) else {
                continue;
            };
            if model.identity.as_ref() != identity
                || !matches!(
                    declared.form,
                    DeclarationForm::ObjectType
                        | DeclarationForm::StateMachine
                        | DeclarationForm::Process
                        | DeclarationForm::SystemsInterface
                )
            {
                continue;
            }
            let mut budget = Budget::new(meter, model.root, model.bytes);
            for operation in model.effective_operation_names(&declared.node, &mut budget)? {
                let binder_types =
                    model.model_operation_binder_types(&declared.node, &operation, &mut budget)?;
                classes.push(ModelOperationClass {
                    object: member_name(&declared.node).into(),
                    operation,
                    node: id.clone(),
                    binder_types,
                });
            }
        }
        Ok(classes)
    }

    /// The selected variant's family, for checked projections after admission.
    pub(super) fn variant_family(&self, key: &CheckedNodeId) -> Option<&'static str> {
        let declared = self.index.0.get(key)?;
        if declared.form != DeclarationForm::VariantType {
            return None;
        }
        let cases = self
            .models
            .get(declared.selection)?
            .variant_types
            .get(&declared.node)?;
        Some(if cases.values().any(|case| case.payload.is_some()) {
            "union"
        } else {
            "enum"
        })
    }

    pub(super) fn variant_body(&self, key: &CheckedNodeId) -> Option<CheckedModelVariantBody<'_>> {
        let declared = self.index.0.get(key)?;
        if declared.form != DeclarationForm::VariantType {
            return None;
        }
        let model = self.models.get(declared.selection)?;
        Some(CheckedModelVariantBody {
            document: &model.identity,
            cases: model.variant_types.get(&declared.node)?,
        })
    }

    pub(super) fn tuple_position_type(
        &self,
        key: &CheckedNodeId,
        position: usize,
    ) -> Option<CheckedMemberType> {
        let declared = self.index.0.get(key)?;
        if declared.form != DeclarationForm::TupleValueType {
            return None;
        }
        let model = self.models.get(declared.selection)?;
        let type_ref = model.tuple_types.get(&declared.node)?.get(position)?;
        model
            .element_type(type_ref)
            .as_ref()
            .map(CheckedMemberType::from)
    }

    pub(super) fn variant_payload_member_type(
        &self,
        key: &CheckedNodeId,
        case: &str,
    ) -> Option<CheckedMemberType> {
        let declared = self.index.0.get(key)?;
        if declared.form != DeclarationForm::VariantType {
            return None;
        }
        let model = self.models.get(declared.selection)?;
        let type_ref = model
            .variant_types
            .get(&declared.node)?
            .get(case)?
            .payload
            .as_ref()?;
        model
            .element_type(type_ref)
            .as_ref()
            .map(CheckedMemberType::from)
    }

    /// The admitted selected variant case for this declaration and exact case label.
    pub(super) fn selected_variant_case(
        &self,
        key: &CheckedNodeId,
        case: &str,
    ) -> Option<&SelectedVariantCase> {
        let declared = self.index.0.get(key)?;
        if declared.form != DeclarationForm::VariantType {
            return None;
        }
        self.models
            .get(declared.selection)?
            .variant_types
            .get(&declared.node)?
            .get(case)
    }

    pub(super) fn refinement_object_fields(
        &self,
        key: &CheckedNodeId,
    ) -> Option<BTreeSet<Box<str>>> {
        let declared = self.index.0.get(key)?;
        if declared.form != DeclarationForm::ObjectType {
            return None;
        }
        self.models
            .get(declared.selection)?
            .fields
            .exposed_names(&declared.node)
    }
    pub(super) fn new(index: DeclarationIndex, models: Vec<DomainModel>) -> Self {
        let by_identity = models
            .iter()
            .enumerate()
            .map(|(selection, model)| (model.identity.clone(), selection))
            .collect();
        let mut object_declarations: BTreeMap<usize, BTreeMap<Box<str>, DeclaringNode>> =
            BTreeMap::new();
        let mut original_declarations = BTreeMap::new();
        for (key, declared) in &index.0 {
            original_declarations
                .entry((declared.selection, declared.node.clone()))
                .and_modify(|entry| *entry = None)
                .or_insert_with(|| Some(key.clone()));
            if declared.form == DeclarationForm::ObjectType {
                object_declarations
                    .entry(declared.selection)
                    .or_default()
                    .entry(declared.node.clone())
                    .and_modify(|entry| *entry = DeclaringNode::Ambiguous)
                    .or_insert_with(|| DeclaringNode::One(key.clone()));
            }
        }
        let mut operation_types = BTreeMap::new();
        for (selection, model) in models.iter().enumerate() {
            for (object, declared) in &model.object_types {
                for operation in &declared.operations {
                    operation_types.insert(
                        (
                            selection,
                            object.clone(),
                            Box::from(member_name(&operation.identity)),
                        ),
                        OperationTypes {
                            has_result: operation.result.is_some(),
                            parameters: operation
                                .parameters
                                .iter()
                                .map(|slot| {
                                    model.slot_type(slot).as_ref().map(CheckedMemberType::from)
                                })
                                .collect(),
                        },
                    );
                }
            }
        }
        let mut relationships = BTreeMap::new();
        for (selection, model) in models.iter().enumerate() {
            for (node, declaration) in &model.relationships {
                relationships.insert(
                    (selection, node.clone()),
                    RetainedRelationship {
                        declaration: declaration.clone(),
                    },
                );
            }
        }
        Self {
            index,
            original_declarations,
            by_identity,
            object_declarations,
            models,
            operation_types,
            relationships,
        }
    }

    pub(super) fn record_binder_types(&self, key: &CheckedNodeId) -> Option<RecordBinderTypes> {
        let declared = self.index.0.get(key)?;
        if declared.form != DeclarationForm::RecordValueType {
            return None;
        }
        self.models
            .get(declared.selection)?
            .record_binder_types(&declared.node)
    }

    pub(super) fn is_record_type(&self, key: &CheckedNodeId) -> bool {
        self.index
            .0
            .get(key)
            .is_some_and(|declared| declared.form == DeclarationForm::RecordValueType)
    }

    pub(super) fn operation_binder_types(
        &self,
        context: &CheckedNodeId,
        name: &str,
        meter: &mut WorkMeter,
    ) -> Result<Option<Vec<CheckedNodeId>>, ValidationFailure> {
        let Some(declared) = self.index.0.get(context) else {
            return Ok(None);
        };
        let Some(model) = self.models.get(declared.selection) else {
            return Ok(None);
        };
        let mut budget = Budget::new(meter, model.root, model.bytes);
        model.effective_operation_binder_types(&declared.node, name, &mut budget)
    }

    /// The retained relationship the model declaration node `node` is.
    pub(super) fn relationship(
        &self,
        node: &super::CheckedSemanticNodeV2,
        term: &CheckedTerm,
    ) -> Option<&RetainedRelationship> {
        let declared = self.index.0.get(&node.node_id)?;
        check_declaration_node(declared.form, node, empty_declaration_term(term)).ok()?;
        // The map holds relationship identities only, so an indexed
        // declaration of another form finds nothing.
        self.relationships
            .get(&(declared.selection, declared.node.clone()))
    }

    /// The selected own operation whose signature the FR-341 clause join checked.
    pub(super) fn operation_types(
        &self,
        context: &super::CheckedSemanticNodeV2,
        term: &CheckedTerm,
        name: &str,
    ) -> Option<&OperationTypes> {
        let declared = self.index.0.get(&context.node_id)?;
        check_declaration_node(declared.form, context, empty_declaration_term(term)).ok()?;
        if !matches!(
            declared.form,
            DeclarationForm::ObjectType | DeclarationForm::StateMachine | DeclarationForm::Process
        ) {
            return None;
        }
        self.operation_types
            .get(&(declared.selection, declared.node.clone(), Box::from(name)))
    }

    /// The table of the object type whose model declaration node `node` is: the
    /// FR-322 step 2 rule of `ModelOwners::recover` over the retained index.
    fn object_table(
        &self,
        node: &super::CheckedSemanticNodeV2,
        term: &CheckedTerm,
    ) -> Option<&FieldTable> {
        let declared = self.index.0.get(&node.node_id)?;
        check_declaration_node(declared.form, node, empty_declaration_term(term)).ok()?;
        if declared.form != DeclarationForm::ObjectType {
            return None;
        }
        self.models
            .get(declared.selection)?
            .fields
            .of(&declared.node)
    }

    /// The selected member type for one unambiguous admitted field read.
    pub(super) fn field_type(
        &self,
        node: &super::CheckedSemanticNodeV2,
        term: &CheckedTerm,
        name: &str,
    ) -> Option<Option<CheckedMemberType>> {
        let mut matching = self
            .object_table(node, term)?
            .exposed
            .iter()
            .filter(|entry| entry.name() == name);
        let entry = matching.next()?;
        if matching.next().is_some() {
            return None;
        }
        Some(entry.member_type.as_ref().map(CheckedMemberType::from))
    }
}

fn empty_declaration_term(term: &CheckedTerm) -> bool {
    matches!(term, CheckedTerm::Aggregate { members } if members.is_empty())
}

/// A field's member type, as FR-322 step 4 derives it from the selected
/// document; mirrors the reader's own member type variant for variant.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum CheckedMemberType {
    /// `Boolean`.
    Boolean,
    /// `Integer`.
    Integer,
    /// Native IEEE binary32 with exact rounding.
    Float32,
    /// Native IEEE binary64 with exact rounding.
    Float64,
    /// Native String, distinct from profiled Text.
    String,
    /// Native UUID.
    Uuid,
    /// Native Timestamp.
    Timestamp,
    /// Native unbounded octet sequence.
    Bytes,
    /// Native opaque JSON value.
    JsonObject,
    /// The native timed-model clock marker.
    Clock,
    /// `Int[lower, upper]`.
    IntRange {
        /// The inclusive lower bound.
        lower: i128,
        /// The inclusive upper bound.
        upper: i128,
    },
    /// `Rational[numerator_min, numerator_max; denominator_min, denominator_max]`.
    RationalRange {
        /// Inclusive minimum numerator.
        numerator_min: i128,
        /// Inclusive maximum numerator.
        numerator_max: i128,
        /// Inclusive minimum denominator.
        denominator_min: i128,
        /// Inclusive maximum denominator.
        denominator_max: i128,
    },
    /// A bounded Rational magnitude carried by an exact unit.
    CarriedRational {
        /// The checked unit node.
        unit: CheckedNodeId,
        /// The rational carrier's exact bounds.
        carrier: Box<CheckedMemberType>,
    },
    /// `Text[min, max; profile]` from a selected value type.
    TextBounds {
        /// Inclusive minimum text length.
        lower: u64,
        /// Inclusive maximum text length.
        upper: u64,
        /// QSpec text profile spelling.
        profile: Box<str>,
    },
    /// `Reference<O>`: the model declaration node of the referenced object type.
    Reference(CheckedNodeId),
    /// A record value with no object identity: its model declaration node.
    Record(CheckedNodeId),
    /// A selected model tuple declaration with ordered positions.
    Tuple(CheckedNodeId),
    /// A selected model variant declaration; selected payloads determine its family.
    Variant(CheckedNodeId),
    /// `Quantity<u>`: the checked unit node of `u`.
    Quantity(CheckedNodeId),
    /// `Option<X>`: the element type.
    Option(Box<CheckedMemberType>),
    /// `K<E>`, or `K<E>[lower, upper]` when bounded.
    Collection {
        /// The collection form.
        kind: CheckedCollectionKind,
        /// The element type.
        element: Box<CheckedMemberType>,
        /// The inclusive `(lower, upper)` bounds, when bounded.
        bounds: Option<(u64, u64)>,
    },
}

/// The form of a [`CheckedMemberType::Collection`].
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum CheckedCollectionKind {
    /// Unordered, without repeats.
    Set,
    /// Unordered, with repeats.
    Bag,
    /// Ordered, with repeats.
    Sequence,
    /// Ordered, without repeats.
    OrderedSet,
}

impl From<CollectionKind> for CheckedCollectionKind {
    fn from(kind: CollectionKind) -> Self {
        match kind {
            CollectionKind::Set => Self::Set,
            CollectionKind::Bag => Self::Bag,
            CollectionKind::Sequence => Self::Sequence,
            CollectionKind::OrderedSet => Self::OrderedSet,
        }
    }
}

impl From<&MemberType> for CheckedMemberType {
    fn from(member_type: &MemberType) -> Self {
        match member_type {
            MemberType::Boolean => Self::Boolean,
            MemberType::Integer => Self::Integer,
            MemberType::Float32 => Self::Float32,
            MemberType::Float64 => Self::Float64,
            MemberType::String => Self::String,
            MemberType::Uuid => Self::Uuid,
            MemberType::Timestamp => Self::Timestamp,
            MemberType::Bytes => Self::Bytes,
            MemberType::JsonObject => Self::JsonObject,
            MemberType::Clock => Self::Clock,
            MemberType::IntRange(range) => Self::IntRange {
                lower: range.lower,
                upper: range.upper,
            },
            MemberType::RationalRange(bounds) => Self::RationalRange {
                numerator_min: bounds.numerator_min,
                numerator_max: bounds.numerator_max,
                denominator_min: bounds.denominator_min,
                denominator_max: bounds.denominator_max,
            },
            MemberType::CarriedRational(bounds, unit) => Self::CarriedRational {
                unit: unit.clone(),
                carrier: Box::new(Self::from(&MemberType::RationalRange(*bounds))),
            },
            MemberType::TextBounds(bounds) => Self::TextBounds {
                lower: bounds.lower,
                upper: bounds.upper,
                profile: bounds.profile.clone(),
            },
            MemberType::Reference(target) => Self::Reference(target.clone()),
            MemberType::Record(target) => Self::Record(target.clone()),
            MemberType::Tuple(target) => Self::Tuple(target.clone()),
            MemberType::Variant(target) => Self::Variant(target.clone()),
            MemberType::Quantity(target) => Self::Quantity(target.clone()),
            MemberType::Option(element) => Self::Option(Box::new(Self::from(element.as_ref()))),
            MemberType::Collection {
                kind,
                element,
                bounds,
            } => Self::Collection {
                kind: (*kind).into(),
                element: Box::new(Self::from(element.as_ref())),
                bounds: *bounds,
            },
        }
    }
}

/// One field of a model object type. Its members are private, so only the
/// reader builds one (FR-038-AC-142):
///
/// ```compile_fail,E0451
/// let id = quire_contract_model::CheckedNodeId { package: "p".into(), path: vec![], ordinal: 0 };
/// let _ = quire_contract_model::CheckedModelField {
///     declaring_node_id: id,
///     name: "x".into(),
///     member_type: None,
/// };
/// ```
///
/// and no single member is public, which a literal cannot show, since it also
/// fails on the members left private:
///
/// ```compile_fail,E0616
/// fn read(field: &quire_contract_model::CheckedModelField) {
///     let _ = &field.name;
/// }
/// ```
///
/// ```compile_fail,E0616
/// fn read(field: &quire_contract_model::CheckedModelField) {
///     let _ = &field.member_type;
/// }
/// ```
///
/// ```compile_fail,E0616
/// fn read(field: &quire_contract_model::CheckedModelField) {
///     let _ = &field.declaring_node_id;
/// }
/// ```
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CheckedModelField {
    declaring_node_id: CheckedNodeId,
    name: Box<str>,
    member_type: Option<CheckedMemberType>,
}

impl CheckedModelField {
    /// The actual declaring object's admitted structural node identity.
    pub fn declaring_node_id(&self) -> &CheckedNodeId {
        &self.declaring_node_id
    }

    /// The field's declared name.
    pub fn name(&self) -> &str {
        &self.name
    }

    /// The field's member type, `None` where FR-322 step 4's tables give the
    /// field no type (a rational, decimal, float, text or
    /// systems interface, a value type not bound as `Int[lo, hi]` or bound past
    /// `i128`, another FR-208 meaning, or a collection with a positive lower
    /// bound and no upper bound).
    pub fn member_type(&self) -> Option<&CheckedMemberType> {
        self.member_type.as_ref()
    }
}

/// The effective fields of a model object type, in effective declaration order.
/// Its member is private, so only the reader builds one (FR-038-AC-142):
///
/// ```compile_fail,E0451
/// let _ = quire_contract_model::CheckedModelObjectFields { fields: Vec::new() };
/// ```
///
/// ```compile_fail,E0616
/// fn read(fields: &quire_contract_model::CheckedModelObjectFields) {
///     let _ = &fields.fields;
/// }
/// ```
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CheckedModelObjectFields {
    fields: Vec<CheckedModelField>,
}

impl CheckedModelObjectFields {
    /// The fields in inherited source order, followed by own declaration order.
    pub fn fields(&self) -> &[CheckedModelField] {
        &self.fields
    }

    /// The field named `name`; `None` for an absent field, which is not an error.
    pub fn field(&self, name: &str) -> Option<&CheckedModelField> {
        self.fields.iter().find(|field| field.name.as_ref() == name)
    }
}

/// One effective field of an admitted selected model record.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CheckedModelRecordField {
    identity: Box<str>,
    name: Box<str>,
    optional: bool,
    member_type: Option<CheckedMemberType>,
}

impl CheckedModelRecordField {
    /// The selected IR identity of this field.
    pub fn identity(&self) -> &str {
        &self.identity
    }

    /// The field's declared name.
    pub fn name(&self) -> &str {
        &self.name
    }

    /// Whether the field may be absent, independently of an `Option` member type.
    pub fn optional(&self) -> bool {
        self.optional
    }

    /// The member type derived at admission, if the selected type has one.
    pub fn member_type(&self) -> Option<&CheckedMemberType> {
        self.member_type.as_ref()
    }
}

/// The complete effective fields of a selected model record, in declaration order.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CheckedModelRecordFields {
    fields: Vec<CheckedModelRecordField>,
}

impl CheckedModelRecordFields {
    /// Every effective field, including inherited and unread fields.
    pub fn fields(&self) -> &[CheckedModelRecordField] {
        &self.fields
    }

    /// The field named `name`, or `None` when no effective field has that name.
    pub fn field(&self, name: &str) -> Option<&CheckedModelRecordField> {
        self.fields.iter().find(|field| field.name() == name)
    }
}

/// Why a selected model record's effective field set cannot be returned.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum CheckedModelRecordFieldsError {
    /// The admitted graph has no node with the requested identity.
    UnknownNode,
    /// The node is not a selected model record declaration.
    NotModelRecordType,
    /// Two effective fields share this retained ambiguous name.
    AmbiguousField(Box<str>),
}

impl fmt::Display for CheckedModelRecordFieldsError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::UnknownNode => formatter.write_str("the graph holds no node with that identity"),
            Self::NotModelRecordType => {
                formatter.write_str("the node is not a selected model record declaration")
            }
            Self::AmbiguousField(name) => {
                write!(
                    formatter,
                    "two effective record fields share the name `{name}`"
                )
            }
        }
    }
}

impl std::error::Error for CheckedModelRecordFieldsError {}

/// Why [`CheckedPackageV2::model_object_fields`] returned no fields.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum CheckedModelFieldsError {
    /// The graph holds no node with the given identity.
    UnknownNode,
    /// The node is not a model object type's model declaration node.
    NotModelObjectType,
    /// Two exposed fields of the object type share a name (the smallest such
    /// name is carried), so its field set has no defined content.
    AmbiguousField(Box<str>),
    /// An effective field's selected declaring owner has no graph declaration.
    MissingDeclaringNode {
        /// The selected IR identity of the first affected effective field.
        field_identity: Box<str>,
    },
    /// An effective field's selected declaring owner has multiple graph declarations.
    AmbiguousDeclaringNode {
        /// The selected IR identity of the first affected effective field.
        field_identity: Box<str>,
    },
}

impl fmt::Display for CheckedModelFieldsError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::UnknownNode => formatter.write_str("the graph holds no node with that identity"),
            Self::NotModelObjectType => {
                formatter.write_str("the node is not a model object type's declaration node")
            }
            Self::AmbiguousField(name) => {
                write!(formatter, "two exposed fields share the name `{name}`")
            }
            Self::MissingDeclaringNode { field_identity } => {
                write!(
                    formatter,
                    "field `{field_identity}` has no declaring graph node"
                )
            }
            Self::AmbiguousDeclaringNode { field_identity } => {
                write!(
                    formatter,
                    "field `{field_identity}` has multiple declaring graph nodes"
                )
            }
        }
    }
}

impl std::error::Error for CheckedModelFieldsError {}

impl CheckedPackageV2 {
    /// The admitted selected type at one tuple position, or `None` for an
    /// unknown declaration or position or an unmapped type.
    pub fn model_tuple_position_type(
        &self,
        node: &CheckedNodeId,
        position: usize,
    ) -> Option<CheckedMemberType> {
        self.models.tuple_position_type(node, position)
    }

    /// The admitted selected payload type of one variant case, or `None` for
    /// an unknown declaration, nullary case, or unmapped type.
    pub fn model_variant_payload_member_type(
        &self,
        node: &CheckedNodeId,
        case: &str,
    ) -> Option<CheckedMemberType> {
        self.models.variant_payload_member_type(node, case)
    }

    /// The complete retained selected-variant body for an admitted declaration
    /// node, borrowed from this package. A non-variant or unselected node has
    /// no body. Cases appear in ascending UTF-8 name order.
    pub fn model_variant_body(&self, node: &CheckedNodeId) -> Option<CheckedModelVariantBody<'_>> {
        self.models.variant_body(node)
    }

    /// The retained effective fields of a selected model record. This read
    /// does not require an Exact identity witness for each member type.
    ///
    /// # Errors
    ///
    /// Returns [`CheckedModelRecordFieldsError::UnknownNode`] for a missing
    /// graph node, [`CheckedModelRecordFieldsError::NotModelRecordType`] for a
    /// non-record node, or [`CheckedModelRecordFieldsError::AmbiguousField`]
    /// when effective fields share a name.
    pub fn model_record_fields(
        &self,
        record: &CheckedNodeId,
    ) -> Result<CheckedModelRecordFields, CheckedModelRecordFieldsError> {
        self.node(record)
            .ok_or(CheckedModelRecordFieldsError::UnknownNode)?;
        let (ambiguous, selected) = self
            .models
            .selected_record_fields(record)
            .ok_or(CheckedModelRecordFieldsError::NotModelRecordType)?;
        if let Some(name) = ambiguous {
            return Err(CheckedModelRecordFieldsError::AmbiguousField(name.into()));
        }
        let fields = selected
            .map(|field| CheckedModelRecordField {
                identity: field.identity.into(),
                name: field.name.into(),
                optional: field.optional,
                member_type: field.member_type.map(CheckedMemberType::from),
            })
            .collect();
        Ok(CheckedModelRecordFields { fields })
    }

    /// The effective fields of the model object type whose model declaration
    /// node is `node`, in effective declaration order, each with the member type
    /// the reader derived from the selected domain package document at
    /// admission. Charges no work and reads no node body other than `node`'s
    /// own fixed members.
    ///
    /// # Errors
    ///
    /// [`CheckedModelFieldsError::UnknownNode`] when the graph holds no such
    /// node; [`CheckedModelFieldsError::NotModelObjectType`] when it is not an
    /// object type's model declaration node;
    /// [`CheckedModelFieldsError::AmbiguousField`] when two exposed fields of
    /// the object type share a name. The first field in effective declaration
    /// order whose owner is absent or ambiguous returns
    /// [`CheckedModelFieldsError::MissingDeclaringNode`] or
    /// [`CheckedModelFieldsError::AmbiguousDeclaringNode`], respectively.
    pub fn model_object_fields(
        &self,
        node: &CheckedNodeId,
    ) -> Result<CheckedModelObjectFields, CheckedModelFieldsError> {
        let found = self
            .node(node)
            .ok_or(CheckedModelFieldsError::UnknownNode)?;
        let table = self
            .models
            .object_table(
                found,
                self.term(node)
                    .ok_or(CheckedModelFieldsError::NotModelObjectType)?,
            )
            .ok_or(CheckedModelFieldsError::NotModelObjectType)?;
        if let Some(name) = &table.ambiguous {
            return Err(CheckedModelFieldsError::AmbiguousField(name.clone()));
        }
        let selection = self
            .models
            .index
            .0
            .get(node)
            .ok_or(CheckedModelFieldsError::NotModelObjectType)?
            .selection;
        let fields = table
            .ordered_exposed
            .iter()
            .map(|slot| {
                let entry = &slot.entry;
                let declaring_node_id = match self
                    .models
                    .object_declarations
                    .get(&selection)
                    .and_then(|owners| owners.get(entry.declaring_owner.as_ref()))
                {
                    Some(DeclaringNode::One(id)) => id.clone(),
                    Some(DeclaringNode::Ambiguous) => {
                        return Err(CheckedModelFieldsError::AmbiguousDeclaringNode {
                            field_identity: entry.identity.as_ref().into(),
                        })
                    }
                    None => {
                        return Err(CheckedModelFieldsError::MissingDeclaringNode {
                            field_identity: entry.identity.as_ref().into(),
                        })
                    }
                };
                Ok(CheckedModelField {
                    declaring_node_id,
                    name: entry.name().into(),
                    member_type: entry.member_type.as_ref().map(CheckedMemberType::from),
                })
            })
            .collect::<Result<Vec<_>, CheckedModelFieldsError>>()?;
        Ok(CheckedModelObjectFields { fields })
    }
}

/// Test probes: how often a table was built.
#[cfg(test)]
pub(super) mod probe {
    use std::cell::Cell;

    thread_local! {
        static BUILT: Cell<u64> = const { Cell::new(0) };
        static NODE_VISITS: Cell<u64> = const { Cell::new(0) };
    }

    pub(in crate::checked_package::v2) fn node_visited() {
        NODE_VISITS.with(|visits| visits.set(visits.get().saturating_add(1)));
    }

    pub(super) fn reset_node_visits() {
        NODE_VISITS.with(|visits| visits.set(0));
    }

    pub(super) fn node_visits() -> u64 {
        NODE_VISITS.with(Cell::get)
    }

    pub(super) fn built() {
        BUILT.with(|built| built.set(built.get().saturating_add(1)));
    }

    /// The components built on this thread so far.
    pub(in crate::checked_package::v2) fn components_built() -> u64 {
        BUILT.with(Cell::get)
    }
}

#[cfg(test)]
mod tests;
