// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (C) 2026 Agent-IX

//! Selected model operation effects carried by the checked package (TC-977).

use super::identity::is_identifier;
use super::model_fields::RetainedModels;
use super::model_members::{member_name, DeclarationForm, FieldBearingTypeDecl, OperationDecl};
use super::{CheckedFieldScope, CheckedFrameModification, CheckedNodeId, CheckedScopedDeletion, WorkMeter};
use crate::checked_package::common::ValidationFailure;
use crate::checked_package::shared::{
    CheckedPackageRefusalCause as Cause, CheckedPackageRefusalCode as Code, JsonPointer,
};
use quire_canonical::{Encode, Error, FixedShape, Sink, Writer};
use serde::{Deserialize, Deserializer, Serialize};
use std::collections::BTreeSet;

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
                (row.document.as_ref(), &row.declaring, row.operation.as_ref())
                    .cmp(&(document, declaring, operation))
            })
            .ok()?;
        rows.get(index)
    }
}

fn row_at(index: usize) -> JsonPointer {
    JsonPointer::root().key("operation_effects").index(index)
}

fn malformed(at: JsonPointer) -> ValidationFailure {
    ValidationFailure::refused(Code::MalformedWire, at)
}

fn refused(code: Code, cause: Cause, at: JsonPointer) -> ValidationFailure {
    ValidationFailure::refused_because(code, at, cause)
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

fn check_shape(row: &CheckedOperationEffectRow, at: &JsonPointer) -> Result<(), ValidationFailure> {
    if row.document.is_empty() {
        return Err(malformed(at.clone().key("document")));
    }
    if !is_identifier(&row.operation) {
        return Err(malformed(at.clone().key("operation")));
    }
    if row.provenance.member.is_empty() {
        return Err(malformed(at.clone().key("provenance").key("member")));
    }
    let effect_at = at.clone().key("effect");
    let mut seen = BTreeSet::new();
    for entry in &row.effect.modifies {
        if let CheckedFrameModification::Field { name, .. } = entry {
            if !is_identifier(name) {
                return Err(malformed(effect_at.clone().key("modifies")));
            }
        }
        if !seen.insert(modification_key(entry)) {
            return Err(malformed(effect_at.clone().key("modifies")));
        }
    }
    for (name, entries) in [
        ("creates", &row.effect.creates),
        ("deletes", &row.effect.deletes),
    ] {
        if entries.iter().collect::<BTreeSet<_>>().len() != entries.len() {
            return Err(malformed(effect_at.clone().key(name)));
        }
    }
    let mut seen = BTreeSet::new();
    for entry in &row.effect.deletes_scoped {
        if let CheckedScopedDeletion::Parameter { name } = entry {
            if !is_identifier(name) {
                return Err(malformed(effect_at.clone().key("deletes_scoped")));
            }
        }
        if !seen.insert(scoped_key(entry)) {
            return Err(malformed(effect_at.clone().key("deletes_scoped")));
        }
    }
    Ok(())
}

fn own_operation<'a>(
    declared: &'a FieldBearingTypeDecl,
    name: &str,
) -> Option<&'a OperationDecl> {
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
    at: &JsonPointer,
) -> Result<(), ValidationFailure> {
    let effect_at = at.clone().key("effect");
    for (index, entry) in row.effect.modifies.iter().enumerate() {
        let entry_at = effect_at.clone().key("modifies").index(index);
        meter.charge(1, || entry_at.clone())?;
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
                        Code::InvalidModelBinding,
                        Cause::MalformedDeclaration,
                        entry_at,
                    ));
                }
                (declaration, Some(name.as_ref()))
            }
        };
        let entry_owner = models.owner_by_key(declaration).ok_or_else(|| {
            refused(
                Code::MissingDeclaration,
                Cause::MissingName,
                entry_at.clone().key("declaration"),
            )
        })?;
        if entry_owner.selection != selection {
            return Err(refused(
                Code::MissingDeclaration,
                Cause::MissingName,
                entry_at.clone().key("declaration"),
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
                    .exposed_names(entry_owner.node)
                    .is_some_and(|names| names.contains(name))
                {
                    return Err(refused(
                        Code::MissingDeclaration,
                        Cause::MissingName,
                        entry_at.key("name"),
                    ));
                }
            }
            _ => {
                return Err(refused(
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
        for (index, entry) in entries.iter().enumerate() {
            let entry_at = effect_at.clone().key(kind).index(index);
            meter.charge(1, || entry_at.clone())?;
            let entry_owner = models.owner_by_key(entry).ok_or_else(|| {
                refused(Code::MissingDeclaration, Cause::MissingName, entry_at.clone())
            })?;
            if entry_owner.selection != selection
                || !matches!(entry_owner.form, DeclarationForm::ObjectType | DeclarationForm::Process)
            {
                return Err(refused(
                    Code::InvalidModelBinding,
                    Cause::MalformedDeclaration,
                    entry_at,
                ));
            }
        }
    }
    for (index, entry) in row.effect.deletes_scoped.iter().enumerate() {
        let entry_at = effect_at.clone().key("deletes_scoped").index(index);
        meter.charge(1, || entry_at.clone())?;
        match entry {
            CheckedScopedDeletion::Receiver if declaring_form != DeclarationForm::ObjectType => {
                return Err(refused(
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
        return Err(ValidationFailure::refused(
            Code::InvalidSemanticGraph,
            effect_at.clone().key("modifies"),
        ));
    }
    for (kind, entries) in [
        ("creates", &row.effect.creates),
        ("deletes", &row.effect.deletes),
    ] {
        if !entries.windows(2).all(|pair| pair[0] < pair[1]) {
            return Err(ValidationFailure::refused(
                Code::InvalidSemanticGraph,
                effect_at.clone().key(kind),
            ));
        }
    }
    if !row
        .effect
        .deletes_scoped
        .windows(2)
        .all(|pair| scoped_key(&pair[0]) < scoped_key(&pair[1]))
    {
        return Err(ValidationFailure::refused(
            Code::InvalidSemanticGraph,
            effect_at.key("deletes_scoped"),
        ));
    }
    Ok(())
}

fn compare_source_frame(
    row: &CheckedOperationEffectRow,
    operation: &OperationDecl,
    models: &RetainedModels,
    at: &JsonPointer,
) -> Result<(), ValidationFailure> {
    let effect_at = at.clone().key("effect");
    let mut actual_modifies = BTreeSet::new();
    for entry in &row.effect.modifies {
        match entry {
            CheckedFrameModification::Relationship { declaration } => {
                let Some((_, original, _)) = models.selected_declaration(declaration) else {
                    continue;
                };
                actual_modifies.insert(("relationship", original.to_owned(), "every-object"));
            }
            CheckedFrameModification::Field {
                declaration,
                name,
                scope,
            } => {
                let Some((_, original, _)) = models.selected_declaration(declaration) else {
                    continue;
                };
                actual_modifies.insert((
                    "field",
                    format!("{original}/{name}"),
                    match scope {
                        CheckedFieldScope::EveryObject => "every-object",
                        CheckedFieldScope::Receiver => "receiver",
                    },
                ));
            }
        }
    }
    let mut expected_modifies = BTreeSet::new();
    for original in &operation.frame.modifies {
        let kind = if models
            .selected_models()
            .iter()
            .any(|model| model.relationships.contains_key(original))
        {
            "relationship"
        } else {
            "field"
        };
        expected_modifies.insert((kind, original.to_string(), "every-object"));
    }
    for original in &operation.frame.modifies_receiver {
        expected_modifies.insert(("field", original.to_string(), "receiver"));
    }
    if actual_modifies != expected_modifies {
        return Err(refused(
            Code::InvalidPackage,
            Cause::InvalidValue,
            effect_at.clone().key("modifies"),
        ));
    }
    for (kind, entries, expected) in [
        ("creates", &row.effect.creates, &operation.frame.creates),
        ("deletes", &row.effect.deletes, &operation.frame.deletes),
    ] {
        let actual: BTreeSet<_> = entries
            .iter()
            .filter_map(|id| models.selected_declaration(id).map(|(_, original, _)| original.to_owned()))
            .collect();
        let expected: BTreeSet<_> = expected.iter().map(|name| name.to_string()).collect();
        if actual != expected {
            return Err(refused(
                Code::InvalidPackage,
                Cause::InvalidValue,
                effect_at.clone().key(kind),
            ));
        }
    }
    let actual_scoped: BTreeSet<_> = row
        .effect
        .deletes_scoped
        .iter()
        .map(|entry| match entry {
            CheckedScopedDeletion::Receiver => ("receiver", "".to_owned()),
            CheckedScopedDeletion::Parameter { name } => ("parameter", name.to_string()),
        })
        .collect();
    let mut expected_scoped: BTreeSet<_> = operation
        .frame
        .deletes_parameters
        .iter()
        .map(|name| ("parameter", name.to_string()))
        .collect();
    if operation.frame.deletes_receiver {
        expected_scoped.insert(("receiver", "".to_owned()));
    }
    if actual_scoped != expected_scoped {
        return Err(refused(
            Code::InvalidPackage,
            Cause::InvalidValue,
            effect_at.key("deletes_scoped"),
        ));
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
        return Err(malformed(JsonPointer::root().key("operation_effects")));
    }
    let mut covered = BTreeSet::new();
    let mut previous: Option<(&str, &CheckedNodeId, &str)> = None;
    for (index, row) in rows.iter().enumerate() {
        let at = row_at(index);
        meter.charge(1, || at.clone())?;
        check_shape(row, &at)?;
        if !models.selected_models().iter().any(|model| model.identity == row.document) {
            return Err(refused(
                Code::MissingDeclaration,
                Cause::MissingSelection,
                at.clone().key("document"),
            ));
        }
        if !models.selected_models().iter().any(|model| model.identity == row.declaring.package) {
            return Err(refused(
                Code::InvalidPackage,
                Cause::InvalidValue,
                at.clone().key("declaring"),
            ));
        }
        let owner = models.owner_by_key(&row.declaring).ok_or_else(|| {
            refused(
                Code::MissingDeclaration,
                Cause::MissingName,
                at.clone().key("declaring"),
            )
        })?;
        if !matches!(
            owner.form,
            DeclarationForm::ObjectType | DeclarationForm::Process | DeclarationForm::StateMachine
        ) {
            return Err(refused(
                Code::IllTyped,
                Cause::OperatorIneligible,
                at.clone().key("declaring"),
            ));
        }
        if owner.package.identity != row.document {
            return Err(refused(
                Code::InvalidPackage,
                Cause::InvalidValue,
                at.clone().key("document"),
            ));
        }
        let declared = owner.package.object_types.get(owner.node).ok_or_else(|| {
            refused(
                Code::IllTyped,
                Cause::OperatorIneligible,
                at.clone().key("declaring"),
            )
        })?;
        let operation = own_operation(declared, &row.operation).ok_or_else(|| {
            refused(
                Code::IllTyped,
                Cause::OperatorIneligible,
                at.clone().key("operation"),
            )
        })?;
        if row.provenance.member != operation.identity {
            return Err(refused(
                Code::InvalidPackage,
                Cause::InvalidValue,
                at.clone().key("provenance").key("member"),
            ));
        }
        validate_effect_entries(row, operation, models, owner.selection, owner.form, meter, &at)?;
        compare_source_frame(row, operation, models, &at)?;
        let key = (row.document.as_ref(), &row.declaring, row.operation.as_ref());
        if !covered.insert((row.document.clone(), row.declaring.clone(), row.operation.clone())) {
            return Err(refused(Code::InvalidPackage, Cause::ConflictingDefinition, at));
        }
        if previous.is_some_and(|earlier| earlier >= key) {
            return Err(refused(Code::InvalidPackage, Cause::InvalidValue, at));
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
            if !covered.contains(&(
                owner.package.identity.clone(),
                key.clone(),
                member_name(&operation.identity).into(),
            )) {
                return Err(refused(
                    Code::InvalidPackage,
                    Cause::InvalidValue,
                    JsonPointer::root().key("operation_effects"),
                ));
            }
        }
    }
    Ok(())
}
