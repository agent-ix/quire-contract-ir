//! FR-038 "Typed accessor for a model object type's fields" (IR-628): the
//! field table the reader builds once per object type at admission, the
//! tables, declaration index and relationship records an admitted
//! [`CheckedPackageV2`] retains, and the accessor
//! [`CheckedPackageV2::model_object_fields`] that reads the field tables (the
//! relationship records are read by `CheckedPackageV2::relationship_end`).
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
    check_declaration_node, member_name, units, Budget, CollectionKind, DeclarationForm,
    DomainModel, FieldBearingTypeDecl, FieldDecl, MemberType, ModelOperationClass,
    RelationshipDecl,
};
use super::{CheckedNodeId, CheckedPackageV2, CheckedTerm, WorkMeter};
use crate::checked_package::common::{ValidationFailure, NODE_DOMAIN};
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
    /// `DomainModel::field_type` of the declaration.
    member_type: Option<MemberType>,
}

/// One effective field and the identities whose inherited position it holds.
#[derive(Clone, Debug, Eq, PartialEq)]
struct OrderedField {
    entry: Arc<FieldEntry>,
    origins: BTreeSet<Arc<str>>,
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
    pub(super) fn binder_types(&self, bytes: u64) -> Option<RecordBinderTypes> {
        if self.ambiguous.is_some() {
            return Some(RecordBinderTypes::Ambiguous);
        }
        let types = self
            .ordered_exposed
            .iter()
            .map(|entry| {
                Some(CheckedNodeId {
                    domain: NODE_DOMAIN.into(),
                    digest: entry
                        .entry
                        .member_type
                        .as_ref()?
                        .node_key(bytes)
                        .ok()?
                        .into(),
                })
            })
            .collect::<Option<Vec<_>>>()?;
        Some(RecordBinderTypes::Known(types))
    }
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
    by_type: BTreeMap<Box<str>, usize>,
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
    for (row, model) in models.iter_mut().enumerate() {
        let mut budget = Budget::new(meter, row, model.bytes);
        let tables = tables_of(model, &mut budget)?;
        model.fields = tables;
    }
    Ok(())
}

/// A strongly connected component of the supertype graph.
struct Component<'m> {
    members: Vec<(&'m str, &'m FieldBearingTypeDecl)>,
    /// More than one member, or a member that extends itself.
    cyclic: bool,
}

fn tables_of(
    model: &DomainModel,
    budget: &mut Budget<'_>,
) -> Result<FieldTables, ValidationFailure> {
    let mut tables = FieldTables::default();
    for component in components(model) {
        let table = build_component(model, &component, &tables, budget)?;
        let index = tables.components.len();
        tables.components.push(table);
        for (node, _) in &component.members {
            tables.by_type.insert(Box::from(*node), index);
        }
    }
    Ok(tables)
}

/// The strongly connected components of the supertype graph of `model`'s
/// objects and records, a component after every component of its supertypes
/// (Tarjan's algorithm over an explicit stack).
fn components(model: &DomainModel) -> Vec<Component<'_>> {
    let mut search = Tarjan {
        model,
        visits: BTreeMap::new(),
        stack: Vec::new(),
        found: Vec::new(),
    };
    for root in model
        .object_types
        .keys()
        .chain(model.record_types.keys())
        .map(AsRef::as_ref)
    {
        if !search.visits.contains_key(root) {
            search.visit(root);
        }
    }
    search.found
}

/// Tarjan's search state: a node's discovery index, its lowest reachable
/// index, and whether it is still on the component stack.
struct Visit {
    index: usize,
    low: usize,
    on_stack: bool,
}

/// An open node with the supertypes it has still to follow.
type Open<'m> = Vec<(&'m str, std::vec::IntoIter<&'m str>)>;

struct Tarjan<'m> {
    model: &'m DomainModel,
    visits: BTreeMap<&'m str, Visit>,
    stack: Vec<&'m str>,
    found: Vec<Component<'m>>,
}

