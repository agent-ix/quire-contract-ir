use ix_trace_rs::trace;
use quire_contract_ir::{
    kani::{
        lower_checked_arithmetic, CapabilityDisposition, CapabilityEntry, DispatchIndex,
        FiniteInput, KaniOutcomeKind, KaniProfile, ModuleDescriptor, PopulationCompleteness,
        ProfileSelection, ResourceBounds, SemanticFamily, PROFILE,
    },
    NumericOperator,
};

fn selection() -> ProfileSelection {
    ProfileSelection {
        profile: PROFILE.into(),
        revision: "kani-bounded/1.0.0".into(),
        executable_digest: "sha256:kani".into(),
        options_digest: "sha256:options".into(),
        abi_revision: "kani-abi/1".into(),
    }
}

fn input() -> quire_contract_ir::kani::ValidatedFiniteInput {
    FiniteInput {
        model_id: "model".into(),
        source_id: "clause".into(),
        profile: selection(),
        completeness: PopulationCompleteness::Complete,
        bounds: ResourceBounds {
            max_objects: 1,
            max_references: 0,
            max_input_bytes: 16,
        },
        input_bytes: 1,
        objects: vec![quire_contract_ir::kani::FiniteObject {
            identity: "self".into(),
            type_id: "Demo::Scalar".into(),
            snapshot_id: "snapshot".into(),
        }],
        references: vec![],
    }
    .validate()
    .expect("finite input")
}

fn profile() -> KaniProfile {
    KaniProfile::new(
        selection(),
        vec![CapabilityEntry {
            construct: "checked-arithmetic".into(),
            disposition: CapabilityDisposition::Supported {
                module: "kani-definedness-arithmetic/1".into(),
            },
        }],
    )
    .expect("profile")
}

fn dispatch() -> DispatchIndex {
    DispatchIndex::new(vec![ModuleDescriptor {
        module_id: "kani-definedness-arithmetic/1".into(),
        family: SemanticFamily::DefinednessArithmetic,
        abi_revision: "kani-abi/1".into(),
        constructs: vec!["checked-arithmetic".into()],
    }])
    .expect("dispatch")
}

#[trace("TC-042", "FR-030-AC-2", "FR-031-AC-1")]
#[test]
fn tc_042_checked_arithmetic_refuses_undefined_and_out_of_range_lowerings() {
    let zero = lower_checked_arithmetic(
        &profile(),
        &dispatch(),
        &input(),
        quire_contract_ir::kani::CheckedArithmeticRequest {
            source_id: "clause",
            operator: NumericOperator::Divide,
            left: 5,
            right: 0,
            minimum: -10,
            maximum: 10,
        },
    )
    .expect_err("zero divisor refuses before a harness");
    assert_eq!(zero.kind, KaniOutcomeKind::Refused);
    assert_eq!(zero.code, "kani_definedness_nonzero_divisor");
    assert_eq!(zero.boolean_claim(), None);

    let out_of_range = lower_checked_arithmetic(
        &profile(),
        &dispatch(),
        &input(),
        quire_contract_ir::kani::CheckedArithmeticRequest {
            source_id: "clause",
            operator: NumericOperator::Multiply,
            left: 5,
            right: 5,
            minimum: -10,
            maximum: 10,
        },
    )
    .expect_err("range overflow refuses before a harness");
    assert_eq!(out_of_range.code, "kani_definedness_checked_range");
    assert_eq!(out_of_range.boolean_claim(), None);
}

#[trace("TC-042", "FR-029-AC-1", "FR-031-AC-1")]
#[test]
fn tc_042_checked_arithmetic_emits_exact_plan_only_after_profile_and_dispatch_selection() {
    let plan = lower_checked_arithmetic(
        &profile(),
        &dispatch(),
        &input(),
        quire_contract_ir::kani::CheckedArithmeticRequest {
            source_id: "clause",
            operator: NumericOperator::Subtract,
            left: 3,
            right: 8,
            minimum: -10,
            maximum: 10,
        },
    )
    .expect("exact arithmetic plan");
    assert_eq!(plan.value, -5);
    assert_eq!(plan.request.operator, NumericOperator::Subtract);
}
