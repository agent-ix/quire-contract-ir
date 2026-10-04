//! TC-043 exercises the target-neutral FS06 output-mapping foundation.

use ix_trace_rs::trace;
use quire_contract_ir::{
    assemble_output_package, map_admitted_request, map_admitted_request_controlled,
    AdmittedMappingObligation, AdmittedMappingRequest, BoundPackage, ClauseId, ClauseRef,
    DiagnosticCode, MappingAllocationPoint, MappingCancellation, MappingCancellationToken,
    MappingCandidate, MappingCause, MappingCondition, MappingDependencyKind, MappingDependencyRef,
    MappingDisposition, MappingExecutionControl, MappingLimits, MappingRequestError,
    MappingRequestErrorCode, MappingRuleDigest, MappingWorkBudget, ModelSourceSelection,
    NativeSourceSelection, ObservationAdequacyRef, ObservationAdequacyState, OutputByteRegion,
    OutputCapability, OutputGeneratorIdentity, OutputMapper, OutputMappingProfile, PackageId,
    ProtocolAdequacyRef, ProtocolAdequacyState, RequestedMappingObligation, RequirementId,
    RequirementRef, RequirementRevision, SemanticSourceSelection, SourceBytesDigest,
    SourceFactState, StructuralObservationOutcome, StructuralObservationRef,
    StructuralObserverIdentity, TargetBytesDigest, EXECUTABLE_PROJECTION_FORMAT,
};
use serde_json::{json, Value};