impl<'m> Tarjan<'m> {
    /// The declared supertypes of `node` that the document declares as
    /// field-bearing types; any other has no table and no members.
    fn supertypes(&self, node: &str) -> std::vec::IntoIter<&'m str> {
        let model = self.model;
        model
            .field_owner(node)
            .map(|declared| {
                declared
                    .supertypes
                    .iter()
                    .map(AsRef::as_ref)
                    .filter(|supertype| model.field_owner(supertype).is_some())
                    .collect::<Vec<_>>()
            })
            .unwrap_or_default()
            .into_iter()
    }

    fn enter(&mut self, node: &'m str) {
        let index = self.visits.len();
        self.visits.insert(
            node,
            Visit {
                index,
                low: index,
                on_stack: true,
            },
        );
        self.stack.push(node);
    }

    fn lower(&mut self, node: &str, to: usize) {
        if let Some(visit) = self.visits.get_mut(node) {
            visit.low = visit.low.min(to);
        }
    }

    /// Searches from `root` over an explicit stack, so a chain of any length
    /// needs no call stack.
    fn visit(&mut self, root: &'m str) {
        self.enter(root);
        let mut open: Open<'m> = vec![(root, self.supertypes(root))];
        while let Some((node, pending)) = open.last_mut() {
            let node = *node;
            match pending.next() {
                Some(next) => self.follow(node, next, &mut open),
                None => {
                    open.pop();
                    self.leave(node, open.last().map(|(parent, _)| *parent));
                }
            }
        }
    }

    fn follow(&mut self, node: &str, next: &'m str, open: &mut Open<'m>) {
        match self.visits.get(next) {
            None => {
                self.enter(next);
                open.push((next, self.supertypes(next)));
            }
            Some(seen) if seen.on_stack => {
                let index = seen.index;
                self.lower(node, index);
            }
            Some(_) => {}
        }
    }

    /// Closes `node`; it roots a component when no node below it reached an
    /// earlier one.
    fn leave(&mut self, node: &'m str, parent: Option<&str>) {
        let Some(&Visit { index, low, .. }) = self.visits.get(node) else {
            return;
        };
        if let Some(parent) = parent {
            self.lower(parent, low);
        }
        if low != index {
            return;
        }
        let mut members = Vec::new();
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
        members.sort_by(|a, b| a.0.cmp(b.0));
        let cyclic = members.len() > 1
            || members.iter().any(|(member, declared)| {
                declared
                    .supertypes
                    .iter()
                    .any(|supertype| supertype.as_ref() == *member)
            });
        self.found.push(Component { members, cyclic });
    }
}

