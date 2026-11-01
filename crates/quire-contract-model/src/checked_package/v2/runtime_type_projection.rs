// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (C) 2026 Agent-IX

//! FR-354: complete selected record and object type reads for a sealed runtime adapter.

use super::exact_projection::project_record_field_type;
use super::model_members::{DeclarationForm, MemberType};
use super::{
    CheckedNodeId, CheckedNodeKind, CheckedPackageV2, ExactTypeProjectionError,
    ExactTypeProjectionErrorCode, ModelForm, ScalarTypeForm, SelectedExactIdentities,
    SelectedExactIdentity, SelectedRecordView,
};
use quire_exact::{EffectiveId, NodeKey, Presence, ValueType};
use std::collections::BTreeMap;

/// One admitted effective object field in selected declaration order.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct SelectedRuntimeObjectField {
    original_identity: Box<str>,
    name: Box<str>,
    value_type: ValueType,
    presence: Presence,
}

impl SelectedRuntimeObjectField {
    /// The selected document's original field identity.
    pub fn original_identity(&self) -> &str {
        &self.original_identity
    }

    /// The selected field name.
    pub fn name(&self) -> &str {
        &self.name
    }

    /// The complete Exact field type.
    pub fn value_type(&self) -> &ValueType {
        &self.value_type
    }

    /// The selected field's presence.
    pub fn presence(&self) -> Presence {
        self.presence
    }
}

/// One selected object declaration and its complete effective fields.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct SelectedRuntimeObject {
    original_identity: Box<str>,
    effective_id: EffectiveId,
    supertypes: Vec<EffectiveId>,
    fields: Vec<SelectedRuntimeObjectField>,
}

impl SelectedRuntimeObject {
    /// The original declaration identity in its selected document.
    pub fn original_identity(&self) -> &str {
        &self.original_identity
    }

    /// The QSL-stage witnessed effective object identity.
    pub fn effective_id(&self) -> &EffectiveId {
        &self.effective_id
    }

    /// Direct witnessed object supertypes in selected declaration order.
    pub fn supertypes(&self) -> &[EffectiveId] {
        &self.supertypes
    }

    /// Every effective field in selected declaration order.
    pub fn fields(&self) -> &[SelectedRuntimeObjectField] {
        &self.fields
    }
}

/// One selected record declaration with its complete Exact view and parents.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct SelectedRuntimeRecord {
    original_identity: Box<str>,
    view: SelectedRecordView,
    supertypes: Vec<NodeKey>,
}

impl SelectedRuntimeRecord {
    /// The original declaration identity in its selected document.
    pub fn original_identity(&self) -> &str {
        &self.original_identity
    }

    /// Complete effective fields and the witnessed record key.
    pub fn view(&self) -> &SelectedRecordView {
        &self.view
    }

    /// Direct witnessed record supertypes in selected declaration order.
    pub fn supertypes(&self) -> &[NodeKey] {
        &self.supertypes
    }
}

/// Complete selected runtime types for a package with supported declarations.
///
/// The view contains no partial tables. A package that needs tuple, enum or
/// unit producer witnesses refuses until those rows and their projections are
/// available; this view never reconstructs that missing authority.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct SelectedRuntimeTypes {
    records: Vec<SelectedRuntimeRecord>,
    objects: Vec<SelectedRuntimeObject>,
    work: u64,
}

impl SelectedRuntimeTypes {
    /// Selected records in structural node order.
    pub fn records(&self) -> &[SelectedRuntimeRecord] {
        &self.records
    }

    /// Selected objects in structural node order.
    pub fn objects(&self) -> &[SelectedRuntimeObject] {
        &self.objects
    }

    /// Work units charged to form the complete view.
    pub fn work(&self) -> u64 {
        self.work
    }
}

/// A typed refusal with the first failing admitted node and charged work.
#[derive(Clone, Debug, Eq, PartialEq, thiserror::Error)]
#[error("selected runtime type projection refused: {code:?} at {form}")]
pub struct SelectedRuntimeTypesError {
    /// Stable refusal category shared with Exact type projection.
    pub code: ExactTypeProjectionErrorCode,
    /// The first node whose selected type could not be projected.
    pub at: Box<CheckedNodeId>,
    /// The admitted family/form, or a named selected join.
    pub form: Box<str>,
    /// Work successfully charged before refusal.
    pub work: u64,
}

impl SelectedRuntimeTypesError {
    fn at(
        code: ExactTypeProjectionErrorCode,
        at: &CheckedNodeId,
        form: impl Into<Box<str>>,
        work: u64,
    ) -> Self {
        Self {
            code,
            at: Box::new(at.clone()),
            form: form.into(),
            work,
        }
    }

