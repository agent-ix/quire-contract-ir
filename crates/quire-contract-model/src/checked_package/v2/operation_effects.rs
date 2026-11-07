// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (C) 2026 Agent-IX

//! Selected model operation effects carried by the checked package (TC-977).

use super::identity::is_identifier;
use super::model_fields::{RetainedModels, Selected};
use super::model_members::{
    member_name, DeclarationForm, DomainModel, FieldBearingTypeDecl, OperationDecl,
};
use super::native_join_budget::PackageReadStage;
use super::stage_storage::{self, Path, Refusal, Site};
use super::{
    CheckedFieldScope, CheckedFrameModification, CheckedNodeId, CheckedScopedDeletion, WorkMeter,
};
use crate::checked_package::common::ValidationFailure;
use crate::checked_package::shared::{
    CheckedPackageRefusalCause as Cause, CheckedPackageRefusalCode as Code,
};
use quire_canonical::{Encode, Error, FixedShape, Sink, Writer};
use serde::{Deserialize, Deserializer, Serialize};

/// The four complete grants of one selected model operation.
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct CheckedOperationEffect {
    /// Field and relationship grants in canonical order.
    pub modifies: Vec<CheckedFrameModification>,
    /// Object types whose instances the operation may create.
    pub creates: Vec<CheckedNodeId>,
    /// Object types whose instances the operation may delete.
    pub deletes: Vec<CheckedNodeId>,
    /// Receiver and parameter scoped deletion grants.
    pub deletes_scoped: Vec<CheckedScopedDeletion>,
}

/// The sole admitted source of a model operation's declared effect.
#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize, Deserialize, FixedShape)]
#[serde(rename_all = "snake_case")]
pub enum CheckedOperationEffectOrigin {
    /// The selected document's operation frame.
    ModelDocumentOperationFrame,
}

/// Source member identity and origin supplied with an effect row.
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize, FixedShape)]
#[serde(deny_unknown_fields)]
pub struct CheckedOperationEffectProvenance {
    /// Fixed selected-document origin.
    pub origin: CheckedOperationEffectOrigin,
    /// Original identity of the operation member.
    pub member: Box<str>,
}

/// A producer-constructible operation effect row; admission validates its claims.
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct CheckedOperationEffectRow {
    /// Selected document identity.
    pub document: Box<str>,
    /// Structural ID of the declaring model object, process or state machine.
    pub declaring: CheckedNodeId,
    /// Original operation name on that declaring node.
    pub operation: Box<str>,
    /// Complete frame lift, including intentionally empty effects.
    pub effect: CheckedOperationEffect,
    /// Selected member identity and fixed source origin.
    pub provenance: CheckedOperationEffectProvenance,
}

/// A borrowed row from a fully admitted checked package.
#[derive(Clone, Copy, Debug)]
pub struct CheckedOperationEffectRef<'a> {
    row: &'a CheckedOperationEffectRow,
}

impl<'a> CheckedOperationEffectRef<'a> {
    pub(super) fn admitted(row: &'a CheckedOperationEffectRow) -> Self {
        Self { row }
    }

    /// Selected document identity.
    pub fn document(self) -> &'a str {
        &self.row.document
    }

    /// Structural declaring node.
    pub fn declaring(self) -> &'a CheckedNodeId {
        &self.row.declaring
    }

    /// Original operation name.
    pub fn operation(self) -> &'a str {
        &self.row.operation
    }

    /// All four admitted frame grants in their canonical order.
    pub fn effect(self) -> &'a CheckedOperationEffect {
        &self.row.effect
    }

    /// Fixed origin and original member identity.
    pub fn provenance(self) -> &'a CheckedOperationEffectProvenance {
        &self.row.provenance
    }
}

// A defaulted optional envelope field must reject explicit JSON null. Deserializing
// the present value as Vec rather than Option preserves that distinction.
pub(super) fn present_rows<'de, D, T>(deserializer: D) -> Result<Option<T>, D::Error>
where
    D: Deserializer<'de>,
    T: Deserialize<'de>,
{
    T::deserialize(deserializer).map(Some)
}

