//! TC-043 exercises the target-neutral FS06 output-mapping foundation.

use quire_contract_ir::{
    AdmittedMappingRequest, BoundPackage, ClauseId, ClauseRef, MappingCancellation, MappingLimits,
    MappingRequestErrorCode, MappingRuleDigest, ModelSourceSelection, NativeSourceSelection,
    OutputCapability, OutputMappingProfile, PackageId, RequestedMappingObligation, RequirementId,
    RequirementRef, RequirementRevision, SemanticSourceSelection, SourceBytesDigest,
    SourceFactState, EXECUTABLE_PROJECTION_FORMAT,
};
use serde_json::{json, Value};

fn projection() -> Value {
    let mut package: Value = serde_json::from_str(include_str!(
        "../corpus/contract-v0.1/inputs/package-constructs.json"
    ))
    .expect("shared package fixture");
    let mut bindings = Vec::new();
    for requirement in package["requirements"]
        .as_array_mut()
        .expect("requirements")
    {
        let owner = json!({
            "package": "agent-ix/conformance",
            "requirement": requirement["id"],
            "revision": requirement["revision"]
        });
        for clause in requirement["clauses"].as_array_mut().expect("clauses") {
            clause["body"] = json!({"node": "literal"});
            if clause["kind"] == "information" {
                continue;
            }
            let mut expression: Value = serde_json::from_str(include_str!(
                "../corpus/contract-v0.1/inputs/expression-boolean-literal.json"
            ))
            .expect("boolean expression fixture");
            expression["owner"] = owner.clone();
            expression["execution_point"] = clause["anchor"].clone();
            expression["clause_root"] = json!(true);
            bindings.push(json!({
                "clause": {"requirement": owner, "clause": clause["id"]},
                "expression": expression
            }));
        }
    }
    json!({
        "format": EXECUTABLE_PROJECTION_FORMAT,
        "package": package,
        "bindings": bindings
    })
}

fn bound_package() -> BoundPackage {
    BoundPackage::from_json_bytes(&serde_json::to_vec(&projection()).expect("projection bytes"))
        .expect("strict bound package")
}

fn digest(seed: u8) -> [u8; 32] {
    [seed; 32]
}

fn ocl_profile() -> OutputMappingProfile {
    OutputMappingProfile::new(
        "ocl",
        vec!["formal/14-02-03"],
        "quire.output.ocl24/v1",
        "1-draft.1",
        MappingRuleDigest::from_bytes(digest(1)),
        vec![OutputCapability::Boolean, OutputCapability::BoundedInteger],
    )
    .expect("accepted OCL profile")
}

fn selections() -> (
    NativeSourceSelection,
    ModelSourceSelection,
    SemanticSourceSelection,
) {
    (
        NativeSourceSelection::new(
            "quire.native/v1",
            "rev-native",
            SourceBytesDigest::from_bytes(digest(2)),
        )
        .expect("native selection"),
        ModelSourceSelection::new(
            "quire.model/v1",
            "rev-model",
            SourceBytesDigest::from_bytes(digest(3)),
        )
        .expect("model selection"),
        SemanticSourceSelection::new(
            "quire.semantic/v1",
            "rev-semantic",
            SourceBytesDigest::from_bytes(digest(4)),
        )
        .expect("semantic selection"),
    )
}

fn limits() -> MappingLimits {
    MappingLimits::new(64 * 1024, 32, 1_024, 128, 4_096, 32, 64 * 1024).expect("positive limits")
}

fn requested(package: &BoundPackage) -> Vec<RequestedMappingObligation> {
    package
        .clauses()
        .iter()
        .take(2)
        .map(|clause| {
            RequestedMappingObligation::new(clause.identity().clone(), SourceFactState::Ready)
        })
        .collect()
}

fn admit(
    package: &BoundPackage,
    obligations: Vec<RequestedMappingObligation>,
    profile: OutputMappingProfile,
    limits: MappingLimits,
) -> Result<AdmittedMappingRequest, quire_contract_ir::MappingRequestError> {
    let (native, model, semantic) = selections();
    AdmittedMappingRequest::admit(
        package,
        obligations,
        native,
        model,
        semantic,
        profile,
        limits,
        MappingCancellation::Active,
    )
}