/// The table of one component, from its members' own fields and the tables of
/// the supertypes outside it, charged: one unit per own field and per
/// declared supertype edge of each member, and one per exposed or hidden entry
/// copied from the table of a supertype outside the component.
fn build_component(
    model: &DomainModel,
    component: &Component<'_>,
    built: &FieldTables,
    budget: &mut Budget<'_>,
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
    budget.charge(units(own_units))?;
    let mut inherited: Vec<Arc<FieldEntry>> = Vec::new();
    let mut ordered_slots = Vec::new();
    let mut hidden: BTreeSet<Arc<str>> = BTreeSet::new();
    for (_, declared) in &component.members {
        for supertype in &declared.supertypes {
            // A supertype of the component itself has no table yet.
            let Some(table) = built.of(supertype) else {
                continue;
            };
            budget.charge(units(table.entries()))?;
            inherited.extend(
                table
                    .ordered_exposed
                    .iter()
                    .map(|slot| Arc::clone(&slot.entry)),
            );
            ordered_slots.extend(table.ordered_exposed.iter().cloned());
            hidden.extend(table.hidden.iter().cloned());
        }
    }
    // A diamond reaches one entry twice; a field one supertype exposes and
    // another hides is hidden.
    inherited.sort_by(|a, b| a.identity.cmp(&b.identity));
    inherited.dedup_by(|a, b| a.identity == b.identity);
    inherited.retain(|entry| !hidden.contains(&entry.identity));
    // A later declaration of one identity replaces an earlier one, as
    // `resolve`'s member map does.
    let mut own: BTreeMap<Arc<str>, Arc<FieldEntry>> = BTreeMap::new();
    let mut own_order = Vec::new();
    for (_, declared) in &component.members {
        for decl in &declared.fields {
            let identity: Arc<str> = Arc::from(decl.identity.as_ref());
            if !own.contains_key(&identity) {
                own_order.push(Arc::clone(&identity));
            }
            own.insert(
                Arc::clone(&identity),
                Arc::new(FieldEntry {
                    identity,
                    member_type: model.field_type(decl),
                    decl: decl.clone(),
                }),
            );
        }
    }
    // FR-322 step 3: a redefinition hides its target; a redefiner of a target
    // that a redefiner in a more derived owner also redefines is hidden, and
    // every owner here is more derived than every owner copied. A member of a
    // cycle is its own ancestor, so each of its redefiners hides itself.
    let mut by_target: BTreeMap<&str, Vec<&Arc<str>>> = BTreeMap::new();
    for entry in &inherited {
        if let Some(target) = entry.decl.redefines.as_deref() {
            by_target.entry(target).or_default().push(&entry.identity);
        }
    }
    for entry in own.values() {
        let Some(target) = entry.decl.redefines.as_deref() else {
            continue;
        };
        hidden.insert(Arc::from(target));
        if component.cyclic {
            hidden.insert(Arc::clone(&entry.identity));
        }
        hidden.extend(
            by_target
                .get(target)
                .into_iter()
                .flatten()
                .map(|id| Arc::clone(*id)),
        );
    }
    drop(by_target);
    for identity in own_order {
        let Some(entry) = own.get(&identity) else {
            continue;
        };
        let mut origins = BTreeSet::from([Arc::clone(&entry.identity)]);
        if let Some(target) = entry.decl.redefines.as_deref() {
            origins.insert(Arc::from(target));
            for slot in &ordered_slots {
                if slot.origins.contains(target) {
                    origins.extend(slot.origins.iter().cloned());
                }
            }
        }
        ordered_slots.push(OrderedField {
            entry: Arc::clone(entry),
            origins,
        });
    }
    let mut exposed: Vec<Arc<FieldEntry>> = inherited;
    exposed.extend(own.into_values());
    exposed.sort_by(|a, b| a.identity.cmp(&b.identity));
    exposed.dedup_by(|a, b| a.identity == b.identity);
    exposed.retain(|entry| !hidden.contains(&entry.identity));
    let mut first_origin = BTreeMap::new();
    let mut first_entry = BTreeMap::new();
    let mut origins_by_identity: BTreeMap<Arc<str>, BTreeSet<Arc<str>>> = BTreeMap::new();
    for (position, slot) in ordered_slots.iter().enumerate() {
        first_entry
            .entry(Arc::clone(&slot.entry.identity))
            .or_insert(position);
        origins_by_identity
            .entry(Arc::clone(&slot.entry.identity))
            .or_default()
            .extend(slot.origins.iter().cloned());
        for origin in &slot.origins {
            first_origin.entry(Arc::clone(origin)).or_insert(position);
        }
    }
    let mut ordered_exposed: Vec<OrderedField> = exposed
        .iter()
        .map(|entry| OrderedField {
            entry: Arc::clone(entry),
            origins: origins_by_identity
                .remove(&entry.identity)
                .unwrap_or_else(|| BTreeSet::from([Arc::clone(&entry.identity)])),
        })
        .collect();
    ordered_exposed.sort_by_key(|slot| {
        let inherited_position = slot
            .origins
            .iter()
            .filter_map(|origin| first_origin.get(origin))
            .copied()
            .min()
            .unwrap_or(usize::MAX);
        let declared_position = first_entry
            .get(&slot.entry.identity)
            .copied()
            .unwrap_or(usize::MAX);
        (inherited_position, declared_position)
    });
    let mut seen = BTreeSet::new();
    let ambiguous = exposed
        .iter()
        .map(|entry| entry.name())
        .filter(|name| !seen.insert(*name))
        .min()
        .map(Box::from);
    Ok(FieldTable {
        exposed,
        ordered_exposed,
        hidden: hidden.into_iter().collect(),
        ambiguous,
    })
}

/// What a selected declaration's model declaration node key names.
#[derive(Clone, Debug, Eq, PartialEq)]
pub(super) struct DeclaredNode {
    pub(super) form: DeclarationForm,
    /// The `lock.model_selections` row of the declaring document.
    pub(super) selection: usize,
    /// The declaration's IR node identity.
    pub(super) node: Box<str>,
}

/// An owned index from each selected declaration's model declaration node key
/// to its selection row, form and node identity.
#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub(super) struct DeclarationIndex(pub(super) BTreeMap<String, DeclaredNode>);

