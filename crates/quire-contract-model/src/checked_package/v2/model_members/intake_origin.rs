//! Private, owned FCD origin retention for selected-document intake refusals.
//! The borrowed view is also the relationship admission grammar.

use super::{units, valid_semantic_identity, valid_semver, valid_source_path, Budget, List, View};
use crate::checked_package::v2::ValidationFailure;

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
        inputs: List<'v>,
    },
}

impl<'v> OriginView<'v> {
    pub(super) fn decode(value: View<'v>) -> Option<Self> {
        let origin = value.as_object()?;
        match (origin.get("source"), origin.get("generated"), origin.len()) {
            (Some(source), None, 1) => {
                let members = source.as_object()?;
                if !(4..=6).contains(&members.len())
                    || !members.keys().all(|key| {
                        matches!(
                            key,
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
                            key,
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
                // Immutable inputs are nonempty and every element is a validated
                // semantic-identity string. The borrowed view preserves that
                // invariant until into_owned; it performs no second parse.
                Some(Self::Generated {
                    generator_identity,
                    generator_version,
                    inputs,
                })
            }
            _ => None,
        }
    }

    /// The owned origin, each text and the input list requested first.
    fn into_owned(
        self,
        budget: &mut Budget<'_, '_>,
    ) -> Result<Option<IntakeDeclarationOrigin>, ValidationFailure> {
        const F: &str = "declarations";
        Ok(Some(match self {
            Self::Source {
                source_identity,
                path,
                start_line,
                start_column,
                end_line,
                end_column,
            } => IntakeDeclarationOrigin::Source {
                source_identity: budget.text(source_identity, F)?,
                path: budget.text(path, F)?,
                start_line,
                start_column,
                end_line,
                end_column,
            },
            Self::Generated {
                generator_identity,
                generator_version,
                inputs,
            } => {
                let generator_identity = budget.text(generator_identity, F)?;
                let generator_version = budget.text(generator_version, F)?;
                let mut input_identities = budget.vec(inputs.len(), F)?;
                // Every input was validated by decode and the borrowed list
                // cannot change before this copy. The fallible collection is
                // deliberately fail-closed if that shared decoder invariant
                // changes: no element is skipped, defaulted or repaired.
                for input in inputs {
                    let Some(text) = input.as_str() else {
                        return Ok(None);
                    };
                    input_identities.push(budget.text(text, F)?);
                }
                IntakeDeclarationOrigin::Generated {
                    generator_identity,
                    generator_version,
                    input_identities,
                }
            }
        }))
    }
}

/// The shallow origin grammar has at most one branch; only its input list is
/// variable-length. Charge its members and text bytes before validation/copy.
fn charge_origin(value: View<'_>, budget: &mut Budget<'_, '_>) -> Result<(), ValidationFailure> {
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
            if let Some(text) = value.as_str() {
                budget.charge(units(text.len()))?;
            } else if let Some(inputs) = value.as_array() {
                for input in inputs {
                    budget.charge(1)?;
                    if let Some(text) = input.as_str() {
                        budget.charge(units(text.len()))?;
                    }
                }
            }
        }
    }
    Ok(())
}

pub(super) fn retained_origin(
    value: Option<View<'_>>,
    budget: &mut Budget<'_, '_>,
) -> Result<Option<IntakeDeclarationOrigin>, ValidationFailure> {
    let Some(value) = value else {
        return Ok(None);
    };
    charge_origin(value, budget)?;
    match OriginView::decode(value) {
        Some(view) => view.into_owned(budget),
        None => Ok(None),
    }
}
