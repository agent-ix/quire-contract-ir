//! Private, owned FCD origin retention for selected-document intake refusals.
//! The borrowed view is also the relationship admission grammar.

use super::{units, valid_semantic_identity, valid_semver, valid_source_path, Budget};
use crate::checked_package::v2::ValidationFailure;
use serde_json::Value;

#[derive(Debug, Eq, PartialEq)]
pub(in crate::checked_package::v2) enum IntakeDeclarationOrigin {
    Source {
        source_identity: Box<str>,
        path: Box<str>,
        start_line: u64,
        start_column: u64,
        end_line: Option<u64>,
        end_column: Option<u64>,
    },
    Generated {
        generator_identity: Box<str>,
        generator_version: Box<str>,
        input_identities: Vec<Box<str>>,
    },
}

pub(super) enum OriginView<'v> {
    Source {
        source_identity: &'v str,
        path: &'v str,
        start_line: u64,
        start_column: u64,
        end_line: Option<u64>,
        end_column: Option<u64>,
    },
    Generated {
        generator_identity: &'v str,
        generator_version: &'v str,
        inputs: &'v [Value],
    },
}

impl<'v> OriginView<'v> {
    pub(super) fn decode(value: &'v Value) -> Option<Self> {
        let origin = value.as_object()?;
        match (origin.get("source"), origin.get("generated"), origin.len()) {
            (Some(source), None, 1) => {
                let members = source.as_object()?;
                if !(4..=6).contains(&members.len())
                    || !members.keys().all(|key| {
                        matches!(
                            key.as_str(),
                            "sourceIdentity"
                                | "path"
                                | "startLine"
                                | "startColumn"
                                | "endLine"
                                | "endColumn"
                        )
                    })
                {
                    return None;
                }
                let source_identity = source.get("sourceIdentity")?.as_str()?;
                let path = source.get("path")?.as_str()?;
                if !valid_semantic_identity(source_identity) || !valid_source_path(path) {
                    return None;
                }
                let positive = |member| source.get(member)?.as_u64().filter(|number| *number > 0);
                let optional = |member| match source.get(member) {
                    None => Some(None),
                    Some(_) => positive(member).map(Some),
                };
                Some(Self::Source {
                    source_identity,
                    path,
                    start_line: positive("startLine")?,
                    start_column: positive("startColumn")?,
                    end_line: optional("endLine")?,
                    end_column: optional("endColumn")?,
                })
            }
            (None, Some(generated), 1) => {
                let members = generated.as_object()?;
                if members.len() != 3
                    || !members.keys().all(|key| {
                        matches!(
                            key.as_str(),
                            "generatorIdentity" | "generatorVersion" | "inputIdentities"
                        )
                    })
                {
                    return None;
                }
                let generator_identity = generated.get("generatorIdentity")?.as_str()?;
                let generator_version = generated.get("generatorVersion")?.as_str()?;
                let inputs = generated.get("inputIdentities")?.as_array()?;
                if !valid_semantic_identity(generator_identity)
                    || !valid_semver(generator_version)
                    || inputs.is_empty()
                    || !inputs
                        .iter()
                        .all(|v| v.as_str().is_some_and(valid_semantic_identity))
                {
                    return None;
                }
                Some(Self::Generated {
                    generator_identity,
                    generator_version,
                    inputs,
                })
            }
            _ => None,
        }
    }

    fn into_owned(self) -> Option<IntakeDeclarationOrigin> {
        Some(match self {
            Self::Source {
                source_identity,
                path,
                start_line,
                start_column,
                end_line,
                end_column,
            } => IntakeDeclarationOrigin::Source {
                source_identity: source_identity.into(),
                path: path.into(),
                start_line,
                start_column,
                end_line,
                end_column,
            },
            Self::Generated {
                generator_identity,
                generator_version,
                inputs,
            } => IntakeDeclarationOrigin::Generated {
                generator_identity: generator_identity.into(),
                generator_version: generator_version.into(),
                // decode has already validated every input. No default or repaired identity.
                input_identities: inputs
                    .iter()
                    .map(|input| input.as_str().map(Box::from))
                    .collect::<Option<Vec<_>>>()?,
            },
        })
    }
}

/// The shallow origin grammar has at most one branch; only its input list is
/// variable-length. Charge its members and text bytes before validation/copy.
fn charge_origin(value: &Value, budget: &mut Budget<'_>) -> Result<(), ValidationFailure> {
    budget.charge(1)?;
    let Some(origin) = value.as_object() else {
        return Ok(());
    };
    for branch in origin.values() {
        budget.charge(1)?;
        let Some(members) = branch.as_object() else {
            continue;
        };
        for value in members.values() {
            budget.charge(1)?;
            match value {
                Value::String(text) => budget.charge(units(text.len()))?,
                Value::Array(inputs) => {
                    for input in inputs {
                        budget.charge(1)?;
                        if let Some(text) = input.as_str() {
                            budget.charge(units(text.len()))?;
                        }
                    }
                }
                _ => {}
            }
        }
    }
    Ok(())
}

pub(super) fn retained_origin(
    value: Option<&Value>,
    budget: &mut Budget<'_>,
) -> Result<Option<IntakeDeclarationOrigin>, ValidationFailure> {
    let Some(value) = value else {
        return Ok(None);
    };
    charge_origin(value, budget)?;
    Ok(OriginView::decode(value).and_then(OriginView::into_owned))
}