/// What an admitted [`CheckedPackageV2`] retains of its selected documents:
/// the declaration index, each document's field tables, the selected
/// operation parameter types needed for clause-context reads, and each
/// selected relationship's [`RetainedRelationship`] record.
/// Not part of the package's equality, which is the equality of its admitted
/// content.
#[derive(Clone, Debug, Default)]
pub(super) struct RetainedModels {
    index: DeclarationIndex,
    /// Admitted selected documents, including their field tables, for
    /// catalog binder resolution only when a refinement requests it.
    models: Vec<DomainModel>,
    operation_types: BTreeMap<(usize, Box<str>, Box<str>), OperationTypes>,
    relationships: BTreeMap<(usize, Box<str>), RetainedRelationship>,
}

/// The relationship record admission consults, kept for the accessor: both
/// ends (role, type reference, multiplicity), the direction, and the derived
/// model declaration node key of each end's type. `None` when no such
/// declaration is selected or the key cannot be derived, which admission
/// refuses for an end it reads.
#[derive(Clone, Debug)]
pub(super) struct RetainedRelationship {
    pub(super) declaration: RelationshipDecl,
    pub(super) source_key: Option<Box<str>>,
    pub(super) target_key: Option<Box<str>>,
}

impl RetainedRelationship {
    /// The derived declaration key of the destination end of a traversal.
    pub(super) fn destination_key(&self, forward: bool) -> Option<&str> {
        let key = if forward {
            &self.target_key
        } else {
            &self.source_key
        };
        key.as_deref()
    }
}

/// The selected model signature already checked against a clause by FR-341.
#[derive(Clone, Debug)]
pub(super) struct OperationTypes {
    pub(super) has_result: bool,
    pub(super) parameters: Vec<Option<CheckedMemberType>>,
}

impl RetainedModels {
    /// Effective operation classes of the selected model document named by its identity.
    pub(super) fn operation_classes(
        &self,
        identity: &str,
        meter: &mut WorkMeter,
    ) -> Result<Vec<ModelOperationClass>, ValidationFailure> {
        let mut classes = Vec::new();
        for (digest, declared) in &self.index.0 {
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
            let mut budget = Budget::new(meter, declared.selection, model.bytes);
            for operation in model.effective_operation_names(&declared.node, &mut budget)? {
                let binder_types =
                    model.model_operation_binder_types(&declared.node, &operation, &mut budget)?;
                classes.push(ModelOperationClass {
                    object: member_name(&declared.node).into(),
                    operation,
                    node: CheckedNodeId {
                        domain: NODE_DOMAIN.into(),
                        digest: digest.clone().into(),
                    },
                    binder_types,
                });
            }
        }
        Ok(classes)
    }