    fn from_exact(error: ExactTypeProjectionError, prior_work: u64) -> Self {
        Self {
            code: error.code,
            at: error.at.unwrap_or(error.requested),
            form: error.form,
            work: prior_work.saturating_add(error.work),
        }
    }
}

fn charge(
    work: &mut u64,
    limit: u64,
    at: &CheckedNodeId,
    form: &str,
) -> Result<(), SelectedRuntimeTypesError> {
    if *work >= limit {
        return Err(SelectedRuntimeTypesError::at(
            ExactTypeProjectionErrorCode::ExhaustedWork,
            at,
            form,
            *work,
        ));
    }
    *work += 1;
    Ok(())
}

fn projected_field_type(
    package: &CheckedPackageV2,
    identities: &SelectedExactIdentities,
    member_type: Option<&MemberType>,
    optional: bool,
    at: &CheckedNodeId,
    work: u64,
) -> Result<ValueType, SelectedRuntimeTypesError> {
    let member_type = member_type.ok_or_else(|| {
        SelectedRuntimeTypesError::at(
            ExactTypeProjectionErrorCode::UnsupportedExactType,
            at,
            "object_field_type",
            work,
        )
    })?;
    let member_type = if optional {
        match member_type {
            MemberType::Option(inner) => inner.as_ref(),
            _ => {
                return Err(SelectedRuntimeTypesError::at(
                    ExactTypeProjectionErrorCode::UnsupportedExactType,
                    at,
                    "object_field_presence",
                    work,
                ));
            }
        }
    } else {
        member_type
    };
    project_record_field_type(package, identities, member_type).map_err(|(code, target)| {
        SelectedRuntimeTypesError::at(
            code,
            target.as_ref().unwrap_or(at),
            "object_field_type",
            work,
        )
    })
}