/// Tracing: TC-043, FR-032-AC-1, FR-032-AC-2, FR-032-AC-3.
#[test]
fn tc_043_profiles_admit_only_exact_fs06_values() {
    let ocl = ocl_profile();
    assert_eq!(ocl.target_family().as_str(), "ocl");
    assert_eq!(ocl.mapping_profile_id(), "quire.output.ocl24/v1");
    assert_eq!(ocl.mapping_revision(), "1-draft.1");
    assert_eq!(ocl.target_standard_refs(), ["formal/14-02-03"]);

    let sysml = OutputMappingProfile::new(
        "sysml-kerml",
        vec!["formal/26-03-02", "formal/26-03-01"],
        "quire.output.sysml2-kerml1/v1",
        "1-draft.1",
        MappingRuleDigest::from_bytes(digest(5)),
        vec![
            OutputCapability::Boolean,
            OutputCapability::AttributeReference,
        ],
    )
    .expect("accepted SysML/KerML profile");
    assert_eq!(
        sysml.target_standard_refs(),
        ["formal/26-03-02", "formal/26-03-01"]
    );

    OutputMappingProfile::new(
        "fretish",
        vec!["v3.1.0"],
        "quire.output.fretish31/v1",
        "1-draft.1",
        MappingRuleDigest::from_bytes(digest(6)),
        vec![
            OutputCapability::Boolean,
            OutputCapability::ImmediateResponse,
        ],
    )
    .expect("accepted FRETish profile");

    let invalid = [
        OutputMappingProfile::new(
            "",
            vec!["formal/14-02-03"],
            "quire.output.ocl24/v1",
            "1-draft.1",
            MappingRuleDigest::from_bytes(digest(1)),
            vec![],
        ),
        OutputMappingProfile::new(
            "ocl",
            vec![],
            "quire.output.ocl24/v1",
            "1-draft.1",
            MappingRuleDigest::from_bytes(digest(1)),
            vec![],
        ),
        OutputMappingProfile::new(
            "ocl",
            vec!["formal/26-03-02"],
            "quire.output.sysml2-kerml1/v1",
            "1-draft.1",
            MappingRuleDigest::from_bytes(digest(1)),
            vec![],
        ),
        OutputMappingProfile::new(
            "ocl",
            vec!["formal/14-02-03"],
            "quire.output.ocl24/v1",
            "1-draft.2",
            MappingRuleDigest::from_bytes(digest(1)),
            vec![],
        ),
        OutputMappingProfile::new(
            "ocl",
            vec!["formal/14-02-03"],
            "quire.output.ocl24/v1",
            "1-draft.1",
            MappingRuleDigest::from_bytes(digest(1)),
            vec![OutputCapability::ImmediateResponse],
        ),
        OutputMappingProfile::new(
            "ocl",
            vec!["formal/14-02-03"],
            "quire.output.ocl24/v1",
            "1-draft.1",
            MappingRuleDigest::from_bytes(digest(1)),
            vec![OutputCapability::Boolean, OutputCapability::Boolean],
        ),
    ];
    assert!(invalid.into_iter().all(|result| result.is_err()));
    assert!(OutputCapability::parse("future-capability").is_err());
    assert!(MappingRuleDigest::parse("").is_err());
    assert!(SourceBytesDigest::parse(&"A".repeat(64)).is_err());
}

/// Tracing: TC-043, FR-032-AC-1, FR-032-AC-3, FR-032-AC-4.
#[test]
fn tc_043_request_admission_preserves_exact_ordered_inputs() {
    let package = bound_package();
    let obligations = requested(&package);
    let admitted = admit(&package, obligations.clone(), ocl_profile(), limits())
        .expect("valid mapping request");

    assert_eq!(admitted.source_package().digest(), package.digest());
    assert_eq!(admitted.obligations().len(), 2);
    assert_eq!(
        admitted
            .obligations()
            .iter()
            .map(|obligation| obligation.identity())
            .collect::<Vec<_>>(),
        obligations
            .iter()
            .map(RequestedMappingObligation::identity)
            .collect::<Vec<_>>()
    );
    assert_eq!(admitted.expression_nodes(), 2);
    assert_eq!(admitted.nesting_depth(), 1);
    assert!(admitted.request_bytes() > 0);
    assert_eq!(admitted.profile(), &ocl_profile());
    assert_eq!(admitted.limits(), &limits());
}

