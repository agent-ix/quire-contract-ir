// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (C) 2026 Agent-IX

//! Bounded reads of explicitly selected model and equality facts (FR-357-AC-5..8).

use super::exact_projection::project_record_field_type;
use super::model_members::MemberType;
use super::{
    AdmittedEqualityReadError, CheckedNodeId, CheckedNodeKind, CheckedOccurrence,
    CheckedPackageV2, ExactTypeProjectionError, ModelForm,
    SelectedEqualityApplicationRow, SelectedEqualityFactRow, SelectedExactDeclarationRow,
    SelectedExactFieldRow, SelectedExactIdentities, SelectedSourceUnitRow,
};
use quire_exact::{Presence, ValueType};
use quire_semantic_value::declaration::AdmittedEqualityDecision;
use std::collections::{BTreeMap, BTreeSet};

/// An explicitly selected original field of an admitted record declaration.
#[derive(Clone, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub struct SelectedFactFieldId {
    /// Final structural node of the declaring record.
    pub record: CheckedNodeId,
    /// Original selected-document field identity.
    pub original_identity: Box<str>,
}

/// One explicitly selected equality application and exact occurrence.
#[derive(Clone, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub struct SelectedFactEqualityId {
    /// Final application node.
    pub application: CheckedNodeId,
    /// Exact admitted occurrence.
    pub occurrence: CheckedOccurrence,
}

/// Admitted node and occurrence IDs to inspect; these IDs do not assert reachability.
pub struct SelectedFactIds<'a> {
    /// Selected model declarations, possibly repeated.
    pub declarations: &'a [CheckedNodeId],
    /// Selected original record fields, possibly repeated.
    pub fields: &'a [SelectedFactFieldId],
    /// Selected equality applications and occurrences, possibly repeated.
    pub equalities: &'a [SelectedFactEqualityId],
}

/// Full-emission producer rows supplied by the caller from one final emission.
pub struct SelectedFactRows<'a> {
    /// QSL-selected Exact declaration identities.
    pub declarations: &'a [SelectedExactDeclarationRow],
    /// QSL-selected record field identities.
    pub fields: &'a [SelectedExactFieldRow],
    /// Source-unit identities used by selected equality decisions.
    pub source_units: &'a [SelectedSourceUnitRow],
    /// Original equality application identities and occurrences.
    pub equality_applications: &'a [SelectedEqualityApplicationRow],
    /// Original typed equality facts.
    pub equality_facts: &'a [SelectedEqualityFactRow],
}

/// One selected field with admitted order, presence and complete Exact type.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct SelectedFactField {
    /// Joined producer identity row.
    pub row: SelectedExactFieldRow,
    /// Admitted field name.
    pub name: Box<str>,
    /// Zero-based position in the admitted record's effective field order.
    pub order: usize,
    /// Required or optional presence.
    pub presence: Presence,
    /// Admitted, representable field type.
    pub value_type: ValueType,
}

/// One selected equality with its original occurrence and opaque admitted decision.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct SelectedFactEquality {
    /// Full final application node and occurrence.
    pub id: SelectedFactEqualityId,
    /// FR-353's complete admitted decision.
    pub decision: AdmittedEqualityDecision,
}

/// Complete facts for only the explicit selection; no monitor closure is certified.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct SelectedFacts {
    /// Selected declaration identities in first-selection order.
    pub declarations: Vec<SelectedExactDeclarationRow>,
    /// Selected fields in first-selection order, with admitted producer order retained.
    pub fields: Vec<SelectedFactField>,
    /// Selected equality decisions in first-selection order.
    pub equalities: Vec<SelectedFactEquality>,
    /// Work charged to produce this complete view.
    pub work: u64,
}

/// Stable cause of a selected-fact refusal.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum SelectedFactsErrorCode {
    /// A selected node is not in this package.
    UnknownNode,
    /// A selected node belongs to another package.
    ForeignNode,
    /// A selected declaration or field has no supported admitted form.
    UnsupportedForm,
    /// A selected field is not in its admitted record.
    UnknownField,
    /// A required selected producer row is absent.
    MissingRow,
    /// A required producer row conflicts with another row.
    DuplicateRow,
    /// An Exact identity or type projection refused.
    Exact,
    /// An admitted equality decision refused.
    Equality,
    /// The next charged inspection exceeds the caller's allowance.
    ExhaustedWork,
}