impl CheckedPackageV2 {
    /// Project every selected record and object declaration as one bounded view.
    ///
    /// This currently admits only packages whose selected type environment
    /// needs records and objects. Tuple, enum and unit selections return a
    /// typed refusal before a view is exposed, pending their QSL-stage witness
    /// rows and the admitted unit resolver. Identity rows must originate from
    /// the stage selecting the same documents; IR checks their structural join
    /// but cannot authenticate an opaque Exact key supplied by another caller.
    pub fn project_selected_runtime_types(
        &self,
        identities: &SelectedExactIdentities,
        work_limit: u64,
    ) -> Result<SelectedRuntimeTypes, SelectedRuntimeTypesError> {
        let mut work = 0;
        for (node, kind) in self.graph().nodes.iter().zip(self.node_kinds()) {
            charge(&mut work, work_limit, &node.node_id, "graph_node")?;
            if matches!(
                kind,
                CheckedNodeKind::ScalarType(
                    ScalarTypeForm::Unit | ScalarTypeForm::Enum | ScalarTypeForm::CompoundUnit
                )
            ) {
                return Err(SelectedRuntimeTypesError::at(
                    ExactTypeProjectionErrorCode::UnavailableIdentityWitness,
                    &node.node_id,
                    format!("{}/{}", node.node_tag, node.semantic_form),
                    work,
                ));
            }
        }

        let mut by_original = BTreeMap::new();
        for (id, declaration) in self.models.selected_declarations() {
            charge(&mut work, work_limit, id, "selected_declaration")?;
            by_original.insert(
                (declaration.selection, declaration.node.as_ref()),
                (id, declaration.form),
            );
            if matches!(
                declaration.form,
                DeclarationForm::TupleValueType | DeclarationForm::VariantType
            ) {
                return Err(SelectedRuntimeTypesError::at(
                    ExactTypeProjectionErrorCode::UnavailableIdentityWitness,
                    id,
                    declaration.form.form(),
                    work,
                ));
            }
        }

        let mut records = Vec::new();
        let mut objects = Vec::new();
        for (id, declaration) in self.models.selected_declarations() {
            let Some(model) = self.models.selected_models().get(declaration.selection) else {
                return Err(SelectedRuntimeTypesError::at(
                    ExactTypeProjectionErrorCode::UnresolvedSelectedDeclaration,
                    id,
                    "selected_model",
                    work,
                ));
            };
            let (supertypes, expected_form) = match declaration.form {
                DeclarationForm::RecordValueType => (
                    model
                        .record_types
                        .get(&declaration.node)
                        .map(|value| value.supertypes.as_slice()),
                    DeclarationForm::RecordValueType,
                ),
                DeclarationForm::ObjectType => (
                    model
                        .object_types
                        .get(&declaration.node)
                        .map(|value| value.supertypes.as_slice()),
                    DeclarationForm::ObjectType,
                ),
                _ => continue,
            };
            let supertypes = supertypes.ok_or_else(|| {
                SelectedRuntimeTypesError::at(
                    ExactTypeProjectionErrorCode::UnresolvedSelectedDeclaration,
                    id,
                    declaration.form.form(),
                    work,
                )
            })?;
            let mut record_parents = Vec::new();
            let mut object_parents = Vec::new();
            for original in supertypes {
                charge(&mut work, work_limit, id, "selected_supertype")?;
                let Some((parent_id, parent_form)) =
                    by_original.get(&(declaration.selection, original.as_ref()))
                else {
                    return Err(SelectedRuntimeTypesError::at(
                        ExactTypeProjectionErrorCode::UnresolvedSelectedDeclaration,
                        id,
                        "selected_supertype",
                        work,
                    ));
                };
                if *parent_form != expected_form {
                    return Err(SelectedRuntimeTypesError::at(
                        ExactTypeProjectionErrorCode::MismatchedIdentityWitness,
                        parent_id,
                        "selected_supertype",
                        work,
                    ));
                }
                let identity =
                    identities
                        .declaration_identity(self, parent_id)
                        .map_err(|code| {
                            SelectedRuntimeTypesError::at(
                                code,
                                parent_id,
                                "selected_supertype",
                                work,
                            )
                        })?;
                match (declaration.form, identity) {
                    (DeclarationForm::RecordValueType, SelectedExactIdentity::Record(key)) => {
                        record_parents.push(key.clone());
                    }
                    (DeclarationForm::ObjectType, SelectedExactIdentity::Object(key)) => {
                        object_parents.push(key.clone());
                    }
                    _ => {
                        return Err(SelectedRuntimeTypesError::at(
                            ExactTypeProjectionErrorCode::MismatchedIdentityWitness,
                            parent_id,
                            "selected_supertype",
                            work,
                        ));
                    }
                }
            }
            match declaration.form {
                DeclarationForm::RecordValueType => {
                    let remaining = work_limit.saturating_sub(work);
                    let projection = self
                        .project_exact_type(id, identities, remaining)
                        .map_err(|error| SelectedRuntimeTypesError::from_exact(error, work))?;
                    work = work.saturating_add(projection.work());
                    let view = projection.record_view().ok_or_else(|| {
                        SelectedRuntimeTypesError::at(
                            ExactTypeProjectionErrorCode::UnsupportedExactType,
                            id,
                            "record_value_type",
                            work,
                        )
                    })?;
                    records.push(SelectedRuntimeRecord {
                        original_identity: declaration.node.clone(),
                        view: view.clone(),
                        supertypes: record_parents,
                    });
                }
                DeclarationForm::ObjectType => {
                    charge(&mut work, work_limit, id, "object_type")?;
                    let effective_id =
                        match identities.declaration_identity(self, id).map_err(|code| {
                            SelectedRuntimeTypesError::at(code, id, "object_type", work)
                        })? {
                            SelectedExactIdentity::Object(key) => key.clone(),
                            _ => {
                                return Err(SelectedRuntimeTypesError::at(
                                    ExactTypeProjectionErrorCode::MismatchedIdentityWitness,
                                    id,
                                    "object_type",
                                    work,
                                ));
                            }
                        };
                    let (ambiguous, fields) =
                        self.models.selected_object_fields(id).ok_or_else(|| {
                            SelectedRuntimeTypesError::at(
                                ExactTypeProjectionErrorCode::UnresolvedSelectedDeclaration,
                                id,
                                "object_type_fields",
                                work,
                            )
                        })?;
                    if ambiguous.is_some() {
                        return Err(SelectedRuntimeTypesError::at(
                            ExactTypeProjectionErrorCode::UnsupportedExactType,
                            id,
                            "ambiguous_object_field",
                            work,
                        ));
                    }
                    let mut projected_fields = Vec::new();
                    for field in fields {
                        charge(&mut work, work_limit, id, "object_field")?;
                        let value_type = projected_field_type(
                            self,
                            identities,
                            field.member_type,
                            field.optional,
                            id,
                            work,
                        )?;
                        projected_fields.push(SelectedRuntimeObjectField {
                            original_identity: field.identity.into(),
                            name: field.name.into(),
                            value_type,
                            presence: if field.optional {
                                Presence::Optional
                            } else {
                                Presence::Required
                            },
                        });
                    }
                    objects.push(SelectedRuntimeObject {
                        original_identity: declaration.node.clone(),
                        effective_id,
                        supertypes: object_parents,
                        fields: projected_fields,
                    });
                }
                _ => unreachable!("selected runtime declaration form was checked"),
            }
        }
        Ok(SelectedRuntimeTypes {
            records,
            objects,
            work,
        })
    }
}