impl Encode for CheckedOperationEffect {
    fn encode_into<S: Sink + ?Sized>(&self, writer: &mut Writer<'_, S>) -> Result<(), Error> {
        writer.begin_object()?;
        super::encode::elements(writer, "modifies", &self.modifies)?;
        super::encode::elements(writer, "creates", &self.creates)?;
        super::encode::elements(writer, "deletes", &self.deletes)?;
        super::encode::member(writer, "deletes_scoped", &self.deletes_scoped)?;
        writer.end_object()
    }
}

impl Encode for CheckedOperationEffectRow {
    fn encode_into<S: Sink + ?Sized>(&self, writer: &mut Writer<'_, S>) -> Result<(), Error> {
        writer.begin_object()?;
        super::encode::member(writer, "document", self.document.as_ref())?;
        super::encode::encoded(writer, "declaring", &self.declaring)?;
        super::encode::member(writer, "operation", self.operation.as_ref())?;
        super::encode::encoded(writer, "effect", &self.effect)?;
        super::encode::member(writer, "provenance", &self.provenance)?;
        writer.end_object()
    }
}

impl super::CheckedPackageV2 {
    /// One validated carrier row keyed by its selected source declaration.
    pub(super) fn selected_operation_effect(
        &self,
        document: &str,
        declaring: &CheckedNodeId,
        operation: &str,
    ) -> Option<&CheckedOperationEffectRow> {
        let rows = self.wire.operation_effects.as_ref()?;
        let index = rows
            .binary_search_by(|row| {
                (
                    row.document.as_ref(),
                    &row.declaring,
                    row.operation.as_ref(),
                )
                    .cmp(&(document, declaring, operation))
            })
            .ok()?;
        rows.get(index)
    }
}

const STAGE_FORM: &str = "operation_effects";

fn rows_at() -> Path<'static> {
    Path::root().key("operation_effects")
}

fn site(row: usize, at: Path<'static>) -> Site<'static> {
    Site::new(PackageReadStage::OperationEffects, row, STAGE_FORM, at)
}

/// A refusal about the value at `at`, in row `row`, its diagnostic charged
/// before it is built.
fn fail(
    meter: &mut WorkMeter,
    row: usize,
    code: Code,
    cause: Option<Cause>,
    at: Path<'static>,
) -> ValidationFailure {
    let refusal = Refusal::new(code, at);
    let refusal = match cause {
        Some(cause) => refusal.because(cause),
        None => refusal,
    };
    stage_storage::refuse(meter, &site(row, at), refusal)
}

fn malformed(meter: &mut WorkMeter, row: usize, at: Path<'static>) -> ValidationFailure {
    fail(meter, row, Code::MalformedWire, None, at)
}

fn refused(
    meter: &mut WorkMeter,
    row: usize,
    code: Code,
    cause: Cause,
    at: Path<'static>,
) -> ValidationFailure {
    fail(meter, row, code, Some(cause), at)
}

fn modification_key(entry: &CheckedFrameModification) -> (&CheckedNodeId, &str, &str) {
    match entry {
        CheckedFrameModification::Relationship { declaration } => (declaration, "", ""),
        CheckedFrameModification::Field {
            declaration,
            name,
            scope,
        } => (
            declaration,
            name,
            match scope {
                CheckedFieldScope::EveryObject => "every-object",
                CheckedFieldScope::Receiver => "receiver",
            },
        ),
    }
}

fn scoped_key(entry: &CheckedScopedDeletion) -> (u8, &str) {
    match entry {
        CheckedScopedDeletion::Receiver => (0, ""),
        CheckedScopedDeletion::Parameter { name } => (1, name),
    }
}

fn first_repeat<K: Ord>(
    meter: &mut WorkMeter,
    row: usize,
    keys: impl ExactSizeIterator<Item = K>,
) -> Result<Option<usize>, ValidationFailure> {
    stage_storage::first_repeat(meter, &site(row, rows_at()), keys)
}

fn check_shape(
    row: &CheckedOperationEffectRow,
    meter: &mut WorkMeter,
    index: usize,
) -> Result<(), ValidationFailure> {
    let at = rows_at().index(index);
    if row.document.is_empty() {
        return Err(malformed(meter, index, at.key("document")));
    }
    if !is_identifier(&row.operation) {
        return Err(malformed(meter, index, at.key("operation")));
    }
    if row.provenance.member.is_empty() {
        return Err(malformed(meter, index, at.key("provenance").key("member")));
    }
    let effect_at = at.key("effect");
    let repeat = first_repeat(
        meter,
        index,
        row.effect.modifies.iter().map(modification_key),
    )?;
    for (position, entry) in row.effect.modifies.iter().enumerate() {
        let list = effect_at.key("modifies");
        stage_storage::work(meter, &site(index, list.index(position)), 1)?;
        if let CheckedFrameModification::Field { name, .. } = entry {
            if !is_identifier(name) {
                return Err(malformed(meter, index, list));
            }
        }
        if repeat == Some(position) {
            return Err(malformed(meter, index, list));
        }
    }
    for (name, entries) in [
        ("creates", &row.effect.creates),
        ("deletes", &row.effect.deletes),
    ] {
        let list = effect_at.key(name);
        let repeat = first_repeat(meter, index, entries.iter())?;
        for position in 0..entries.len() {
            stage_storage::work(meter, &site(index, list.index(position)), 1)?;
            if repeat == Some(position) {
                return Err(malformed(meter, index, list));
            }
        }
    }
    let list = effect_at.key("deletes_scoped");
    let repeat = first_repeat(
        meter,
        index,
        row.effect.deletes_scoped.iter().map(scoped_key),
    )?;
    for (position, entry) in row.effect.deletes_scoped.iter().enumerate() {
        stage_storage::work(meter, &site(index, list.index(position)), 1)?;
        if let CheckedScopedDeletion::Parameter { name } = entry {
            if !is_identifier(name) {
                return Err(malformed(meter, index, list));
            }
        }
        if repeat == Some(position) {
            return Err(malformed(meter, index, list));
        }
    }
    Ok(())
}

fn own_operation<'a>(declared: &'a FieldBearingTypeDecl, name: &str) -> Option<&'a OperationDecl> {
    declared
        .operations
        .iter()
        .find(|operation| member_name(&operation.identity) == name)
}