impl SelectedFactsErrorCode {
    /// Stable machine-readable spelling.
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::UnknownNode => "unknown_node",
            Self::ForeignNode => "foreign_node",
            Self::UnsupportedForm => "unsupported_form",
            Self::UnknownField => "unknown_field",
            Self::MissingRow => "missing_row",
            Self::DuplicateRow => "duplicate_row",
            Self::Exact => "exact",
            Self::Equality => "equality",
            Self::ExhaustedWork => "exhausted_work",
        }
    }

    /// All stable refusal codes.
    pub const fn all() -> &'static [Self] {
        &[
            Self::UnknownNode,
            Self::ForeignNode,
            Self::UnsupportedForm,
            Self::UnknownField,
            Self::MissingRow,
            Self::DuplicateRow,
            Self::Exact,
            Self::Equality,
            Self::ExhaustedWork,
        ]
    }

    /// Decode a stable refusal code.
    pub fn from_code(code: &str) -> Option<Self> {
        Self::all().iter().copied().find(|value| value.as_str() == code)
    }
}

/// Typed first refusal; no partial selected view accompanies it.
#[derive(Clone, Debug, Eq, PartialEq, thiserror::Error)]
#[error("selected facts refused: {code:?} at {at:?}")]
pub struct SelectedFactsError {
    /// Stable category.
    pub code: SelectedFactsErrorCode,
    /// First native graph or row locus.
    pub at: Box<CheckedNodeId>,
    /// Selected occurrence, when the failing request is equality.
    pub occurrence: Option<CheckedOccurrence>,
    /// Original field identity, when the failing request is a field.
    pub field: Option<Box<str>>,
    /// Charged work before refusal.
    pub work: u64,
    /// First refused charge, zero for a semantic refusal.
    pub attempted_work: u64,
    /// Native Exact refusal, when one occurred.
    pub exact: Option<Box<ExactTypeProjectionError>>,
    /// Native FR-353 refusal, when one occurred.
    pub equality: Option<Box<AdmittedEqualityReadError>>,
}

impl SelectedFactsError {
    fn new(code: SelectedFactsErrorCode, at: &CheckedNodeId, work: u64) -> Self {
        Self {
            code,
            at: Box::new(at.clone()),
            occurrence: None,
            field: None,
            work,
            attempted_work: 0,
            exact: None,
            equality: None,
        }
    }
}

struct Meter {
    used: u64,
    limit: u64,
}

impl Meter {
    fn charge(&mut self, at: &CheckedNodeId, amount: u64) -> Result<(), SelectedFactsError> {
        if self.used.checked_add(amount).is_none_or(|next| next > self.limit) {
            let mut error = SelectedFactsError::new(SelectedFactsErrorCode::ExhaustedWork, at, self.used);
            error.attempted_work = amount;
            return Err(error);
        }
        self.used += amount;
        Ok(())
    }

    fn text(&mut self, at: &CheckedNodeId, value: &str) -> Result<(), SelectedFactsError> {
        self.charge(at, u64::try_from(value.len()).unwrap_or(u64::MAX))
    }
}

fn exact_error(error: ExactTypeProjectionError, prior: u64) -> SelectedFactsError {
    SelectedFactsError {
        code: SelectedFactsErrorCode::Exact,
        at: error.at.clone().unwrap_or_else(|| error.requested.clone()),
        occurrence: None,
        field: None,
        work: prior.saturating_add(error.work),
        attempted_work: 0,
        exact: Some(Box::new(error)),
        equality: None,
    }
}

fn equality_error(
    error: AdmittedEqualityReadError,
    occurrence: &CheckedOccurrence,
    prior: u64,
) -> SelectedFactsError {
    SelectedFactsError {
        code: SelectedFactsErrorCode::Equality,
        at: error.at.clone(),
        occurrence: Some(occurrence.clone()),
        field: None,
        work: prior.saturating_add(error.consumed_work),
        attempted_work: error.attempted_work,
        exact: None,
        equality: Some(Box::new(error)),
    }
}