fn projection() -> Value {
    let mut package: Value = serde_json::from_str(include_str!(
        "../../corpus/contract-v0.1/inputs/package-constructs.json"
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
                "../../corpus/contract-v0.1/inputs/expression-boolean-literal.json"
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

fn bound_package_with_id(id: &str) -> BoundPackage {
    let mut value = projection();
    value["package"]["id"] = json!(id);
    for binding in value["bindings"].as_array_mut().expect("bindings") {
        binding["clause"]["requirement"]["package"] = json!(id);
        binding["expression"]["owner"]["package"] = json!(id);
    }
    BoundPackage::from_json_bytes(&serde_json::to_vec(&value).expect("projection bytes"))
        .expect("strict renamed bound package")
}

fn nested_bound_package() -> BoundPackage {
    let mut value = projection();
    for binding in value["bindings"].as_array_mut().expect("bindings") {
        let owner = binding["expression"]["owner"].clone();
        let execution_point = binding["expression"]["execution_point"].clone();
        let mut nested: Value = serde_json::from_str(include_str!(
            "../../corpus/contract-v0.1/inputs/expression-boolean-not.json"
        ))
        .expect("nested Boolean expression fixture");
        nested["owner"] = owner;
        nested["execution_point"] = execution_point;
        nested["clause_root"] = json!(true);
        binding["expression"] = nested;
    }
    BoundPackage::from_json_bytes(&serde_json::to_vec(&value).expect("nested projection bytes"))
        .expect("strict nested bound package")
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
#[trace("TC-043", "FR-032-AC-1", "FR-032-AC-2", "FR-032-AC-3")]
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
#[trace("TC-043", "FR-032-AC-1", "FR-032-AC-3", "FR-032-AC-4")]
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
#[trace("TC-043", "FR-032-AC-2")]
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

    let nested = nested_bound_package();
    let nested_request = admit(
        &nested,
        requested(&nested),
        ocl_profile(),
        MappingLimits::new(64 * 1024, 2, 4, 2, 4_096, 2, 64 * 1024)
            .expect("exact nested-expression limits"),
    )
    .expect("exact nested-expression boundary");
    assert_eq!(nested_request.expression_nodes(), 4);
    assert_eq!(nested_request.nesting_depth(), 2);
    assert_eq!(
        admit(
            &nested,
            requested(&nested),
            ocl_profile(),
            MappingLimits::new(64 * 1024, 2, 4, 1, 4_096, 2, 64 * 1024)
                .expect("one-short depth limit"),
        )
        .expect_err("nesting-depth overage accepted")
        .code(),
        MappingRequestErrorCode::NestingDepthLimitExceeded
    );
}

/// Tracing: TC-043, FR-032-AC-3, FR-032-AC-4.
#[trace("TC-043", "FR-032-AC-3", "FR-032-AC-4")]
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

fn condition(code: &str) -> MappingCondition {
    MappingCondition::new(
        "agent-ix/quire-specification",
        "quire.output.ocl24/v1",
        "1-draft.1",
        SourceBytesDigest::from_bytes(digest(7)),
        code,
    )
    .expect("qualified mapping condition")
}

fn cause(code: &str) -> MappingCause {
    MappingCause::new(
        "agent-ix/quire-specification",
        "quire.output.mapping/v1",
        "1-draft.1",
        SourceBytesDigest::from_bytes(digest(8)),
        code,
    )
    .expect("qualified mapping cause")
}

fn dependency(kind: MappingDependencyKind, identity: &str) -> MappingDependencyRef {
    MappingDependencyRef::new(
        kind,
        "agent-ix/owner",
        identity,
        "rev-owner",
        SourceBytesDigest::from_bytes(digest(9)),
    )
    .expect("qualified mapping dependency")
}

fn region_for(fragment: &[u8]) -> Vec<OutputByteRegion> {
    let length = u64::try_from(fragment.len()).expect("test fragment length fits u64");
    vec![OutputByteRegion::new(0, length).expect("local output region")]
}

fn candidate(
    obligation: ClauseRef,
    source_state: SourceFactState,
    disposition: MappingDisposition,
    fragment: &[u8],
    conditions: Vec<MappingCondition>,
    causes: Vec<MappingCause>,
) -> Result<MappingCandidate, MappingRequestError> {
    MappingCandidate::new(
        obligation,
        source_state,
        fragment.to_vec(),
        if fragment.is_empty() {
            vec![]
        } else {
            region_for(fragment)
        },
        vec![dependency(MappingDependencyKind::Semantic, "boolean")],
        disposition,
        conditions,
        causes,
        Some(
            ObservationAdequacyRef::new(
                "agent-ix/quire-observation",
                "quire.observation.result/v1",
                "rev-observation",
                SourceBytesDigest::from_bytes(digest(10)),
                ObservationAdequacyState::Adequate,
            )
            .expect("observation adequacy"),
        ),
        Some(
            ProtocolAdequacyRef::new(
                "agent-ix/quire-protocol",
                "quire.protocol.result/v1",
                "rev-protocol",
                SourceBytesDigest::from_bytes(digest(11)),
                ProtocolAdequacyState::Demonstrated,
            )
            .expect("protocol adequacy"),
        ),
        2,
    )
}

/// Tracing: TC-043, FR-033-AC-2, FR-033-AC-4, FR-033-AC-5.
#[trace("TC-043", "FR-033-AC-2", "FR-033-AC-4", "FR-033-AC-5")]
#[test]
fn tc_043_mapping_candidate_invariants_are_closed_and_non_boolean() {
    let package = bound_package();
    let identity = package.clauses()[0].identity().clone();

    let qualified = candidate(
        identity.clone(),
        SourceFactState::Ready,
        MappingDisposition::Preserved,
        b"true",
        vec![],
        vec![],
    )
    .expect("valid preserved candidate");
    candidate(
        identity.clone(),
        SourceFactState::Ready,
        MappingDisposition::Conditional,
        b"native_bound",
        vec![condition("native-domain-bound")],
        vec![],
    )
    .expect("valid conditional candidate");
    assert_eq!(
        qualified
            .observation_adequacy()
            .map(ObservationAdequacyRef::state),
        Some(ObservationAdequacyState::Adequate)
    );
    assert_eq!(
        qualified
            .protocol_adequacy()
            .map(ProtocolAdequacyRef::state),
        Some(ProtocolAdequacyState::Demonstrated)
    );
    candidate(
        identity.clone(),
        SourceFactState::Pending,
        MappingDisposition::Unrepresented,
        b"",
        vec![],
        vec![cause("source-pending")],
    )
    .expect("valid pending unrepresented candidate");
    candidate(
        identity.clone(),
        SourceFactState::Incomplete,
        MappingDisposition::Unrepresented,
        b"",
        vec![],
        vec![cause("source-incomplete")],
    )
    .expect("valid unrepresented candidate");
    candidate(
        identity.clone(),
        SourceFactState::Refused,
        MappingDisposition::Refused,
        b"",
        vec![],
        vec![cause("source-refused")],
    )
    .expect("valid refused candidate");

    let invalid = [
        candidate(
            identity.clone(),
            SourceFactState::Ready,
            MappingDisposition::Preserved,
            b"",
            vec![],
            vec![],
        ),
        candidate(
            identity.clone(),
            SourceFactState::Ready,
            MappingDisposition::Preserved,
            b"true",
            vec![condition("unexpected")],
            vec![],
        ),
        candidate(
            identity.clone(),
            SourceFactState::Pending,
            MappingDisposition::Preserved,
            b"true",
            vec![],
            vec![],
        ),
        candidate(
            identity.clone(),
            SourceFactState::Ready,
            MappingDisposition::Conditional,
            b"true",
            vec![],
            vec![],
        ),
        candidate(
            identity.clone(),
            SourceFactState::Ready,
            MappingDisposition::Unrepresented,
            b"substitute",
            vec![],
            vec![cause("unsupported")],
        ),
        candidate(
            identity,
            SourceFactState::Ready,
            MappingDisposition::Refused,
            b"",
            vec![],
            vec![],
        ),
    ];
    assert!(invalid.into_iter().all(|result| result.is_err()));
}

/// Tracing: TC-043, FR-033-AC-2, FR-033-AC-4, NFR-060.
#[trace("TC-043", "FR-033-AC-2", "FR-033-AC-4")]
#[test]
fn tc_043_candidate_regions_dependencies_and_work_fail_closed() {
    let package = bound_package();
    let identity = package.clauses()[0].identity().clone();
    let dependency = dependency(MappingDependencyKind::Semantic, "boolean");
    assert!(MappingCandidate::new(
        identity.clone(),
        SourceFactState::Ready,
        vec![0xff],
        vec![OutputByteRegion::new(0, 1).expect("byte region")],
        vec![],
        MappingDisposition::Preserved,
        vec![],
        vec![],
        None,
        None,
        1,
    )
    .is_err());
    assert!(MappingCandidate::new(
        identity.clone(),
        SourceFactState::Ready,
        "β".as_bytes().to_vec(),
        vec![OutputByteRegion::new(1, 2).expect("numerical byte region")],
        vec![],
        MappingDisposition::Preserved,
        vec![],
        vec![],
        None,
        None,
        1,
    )
    .is_err());
    assert!(MappingCandidate::new(
        identity.clone(),
        SourceFactState::Ready,
        b"x".to_vec(),
        vec![OutputByteRegion::new(0, 2).expect("numerical byte region")],
        vec![],
        MappingDisposition::Preserved,
        vec![],
        vec![],
        None,
        None,
        1,
    )
    .is_err());
    assert!(MappingCandidate::new(
        identity.clone(),
        SourceFactState::Ready,
        b"x".to_vec(),
        vec![OutputByteRegion::new(0, 1).expect("byte region")],
        vec![dependency.clone(), dependency],
        MappingDisposition::Preserved,
        vec![],
        vec![],
        None,
        None,
        1,
    )
    .is_err());
    assert!(MappingCandidate::new(
        identity,
        SourceFactState::Ready,
        b"x".to_vec(),
        vec![OutputByteRegion::new(0, 1).expect("byte region")],
        vec![],
        MappingDisposition::Preserved,
        vec![],
        vec![],
        None,
        None,
        0,
    )
    .is_err());
}

struct FixedMapper {
    profile: OutputMappingProfile,
    candidate: MappingCandidate,
}

impl OutputMapper for FixedMapper {
    fn profile(&self) -> &OutputMappingProfile {
        &self.profile
    }

    fn map_obligation(
        &mut self,
        _obligation: &AdmittedMappingObligation,
        _budget: MappingWorkBudget,
    ) -> Result<MappingCandidate, MappingRequestError> {
        Ok(self.candidate.clone())
    }
}

struct OperationalFailureMapper {
    profile: OutputMappingProfile,
    invoked: bool,
}

impl OutputMapper for OperationalFailureMapper {
    fn profile(&self) -> &OutputMappingProfile {
        &self.profile
    }

    fn map_obligation(
        &mut self,
        _obligation: &AdmittedMappingObligation,
        _budget: MappingWorkBudget,
    ) -> Result<MappingCandidate, MappingRequestError> {
        self.invoked = true;
        Err(MappingRequestError::mapper_failed(
            "deterministic target mapper failure",
        ))
    }
}

fn mapped_record_id(
    request: &AdmittedMappingRequest,
    candidate: MappingCandidate,
) -> quire_contract_ir::MappingRecordId {
    let mut mapper = FixedMapper {
        profile: request.profile().clone(),
        candidate,
    };
    map_admitted_request(request, &mut mapper, MappingCancellation::Active)
        .expect("mapped record")
        .records()[0]
        .record_id()
}

fn one_request(
    package: &BoundPackage,
    index: usize,
    source_state: SourceFactState,
    profile: OutputMappingProfile,
) -> AdmittedMappingRequest {
    admit(
        package,
        vec![RequestedMappingObligation::new(
            package.clauses()[index].identity().clone(),
            source_state,
        )],
        profile,
        limits(),
    )
    .expect("single-obligation request")
}

/// Tracing: TC-043, FR-033-AC-3, FR-033-AC-5.
#[trace("TC-043", "FR-033-AC-3", "FR-033-AC-5")]
#[test]
fn tc_043_record_identity_binds_each_semantic_axis() {
    let package = bound_package();
    let request = one_request(&package, 0, SourceFactState::Ready, ocl_profile());
    let identity = request.obligations()[0].identity().clone();
    let baseline = MappingCandidate::new(
        identity.clone(),
        SourceFactState::Ready,
        b"ab".to_vec(),
        vec![OutputByteRegion::new(0, 2).expect("full region")],
        vec![dependency(MappingDependencyKind::Semantic, "boolean")],
        MappingDisposition::Preserved,
        vec![],
        vec![],
        None,
        None,
        1,
    )
    .expect("baseline candidate");
    let baseline_id = mapped_record_id(&request, baseline.clone());

    let changed_dependency = MappingCandidate::new(
        identity.clone(),
        SourceFactState::Ready,
        b"ab".to_vec(),
        vec![OutputByteRegion::new(0, 2).expect("full region")],
        vec![dependency(MappingDependencyKind::Model, "model-binding")],
        MappingDisposition::Preserved,
        vec![],
        vec![],
        None,
        None,
        1,
    )
    .expect("dependency mutation");
    assert_ne!(baseline_id, mapped_record_id(&request, changed_dependency));

    let changed_region = MappingCandidate::new(
        identity.clone(),
        SourceFactState::Ready,
        b"ab".to_vec(),
        vec![OutputByteRegion::new(0, 1).expect("short region")],
        vec![dependency(MappingDependencyKind::Semantic, "boolean")],
        MappingDisposition::Preserved,
        vec![],
        vec![],
        None,
        None,
        1,
    )
    .expect("region mutation");
    assert_ne!(baseline_id, mapped_record_id(&request, changed_region));

    let conditional = MappingCandidate::new(
        identity.clone(),
        SourceFactState::Ready,
        b"ab".to_vec(),
        vec![OutputByteRegion::new(0, 2).expect("full region")],
        vec![dependency(MappingDependencyKind::Semantic, "boolean")],
        MappingDisposition::Conditional,
        vec![condition("native-domain-bound")],
        vec![],
        None,
        None,
        1,
    )
    .expect("conditional mutation");
    assert_ne!(baseline_id, mapped_record_id(&request, conditional));
    let conditional_other = MappingCandidate::new(
        identity.clone(),
        SourceFactState::Ready,
        b"ab".to_vec(),
        vec![OutputByteRegion::new(0, 2).expect("full region")],
        vec![dependency(MappingDependencyKind::Semantic, "boolean")],
        MappingDisposition::Conditional,
        vec![condition("different-condition")],
        vec![],
        None,
        None,
        1,
    )
    .expect("condition-code mutation");
    let conditional_original = MappingCandidate::new(
        identity.clone(),
        SourceFactState::Ready,
        b"ab".to_vec(),
        vec![OutputByteRegion::new(0, 2).expect("full region")],
        vec![dependency(MappingDependencyKind::Semantic, "boolean")],
        MappingDisposition::Conditional,
        vec![condition("native-domain-bound")],
        vec![],
        None,
        None,
        1,
    )
    .expect("original condition code");
    assert_ne!(
        mapped_record_id(&request, conditional_original),
        mapped_record_id(&request, conditional_other)
    );

    let unrepresented_original = MappingCandidate::new(
        identity.clone(),
        SourceFactState::Ready,
        vec![],
        vec![],
        vec![dependency(MappingDependencyKind::Semantic, "boolean")],
        MappingDisposition::Unrepresented,
        vec![],
        vec![cause("unsupported")],
        None,
        None,
        1,
    )
    .expect("original unrepresented cause");
    let unrepresented_other = MappingCandidate::new(
        identity.clone(),
        SourceFactState::Ready,
        vec![],
        vec![],
        vec![dependency(MappingDependencyKind::Semantic, "boolean")],
        MappingDisposition::Unrepresented,
        vec![],
        vec![cause("different-cause")],
        None,
        None,
        1,
    )
    .expect("changed unrepresented cause");
    assert_ne!(
        mapped_record_id(&request, unrepresented_original),
        mapped_record_id(&request, unrepresented_other)
    );

    let with_adequacy = MappingCandidate::new(
        identity.clone(),
        SourceFactState::Ready,
        b"ab".to_vec(),
        vec![OutputByteRegion::new(0, 2).expect("full region")],
        vec![dependency(MappingDependencyKind::Semantic, "boolean")],
        MappingDisposition::Preserved,
        vec![],
        vec![],
        Some(
            ObservationAdequacyRef::new(
                "agent-ix/quire-observation",
                "quire.observation.result/v1",
                "rev-observation",
                SourceBytesDigest::from_bytes(digest(10)),
                ObservationAdequacyState::Unknown,
            )
            .expect("observation adequacy"),
        ),
        None,
        1,
    )
    .expect("adequacy mutation");
    assert_ne!(baseline_id, mapped_record_id(&request, with_adequacy));

    let with_protocol_adequacy = MappingCandidate::new(
        identity.clone(),
        SourceFactState::Ready,
        b"ab".to_vec(),
        vec![OutputByteRegion::new(0, 2).expect("full region")],
        vec![dependency(MappingDependencyKind::Semantic, "boolean")],
        MappingDisposition::Preserved,
        vec![],
        vec![],
        None,
        Some(
            ProtocolAdequacyRef::new(
                "agent-ix/quire-protocol",
                "quire.protocol.result/v1",
                "rev-protocol",
                SourceBytesDigest::from_bytes(digest(11)),
                ProtocolAdequacyState::Unknown,
            )
            .expect("protocol adequacy"),
        ),
        1,
    )
    .expect("protocol-adequacy mutation");
    assert_ne!(
        baseline_id,
        mapped_record_id(&request, with_protocol_adequacy)
    );

    let incomplete_request = one_request(&package, 0, SourceFactState::Incomplete, ocl_profile());
    let incomplete_candidate = MappingCandidate::new(
        incomplete_request.obligations()[0].identity().clone(),
        SourceFactState::Incomplete,
        vec![],
        vec![],
        vec![dependency(MappingDependencyKind::Semantic, "boolean")],
        MappingDisposition::Unrepresented,
        vec![],
        vec![cause("unsupported")],
        None,
        None,
        1,
    )
    .expect("incomplete-source candidate");
    let ready_unrepresented = MappingCandidate::new(
        request.obligations()[0].identity().clone(),
        SourceFactState::Ready,
        vec![],
        vec![],
        vec![dependency(MappingDependencyKind::Semantic, "boolean")],
        MappingDisposition::Unrepresented,
        vec![],
        vec![cause("unsupported")],
        None,
        None,
        1,
    )
    .expect("ready-source unrepresented candidate");
    assert_ne!(
        mapped_record_id(&request, ready_unrepresented),
        mapped_record_id(&incomplete_request, incomplete_candidate)
    );

    let sysml_profile = OutputMappingProfile::new(
        "sysml-kerml",
        vec!["formal/26-03-02", "formal/26-03-01"],
        "quire.output.sysml2-kerml1/v1",
        "1-draft.1",
        MappingRuleDigest::from_bytes(digest(12)),
        vec![OutputCapability::Boolean],
    )
    .expect("SysML profile");
    let sysml_request = one_request(&package, 0, SourceFactState::Ready, sysml_profile);
    let sysml_candidate = MappingCandidate::new(
        sysml_request.obligations()[0].identity().clone(),
        SourceFactState::Ready,
        b"ab".to_vec(),
        vec![OutputByteRegion::new(0, 2).expect("full region")],
        vec![dependency(MappingDependencyKind::Semantic, "boolean")],
        MappingDisposition::Preserved,
        vec![],
        vec![],
        None,
        None,
        1,
    )
    .expect("SysML candidate");
    assert_ne!(
        baseline_id,
        mapped_record_id(&sysml_request, sysml_candidate)
    );

    let second_request = one_request(&package, 1, SourceFactState::Ready, ocl_profile());
    let second_candidate = MappingCandidate::new(
        second_request.obligations()[0].identity().clone(),
        SourceFactState::Ready,
        b"ab".to_vec(),
        vec![OutputByteRegion::new(0, 2).expect("full region")],
        vec![dependency(MappingDependencyKind::Semantic, "boolean")],
        MappingDisposition::Preserved,
        vec![],
        vec![],
        None,
        None,
        1,
    )
    .expect("second source candidate");
    assert_ne!(
        baseline_id,
        mapped_record_id(&second_request, second_candidate)
    );
}

struct DeterministicMapper {
    profile: OutputMappingProfile,
    seen: Vec<ClauseRef>,
    override_identity: Option<ClauseRef>,
    override_source_state: Option<SourceFactState>,
    work: u64,
    prefix: Box<str>,
    cancel_after_first: Option<MappingCancellationToken>,
}

impl DeterministicMapper {
    fn new(profile: OutputMappingProfile) -> Self {
        Self {
            profile,
            seen: vec![],
            override_identity: None,
            override_source_state: None,
            work: 2,
            prefix: "-- ".into(),
            cancel_after_first: None,
        }
    }
}

impl OutputMapper for DeterministicMapper {
    fn profile(&self) -> &OutputMappingProfile {
        &self.profile
    }

    fn map_obligation(
        &mut self,
        obligation: &AdmittedMappingObligation,
        budget: MappingWorkBudget,
    ) -> Result<MappingCandidate, MappingRequestError> {
        assert!(budget.remaining() > 0);
        self.seen.push(obligation.identity().clone());
        let fragment = format!("{}{}\n", self.prefix, obligation.identity().clause()).into_bytes();
        if self.seen.len() == 1 {
            if let Some(token) = &self.cancel_after_first {
                token.cancel();
            }
        }
        MappingCandidate::new(
            self.override_identity
                .clone()
                .unwrap_or_else(|| obligation.identity().clone()),
            self.override_source_state
                .unwrap_or_else(|| obligation.source_state()),
            fragment.clone(),
            region_for(&fragment),
            vec![dependency(MappingDependencyKind::Semantic, "boolean")],
            MappingDisposition::Preserved,
            vec![],
            vec![],
            None,
            None,
            self.work,
        )
    }
}

/// Tracing: TC-043, FR-033-AC-1, FR-033-AC-3, FR-033-AC-5.
#[trace("TC-043", "FR-033-AC-1", "FR-033-AC-3", "FR-033-AC-5")]
#[test]
fn tc_043_mapper_dispatch_is_ordered_complete_and_identity_bearing() {
    let package = bound_package();
    let request = admit(&package, requested(&package), ocl_profile(), limits())
        .expect("admitted mapping request");
    let mut mapper = DeterministicMapper::new(ocl_profile());
    let mapped = map_admitted_request(&request, &mut mapper, MappingCancellation::Active)
        .expect("complete mapped record set");

    assert_eq!(mapper.seen.len(), request.obligations().len());
    assert_eq!(
        mapper.seen.iter().collect::<Vec<_>>(),
        request
            .obligations()
            .iter()
            .map(AdmittedMappingObligation::identity)
            .collect::<Vec<_>>()
    );
    assert_eq!(mapped.records().len(), request.obligations().len());
    assert_eq!(mapped.mapping_work(), 4);
    assert!(mapped.emitted_bytes() > 0);
    assert_ne!(
        mapped.records()[0].record_id(),
        mapped.records()[1].record_id()
    );
    assert_eq!(
        mapped.records()[0]
            .observation_adequacy()
            .map(ObservationAdequacyRef::state),
        None
    );
    assert_eq!(mapped.records()[0].protocol_adequacy(), None);
}

/// Tracing: TC-043, FR-033-AC-1, FR-033-AC-4, NFR-060.
#[trace("TC-043", "FR-033-AC-1", "FR-033-AC-4")]
#[test]
fn tc_043_mapper_mismatch_cross_wiring_and_exhaustion_refuse_atomically() {
    let package = bound_package();
    let request = admit(&package, requested(&package), ocl_profile(), limits())
        .expect("admitted mapping request");

    let sysml = OutputMappingProfile::new(
        "sysml-kerml",
        vec!["formal/26-03-02", "formal/26-03-01"],
        "quire.output.sysml2-kerml1/v1",
        "1-draft.1",
        MappingRuleDigest::from_bytes(digest(12)),
        vec![OutputCapability::Boolean],
    )
    .expect("SysML profile");
    let mut wrong_profile = DeterministicMapper::new(sysml);
    assert_eq!(
        map_admitted_request(&request, &mut wrong_profile, MappingCancellation::Active,)
            .expect_err("cross-profile mapper accepted")
            .code(),
        MappingRequestErrorCode::TargetProfileMismatch
    );
    assert!(wrong_profile.seen.is_empty());

    let mut failed = OperationalFailureMapper {
        profile: ocl_profile(),
        invoked: false,
    };
    assert_eq!(
        map_admitted_request(&request, &mut failed, MappingCancellation::Active)
            .expect_err("operational mapper failure exposed records")
            .code(),
        MappingRequestErrorCode::MapperFailed
    );
    assert!(failed.invoked);

    let mut cross_wired = DeterministicMapper::new(ocl_profile());
    cross_wired.override_identity = Some(request.obligations()[0].identity().clone());
    assert_eq!(
        map_admitted_request(&request, &mut cross_wired, MappingCancellation::Active)
            .expect_err("cross-wired candidate accepted")
            .code(),
        MappingRequestErrorCode::CandidateObligationMismatch
    );

    let tight_limits = MappingLimits::new(64 * 1024, 32, 1_024, 128, 2, 32, 64 * 1024)
        .expect("tight positive work limit");
    let tight_request = admit(&package, requested(&package), ocl_profile(), tight_limits)
        .expect("minimum admitted work budget");
    let mut excessive = DeterministicMapper::new(ocl_profile());
    assert_eq!(
        map_admitted_request(&tight_request, &mut excessive, MappingCancellation::Active,)
            .expect_err("aggregate mapping work overage accepted")
            .code(),
        MappingRequestErrorCode::MappingWorkLimitExceeded
    );

    let tiny_output_limits = MappingLimits::new(64 * 1024, 32, 1_024, 128, 4_096, 32, 1)
        .expect("tiny positive emitted-byte limit");
    let tiny_output_request = admit(
        &package,
        requested(&package),
        ocl_profile(),
        tiny_output_limits,
    )
    .expect("request with tiny output budget");
    let mut too_many_bytes = DeterministicMapper::new(ocl_profile());
    assert_eq!(
        map_admitted_request(
            &tiny_output_request,
            &mut too_many_bytes,
            MappingCancellation::Active,
        )
        .expect_err("emitted-byte overage accepted")
        .code(),
        MappingRequestErrorCode::EmittedBytesLimitExceeded
    );

    let mismatched_state = MappingCandidate::new(
        request.obligations()[0].identity().clone(),
        SourceFactState::Pending,
        vec![],
        vec![],
        vec![],
        MappingDisposition::Unrepresented,
        vec![],
        vec![cause("source-pending")],
        None,
        None,
        1,
    )
    .expect("internally valid pending candidate");
    let mut wrong_state = FixedMapper {
        profile: ocl_profile(),
        candidate: mismatched_state,
    };
    assert_eq!(
        map_admitted_request(&request, &mut wrong_state, MappingCancellation::Active)
            .expect_err("cross-state candidate accepted")
            .code(),
        MappingRequestErrorCode::CandidateSourceStateMismatch
    );

    let mut cancelled = DeterministicMapper::new(ocl_profile());
    assert_eq!(
        map_admitted_request(&request, &mut cancelled, MappingCancellation::Cancelled)
            .expect_err("cancelled mapping accepted")
            .code(),
        MappingRequestErrorCode::Cancelled
    );
    assert!(cancelled.seen.is_empty());
}

/// Tracing: TC-043, FR-033-AC-4, FR-034-AC-3, NFR-060.
#[trace("TC-043", "FR-033-AC-4", "FR-034-AC-3")]
#[test]
fn tc_043_mapping_aggregates_accept_exact_and_refuse_just_over_bounds() {
    let package = bound_package();
    let baseline = mapped(&package, limits());
    let exact_limits = MappingLimits::new(
        64 * 1024,
        2,
        2,
        1,
        baseline.mapping_work(),
        2,
        baseline.emitted_bytes(),
    )
    .expect("exact aggregate limits");
    let exact = mapped(&package, exact_limits);
    assert_eq!(exact.mapping_work(), baseline.mapping_work());
    assert_eq!(exact.emitted_bytes(), baseline.emitted_bytes());

    let short_work = MappingLimits::new(
        64 * 1024,
        2,
        2,
        1,
        baseline.mapping_work() - 1,
        2,
        baseline.emitted_bytes(),
    )
    .expect("one-short work limit");
    let request = admit(&package, requested(&package), ocl_profile(), short_work)
        .expect("request within pre-dispatch work minimum");
    let mut mapper = DeterministicMapper::new(ocl_profile());
    assert_eq!(
        map_admitted_request(&request, &mut mapper, MappingCancellation::Active)
            .expect_err("mapping-work overage accepted")
            .code(),
        MappingRequestErrorCode::MappingWorkLimitExceeded
    );

    let short_output = MappingLimits::new(
        64 * 1024,
        2,
        2,
        1,
        baseline.mapping_work(),
        2,
        baseline.emitted_bytes() - 1,
    )
    .expect("one-short emitted-byte limit");
    let request = admit(&package, requested(&package), ocl_profile(), short_output)
        .expect("request before target bytes exist");
    let mut mapper = DeterministicMapper::new(ocl_profile());
    assert_eq!(
        map_admitted_request(&request, &mut mapper, MappingCancellation::Active)
            .expect_err("emitted-byte overage accepted")
            .code(),
        MappingRequestErrorCode::EmittedBytesLimitExceeded
    );
}

fn generator() -> OutputGeneratorIdentity {
    OutputGeneratorIdentity::new("agent-ix/quire-contract-ir").expect("Rust generator identity")
}

fn mapped(package: &BoundPackage, limits: MappingLimits) -> quire_contract_ir::CompletedMappings {
    let request = admit(package, requested(package), ocl_profile(), limits)
        .expect("admitted mapping request");
    let mut mapper = DeterministicMapper::new(ocl_profile());
    map_admitted_request(&request, &mut mapper, MappingCancellation::Active)
        .expect("complete mapped population")
}

/// Tracing: TC-043, FR-034-AC-1, FR-034-AC-2, FR-034-AC-3.
#[trace("TC-043", "FR-034-AC-1", "FR-034-AC-2", "FR-034-AC-3")]
#[test]
fn tc_043_package_assembly_is_deterministic_complete_and_region_safe() {
    let package = bound_package();
    let mapped = mapped(&package, limits());
    let generated =
        assemble_output_package(&mapped, generator(), &MappingExecutionControl::active())
            .expect("generated output package");
    let replay = assemble_output_package(&mapped, generator(), &MappingExecutionControl::active())
        .expect("replayed output package");

    assert_eq!(generated, replay);
    assert_eq!(generated.target_bytes(), b"-- a_assert\n-- b_case\n");
    assert_eq!(
        generated.target_bytes_digest(),
        TargetBytesDigest::digest(generated.target_bytes())
    );
    assert_eq!(
        generated.records().len(),
        mapped.request().obligations().len()
    );
    assert_eq!(generated.records()[0].output_regions()[0].start(), 0);
    assert_eq!(
        generated.records()[1].output_regions()[0].start(),
        b"-- a_assert\n".len() as u64
    );
    assert_eq!(generated.limits(), mapped.request().limits());
    assert_eq!(
        generated.source_package(),
        mapped.request().source_package()
    );
    assert_eq!(generated.generator(), &generator());
}

/// Tracing: TC-043, FR-032-AC-1, FR-034-AC-1.
#[trace("TC-043", "FR-032-AC-1", "FR-034-AC-1")]
#[test]
fn tc_043_common_assembly_accepts_each_exact_profile_without_claiming_target_semantics() {
    let package = bound_package();
    let profiles = [
        ocl_profile(),
        OutputMappingProfile::new(
            "sysml-kerml",
            vec!["formal/26-03-02", "formal/26-03-01"],
            "quire.output.sysml2-kerml1/v1",
            "1-draft.1",
            MappingRuleDigest::from_bytes(digest(30)),
            vec![OutputCapability::Boolean],
        )
        .expect("SysML/KerML profile"),
        OutputMappingProfile::new(
            "fretish",
            vec!["v3.1.0"],
            "quire.output.fretish31/v1",
            "1-draft.1",
            MappingRuleDigest::from_bytes(digest(31)),
            vec![OutputCapability::Boolean],
        )
        .expect("FRETish profile"),
    ];

    for profile in profiles {
        let request = admit(&package, requested(&package), profile.clone(), limits())
            .expect("profile-specific request");
        let mut mapper = DeterministicMapper::new(profile.clone());
        let mapped = map_admitted_request(&request, &mut mapper, MappingCancellation::Active)
            .expect("target-neutral mapper coordination");
        let generated =
            assemble_output_package(&mapped, generator(), &MappingExecutionControl::active())
                .expect("target-neutral package assembly");
        assert_eq!(generated.target_profile(), &profile);
    }
}

/// Tracing: TC-043, FR-034-AC-1, FR-034-AC-4.
#[trace("TC-043", "FR-034-AC-1", "FR-034-AC-4")]
#[test]
fn tc_043_package_identity_binds_target_generator_source_profile_records_and_limits() {
    let package = bound_package();
    let baseline_mapped = mapped(&package, limits());
    let baseline = assemble_output_package(
        &baseline_mapped,
        generator(),
        &MappingExecutionControl::active(),
    )
    .expect("baseline package");

    let changed_owner = assemble_output_package(
        &baseline_mapped,
        OutputGeneratorIdentity::new("agent-ix/another-generator").expect("other generator owner"),
        &MappingExecutionControl::active(),
    )
    .expect("changed generator owner package");
    assert_ne!(baseline.package_id(), changed_owner.package_id());

    let changed_profile = OutputMappingProfile::new(
        "ocl",
        vec!["formal/14-02-03"],
        "quire.output.ocl24/v1",
        "1-draft.1",
        MappingRuleDigest::from_bytes(digest(29)),
        vec![OutputCapability::Boolean, OutputCapability::BoundedInteger],
    )
    .expect("changed profile digest");
    let changed_profile_request = admit(
        &package,
        requested(&package),
        changed_profile.clone(),
        limits(),
    )
    .expect("changed-profile request");
    let mut changed_profile_mapper = DeterministicMapper::new(changed_profile);
    let changed_profile_mapped = map_admitted_request(
        &changed_profile_request,
        &mut changed_profile_mapper,
        MappingCancellation::Active,
    )
    .expect("changed-profile mapping");
    let changed_profile_package = assemble_output_package(
        &changed_profile_mapped,
        generator(),
        &MappingExecutionControl::active(),
    )
    .expect("changed-profile package");
    assert_ne!(baseline.package_id(), changed_profile_package.package_id());

    let changed_limits = MappingLimits::new(64 * 1024, 33, 1_024, 128, 4_096, 32, 64 * 1024)
        .expect("changed valid limits");
    let changed_limits_mapped = mapped(&package, changed_limits);
    let changed_limits_package = assemble_output_package(
        &changed_limits_mapped,
        generator(),
        &MappingExecutionControl::active(),
    )
    .expect("changed limits package");
    assert_ne!(baseline.package_id(), changed_limits_package.package_id());

    let request = admit(&package, requested(&package), ocl_profile(), limits())
        .expect("admitted mapping request");
    let mut changed_text_mapper = DeterministicMapper::new(ocl_profile());
    changed_text_mapper.prefix = "// ".into();
    let changed_text_mapped = map_admitted_request(
        &request,
        &mut changed_text_mapper,
        MappingCancellation::Active,
    )
    .expect("changed target text mapping");
    let changed_text = assemble_output_package(
        &changed_text_mapped,
        generator(),
        &MappingExecutionControl::active(),
    )
    .expect("changed target text package");
    assert_ne!(
        baseline.target_bytes_digest(),
        changed_text.target_bytes_digest()
    );
    assert_ne!(baseline.package_id(), changed_text.package_id());

    let changed_source_package = bound_package_with_id("agent-ix/conformance-mutated");
    let changed_source_request = admit(
        &changed_source_package,
        requested(&changed_source_package),
        ocl_profile(),
        limits(),
    )
    .expect("changed source-package request");
    let mut changed_source_mapper = DeterministicMapper::new(ocl_profile());
    let changed_source_mapped = map_admitted_request(
        &changed_source_request,
        &mut changed_source_mapper,
        MappingCancellation::Active,
    )
    .expect("changed source mapping");
    let changed_source = assemble_output_package(
        &changed_source_mapped,
        generator(),
        &MappingExecutionControl::active(),
    )
    .expect("changed source package");
    assert_ne!(baseline.package_id(), changed_source.package_id());

    for refused_owner in [
        String::new(),
        "x".repeat(257),
        "agent ix".to_string(),
        "agent-ix/\u{e9}".to_string(),
    ] {
        let error = OutputGeneratorIdentity::new(refused_owner.as_str())
            .expect_err("invalid generator owner accepted");
        assert_eq!(error.code(), MappingRequestErrorCode::InvalidGenerator);
        assert_eq!(error.path(), "generator");
    }
    assert_eq!(
        OutputGeneratorIdentity::new("x".repeat(256))
            .expect("256-byte generator owner is within the bound")
            .owner()
            .len(),
        256
    );
}

/// Tracing: TC-043, FR-034-AC-2, FR-034-AC-3, NFR-060.
#[trace("TC-043", "FR-034-AC-2", "FR-034-AC-3")]
#[test]
fn tc_043_cancellation_and_allocation_failures_expose_no_package() {
    let package = bound_package();
    for point in [
        MappingAllocationPoint::RequestObligations,
        MappingAllocationPoint::RequestIdentity,
    ] {
        let control = MappingExecutionControl::fail_allocation_at(point);
        let (native, model, semantic) = selections();
        assert_eq!(
            AdmittedMappingRequest::admit_controlled(
                &package,
                requested(&package),
                native,
                model,
                semantic,
                ocl_profile(),
                limits(),
                &control,
            )
            .expect_err("request allocation failure exposed a request")
            .code(),
            MappingRequestErrorCode::AllocationFailed
        );
    }
    let (native, model, semantic) = selections();
    assert_eq!(
        AdmittedMappingRequest::admit_controlled(
            &package,
            requested(&package),
            native,
            model,
            semantic,
            ocl_profile(),
            limits(),
            &MappingExecutionControl::cancelled(),
        )
        .expect_err("cancelled controlled admission exposed a request")
        .code(),
        MappingRequestErrorCode::Cancelled
    );

    let mapped = mapped(&package, limits());

    let cancelled = MappingExecutionControl::cancelled();
    assert_eq!(
        assemble_output_package(&mapped, generator(), &cancelled)
            .expect_err("cancelled assembly emitted a package")
            .code(),
        MappingRequestErrorCode::Cancelled
    );
    for point in [
        MappingAllocationPoint::TargetBytes,
        MappingAllocationPoint::PackageRecords,
        MappingAllocationPoint::PackageIdentity,
    ] {
        let control = MappingExecutionControl::fail_allocation_at(point);
        assert_eq!(
            assemble_output_package(&mapped, generator(), &control)
                .expect_err("allocation failure emitted a package")
                .code(),
            MappingRequestErrorCode::AllocationFailed
        );
    }

    let token = MappingCancellationToken::new();
    let control = MappingExecutionControl::with_token(token.clone());
    let request = admit(&package, requested(&package), ocl_profile(), limits())
        .expect("admitted mapping request");
    let mut cancelling_mapper = DeterministicMapper::new(ocl_profile());
    cancelling_mapper.cancel_after_first = Some(token);
    assert_eq!(
        map_admitted_request_controlled(&request, &mut cancelling_mapper, &control)
            .expect_err("mid-mapping cancellation exposed records")
            .code(),
        MappingRequestErrorCode::Cancelled
    );
    assert_eq!(cancelling_mapper.seen.len(), 1);

    for point in [
        MappingAllocationPoint::MappingRecords,
        MappingAllocationPoint::MappingFragments,
        MappingAllocationPoint::MappingRegions,
    ] {
        let control = MappingExecutionControl::fail_allocation_at(point);
        let mut mapper = DeterministicMapper::new(ocl_profile());
        assert_eq!(
            map_admitted_request_controlled(&request, &mut mapper, &control)
                .expect_err("mapping allocation failure exposed records")
                .code(),
            MappingRequestErrorCode::AllocationFailed
        );
    }
}

fn observer() -> StructuralObserverIdentity {
    StructuralObserverIdentity::new("agent-ix/observer").expect("qualified observer")
}

/// Tracing: TC-043, FR-034-AC-4, FR-034-AC-5, NFR-061.
#[trace("TC-043", "FR-034-AC-4", "FR-034-AC-5")]
#[test]
fn tc_043_structural_observations_are_downstream_and_package_immutable() {
    let package = bound_package();
    let mapped = mapped(&package, limits());
    let generated =
        assemble_output_package(&mapped, generator(), &MappingExecutionControl::active())
            .expect("generated package");
    let package_id = generated.package_id();
    let bytes = generated.target_bytes().to_vec();
    let records = generated.records().to_vec();

    for refused_owner in [
        String::new(),
        "x".repeat(257),
        "agent ix".to_string(),
        "agent-ix/\u{e9}".to_string(),
    ] {
        let error = StructuralObserverIdentity::new(refused_owner.as_str())
            .expect_err("invalid observer owner accepted");
        assert_eq!(error.code(), MappingRequestErrorCode::InvalidObserver);
        assert_eq!(error.path(), "observer");
    }
    assert_eq!(
        StructuralObserverIdentity::new("x".repeat(256))
            .expect("256-byte observer owner is within the bound")
            .owner()
            .len(),
        256
    );
    let accepted = StructuralObservationRef::accepted(&generated, observer());
    let refused = StructuralObservationRef::refused(
        &generated,
        StructuralObserverIdentity::new("agent-ix/other-observer")
            .expect("changed observer identity"),
        cause("observer-rights-unavailable"),
    );
    assert_eq!(accepted.package_id(), package_id);
    assert_eq!(accepted.outcome(), StructuralObservationOutcome::Accepted);
    assert_eq!(accepted.observer().owner(), "agent-ix/observer");
    assert_eq!(accepted.refusal_cause(), None);
    assert_eq!(refused.package_id(), package_id);
    assert_eq!(refused.outcome(), StructuralObservationOutcome::Refused);
    assert_eq!(refused.observer().owner(), "agent-ix/other-observer");
    assert_eq!(
        refused.refusal_cause().map(MappingCause::code),
        Some("observer-rights-unavailable")
    );
    assert_eq!(generated.package_id(), package_id);
    assert_eq!(generated.target_bytes(), bytes);
    assert_eq!(generated.records(), records);
    let serialized = serde_json::to_string(&generated).expect("serializable package");
    for excluded in ["observer", "timestamp", "locale", "display", "path"] {
        assert!(!serialized.contains(excluded));
    }
}

/// Tracing: TC-051, STD-003, FR-032-AC-5.
#[trace("TC-051", "STD-003", "FR-032-AC-5")]
#[test]
fn tc_051_mapping_refusal_catalog_is_closed_and_registered() {
    for code in MappingRequestErrorCode::ALL {
        let encoded = serde_json::to_string(code).expect("code serializes");
        assert_eq!(encoded, format!("{:?}", code.as_str()));
        assert_eq!(
            MappingRequestErrorCode::from_code(code.as_str()),
            Some(*code)
        );
    }

    // STD-003: the two registries share no spelling in either direction.
    for code in MappingRequestErrorCode::ALL {
        assert!(
            !DiagnosticCode::ALL
                .iter()
                .any(|diagnostic| diagnostic.as_str() == code.as_str()),
            "{} appears in both refusal registries",
            code.as_str()
        );
    }
    for diagnostic in DiagnosticCode::ALL {
        assert!(
            MappingRequestErrorCode::from_code(diagnostic.as_str()).is_none(),
            "{} appears in both refusal registries",
            diagnostic.as_str()
        );
    }
}

/// Tracing: TC-043, FR-032-AC-5, STD-003.
#[trace("TC-043", "FR-032-AC-5", "STD-003")]
#[test]
fn tc_043_unresolved_obligation_precedence_is_total() {
    let package = bound_package();
    let valid = requested(&package);
    let identity = valid[0].identity();

    // STD-003: an obligation that is foreign *and* carries a revision that
    // would be stale in the bound package classifies as foreign, because
    // package identity is compared before any revision is read. Without the
    // declared order this case could report either code.
    let foreign_and_stale = RequestedMappingObligation::new(
        ClauseRef::new(
            RequirementRef::new(
                PackageId::new("foreign/package").expect("foreign package id"),
                identity.requirement().requirement().clone(),
                identity
                    .requirement()
                    .revision()
                    .advance(2)
                    .expect("stale revision"),
            ),
            identity.clause().clone(),
        ),
        SourceFactState::Ready,
    );
    assert_eq!(
        admit(&package, vec![foreign_and_stale], ocl_profile(), limits())
            .expect_err("foreign obligation accepted")
            .code(),
        MappingRequestErrorCode::ForeignObligation
    );

    // STD-003: a same-package obligation naming a present requirement at
    // another revision is stale, never unknown, even though its clause also
    // fails to resolve.
    let stale = RequestedMappingObligation::new(
        ClauseRef::new(
            RequirementRef::new(
                package.package().id().clone(),
                identity.requirement().requirement().clone(),
                identity
                    .requirement()
                    .revision()
                    .advance(3)
                    .expect("stale revision"),
            ),
            ClauseId::new("absent").expect("clause id"),
        ),
        SourceFactState::Ready,
    );
    assert_eq!(
        admit(&package, vec![stale], ocl_profile(), limits())
            .expect_err("stale obligation accepted")
            .code(),
        MappingRequestErrorCode::StaleObligation
    );

    // STD-003: neither foreign nor stale leaves exactly unknown.
    let unknown = RequestedMappingObligation::new(
        ClauseRef::new(
            RequirementRef::new(
                package.package().id().clone(),
                RequirementId::new("absent").expect("requirement id"),
                RequirementRevision::new(1).expect("revision"),
            ),
            ClauseId::new("absent").expect("clause id"),
        ),
        SourceFactState::Ready,
    );
    assert_eq!(
        admit(&package, vec![unknown], ocl_profile(), limits())
            .expect_err("unknown obligation accepted")
            .code(),
        MappingRequestErrorCode::UnknownObligation
    );
}

/// The smallest `maximum_request_bytes` in `1..=1 << 20` under which `attempt`
/// succeeds. `attempt` succeeds exactly when the limit is at least the longest
/// canonical identity material it encodes, so this is that material's length.
fn smallest_passing_request_limit<T>(
    attempt: impl Fn(u64) -> Result<T, MappingRequestError>,
) -> u64 {
    let (mut low, mut high) = (1_u64, 1_u64 << 20);
    assert!(attempt(high).is_ok(), "the search ceiling passes");
    while low < high {
        let middle = low + (high - low) / 2;
        if attempt(middle).is_ok() {
            high = middle;
        } else {
            low = middle + 1;
        }
    }
    low
}

fn limits_with_request_bytes(maximum_request_bytes: u64) -> MappingLimits {
    MappingLimits::new(maximum_request_bytes, 32, 1_024, 128, 4_096, 32, 64 * 1024)
        .expect("positive limits")
}

/// A record whose condition text is long enough that its identity material is
/// longer than the request's, so the record step is the longest one.
fn long_condition_record(
    package: &BoundPackage,
    maximum_request_bytes: u64,
) -> Result<quire_contract_ir::CompletedMappings, MappingRequestError> {
    let request = admit(
        package,
        vec![RequestedMappingObligation::new(
            package.clauses()[0].identity().clone(),
            SourceFactState::Ready,
        )],
        ocl_profile(),
        limits_with_request_bytes(maximum_request_bytes),
    )?;
    let mut mapper = FixedMapper {
        profile: ocl_profile(),
        candidate: candidate(
            request.obligations()[0].identity().clone(),
            SourceFactState::Ready,
            MappingDisposition::Conditional,
            b"x",
            vec![condition(&"c".repeat(256))],
            vec![],
        )
        .expect("long-condition candidate"),
    };
    map_admitted_request(&request, &mut mapper, MappingCancellation::Active)
}

/// Tracing: TC-043, FR-033-AC-6, FR-034-AC-6.
#[trace("TC-043", "FR-033-AC-6", "FR-034-AC-6")]
#[test]
fn tc_043_record_identity_material_is_metered_by_the_request_byte_limit() {
    let package = bound_package();
    let exact = smallest_passing_request_limit(|limit| long_condition_record(&package, limit));
    assert!(long_condition_record(&package, exact).is_ok());
    let refusal = long_condition_record(&package, exact - 1)
        .expect_err("record material one byte over the limit");
    assert_eq!(
        refusal.code(),
        MappingRequestErrorCode::RequestLimitExceeded
    );
    assert_eq!(
        refusal.path(),
        "record.identity",
        "the request still admits one byte lower, so the record step refuses"
    );
    // The same candidate maps under a larger limit.
    assert!(long_condition_record(&package, exact + 1).is_ok());
}

fn long_generator_package(
    package: &BoundPackage,
    maximum_request_bytes: u64,
) -> Result<quire_contract_ir::GeneratedOutputPackage, MappingRequestError> {
    let request = admit(
        package,
        requested(package).into_iter().take(1).collect(),
        ocl_profile(),
        limits_with_request_bytes(maximum_request_bytes),
    )?;
    let mut mapper = DeterministicMapper::new(ocl_profile());
    let completed = map_admitted_request(&request, &mut mapper, MappingCancellation::Active)?;
    assemble_output_package(
        &completed,
        // Each quote escapes to two bytes, so the owner is 512 bytes of material.
        OutputGeneratorIdentity::new("\"".repeat(256)).expect("long generator identity"),
        &MappingExecutionControl::active(),
    )
}

/// Tracing: TC-043, FR-034-AC-6.
#[trace("TC-043", "FR-034-AC-6")]
#[test]
fn tc_043_package_identity_material_is_metered_by_the_request_byte_limit() {
    let package = bound_package();
    let exact = smallest_passing_request_limit(|limit| long_generator_package(&package, limit));
    let refusal = long_generator_package(&package, exact - 1)
        .expect_err("package material one byte over the limit");
    assert_eq!(
        refusal.code(),
        MappingRequestErrorCode::RequestLimitExceeded
    );
    assert_eq!(
        refusal.path(),
        "package.identity",
        "the request and the records still pass one byte lower, so the package step refuses"
    );
}

/// Tracing: TC-043, FR-032-AC-6, FR-034-AC-6, FR-034-AC-9.
#[trace("TC-043", "FR-032-AC-6", "FR-034-AC-6", "FR-034-AC-9")]
#[test]
fn tc_043_request_material_is_metered_and_admits_a_two_to_the_64_limit() {
    let package = bound_package();
    let obligations = requested(&package);
    let measured = admit(
        &package,
        obligations.clone(),
        ocl_profile(),
        limits_with_request_bytes(1 << 20),
    )
    .expect("measure the request")
    .request_bytes();
    let refusal = admit(
        &package,
        obligations.clone(),
        ocl_profile(),
        limits_with_request_bytes(measured - 1),
    )
    .expect_err("request material one byte over the limit");
    assert_eq!(
        refusal.code(),
        MappingRequestErrorCode::RequestLimitExceeded
    );
    assert_eq!(refusal.path(), "request");
    assert!(admit(
        &package,
        obligations.clone(),
        ocl_profile(),
        limits_with_request_bytes(measured),
    )
    .is_ok());
    for emitted in [u64::MAX, u64::MAX - 1] {
        let limits = MappingLimits::new(1 << 20, 32, 1_024, 128, 4_096, 32, emitted)
            .expect("limit near 2^64");
        assert!(admit(&package, obligations.clone(), ocl_profile(), limits).is_ok());
    }
}

/// Tracing: TC-043, FR-034-AC-7.
#[trace("TC-043", "FR-034-AC-7")]
#[test]
fn tc_043_admitted_request_bytes_are_the_length_of_the_hand_written_request_text() {
    // Every request limit member has a value of its own digit count (1 to 6),
    // so a member swapped with, or copied from, another in the request material
    // changes the length of the material and so `request_bytes`.
    let limits = MappingLimits::new(1 << 20, 55, 7_777, 4, 88_888, 666, 999_999)
        .expect("distinct positive limits");
    let package = bound_package();
    let obligations = requested(&package);
    let admitted = admit(&package, obligations.clone(), ocl_profile(), limits)
        .expect("request with distinct limits admits");

    let obligation_texts: Vec<String> = obligations
        .iter()
        .map(|obligation| {
            let identity = obligation.identity();
            let requirement = identity.requirement();
            format!(
                "{{\"identity\":{{\"clause\":\"{}\",\"requirement\":{{\"package\":\"{}\",\
                 \"requirement\":\"{}\",\"revision\":{}}}}},\"source_state\":\"ready\"}}",
                identity.clause().as_str(),
                requirement.package().as_str(),
                requirement.requirement().as_str(),
                requirement.revision().get(),
            )
        })
        .collect();
    let selection = |seed: u8, identity: &str, revision: &str| {
        format!(
            "{{\"digest\":\"{}\",\"identity\":\"{identity}\",\"revision\":\"{revision}\"}}",
            hex_of(&digest(seed))
        )
    };
    let expected = format!(
        "{{\"identity_version\":\"quire.output.mapping-request-identity/v1-draft.1\",\
         \"model_selection\":{model},\"native_selection\":{native},\
         \"obligations\":[{obligations}],\
         \"resource_shape\":{{\"maximum_emitted_bytes\":\"999999\",\
         \"maximum_expression_nodes\":\"7777\",\"maximum_mapping_work\":\"88888\",\
         \"maximum_nesting_depth\":\"4\",\"maximum_obligations\":\"55\",\
         \"maximum_records\":\"666\"}},\
         \"semantic_selection\":{semantic},\
         \"source_package\":{{\"digest\":\"{package_digest}\",\"package\":\"{package_id}\",\
         \"schema_version\":{{\"major\":1,\"minor\":1}}}},\
         \"target_profile\":{profile}}}",
        model = selection(3, "quire.model/v1", "rev-model"),
        native = selection(2, "quire.native/v1", "rev-native"),
        semantic = selection(4, "quire.semantic/v1", "rev-semantic"),
        obligations = obligation_texts.join(","),
        package_digest = package.digest(),
        package_id = package.package().id().as_str(),
        profile = expected_ocl_profile(),
    );
    assert_eq!(
        admitted.request_bytes(),
        u64::try_from(expected.len()).expect("text length fits u64"),
        "admission keeps the length of the written-out request material"
    );
}

/// The fixture clause `identity` as written in the shared input, so the
/// expected text below is built from the input and not from the code under test.
fn fixture_clause(identity: &ClauseRef) -> Value {
    projection()["package"]["requirements"]
        .as_array()
        .expect("requirements")
        .iter()
        .find(|requirement| requirement["id"] == identity.requirement().requirement().as_str())
        .expect("requirement")["clauses"]
        .as_array()
        .expect("clauses")
        .iter()
        .find(|clause| clause["id"] == identity.clause().as_str())
        .expect("clause")
        .clone()
}

fn fixture_location(location: &Value) -> String {
    format!(
        "{{\"byte_offset\":{},\"column\":{},\"line\":{},\
         \"source\":{{\"document\":{},\"revision\":{}}}}}",
        location["byte_offset"],
        location["column"],
        location["line"],
        location["source"]["document"],
        location["source"]["revision"],
    )
}

fn fixture_anchor(anchor: &Value) -> String {
    let (member, value) = ["operation", "name"]
        .into_iter()
        .find_map(|member| anchor.get(member).map(|value| (member, value)))
        .expect("anchor member");
    format!("{{\"kind\":{},\"{member}\":{value}}}", anchor["kind"])
}

fn hex_of(bytes: &[u8; 32]) -> String {
    bytes.iter().map(|byte| format!("{byte:02x}")).collect()
}

fn expected_ocl_profile() -> String {
    format!(
        "{{\"mapping_digest\":\"{}\",\"mapping_profile_id\":\"quire.output.ocl24/v1\",\
         \"mapping_revision\":\"1-draft.1\",\
         \"required_capabilities\":[\"boolean\",\"bounded-integer\"],\
         \"target_family\":\"ocl\",\"target_standard_refs\":[\"formal/14-02-03\"]}}",
        hex_of(&digest(1))
    )
}

/// Tracing: TC-043, FR-034-AC-7, FR-034-AC-8.
#[trace("TC-043", "FR-034-AC-7", "FR-034-AC-8")]
#[test]
fn tc_043_record_and_package_identities_are_the_digests_of_hand_written_text() {
    use sha2::{Digest, Sha256};

    let package = bound_package();
    let request = one_request(&package, 0, SourceFactState::Ready, ocl_profile());
    let clause = &package.clauses()[0];
    let identity = clause.identity();
    let mut mapper = FixedMapper {
        profile: ocl_profile(),
        candidate: candidate(
            identity.clone(),
            SourceFactState::Ready,
            MappingDisposition::Conditional,
            b"x",
            vec![condition("bounded")],
            vec![],
        )
        .expect("candidate"),
    };
    let completed = map_admitted_request(&request, &mut mapper, MappingCancellation::Active)
        .expect("mapped record");

    let fixture = fixture_clause(identity);
    let requirement = identity.requirement();
    let clause_text = format!(
        "{{\"clause\":\"{}\",\"requirement\":{{\"package\":\"{}\",\"requirement\":\"{}\",\
         \"revision\":{}}}}}",
        identity.clause().as_str(),
        requirement.package().as_str(),
        requirement.requirement().as_str(),
        requirement.revision().get(),
    );
    let record_text = format!(
        "{{\"causes\":[],\
         \"conditions\":[{{\"code\":\"bounded\",\"contract\":\"quire.output.ocl24/v1\",\
         \"digest\":\"{c7}\",\"owner\":\"agent-ix/quire-specification\",\
         \"revision\":\"1-draft.1\"}}],\
         \"dependencies\":[{{\"digest\":\"{c9}\",\"identity\":\"boolean\",\
         \"kind\":\"semantic\",\"owner\":\"agent-ix/owner\",\"revision\":\"rev-owner\"}}],\
         \"disposition\":\"conditional\",\
         \"identity_version\":\"quire.output.mapping-record-identity/v1-draft.1\",\
         \"observation_adequacy\":{{\"contract\":\"quire.observation.result/v1\",\
         \"digest\":\"{c10}\",\"owner\":\"agent-ix/quire-observation\",\
         \"revision\":\"rev-observation\",\"state\":\"adequate\"}},\
         \"output_regions\":[{{\"end\":\"1\",\"start\":\"0\"}}],\
         \"protocol_adequacy\":{{\"contract\":\"quire.protocol.result/v1\",\
         \"digest\":\"{c11}\",\"owner\":\"agent-ix/quire-protocol\",\
         \"revision\":\"rev-protocol\",\"state\":\"demonstrated\"}},\
         \"source\":{{\"anchor\":{anchor},\"declaration_digest\":\"{declaration}\",\
         \"expression_digest\":\"{expression}\",\"identity\":{clause_text},\
         \"kind\":{kind},\
         \"source\":{{\"end\":{end},\"start\":{start}}}}},\
         \"source_state\":\"ready\",\"target_profile\":{profile}}}",
        c7 = hex_of(&digest(7)),
        c9 = hex_of(&digest(9)),
        c10 = hex_of(&digest(10)),
        c11 = hex_of(&digest(11)),
        anchor = fixture_anchor(&fixture["anchor"]),
        declaration = clause.declaration_digest(),
        expression = clause.expression_digest(),
        kind = fixture["kind"],
        end = fixture_location(&fixture["source"]["end"]),
        start = fixture_location(&fixture["source"]["start"]),
        profile = expected_ocl_profile(),
    );
    let record_id = completed.records()[0].record_id();
    assert_eq!(
        record_id.as_bytes().as_slice(),
        Sha256::digest(record_text.as_bytes()).as_slice(),
        "record identity is the digest of the written-out text"
    );

    let assembled =
        assemble_output_package(&completed, generator(), &MappingExecutionControl::active())
            .expect("package");
    let limits = limits();
    let package_text = format!(
        "{{\"generator\":{{\"owner\":\"agent-ix/quire-contract-ir\"}},\
         \"identity_version\":\"quire.output.package-identity/v1-draft.1\",\
         \"limits\":{{\"maximum_emitted_bytes\":\"{emitted}\",\
         \"maximum_expression_nodes\":\"{nodes}\",\"maximum_mapping_work\":\"{work}\",\
         \"maximum_nesting_depth\":\"{depth}\",\"maximum_obligations\":\"{obligations}\",\
         \"maximum_records\":\"{records}\",\"maximum_request_bytes\":\"{request_bytes}\"}},\
         \"record_ids\":[\"{record_id}\"],\
         \"source_package\":{{\"digest\":\"{package_digest}\",\"package\":\"{package_id}\",\
         \"schema_version\":{{\"major\":1,\"minor\":1}}}},\
         \"target_bytes_digest\":\"{target}\",\"target_profile\":{profile}}}",
        emitted = limits.maximum_emitted_bytes(),
        nodes = limits.maximum_expression_nodes(),
        work = limits.maximum_mapping_work(),
        depth = limits.maximum_nesting_depth(),
        obligations = limits.maximum_obligations(),
        records = limits.maximum_records(),
        request_bytes = limits.maximum_request_bytes(),
        record_id = hex_of(record_id.as_bytes()),
        package_digest = package.digest(),
        package_id = package.package().id().as_str(),
        target = hex_of(&Sha256::digest(b"x").into()),
        profile = expected_ocl_profile(),
    );
    assert_eq!(assembled.target_bytes(), b"x");
    assert_eq!(
        assembled.package_id().as_bytes().as_slice(),
        Sha256::digest(package_text.as_bytes()).as_slice(),
        "package identity is the digest of the written-out text"
    );
}