fn validate_effect_entries(
    row: &CheckedOperationEffectRow,
    operation: &OperationDecl,
    models: &RetainedModels,
    selection: usize,
    declaring_form: DeclarationForm,
    meter: &mut WorkMeter,
    index: usize,
) -> Result<(), ValidationFailure> {
    let effect_at = rows_at().index(index).key("effect");
    for (position, entry) in row.effect.modifies.iter().enumerate() {
        let entry_at = effect_at.key("modifies").index(position);
        let (declaration, field) = match entry {
            CheckedFrameModification::Relationship { declaration } => (declaration, None),
            CheckedFrameModification::Field {
                declaration,
                name,
                scope,
            } => {
                if *scope == CheckedFieldScope::Receiver
                    && declaring_form != DeclarationForm::ObjectType
                {
                    return Err(refused(
                        meter,
                        index,
                        Code::InvalidModelBinding,
                        Cause::MalformedDeclaration,
                        entry_at,
                    ));
                }
                (declaration, Some(name.as_ref()))
            }
        };
        let Some(entry_owner) = models.owner_by_key(declaration) else {
            return Err(refused(
                meter,
                index,
                Code::MissingDeclaration,
                Cause::MissingName,
                entry_at.key("declaration"),
            ));
        };
        if entry_owner.selection != selection {
            return Err(refused(
                meter,
                index,
                Code::MissingDeclaration,
                Cause::MissingName,
                entry_at.key("declaration"),
            ));
        }
        match field {
            None if entry_owner.form == DeclarationForm::Relationship => {}
            Some(name)
                if matches!(
                    entry_owner.form,
                    DeclarationForm::ObjectType
                        | DeclarationForm::RecordValueType
                        | DeclarationForm::Process
                        | DeclarationForm::StateMachine
                ) =>
            {
                if !entry_owner
                    .package
                    .fields
                    .of(entry_owner.node)
                    .is_some_and(|table| !matches!(table.select(name), Selected::Absent))
                {
                    return Err(refused(
                        meter,
                        index,
                        Code::MissingDeclaration,
                        Cause::MissingName,
                        entry_at.key("name"),
                    ));
                }
            }
            _ => {
                return Err(refused(
                    meter,
                    index,
                    Code::InvalidModelBinding,
                    Cause::MalformedDeclaration,
                    entry_at,
                ));
            }
        }
    }
    for (kind, entries) in [
        ("creates", &row.effect.creates),
        ("deletes", &row.effect.deletes),
    ] {
        for (position, entry) in entries.iter().enumerate() {
            let entry_at = effect_at.key(kind).index(position);
            let Some(entry_owner) = models.owner_by_key(entry) else {
                return Err(refused(
                    meter,
                    index,
                    Code::MissingDeclaration,
                    Cause::MissingName,
                    entry_at,
                ));
            };
            if entry_owner.selection != selection
                || !matches!(
                    entry_owner.form,
                    DeclarationForm::ObjectType | DeclarationForm::Process
                )
            {
                return Err(refused(
                    meter,
                    index,
                    Code::InvalidModelBinding,
                    Cause::MalformedDeclaration,
                    entry_at,
                ));
            }
        }
    }
    for (position, entry) in row.effect.deletes_scoped.iter().enumerate() {
        let entry_at = effect_at.key("deletes_scoped").index(position);
        match entry {
            CheckedScopedDeletion::Receiver if declaring_form != DeclarationForm::ObjectType => {
                return Err(refused(
                    meter,
                    index,
                    Code::InvalidModelBinding,
                    Cause::MalformedDeclaration,
                    entry_at,
                ));
            }
            CheckedScopedDeletion::Parameter { name }
                if !operation
                    .parameters
                    .iter()
                    .any(|parameter| parameter.name.as_deref() == Some(name.as_ref())) =>
            {
                return Err(refused(
                    meter,
                    index,
                    Code::MissingDeclaration,
                    Cause::MissingName,
                    entry_at.key("name"),
                ));
            }
            _ => {}
        }
    }
    let ordered_modifies = row
        .effect
        .modifies
        .windows(2)
        .all(|pair| modification_key(&pair[0]) < modification_key(&pair[1]));
    if !ordered_modifies {
        return Err(fail(
            meter,
            index,
            Code::InvalidSemanticGraph,
            None,
            effect_at.key("modifies"),
        ));
    }
    for (kind, entries) in [
        ("creates", &row.effect.creates),
        ("deletes", &row.effect.deletes),
    ] {
        if !entries.windows(2).all(|pair| pair[0] < pair[1]) {
            return Err(fail(
                meter,
                index,
                Code::InvalidSemanticGraph,
                None,
                effect_at.key(kind),
            ));
        }
    }
    if !row
        .effect
        .deletes_scoped
        .windows(2)
        .all(|pair| scoped_key(&pair[0]) < scoped_key(&pair[1]))
    {
        return Err(fail(
            meter,
            index,
            Code::InvalidSemanticGraph,
            None,
            effect_at.key("deletes_scoped"),
        ));
    }
    Ok(())
}