impl CheckedPackageV2 {
    /// Read complete, explicitly selected model and equality facts under one finite allowance.
    ///
    /// The caller supplies rows from one final QSL emission. Structural joins are checked here,
    /// but an opaque Exact payload cannot be authenticated from caller-created rows. The result
    /// does not prove entry reachability or a generated-source declaration closure.
    pub fn project_selected_facts(
        &self,
        ids: SelectedFactIds<'_>,
        rows: SelectedFactRows<'_>,
        work_limit: u64,
    ) -> Result<SelectedFacts, SelectedFactsError> {
        let mut meter = Meter { used: 0, limit: work_limit };
        let mut selected_declarations = Vec::new();
        let mut selected_fields = Vec::new();
        let mut selected_equalities = Vec::new();
        let mut seen_declarations = BTreeSet::new();
        let mut seen_fields = BTreeSet::new();
        let mut seen_equalities = BTreeSet::new();
        let mut joined_declarations = BTreeMap::new();

        for id in ids.declarations {
            meter.charge(id, 1)?;
            if !seen_declarations.insert(id) {
                continue;
            }
            self.selected_fact_declaration(id, &rows, &mut meter, &mut joined_declarations)?;
            selected_declarations.push(joined_declarations[id].clone());
        }

        for id in ids.fields {
            meter.charge(&id.record, 1)?;
            meter.text(&id.record, &id.original_identity)?;
            if !seen_fields.insert(id) {
                continue;
            }
            self.selected_fact_declaration(&id.record, &rows, &mut meter, &mut joined_declarations)?;
            let (ambiguous, fields) = self.models.selected_record_fields(&id.record).ok_or_else(|| {
                let mut error = SelectedFactsError::new(SelectedFactsErrorCode::UnsupportedForm, &id.record, meter.used);
                error.field = Some(id.original_identity.clone());
                error
            })?;
            if ambiguous.is_some() {
                return Err(SelectedFactsError::new(SelectedFactsErrorCode::UnsupportedForm, &id.record, meter.used));
            }
            let mut selected = None;
            for (order, field) in fields.enumerate() {
                meter.charge(&id.record, 1)?;
                if field.identity == id.original_identity.as_ref() {
                    selected = Some((order, field));
                    break;
                }
            }
            let (order, field) = selected.ok_or_else(|| {
                let mut error = SelectedFactsError::new(SelectedFactsErrorCode::UnknownField, &id.record, meter.used);
                error.field = Some(id.original_identity.clone());
                error
            })?;
            let row = find_field_row(rows.fields, id, &mut meter)?;
            let declaration = joined_declarations.get(&id.record).expect("joined record identity").clone();
            let identities = SelectedExactIdentities::from_stage_rows(self, [declaration], [row.clone()])
                .map_err(|error| exact_error(error, meter.used))?;
            let member_type = field.member_type.ok_or_else(|| SelectedFactsError::new(
                SelectedFactsErrorCode::UnsupportedForm, &id.record, meter.used
            ))?;
            let member_type = if field.optional {
                match member_type {
                    MemberType::Option(inner) => inner.as_ref(),
                    _ => return Err(SelectedFactsError::new(SelectedFactsErrorCode::UnsupportedForm, &id.record, meter.used)),
                }
            } else {
                member_type
            };
            meter.charge(&id.record, 1)?;
            let value_type = project_record_field_type(self, &identities, member_type).map_err(|(code, target)| {
                let at = target.as_ref().unwrap_or(&id.record);
                let native = ExactTypeProjectionError {
                    code,
                    requested: Box::new(id.record.clone()),
                    at: Some(Box::new(at.clone())),
                    form: "record_field_type".into(),
                    work: 0,
                };
                let mut error = exact_error(native, meter.used);
                error.field = Some(id.original_identity.clone());
                error
            })?;
            meter.text(&id.record, field.name)?;
            selected_fields.push(SelectedFactField {
                row,
                name: field.name.into(),
                order,
                presence: if field.optional { Presence::Optional } else { Presence::Required },
                value_type,
            });
        }

        for id in ids.equalities {
            meter.charge(&id.application, 1)?;
            if !seen_equalities.insert(id) {
                continue;
            }
            let application_row = find_equality_row(rows.equality_applications, id, &mut meter)?;
            let mut facts = Vec::new();
            for row in rows.equality_facts {
                meter.charge(&row.structural_node, 1)?;
                if row.checked_application == id.application && row.occurrence == id.occurrence {
                    facts.push(row.clone());
                }
            }
            let mut units = Vec::new();
            for row in rows.source_units {
                meter.charge(&row.structural_node, 1)?;
                if facts.iter().any(|fact| fact.structural_node == row.structural_node) {
                    units.push(row.clone());
                }
            }
            let prior = meter.used;
            let (decision, work) = self.admitted_equality_metered(
                &id.application, &id.occurrence, &[application_row], &facts, &units,
                work_limit.saturating_sub(meter.used),
            ).map_err(|error| equality_error(error, &id.occurrence, prior))?;
            meter.charge(&id.application, work)?;
            selected_equalities.push(SelectedFactEquality { id: id.clone(), decision });
        }

        Ok(SelectedFacts {
            declarations: selected_declarations,
            fields: selected_fields,
            equalities: selected_equalities,
            work: meter.used,
        })
    }

