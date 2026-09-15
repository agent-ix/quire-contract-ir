//! Definedness-preserving checked-arithmetic lowering preparation.

use crate::NumericOperator;

use super::{
    CapabilityDisposition, DispatchIndex, KaniOutcome, KaniOutcomeKind, KaniProfile,
    SemanticFamily, ValidatedFiniteInput,
};

/// A statically admitted concrete arithmetic operation and its exact named range.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct CheckedArithmeticRequest {
    /// Source clause or checked-expression identity.
    pub source_id: &'static str,
    /// Native checked arithmetic operation.
    pub operator: NumericOperator,
    /// Exact left operand in the selected finite domain.
    pub left: i128,
    /// Exact right operand in the selected finite domain.
    pub right: i128,
    /// Inclusive selected result-domain minimum.
    pub minimum: i128,
    /// Inclusive selected result-domain maximum.
    pub maximum: i128,
}

/// Exact arithmetic lowering preimage, not a Kani verdict.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct ArithmeticLowering {
    /// Original checked request.
    pub request: CheckedArithmeticRequest,
    /// Checked mathematical result within the named domain.
    pub value: i128,
}

/// Validates and prepares one definedness-preserving arithmetic lowering.
///
/// This function never invokes Kani and therefore never returns a Boolean proof
/// or counterexample. It only produces an exact plan for a later harness stage.
pub fn lower_checked_arithmetic(
    profile: &KaniProfile,
    dispatch: &DispatchIndex,
    _input: &ValidatedFiniteInput,
    request: CheckedArithmeticRequest,
) -> Result<ArithmeticLowering, KaniOutcome> {
    let entries = profile.classify(&["checked-arithmetic".to_owned()], request.source_id)?;
    match &entries[0].disposition {
        CapabilityDisposition::Supported { module } => {
            let descriptor = dispatch.resolve("checked-arithmetic").map_err(|_| {
                KaniOutcome::non_success(
                    KaniOutcomeKind::Refused,
                    "kani_dispatch_unowned",
                    request.source_id,
                    profile.selection.revision.clone(),
                )
            })?;
            if descriptor.family != SemanticFamily::DefinednessArithmetic
                || descriptor.module_id != *module
            {
                return Err(KaniOutcome::non_success(
                    KaniOutcomeKind::Refused,
                    "kani_dispatch_profile_mismatch",
                    request.source_id,
                    profile.selection.revision.clone(),
                ));
            }
        }
        CapabilityDisposition::Refused { code } => {
            return Err(KaniOutcome::non_success(
                KaniOutcomeKind::Refused,
                code.clone(),
                request.source_id,
                profile.selection.revision.clone(),
            ))
        }
        CapabilityDisposition::Inconclusive { code } => {
            return Err(KaniOutcome::non_success(
                KaniOutcomeKind::Inconclusive,
                code.clone(),
                request.source_id,
                profile.selection.revision.clone(),
            ))
        }
    }
    if request.minimum > request.maximum {
        return Err(KaniOutcome::non_success(
            KaniOutcomeKind::InvalidInput,
            "kani_arithmetic_range_invalid",
            request.source_id,
            profile.selection.revision.clone(),
        ));
    }
    let value = match request.operator {
        NumericOperator::Add => request.left.checked_add(request.right),
        NumericOperator::Subtract => request.left.checked_sub(request.right),
        NumericOperator::Multiply => request.left.checked_mul(request.right),
        NumericOperator::Divide => {
            if request.right == 0 {
                return Err(KaniOutcome::non_success(
                    KaniOutcomeKind::Refused,
                    "kani_definedness_nonzero_divisor",
                    request.source_id,
                    profile.selection.revision.clone(),
                ));
            }
            request.left.checked_div(request.right)
        }
        NumericOperator::Remainder => {
            if request.right == 0 {
                return Err(KaniOutcome::non_success(
                    KaniOutcomeKind::Refused,
                    "kani_definedness_nonzero_divisor",
                    request.source_id,
                    profile.selection.revision.clone(),
                ));
            }
            request.left.checked_rem(request.right)
        }
    };
    let Some(value) = value else {
        return Err(KaniOutcome::non_success(
            KaniOutcomeKind::Refused,
            "kani_definedness_checked_range",
            request.source_id,
            profile.selection.revision.clone(),
        ));
    };
    if value < request.minimum || value > request.maximum {
        return Err(KaniOutcome::non_success(
            KaniOutcomeKind::Refused,
            "kani_definedness_checked_range",
            request.source_id,
            profile.selection.revision.clone(),
        ));
    }
    Ok(ArithmeticLowering { request, value })
}