/// Tracing: TC-043, FR-032-AC-2, NFR-060.
#[test]
fn tc_043_request_admission_refuses_population_and_resource_mutations() {
    let package = bound_package();
    let valid = requested(&package);

    let cases = [
        (vec![], MappingRequestErrorCode::EmptyObligationSelection),
        (
            vec![valid[0].clone(), valid[0].clone()],
            MappingRequestErrorCode::DuplicateObligation,
        ),
        (
            vec![valid[1].clone(), valid[0].clone()],
            MappingRequestErrorCode::ObligationOrderMismatch,
        ),
        (
            vec![RequestedMappingObligation::new(
                package.informational()[0].clone(),
                SourceFactState::Ready,
            )],
            MappingRequestErrorCode::InformationalObligation,
        ),
        (
            vec![RequestedMappingObligation::new(
                ClauseRef::new(
                    RequirementRef::new(
                        package.package().id().clone(),
                        RequirementId::new("absent").expect("requirement id"),
                        RequirementRevision::new(1).expect("revision"),
                    ),
                    ClauseId::new("absent").expect("clause id"),
                ),
                SourceFactState::Ready,
            )],
            MappingRequestErrorCode::UnknownObligation,
        ),
        (
            vec![RequestedMappingObligation::new(
                ClauseRef::new(
                    RequirementRef::new(
                        PackageId::new("foreign/package").expect("foreign package id"),
                        valid[0].identity().requirement().requirement().clone(),
                        valid[0].identity().requirement().revision(),
                    ),
                    valid[0].identity().clause().clone(),
                ),
                SourceFactState::Ready,
            )],
            MappingRequestErrorCode::ForeignObligation,
        ),
        (
            vec![RequestedMappingObligation::new(
                ClauseRef::new(
                    RequirementRef::new(
                        package.package().id().clone(),
                        valid[0].identity().requirement().requirement().clone(),
                        valid[0]
                            .identity()
                            .requirement()
                            .revision()
                            .advance(2)
                            .expect("stale revision"),
                    ),
                    valid[0].identity().clause().clone(),
                ),
                SourceFactState::Ready,
            )],
            MappingRequestErrorCode::StaleObligation,
        ),
    ];
    for (obligations, code) in cases {
        assert_eq!(
            admit(&package, obligations, ocl_profile(), limits())
                .expect_err("invalid population accepted")
                .code(),
            code
        );
    }

    assert_eq!(
        MappingLimits::new(0, 1, 1, 1, 1, 1, 1)
            .expect_err("zero request limit accepted")
            .code(),
        MappingRequestErrorCode::ZeroLimit
    );
    let one_short = MappingLimits::new(64 * 1024, 1, 1_024, 128, 4_096, 32, 64 * 1024)
        .expect("positive limits");
    assert_eq!(
        admit(&package, valid.clone(), ocl_profile(), one_short)
            .expect_err("over-limit request accepted")
            .code(),
        MappingRequestErrorCode::ObligationLimitExceeded
    );
    let one_short_nodes =
        MappingLimits::new(64 * 1024, 32, 1, 128, 4_096, 32, 64 * 1024).expect("positive limits");
    assert_eq!(
        admit(&package, valid.clone(), ocl_profile(), one_short_nodes,)
            .expect_err("expression node overage accepted")
            .code(),
        MappingRequestErrorCode::ExpressionNodeLimitExceeded
    );
    let one_short_work =
        MappingLimits::new(64 * 1024, 32, 1_024, 128, 1, 32, 64 * 1024).expect("positive limits");
    assert_eq!(
        admit(&package, valid.clone(), ocl_profile(), one_short_work)
            .expect_err("minimum mapping-work overage accepted")
            .code(),
        MappingRequestErrorCode::MappingWorkLimitExceeded
    );
    let one_short_records = MappingLimits::new(64 * 1024, 32, 1_024, 128, 4_096, 1, 64 * 1024)
        .expect("positive limits");
    assert_eq!(
        admit(&package, valid.clone(), ocl_profile(), one_short_records,)
            .expect_err("minimum record overage accepted")
            .code(),
        MappingRequestErrorCode::RecordLimitExceeded
    );
    let measured = admit(&package, valid.clone(), ocl_profile(), limits())
        .expect("measure baseline request")
        .request_bytes();
    let exact_request = MappingLimits::new(measured, 32, 1_024, 128, 4_096, 32, 64 * 1024)
        .expect("exact request-byte limit");
    assert_eq!(
        admit(&package, valid.clone(), ocl_profile(), exact_request)
            .expect("exact request-byte boundary")
            .request_bytes(),
        measured
    );
    let short_request = MappingLimits::new(
        measured.checked_sub(1).expect("nonzero request bytes"),
        32,
        1_024,
        128,
        4_096,
        32,
        64 * 1024,
    )
    .expect("one-short request-byte limit");
    assert_eq!(
        admit(&package, valid.clone(), ocl_profile(), short_request)
            .expect_err("request-byte overage accepted")
            .code(),
        MappingRequestErrorCode::RequestLimitExceeded
    );
    assert_eq!(
        admit(&package, valid.clone(), ocl_profile(), limits(),)
            .expect("baseline request")
            .limits()
            .maximum_obligations(),
        32
    );
    assert_eq!(
        admit(&package, requested(&package), ocl_profile(), limits(),)
            .expect("baseline request")
            .obligations()
            .len(),
        2
    );
    let (native, model, semantic) = selections();
    assert_eq!(
        AdmittedMappingRequest::admit(
            &package,
            requested(&package),
            native,
            model,
            semantic,
            ocl_profile(),
            limits(),
            MappingCancellation::Cancelled,
        )
        .expect_err("cancelled request accepted")
        .code(),
        MappingRequestErrorCode::Cancelled
    );
    assert!(
        NativeSourceSelection::new("", "rev-native", SourceBytesDigest::from_bytes(digest(2)))
            .is_err()
    );
}

/// Tracing: TC-043, FR-032-AC-3, FR-032-AC-4.
#[test]
fn tc_043_request_equality_is_semantic_and_ambient_free() {
    let package = bound_package();
    let first =
        admit(&package, requested(&package), ocl_profile(), limits()).expect("first request");
    let replay =
        admit(&package, requested(&package), ocl_profile(), limits()).expect("replayed request");
    assert_eq!(first, replay);

    let (native, model, semantic) = selections();
    let changed_native =
        NativeSourceSelection::new(native.identity(), "different-revision", native.digest())
            .expect("changed native selection");
    let changed = AdmittedMappingRequest::admit(
        &package,
        requested(&package),
        changed_native,
        model,
        semantic,
        ocl_profile(),
        limits(),
        MappingCancellation::Active,
    )
    .expect("changed request");
    assert_ne!(first, changed);
}
