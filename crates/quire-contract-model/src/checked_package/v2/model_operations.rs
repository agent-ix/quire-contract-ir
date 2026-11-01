// SPDX-License-Identifier: AGPL-3.0-or-later
//! Bounded effective-operation declaration reads from admitted model evidence.

use super::model_members::{
    member_name, Budget, DeclarationForm, MemberKind, ModelFailure, OperationDecl, Resolved,
};
use super::{CheckedMemberType, CheckedNodeId, CheckedPackageV2, WorkMeter};
use crate::{CheckedPackageRefusalCode, JsonPointer};
use std::collections::BTreeMap;
use thiserror::Error;

/// Stable reason a complete model-operation census is unavailable.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum CheckedOperationReadErrorCode {
    /// The context belongs to no selected model package.
    ForeignContext,
    /// The selected package has no admitted node with this identity.
    UnknownNode,
    /// The node is not an operation-bearing model declaration.
    WrongContextKind,
    /// An effective name selects multiple declarations.
    AmbiguousOperation,
    /// The effective member's declaring model node is absent.
    MissingDeclaringNode,
    /// Multiple admitted nodes name the declaring model owner.
    AmbiguousDeclaringNode,
    /// An admitted parameter lacks its required name.
    UnavailableParameterName,
    /// The caller's work provision was exhausted.
    WorkLimit,
}

/// A refused read, containing no partial census.
#[derive(Clone, Debug, Eq, PartialEq, Error)]
#[error("operation census refused: {code:?} at {context:?} (work {work}, limit {limit})")]
pub struct CheckedOperationReadError {
    /// Stable reason.
    pub code: CheckedOperationReadErrorCode,
    /// Requested context.
    pub context: CheckedNodeId,
    /// Charged work, including a denied charge.
    pub work: u64,
    /// Configured work provision.
    pub limit: u64,
}

/// One parameter projected from the retained canonical operation declaration.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CheckedOperationParameter<'a> {
    name: &'a str,
    member_type: Option<CheckedMemberType>,
}

impl CheckedOperationParameter<'_> {
    /// Original parameter name.
    pub fn name(&self) -> &str {
        self.name
    }

    /// Canonical domain, or explicit unavailable-domain information.
    pub fn member_type(&self) -> Option<&CheckedMemberType> {
        self.member_type.as_ref()
    }
}

/// A read-only view borrowing its admitted effective operation declaration.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CheckedModelOperation<'a> {
    declaration: &'a OperationDecl,
    context: &'a CheckedNodeId,
    declaring: &'a CheckedNodeId,
    parameters: Vec<CheckedOperationParameter<'a>>,
    result: Option<Option<CheckedMemberType>>,
}

impl CheckedModelOperation<'_> {
    /// Original selected model member identity; not an inferred Exact stage identity.
    pub fn identity(&self) -> &str {
        &self.declaration.identity
    }

    /// Effective operation name within the requested context.
    pub fn name(&self) -> &str {
        member_name(&self.declaration.identity)
    }

    /// Actual admitted context of this read.
    pub fn context(&self) -> &CheckedNodeId {
        self.context
    }

    /// Actual admitted declaration node of the owner of this member.
    pub fn declaring_node_id(&self) -> &CheckedNodeId {
        self.declaring
    }

    /// Parameters in producer order, without an implicit receiver or result slot.
    pub fn parameters(&self) -> &[CheckedOperationParameter<'_>] {
        &self.parameters
    }

    /// Absent result, present unavailable domain, or present canonical result domain.
    pub fn result(&self) -> Option<Option<&CheckedMemberType>> {
        self.result.as_ref().map(Option::as_ref)
    }
}

/// A complete declaration census, independent of executable-body and dispatch reads.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CheckedModelOperations<'a> {
    operations: Vec<CheckedModelOperation<'a>>,
    work: u64,
}

impl CheckedModelOperations<'_> {
    /// Every effective operation, in ascending name order, including unimplemented ones.
    pub fn operations(&self) -> &[CheckedModelOperation<'_>] {
        &self.operations
    }

    /// Work charged by this complete read.
    pub const fn work(&self) -> u64 {
        self.work
    }
}

fn eligible(form: DeclarationForm) -> bool {
    matches!(
        form,
        DeclarationForm::ObjectType
            | DeclarationForm::Process
            | DeclarationForm::StateMachine
            | DeclarationForm::PersistenceInterface
            | DeclarationForm::SystemsInterface
    )
}

