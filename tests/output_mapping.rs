//! TC-043 exercises the target-neutral FS06 output-mapping foundation.

use quire_contract_ir::{
    map_admitted_request, AdmittedMappingObligation, AdmittedMappingRequest, BoundPackage,
    ClauseId, ClauseRef, MappingCancellation, MappingCandidate, MappingCause, MappingCondition,
    MappingDependencyKind, MappingDependencyRef, MappingDisposition, MappingLimits,
    MappingRequestError, MappingRequestErrorCode, MappingRuleDigest, MappingWorkBudget,
    ModelSourceSelection, NativeSourceSelection, ObservationAdequacyRef, ObservationAdequacyState,
    OutputByteRegion, OutputCapability, OutputMapper, OutputMappingProfile, PackageId,
    ProtocolAdequacyRef, ProtocolAdequacyState, RequestedMappingObligation, RequirementId,
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
    vec![OutputByteRegion::new(0, fragment.len() as u64).expect("local output region")]
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
}

impl DeterministicMapper {
    fn new(profile: OutputMappingProfile) -> Self {
        Self {
            profile,
            seen: vec![],
            override_identity: None,
            override_source_state: None,
            work: 2,
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
        let fragment = format!("-- {}\n", obligation.identity().clause()).into_bytes();
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