    fn selected_fact_declaration(
        &self,
        id: &CheckedNodeId,
        rows: &SelectedFactRows<'_>,
        meter: &mut Meter,
        joined: &mut BTreeMap<CheckedNodeId, SelectedExactDeclarationRow>,
    ) -> Result<(), SelectedFactsError> {
        if joined.contains_key(id) {
            return Ok(());
        }
        meter.charge(id, 1)?;
        if id.package != self.name() {
            return Err(SelectedFactsError::new(SelectedFactsErrorCode::ForeignNode, id, meter.used));
        }
        if self.node(id).is_none() {
            return Err(SelectedFactsError::new(SelectedFactsErrorCode::UnknownNode, id, meter.used));
        }
        if !matches!(self.node_kind(id), Some(CheckedNodeKind::Model(ModelForm::RecordValueType | ModelForm::ObjectType))) {
            return Err(SelectedFactsError::new(SelectedFactsErrorCode::UnsupportedForm, id, meter.used));
        }
        let row = find_declaration_row(rows.declarations, id, meter)?;
        SelectedExactIdentities::from_stage_rows(self, [row.clone()], [])
            .map_err(|error| exact_error(error, meter.used))?;
        joined.insert(id.clone(), row);
        Ok(())
    }
}

fn find_declaration_row(
    rows: &[SelectedExactDeclarationRow],
    id: &CheckedNodeId,
    meter: &mut Meter,
) -> Result<SelectedExactDeclarationRow, SelectedFactsError> {
    let mut selected = None;
    for row in rows {
        meter.charge(&row.structural_node, 1)?;
        if &row.structural_node == id {
            if selected.is_some() {
                return Err(SelectedFactsError::new(SelectedFactsErrorCode::DuplicateRow, id, meter.used));
            }
            selected = Some(row.clone());
        }
    }
    selected.ok_or_else(|| SelectedFactsError::new(SelectedFactsErrorCode::MissingRow, id, meter.used))
}

fn find_field_row(
    rows: &[SelectedExactFieldRow],
    id: &SelectedFactFieldId,
    meter: &mut Meter,
) -> Result<SelectedExactFieldRow, SelectedFactsError> {
    let mut selected = None;
    for row in rows {
        meter.charge(&row.record_node, 1)?;
        if row.record_node == id.record && row.field_original_identity == id.original_identity {
            if selected.is_some() {
                let mut error = SelectedFactsError::new(SelectedFactsErrorCode::DuplicateRow, &id.record, meter.used);
                error.field = Some(id.original_identity.clone());
                return Err(error);
            }
            selected = Some(row.clone());
        }
    }
    selected.ok_or_else(|| {
        let mut error = SelectedFactsError::new(SelectedFactsErrorCode::MissingRow, &id.record, meter.used);
        error.field = Some(id.original_identity.clone());
        error
    })
}

fn find_equality_row(
    rows: &[SelectedEqualityApplicationRow],
    id: &SelectedFactEqualityId,
    meter: &mut Meter,
) -> Result<SelectedEqualityApplicationRow, SelectedFactsError> {
    let mut selected = None;
    for row in rows {
        meter.charge(&row.application, 1)?;
        if row.application == id.application && row.occurrence == id.occurrence {
            if selected.is_some() {
                let mut error = SelectedFactsError::new(SelectedFactsErrorCode::DuplicateRow, &id.application, meter.used);
                error.occurrence = Some(id.occurrence.clone());
                return Err(error);
            }
            selected = Some(row.clone());
        }
    }
    selected.ok_or_else(|| {
        let mut error = SelectedFactsError::new(SelectedFactsErrorCode::MissingRow, &id.application, meter.used);
        error.occurrence = Some(id.occurrence.clone());
        error
    })
}