impl CheckedPackageV2 {
    /// Read the complete effective operation declarations of an admitted model context.
    ///
    /// Borrows retained declarations and preserves unavailable domains. No executable body,
    /// selected-stage identity, dispatch candidate or successor is inferred from this read.
    ///
    /// # Errors
    ///
    /// Returns a typed context, ambiguity, declaring-node, parameter-name or work refusal
    /// before exposing any partial census.
    pub fn model_operations(
        &self,
        context: &CheckedNodeId,
        work_limit: u64,
    ) -> Result<CheckedModelOperations<'_>, CheckedOperationReadError> {
        use CheckedOperationReadErrorCode as Code;
        let mut meter = WorkMeter::new(work_limit);
        let failure = |code, work| CheckedOperationReadError {
            code,
            context: context.clone(),
            work,
            limit: work_limit,
        };
        meter
            .charge(1, JsonPointer::root)
            .map_err(|_| failure(Code::WorkLimit, meter.consumed()))?;
        let node = match self.node(context) {
            Some(node) => node,
            None => {
                let mut selected_package = context.package.as_ref() == self.name();
                for model in self.models.selected_models() {
                    meter
                        .charge(1, JsonPointer::root)
                        .map_err(|_| failure(Code::WorkLimit, meter.consumed()))?;
                    selected_package |= model.identity.as_ref() == context.package.as_ref();
                }
                return Err(failure(
                    if selected_package {
                        Code::UnknownNode
                    } else {
                        Code::ForeignContext
                    },
                    meter.consumed(),
                ));
            }
        };
        let owner = self
            .models
            .owner_by_key(context)
            .filter(|owner| eligible(owner.form))
            .ok_or_else(|| failure(Code::WrongContextKind, meter.consumed()))?;
        let model = owner.package;
        let mut declarations = BTreeMap::new();
        for (key, declared) in self.models.selected_declarations() {
            meter
                .charge(1, JsonPointer::root)
                .map_err(|_| failure(Code::WorkLimit, meter.consumed()))?;
            if declared.selection == owner.selection && eligible(declared.form) {
                declarations
                    .entry(declared.node.as_ref())
                    .and_modify(|value| *value = None)
                    .or_insert(Some(key));
            }
        }
        let mut budget = Budget::new(&mut meter, model.root, model.bytes);
        let persistence = model.persistence_operations.get(owner.node);
        let names = if let Some(declared) = persistence {
            budget
                .charge(super::model_members::units(declared.operations.len()))
                .map_err(|_| failure(Code::WorkLimit, budget.work()))?;
            declared
                .operations
                .iter()
                .map(|op| Box::<str>::from(member_name(&op.identity)))
                .collect()
        } else {
            model
                .operation_candidate_names(owner.node, &mut budget)
                .map_err(|_| failure(Code::WorkLimit, budget.work()))?
        };
        let mut operations = Vec::new();
        for name in names {
            let resolved = if let Some(declared) = persistence {
                budget
                    .charge(super::model_members::units(declared.operations.len()))
                    .map_err(|_| failure(Code::WorkLimit, budget.work()))?;
                declared
                    .operations
                    .iter()
                    .find(|op| member_name(&op.identity) == name.as_ref())
                    .map(Resolved::Operation)
                    .ok_or_else(|| {
                        ModelFailure::Refused(super::model_members::ModelRefusal::ineligible())
                    })
            } else {
                model.resolve(owner.node, MemberKind::Operation, &name, &mut budget)
            };
            let declaration = match resolved {
                Ok(Resolved::Operation(operation)) => operation,
                // A differently named redefiner may hide this candidate entirely.
                Ok(Resolved::Field(_)) => continue,
                Err(ModelFailure::Refused(reason)) => {
                    if reason.code == CheckedPackageRefusalCode::AmbiguousDeclaration {
                        return Err(failure(Code::AmbiguousOperation, budget.work()));
                    }
                    continue;
                }
                Err(ModelFailure::Limit(_)) => {
                    return Err(failure(Code::WorkLimit, budget.work()));
                }
            };
            let declaring_name = declaration
                .identity
                .rsplit_once('/')
                .map(|(owner, _)| owner);
            let declaring = match declaring_name.and_then(|name| declarations.get(name)) {
                Some(Some(node)) => *node,
                Some(None) => return Err(failure(Code::AmbiguousDeclaringNode, budget.work())),
                None => return Err(failure(Code::MissingDeclaringNode, budget.work())),
            };
            let mut parameters = Vec::new();
            for parameter in &declaration.parameters {
                budget
                    .charge(1)
                    .map_err(|_| failure(Code::WorkLimit, budget.work()))?;
                parameters.push(CheckedOperationParameter {
                    name: parameter
                        .name
                        .as_deref()
                        .ok_or_else(|| failure(Code::UnavailableParameterName, budget.work()))?,
                    member_type: model
                        .slot_type(parameter)
                        .as_ref()
                        .map(CheckedMemberType::from),
                });
            }
            let result = if let Some(result) = &declaration.result {
                budget
                    .charge(1)
                    .map_err(|_| failure(Code::WorkLimit, budget.work()))?;
                Some(
                    model
                        .slot_type(result)
                        .as_ref()
                        .map(CheckedMemberType::from),
                )
            } else {
                None
            };
            operations.push(CheckedModelOperation {
                declaration,
                context: &node.node_id,
                declaring,
                parameters,
                result,
            });
        }
        Ok(CheckedModelOperations {
            operations,
            work: budget.work(),
        })
    }
}