/// A name that is `head`, or `head/tail`, compared as that text without
/// building it.
#[derive(Clone, Copy, Debug)]
struct Joined<'a>(&'a str, Option<&'a str>);

impl Joined<'_> {
    fn bytes(&self) -> impl Iterator<Item = u8> + '_ {
        self.0.bytes().chain(
            self.1
                .into_iter()
                .flat_map(|tail| std::iter::once(b'/').chain(tail.bytes())),
        )
    }
}

impl PartialEq for Joined<'_> {
    fn eq(&self, other: &Self) -> bool {
        self.bytes().eq(other.bytes())
    }
}

impl Eq for Joined<'_> {}

impl PartialOrd for Joined<'_> {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for Joined<'_> {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        self.bytes().cmp(other.bytes())
    }
}

/// A modification grant as the source frame spells it: a kind, the member's
/// name and a scope.
type Grant<'a> = (&'static str, Joined<'a>, &'static str);

/// Sorts and de-duplicates a table, as a set compares.
fn as_set<T: Ord>(table: &mut Vec<T>) {
    table.sort_unstable();
    table.dedup();
}

fn compare_source_frame(
    row: &CheckedOperationEffectRow,
    operation: &OperationDecl,
    models: &RetainedModels,
    source: &DomainModel,
    meter: &mut WorkMeter,
    index: usize,
) -> Result<(), ValidationFailure> {
    let effect_at = rows_at().index(index).key("effect");
    let at = site(index, effect_at);
    let mismatch = |meter: &mut WorkMeter, member: &'static str| {
        refused(
            meter,
            index,
            Code::InvalidPackage,
            Cause::InvalidValue,
            effect_at.key(member),
        )
    };
    let frame = &operation.frame;
    let mut actual_modifies: Vec<Grant<'_>> =
        stage_storage::vec(meter, &at, row.effect.modifies.len())?;
    for entry in &row.effect.modifies {
        match entry {
            CheckedFrameModification::Relationship { declaration } => {
                let Some((_, original, _)) = models.selected_declaration(declaration) else {
                    continue;
                };
                actual_modifies.push(("relationship", Joined(original, None), "every-object"));
            }
            CheckedFrameModification::Field {
                declaration,
                name,
                scope,
            } => {
                let Some((_, original, _)) = models.selected_declaration(declaration) else {
                    continue;
                };
                actual_modifies.push((
                    "field",
                    Joined(original, Some(name)),
                    match scope {
                        CheckedFieldScope::EveryObject => "every-object",
                        CheckedFieldScope::Receiver => "receiver",
                    },
                ));
            }
        }
    }
    as_set(&mut actual_modifies);
    let mut expected_modifies: Vec<Grant<'_>> = stage_storage::vec(
        meter,
        &at,
        frame
            .modifies
            .len()
            .saturating_add(frame.modifies_receiver.len()),
    )?;
    for original in &frame.modifies {
        let original: &str = original.as_ref();
        let kind = if source.relationships.contains_key(original) {
            "relationship"
        } else {
            "field"
        };
        expected_modifies.push((kind, Joined(original, None), "every-object"));
    }
    for original in &frame.modifies_receiver {
        expected_modifies.push(("field", Joined(original.as_ref(), None), "receiver"));
    }
    as_set(&mut expected_modifies);
    if actual_modifies != expected_modifies {
        return Err(mismatch(meter, "modifies"));
    }
    for (kind, entries, expected) in [
        ("creates", &row.effect.creates, &frame.creates),
        ("deletes", &row.effect.deletes, &frame.deletes),
    ] {
        let mut actual: Vec<&str> = stage_storage::vec(meter, &at, entries.len())?;
        actual.extend(entries.iter().filter_map(|id| {
            models
                .selected_declaration(id)
                .map(|(_, original, _)| original)
        }));
        as_set(&mut actual);
        let mut wanted: Vec<&str> = stage_storage::vec(meter, &at, expected.len())?;
        wanted.extend(expected.iter().map(|name| -> &str { name.as_ref() }));
        as_set(&mut wanted);
        if actual != wanted {
            return Err(mismatch(meter, kind));
        }
    }
    let mut actual_scoped: Vec<(u8, &str)> =
        stage_storage::vec(meter, &at, row.effect.deletes_scoped.len())?;
    actual_scoped.extend(row.effect.deletes_scoped.iter().map(scoped_key));
    as_set(&mut actual_scoped);
    let mut expected_scoped: Vec<(u8, &str)> = stage_storage::vec(
        meter,
        &at,
        frame
            .deletes_parameters
            .len()
            .saturating_add(usize::from(frame.deletes_receiver)),
    )?;
    expected_scoped.extend(
        frame
            .deletes_parameters
            .iter()
            .map(|name| (1, name.as_ref())),
    );
    if frame.deletes_receiver {
        expected_scoped.push((0, ""));
    }
    as_set(&mut expected_scoped);
    if actual_scoped != expected_scoped {
        return Err(mismatch(meter, "deletes_scoped"));
    }
    Ok(())
}