    pub(super) fn refinement_object_fields(
        &self,
        key: &CheckedNodeId,
    ) -> Option<BTreeSet<Box<str>>> {
        let declared = self.index.0.get(key.digest.as_ref())?;
        if declared.form != DeclarationForm::ObjectType {
            return None;
        }
        self.models
            .get(declared.selection)?
            .fields
            .exposed_names(&declared.node)
    }
    pub(super) fn new(index: DeclarationIndex, models: Vec<DomainModel>) -> Self {
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
            let key = |type_ref: &str| {
                model
                    .end_declaration_key(type_ref)
                    .and_then(Result::ok)
                    .map(Box::<str>::from)
            };
            for (node, declaration) in &model.relationships {
                relationships.insert(
                    (selection, node.clone()),
                    RetainedRelationship {
                        source_key: key(&declaration.source.type_ref),
                        target_key: key(&declaration.target.type_ref),
                        declaration: declaration.clone(),
                    },
                );
            }
        }
        Self {
            index,
            models,
            operation_types,
            relationships,
        }
    }

    pub(super) fn record_binder_types(&self, key: &CheckedNodeId) -> Option<RecordBinderTypes> {
        let declared = self.index.0.get(key.digest.as_ref())?;
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
            .get(key.digest.as_ref())
            .is_some_and(|declared| declared.form == DeclarationForm::RecordValueType)
    }

    pub(super) fn operation_binder_types(
        &self,
        context: &CheckedNodeId,
        name: &str,
        meter: &mut WorkMeter,
    ) -> Result<Option<Vec<CheckedNodeId>>, ValidationFailure> {
        let Some(declared) = self.index.0.get(context.digest.as_ref()) else {
            return Ok(None);
        };
        let Some(model) = self.models.get(declared.selection) else {
            return Ok(None);
        };
        let mut budget = Budget::new(meter, declared.selection, model.bytes);
        model.effective_operation_binder_types(&declared.node, name, &mut budget)
    }

    /// A retained index of one relationship, for tests that build a package
    /// by hand instead of admitting one.
    #[cfg(test)]
    pub(super) fn with_relationship(
        digest: &str,
        selection: usize,
        node: &str,
        relationship: RetainedRelationship,
    ) -> Self {
        let mut index = DeclarationIndex::default();
        index.0.insert(
            digest.to_owned(),
            DeclaredNode {
                form: DeclarationForm::Relationship,
                selection,
                node: node.into(),
            },
        );
        let mut retained = Self {
            index,
            ..Self::default()
        };
        retained
            .relationships
            .insert((selection, node.into()), relationship);
        retained
    }

    /// The retained relationship the model declaration node `node` is.
    pub(super) fn relationship(
        &self,
        node: &super::CheckedSemanticNodeV2,
        term: &CheckedTerm,
    ) -> Option<&RetainedRelationship> {
        let declared = self.index.0.get(node.node_id.digest.as_ref())?;
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
        let declared = self.index.0.get(context.node_id.digest.as_ref())?;
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
        let declared = self.index.0.get(node.node_id.digest.as_ref())?;
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
    /// Native String, distinct from profiled Text.
    String,
    /// Native UUID.
    Uuid,
    /// Native Timestamp.
    Timestamp,
    /// The native timed-model clock marker.
    Clock,
    /// `Int[lower, upper]`.
    IntRange {
        /// The inclusive lower bound.
        lower: i128,
        /// The inclusive upper bound.
        upper: i128,
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
    /// A selected model variant declaration with unordered enum cases.
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
            MemberType::String => Self::String,
            MemberType::Uuid => Self::Uuid,
            MemberType::Timestamp => Self::Timestamp,
            MemberType::Clock => Self::Clock,
            MemberType::IntRange(range) => Self::IntRange {
                lower: range.lower,
                upper: range.upper,
            },
            MemberType::TextBounds(bounds) => Self::TextBounds {
                lower: bounds.lower,
                upper: bounds.upper,
                profile: bounds.profile.clone(),
            },
            MemberType::Reference(target) => Self::Reference(CheckedNodeId {
                domain: NODE_DOMAIN.into(),
                digest: target.clone(),
            }),
            MemberType::Record(target) => Self::Record(CheckedNodeId {
                domain: NODE_DOMAIN.into(),
                digest: target.clone(),
            }),
            MemberType::Variant(target) => Self::Variant(CheckedNodeId {
                domain: NODE_DOMAIN.into(),
                digest: target.clone(),
            }),
            MemberType::Quantity(target) => Self::Quantity(CheckedNodeId {
                domain: NODE_DOMAIN.into(),
                digest: target.clone(),
            }),
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
/// let _ = quire_contract_model::CheckedModelField {
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
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CheckedModelField {
    name: Box<str>,
    member_type: Option<CheckedMemberType>,
}

impl CheckedModelField {
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

/// The effective fields of a model object type, in ascending order of name.
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
    /// The fields in ascending order of name (bytewise UTF-8), each name once.
    pub fn fields(&self) -> &[CheckedModelField] {
        &self.fields
    }

    /// The field named `name`; `None` for an absent field, which is not an error.
    pub fn field(&self, name: &str) -> Option<&CheckedModelField> {
        self.fields
            .binary_search_by(|field| field.name.as_ref().cmp(name))
            .ok()
            .and_then(|position| self.fields.get(position))
    }
}

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
        }
    }
}

impl std::error::Error for CheckedModelFieldsError {}

impl CheckedPackageV2 {
    /// The effective fields of the model object type whose model declaration
    /// node is `node`, in ascending order of name, each with the member type
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
    /// the object type share a name.
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
        let mut fields: Vec<CheckedModelField> = table
            .exposed
            .iter()
            .map(|entry| CheckedModelField {
                name: entry.name().into(),
                member_type: entry.member_type.as_ref().map(CheckedMemberType::from),
            })
            .collect();
        fields.sort_by(|a, b| a.name.cmp(&b.name));
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