/// Validate a present carrier against selected declarations and the selected
/// document's parsed operation frames before exposing any borrowed row.
pub(super) fn validate(
    rows: Option<&[CheckedOperationEffectRow]>,
    models: &RetainedModels,
    meter: &mut WorkMeter,
) -> Result<(), ValidationFailure> {
    let Some(rows) = rows else {
        return Ok(());
    };
    if rows.is_empty() {
        return Err(malformed(meter, 0, rows_at()));
    }
    type Key<'a> = (&'a str, &'a CheckedNodeId, &'a str);
    fn key_of(row: &CheckedOperationEffectRow) -> Key<'_> {
        (
            row.document.as_ref(),
            &row.declaring,
            row.operation.as_ref(),
        )
    }
    // The first row that repeats an earlier row's operation, and the sorted
    // table the final coverage check searches.
    let mut covered: Vec<(Key<'_>, usize)> =
        stage_storage::vec(meter, &site(0, rows_at()), rows.len())?;
    covered.extend(
        rows.iter()
            .enumerate()
            .map(|(position, row)| (key_of(row), position)),
    );
    covered.sort_unstable();
    let repeated = covered
        .windows(2)
        .filter(|pair| pair[0].0 == pair[1].0)
        .map(|pair| pair[1].1)
        .min();
    let mut previous: Option<Key<'_>> = None;
    for (index, row) in rows.iter().enumerate() {
        let at = rows_at().index(index);
        stage_storage::work(meter, &site(index, at), 1)?;
        check_shape(row, meter, index)?;
        if !models
            .selected_models()
            .iter()
            .any(|model| model.identity == row.document)
        {
            return Err(refused(
                meter,
                index,
                Code::MissingDeclaration,
                Cause::MissingSelection,
                at.key("document"),
            ));
        }
        if !models
            .selected_models()
            .iter()
            .any(|model| model.identity == row.declaring.package)
        {
            return Err(refused(
                meter,
                index,
                Code::InvalidPackage,
                Cause::InvalidValue,
                at.key("declaring"),
            ));
        }
        let Some(owner) = models.owner_by_key(&row.declaring) else {
            return Err(refused(
                meter,
                index,
                Code::MissingDeclaration,
                Cause::MissingName,
                at.key("declaring"),
            ));
        };
        if !matches!(
            owner.form,
            DeclarationForm::ObjectType | DeclarationForm::Process | DeclarationForm::StateMachine
        ) {
            return Err(refused(
                meter,
                index,
                Code::IllTyped,
                Cause::OperatorIneligible,
                at.key("declaring"),
            ));
        }
        if owner.package.identity != row.document {
            return Err(refused(
                meter,
                index,
                Code::InvalidPackage,
                Cause::InvalidValue,
                at.key("document"),
            ));
        }
        let Some(declared) = owner.package.object_types.get(owner.node) else {
            return Err(refused(
                meter,
                index,
                Code::IllTyped,
                Cause::OperatorIneligible,
                at.key("declaring"),
            ));
        };
        let Some(operation) = own_operation(declared, &row.operation) else {
            return Err(refused(
                meter,
                index,
                Code::IllTyped,
                Cause::OperatorIneligible,
                at.key("operation"),
            ));
        };
        if row.provenance.member != operation.identity {
            return Err(refused(
                meter,
                index,
                Code::InvalidPackage,
                Cause::InvalidValue,
                at.key("provenance").key("member"),
            ));
        }
        validate_effect_entries(
            row,
            operation,
            models,
            owner.selection,
            owner.form,
            meter,
            index,
        )?;
        compare_source_frame(row, operation, models, owner.package, meter, index)?;
        if repeated == Some(index) {
            return Err(refused(
                meter,
                index,
                Code::InvalidPackage,
                Cause::ConflictingDefinition,
                at,
            ));
        }
        let key = key_of(row);
        if previous.is_some_and(|earlier| earlier >= key) {
            return Err(refused(
                meter,
                index,
                Code::InvalidPackage,
                Cause::InvalidValue,
                at,
            ));
        }
        previous = Some(key);
    }
    for (key, declaration) in models.selected_declarations() {
        if !matches!(
            declaration.form,
            DeclarationForm::ObjectType | DeclarationForm::Process | DeclarationForm::StateMachine
        ) {
            continue;
        }
        let Some(owner) = models.owner_by_key(key) else {
            continue;
        };
        let Some(declared) = owner.package.object_types.get(owner.node) else {
            continue;
        };
        for operation in &declared.operations {
            let probe: Key<'_> = (
                owner.package.identity.as_ref(),
                key,
                member_name(&operation.identity),
            );
            if covered
                .binary_search_by(|(held, _)| held.cmp(&probe))
                .is_err()
            {
                return Err(refused(
                    meter,
                    0,
                    Code::InvalidPackage,
                    Cause::InvalidValue,
                    rows_at(),
                ));
            }
        }
    }
    Ok(())
}
