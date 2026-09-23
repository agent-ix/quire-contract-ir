//! Real observation/QSL/QProtocol owner fixtures for FR-025 integration tests.

use quire_observation::authority::{
    self, AuthoritySelection, Context, History, Limits as ObservationLimits, OpenClosed,
    SubjectSelection, TemporalBoundary,
};
use quire_observation::{
    admit, AdmissionOutcome, AdmissionRequest, AdmittedRecord, Anchor, ClockRange, Digest,
    Identity, Member, ObservationBinding, PackageSelection, ProducerSelection,
    QualifiedObservation, ResourceLimits, ScopeKind, ScopeSelection, Subject, SubjectKind,
    ValueState, Visibility, NATIVE_LINKED_PACKAGE_FORMAT, PRODUCER_INTERFACE_VERSION,
};
use quire_protocol::closure::{AssessmentExecution, GlobalConformanceClosure};
use quire_protocol::repro::{CorpusIdentity, ReproductionInputs, Seed, ToolchainIdentity};
use quire_protocol::result::{
    self, Activation, Adequacy, Assumption, AssumptionState, AxisInputs, Claim, ClaimKind,
    ClaimStrength, CorrectionInput, CorrectionRelation, DecisionSupport, DependenceRelation,
    GlobalConformanceInput, GlobalPremise, Input, Limitation, Limits, Participation, PremiseKind,
    ProtocolAdequacy, ProvenanceRecord, Settlement, SettlementBasis, Truth,
};
use quire_protocol::{v2, wire as w};
use quire_spec_language::protocol_artifact::{checked_predicate, temporal_subject};

use super::v2_handoff;

pub struct Fixture {
    predicates: Vec<checked_predicate::ValidatedCheckedPredicate>,
    temporal: temporal_subject::ValidatedTemporalSubject,
    decision: ScopeViews,
    surrounding: ScopeViews,
}

impl Fixture {
    pub fn predicate(&self) -> &checked_predicate::ValidatedCheckedPredicate {
        self.predicates.first().expect("fixture checked predicate")
    }

    #[allow(
        dead_code,
        reason = "shared fixture; only the FR-026 test binary consumes every temporal leaf"
    )]
    pub fn predicates(&self) -> &[checked_predicate::ValidatedCheckedPredicate] {
        &self.predicates
    }

    #[allow(
        dead_code,
        reason = "shared fixture; only the FR-026 test binary consumes the temporal owner view"
    )]
    pub fn temporal(&self) -> &temporal_subject::ValidatedTemporalSubject {
        &self.temporal
    }
}

struct ScopeViews {
    progress_open: authority::progress::View,
    progress_closed: authority::progress::View,
    closure_open: authority::closure::View,
    closure_closed: authority::closure::View,
    completeness_complete: authority::completeness::View,
    completeness_incomplete: authority::completeness::View,
    completeness_contradicted: authority::completeness::View,
}

pub fn fixture() -> Fixture {
    fixture_for_temporal(0)
}

#[allow(
    dead_code,
    reason = "shared fixture; FR-026 selects multiple published temporal declarations"
)]
pub fn fixture_for_temporal(temporal_ordinal: usize) -> Fixture {
    let loaded = v2_handoff::Handoff::load().expect("published v2 handoff");
    let declarations = loaded.declarations();
    let inventories = loaded.inventories(&declarations);
    let expected = loaded.expected(&inventories);
    let admitted = quire_protocol::intake::v2::admit(loaded.offer(), &expected, loaded.limits())
        .into_result()
        .expect("strict v2 package");
    let temporal_declaration = admitted
        .inherited()
        .declarations
        .iter()
        .enumerate()
        .filter(|(_, declaration)| matches!(declaration.body, w::Body::Temporal { .. }))
        .nth(temporal_ordinal)
        .and_then(|(index, _)| u32::try_from(index).ok())
        .expect("temporal declaration");
    fixture_from_package(&admitted, temporal_declaration)
}

#[allow(
    dead_code,
    reason = "shared fixture; FR-026 authors targeted temporal owner packages"
)]
pub fn fixture_from_package(admitted: &v2::AdmittedPackage, temporal_declaration: u32) -> Fixture {
    let predicates = temporal_checked_leaves(admitted, temporal_declaration)
        .into_iter()
        .map(|(predicate_declaration, predicate_root)| {
            let predicate_selection =
                checked_predicate::ClauseSelection::new(predicate_declaration, predicate_root);
            let predicate_document = checked_predicate::derive(
                admitted,
                predicate_selection.clone(),
                checked_predicate::Limits::default(),
            )
            .into_result()
            .expect("checked predicate document");
            checked_predicate::read(
                predicate_document.bytes(),
                admitted,
                predicate_selection,
                checked_predicate::Limits::default(),
            )
            .into_result()
            .expect("checked predicate view")
        })
        .collect();
    let temporal_selection = temporal_subject::DeclarationSelection::new(temporal_declaration);
    let temporal_document = temporal_subject::derive(
        admitted,
        temporal_selection,
        temporal_subject::Limits::default(),
    )
    .into_result()
    .expect("temporal subject document");
    let temporal = temporal_subject::read(
        temporal_document.bytes(),
        admitted,
        temporal_selection,
        temporal_subject::Limits::default(),
    )
    .into_result()
    .expect("temporal subject view");
    Fixture {
        predicates,
        temporal,
        decision: scope_views("decision"),
        surrounding: scope_views("surrounding"),
    }
}

pub fn validated_result(fixture: &Fixture) -> result::ValidatedResult {
    validated_result_with(
        fixture,
        AssessmentExecution::Completed,
        Truth::Satisfied,
        SettlementBasis::ClosedScope,
    )
}

pub fn validated_result_with(
    fixture: &Fixture,
    execution: AssessmentExecution,
    truth: Truth,
    settlement: SettlementBasis,
) -> result::ValidatedResult {
    let mut input = base_input(fixture);
    input.execution = execution;
    input.truth = truth;
    input.settlement.basis = settlement;
    let document = result::produce(input.clone(), Limits::owner_max()).expect("canonical result");
    result::read(
        document.bytes(),
        result::Expected {
            input,
            enclosing_digest: document.digest(),
        },
        Limits::owner_max(),
    )
    .expect("strict result reader")
}

#[allow(clippy::too_many_arguments)]
#[allow(
    dead_code,
    reason = "shared fixture; only the FR-026 test binary constructs temporal result axes"
)]
pub fn validated_result_for_temporal<'a>(
    fixture: &'a Fixture,
    decision_progress: &'a authority::progress::View,
    decision_closure: &'a authority::closure::View,
    surrounding_progress: &'a authority::progress::View,
    surrounding_closure: &'a authority::closure::View,
    completeness: &'a authority::completeness::View,
    observation_identity: &'a str,
    truth: Truth,
) -> result::ValidatedResult {
    validated_result_for_temporal_predicate(
        fixture,
        fixture.predicate(),
        decision_progress,
        decision_closure,
        surrounding_progress,
        surrounding_closure,
        completeness,
        observation_identity,
        truth,
    )
}

#[allow(clippy::too_many_arguments)]
pub fn validated_result_for_temporal_predicate<'a>(
    fixture: &'a Fixture,
    checked_predicate: &'a checked_predicate::ValidatedCheckedPredicate,
    decision_progress: &'a authority::progress::View,
    decision_closure: &'a authority::closure::View,
    surrounding_progress: &'a authority::progress::View,
    surrounding_closure: &'a authority::closure::View,
    completeness: &'a authority::completeness::View,
    observation_identity: &'a str,
    truth: Truth,
) -> result::ValidatedResult {
    let mut input = base_input_for(fixture, checked_predicate);
    input.axes = AxisInputs {
        decision_scope_progress: decision_progress,
        decision_scope_closure: decision_closure,
        surrounding_execution_progress: surrounding_progress,
        surrounding_execution_closure: surrounding_closure,
    };
    input.completeness = completeness;
    input.truth = truth;
    input.settlement.basis = match (decision_closure.payload().state(), truth) {
        (_, Truth::Unavailable) => SettlementBasis::Unavailable,
        (_, Truth::Pending) => SettlementBasis::Unsettled,
        (OpenClosed::Open, Truth::Satisfied) => SettlementBasis::DecisiveWitness,
        (OpenClosed::Open, Truth::Violated) => SettlementBasis::DecisiveCounterexample,
        (OpenClosed::Closed, Truth::Satisfied | Truth::Violated) => SettlementBasis::ClosedScope,
    };
    input.settlement.supporting_fact_identities = vec![observation_identity.to_owned()];
    input.settlement.supporting_progress_identities =
        vec![decision_progress.identity().as_str().to_owned()];
    input.decision_support[0].observation_identity = observation_identity.to_owned();
    let document = result::produce(input.clone(), Limits::owner_max()).expect("temporal result");
    result::read(
        document.bytes(),
        result::Expected {
            input,
            enclosing_digest: document.digest(),
        },
        Limits::owner_max(),
    )
    .expect("temporal result strict reader")
}

pub fn validated_result_with_outside_gap(
    fixture: &Fixture,
    contradicted: bool,
) -> result::ValidatedResult {
    let mut input = base_input(fixture);
    input.axes.decision_scope_progress = &fixture.decision.progress_open;
    input.axes.decision_scope_closure = &fixture.decision.closure_open;
    input.settlement.basis = SettlementBasis::DecisiveWitness;
    input.settlement.supporting_progress_identities = vec![fixture
        .decision
        .progress_open
        .identity()
        .as_str()
        .to_owned()];
    input.completeness = if contradicted {
        &fixture.decision.completeness_contradicted
    } else {
        &fixture.decision.completeness_incomplete
    };
    let document = result::produce(input.clone(), Limits::owner_max()).expect("gap result");
    result::read(
        document.bytes(),
        result::Expected {
            input,
            enclosing_digest: document.digest(),
        },
        Limits::owner_max(),
    )
    .expect("gap result strict reader")
}

pub fn corrected_result(
    fixture: &Fixture,
    predecessor: &result::ValidatedResult,
) -> result::ValidatedResult {
    let mut input = base_input(fixture);
    input.execution = AssessmentExecution::Unsupported;
    input.truth = Truth::Unavailable;
    input.settlement.basis = SettlementBasis::Unavailable;
    input.provenance.input_identity = "input:corrected".into();
    input.correction = Some(CorrectionInput {
        relation: CorrectionRelation::Invalidates,
        predecessor,
        contradicted_premise_identity: "premise:branch",
        corrected_input_identity: "input:corrected",
    });
    let document = result::produce(input.clone(), Limits::owner_max()).expect("corrected result");
    result::read(
        document.bytes(),
        result::Expected {
            input,
            enclosing_digest: document.digest(),
        },
        Limits::owner_max(),
    )
    .expect("corrected result strict reader")
}

pub fn contradicted_non_value(fixture: &Fixture) -> result::ValidatedResult {
    let mut input = base_input(fixture);
    input.execution = AssessmentExecution::Unsupported;
    input.truth = Truth::Unavailable;
    input.settlement.basis = SettlementBasis::Unavailable;
    input.completeness = &fixture.decision.completeness_contradicted;
    let document =
        result::produce(input.clone(), Limits::owner_max()).expect("contradicted result");
    result::read(
        document.bytes(),
        result::Expected {
            input,
            enclosing_digest: document.digest(),
        },
        Limits::owner_max(),
    )
    .expect("contradicted result strict reader")
}

pub fn availability(
    result_identity: &str,
    state: authority::availability::State,
) -> authority::availability::View {
    let qualified = qualified("decision");
    let owner = owner("decision");
    let subject = subject("decision", &qualified);
    let context = Context::new(History::batch(&qualified), &owner, &subject, 1, None);
    let required = vec![id(result_identity)];
    let available = if state == authority::availability::State::Available {
        required.clone()
    } else {
        Vec::new()
    };
    let producer = if state == authority::availability::State::ProducerUnavailable {
        authority::availability::DependencyState::Unavailable
    } else {
        authority::availability::DependencyState::Available
    };
    let contract = if state == authority::availability::State::ContractUnavailable {
        authority::availability::DependencyState::Unavailable
    } else {
        authority::availability::DependencyState::Available
    };
    let selection =
        authority::availability::Selection::new(required, available, producer, contract);
    let document =
        authority::availability::derive(context, &selection, ObservationLimits::owner_max())
            .expect("availability document");
    authority::availability::read(
        document.bytes(),
        context,
        &selection,
        ObservationLimits::owner_max(),
    )
    .expect("availability strict reader")
}

fn temporal_checked_leaves(
    package: &v2::AdmittedPackage,
    temporal_declaration: u32,
) -> Vec<(u32, w::Handle)> {
    package
        .inherited()
        .declarations
        .iter()
        .enumerate()
        .find_map(|(index, declaration)| {
            (u32::try_from(index).ok()? == temporal_declaration).then_some(())?;
            let roots = match &declaration.body {
                w::Body::Temporal { activation, .. } => {
                    let mut roots = declaration
                        .temporal
                        .iter()
                        .filter_map(|node| match &node.operation {
                            w::TemporalOperation::Holds { value } => Some(value.clone()),
                            _ => None,
                        })
                        .collect::<Vec<_>>();
                    if let w::Activation::Each {
                        guard: w::Nullable(Some(guard)),
                        ..
                    } = activation
                    {
                        if !roots.contains(guard) {
                            roots.push(guard.clone());
                        }
                    }
                    roots
                }
                _ => return None,
            };
            let declaration = u32::try_from(index).ok()?;
            Some(roots.into_iter().map(|root| (declaration, root)).collect())
        })
        .expect("checked Boolean leaf")
}

fn scope_views(tag: &str) -> ScopeViews {
    let qualified = qualified(tag);
    let owner = owner(tag);
    let subject = subject(tag, &qualified);
    let context = Context::new(History::batch(&qualified), &owner, &subject, 1, None);
    ScopeViews {
        progress_open: progress_view(context, OpenClosed::Open, tag),
        progress_closed: progress_view(context, OpenClosed::Closed, tag),
        closure_open: closure_view(context, OpenClosed::Open, tag),
        closure_closed: closure_view(context, OpenClosed::Closed, tag),
        completeness_complete: completeness_view(
            context,
            &qualified,
            authority::completeness::FactStatus::Available,
            tag,
        ),
        completeness_incomplete: completeness_view(
            context,
            &qualified,
            authority::completeness::FactStatus::Incomplete,
            tag,
        ),
        completeness_contradicted: completeness_view(
            context,
            &qualified,
            authority::completeness::FactStatus::Contradicted,
            tag,
        ),
    }
}

fn id(value: impl Into<String>) -> Identity {
    Identity::new(value)
}

fn digest(value: u8) -> Digest {
    Digest::new([value; 32])
}

fn owner(tag: &str) -> AuthoritySelection {
    AuthoritySelection {
        definition_identity: id(format!("definition:{tag}")),
        definition_revision: id("1"),
        definition_digest: digest(9),
    }
}

fn subject(tag: &str, qualified: &QualifiedObservation) -> SubjectSelection {
    SubjectSelection {
        scope_identity: id(format!("window:{tag}")),
        population_identity: qualified.scope().population_identity.clone(),
    }
}

fn qualified(tag: &str) -> Box<QualifiedObservation> {
    let record_subject = Subject {
        kind: SubjectKind::Order,
        identity: id(format!("order:{tag}")),
    };
    let mut request = AdmissionRequest {
        package: PackageSelection {
            format: NATIVE_LINKED_PACKAGE_FORMAT.to_owned(),
            identity: id(format!("package:{tag}")),
            revision: id("1"),
            digest: digest(1),
        },
        producer: ProducerSelection {
            interface_version: PRODUCER_INTERFACE_VERSION.to_owned(),
            document_identity: id(format!("producer:{tag}")),
            document_digest: digest(2),
            model_identity: id(format!("model:{tag}")),
            configuration_identity: id(format!("configuration:{tag}")),
            configuration_digest: digest(3),
        },
        binding: ObservationBinding {
            identity: id(format!("binding:{tag}")),
            source_identity: id(format!("source:{tag}")),
            schema_identity: id(format!("schema:{tag}")),
            signal_identity: id(format!("signal:{tag}")),
            trigger_identity: id(format!("trigger:{tag}")),
            unit: id("unit"),
            subject_kind: SubjectKind::Order,
            required: true,
        },
        expected_subject: record_subject.clone(),
        relationships: vec![],
        required_relationships: vec![],
        scope: ScopeSelection {
            population_identity: id("unsealed-population"),
            membership_rule_identity: id(format!("membership:{tag}")),
            membership_digest: digest(4),
            membership_document: vec![],
            required_member_identities: vec![id(format!("member:{tag}"))],
            observation_sources: vec![id(format!("source:{tag}"))],
            completeness_dependencies: vec![id(format!("completeness:{tag}"))],
            progress_dependencies: vec![id(format!("progress:{tag}"))],
            clock_identity: id(format!("clock:{tag}")),
            clock_revision: id("1"),
            membership_complete: true,
            closure_identity: Some(id(format!("closure:{tag}"))),
            closure_digest: Some(digest(5)),
            kind: ScopeKind::Window {
                window_identity: id(format!("window:{tag}")),
            },
            range: ClockRange::Timestamp {
                start_nanos: 0,
                end_nanos: 30,
            },
            members: vec![Member {
                object_identity: id(format!("member:{tag}")),
                record_identity: id("unsealed-record"),
                anchor: Anchor::TimestampNanos(10),
            }],
        },
        records: vec![AdmittedRecord {
            identity: id("unsealed-record"),
            binding_identity: id(format!("binding:{tag}")),
            source_identity: id(format!("source:{tag}")),
            schema_identity: id(format!("schema:{tag}")),
            subject: record_subject,
            signal_identity: id(format!("signal:{tag}")),
            trigger_identity: id(format!("trigger:{tag}")),
            unit: id("unit"),
            value: ValueState::Present {
                value_type: id("text"),
                canonical_value: tag.to_owned(),
            },
            visibility: Visibility::External,
            anchor: Anchor::TimestampNanos(10),
            event_time_nanos: 10,
            ingestion_time_nanos: 11,
            causal_relationship_identity: None,
            clock_identity: id(format!("clock:{tag}")),
            clock_revision: id("1"),
            clock_uncertainty_nanos: 1,
        }],
        limits: ResourceLimits {
            max_records: 1,
            max_members: 1,
            max_relationships: 0,
            max_required_relationships: 0,
        },
    };
    let mut outside_record = request.records[0].clone();
    outside_record.identity = id("unsealed-record-outside");
    outside_record.value = ValueState::Present {
        value_type: id("text"),
        canonical_value: format!("{tag}:outside"),
    };
    outside_record.anchor = Anchor::TimestampNanos(20);
    outside_record.event_time_nanos = 20;
    outside_record.ingestion_time_nanos = 21;
    request.records.push(outside_record);
    request
        .scope
        .required_member_identities
        .push(id(format!("member:{tag}:outside")));
    request.scope.members.push(Member {
        object_identity: id(format!("member:{tag}:outside")),
        record_identity: id("unsealed-record-outside"),
        anchor: Anchor::TimestampNanos(20),
    });
    request.limits.max_records = 2;
    request.limits.max_members = 2;
    for index in 0..request.records.len() {
        let record_identity = authority::observation::record_identity(
            &request.records[index],
            ObservationLimits::owner_max(),
        )
        .expect("record identity");
        request.records[index].identity = record_identity.clone();
        request.scope.members[index].record_identity = record_identity;
    }
    request
        .records
        .sort_by(|left, right| left.identity.cmp(&right.identity));
    request
        .scope
        .members
        .sort_by(|left, right| left.object_identity.cmp(&right.object_identity));
    authority::population::assign_request_identities(&mut request, ObservationLimits::owner_max())
        .expect("population identities");
    match admit(request) {
        AdmissionOutcome::Available { observation } => observation,
        other => panic!("qualification failed: {other:?}"),
    }
}

fn boundary(state: OpenClosed) -> TemporalBoundary {
    TemporalBoundary::TimestampedEvent {
        lower_nanos: 0,
        upper_inclusive_nanos: 29,
        carrier_end_exclusive_nanos: 30,
        watermark_nanos: if state == OpenClosed::Closed { 30 } else { 20 },
    }
}

fn cutoff(tag: &str) -> authority::observation::CutoffSelection {
    authority::observation::CutoffSelection::new(
        id(format!("clock:{tag}")),
        id("1"),
        40,
        authority::observation::CutoffRule::IngestionTimeAtOrBefore,
        id("1"),
    )
}

fn progress_view(context: Context<'_>, state: OpenClosed, tag: &str) -> authority::progress::View {
    let selection = authority::progress::Selection::new(
        authority::clock::Selection::new(id(format!("clock:{tag}")), id("1")),
        vec![id(format!("source:{tag}"))],
        boundary(state),
        state,
        id(format!("trigger:{tag}")),
        cutoff(tag),
        id(format!("restoration:{tag}")),
    );
    let document = authority::progress::derive(context, &selection, ObservationLimits::owner_max())
        .expect("progress document");
    authority::progress::read(
        document.bytes(),
        context,
        &selection,
        ObservationLimits::owner_max(),
    )
    .expect("progress reader")
}

fn closure_view(context: Context<'_>, state: OpenClosed, tag: &str) -> authority::closure::View {
    let selection = authority::closure::Selection::new(
        id(format!("clock:{tag}")),
        id("1"),
        vec![id(format!("source:{tag}"))],
        boundary(state),
        state,
    );
    let document = authority::closure::derive(context, &selection, ObservationLimits::owner_max())
        .expect("closure document");
    authority::closure::read(
        document.bytes(),
        context,
        &selection,
        ObservationLimits::owner_max(),
    )
    .expect("closure reader")
}

fn completeness_view(
    context: Context<'_>,
    qualified: &QualifiedObservation,
    outside_status: authority::completeness::FactStatus,
    tag: &str,
) -> authority::completeness::View {
    let first_observation = qualified
        .scope()
        .members
        .iter()
        .find(|member| member.object_identity == id(format!("member:{tag}")))
        .map(|member| member.record_identity.clone())
        .expect("deciding member");
    let outside_observation = qualified
        .scope()
        .members
        .iter()
        .find(|member| member.object_identity == id(format!("member:{tag}:outside")))
        .map(|member| member.record_identity.clone())
        .expect("outside member");
    let selection = authority::completeness::Selection::new(
        id(format!("boundary:{tag}")),
        vec![
            authority::completeness::Fact::new(
                id(format!("member:{tag}")),
                Some(first_observation),
                authority::completeness::FactStatus::Available,
            ),
            authority::completeness::Fact::new(
                id(format!("member:{tag}:outside")),
                (outside_status != authority::completeness::FactStatus::Incomplete)
                    .then_some(outside_observation),
                outside_status,
            ),
        ],
    );
    let document =
        authority::completeness::derive(context, &selection, ObservationLimits::owner_max())
            .expect("completeness document");
    authority::completeness::read(
        document.bytes(),
        context,
        &selection,
        ObservationLimits::owner_max(),
    )
    .expect("completeness reader")
}

fn global_premises() -> Vec<GlobalPremise> {
    [
        PremiseKind::Execution,
        PremiseKind::Branch,
        PremiseKind::Workflow,
    ]
    .into_iter()
    .map(|kind| GlobalPremise {
        kind,
        identity: format!("premise:{kind:?}").to_lowercase(),
        scope_identity: "scope:global".into(),
        authority_identity: "authority:global".into(),
    })
    .collect()
}

fn base_input<'a>(fixture: &'a Fixture) -> Input<'a> {
    base_input_for(fixture, fixture.predicate())
}

fn base_input_for<'a>(
    fixture: &'a Fixture,
    checked_predicate: &'a checked_predicate::ValidatedCheckedPredicate,
) -> Input<'a> {
    let observation_identity = fixture
        .decision
        .completeness_complete
        .payload()
        .facts()
        .next()
        .and_then(|fact| fact.observation_identity)
        .expect("available fact")
        .to_owned();
    let premises = global_premises();
    Input {
        checked_predicate,
        temporal_subject: &fixture.temporal,
        axes: AxisInputs {
            decision_scope_progress: &fixture.decision.progress_closed,
            decision_scope_closure: &fixture.decision.closure_closed,
            surrounding_execution_progress: &fixture.surrounding.progress_closed,
            surrounding_execution_closure: &fixture.surrounding.closure_closed,
        },
        completeness: &fixture.decision.completeness_complete,
        execution: AssessmentExecution::Completed,
        truth: Truth::Satisfied,
        settlement: Settlement {
            basis: SettlementBasis::ClosedScope,
            supporting_fact_identities: vec![observation_identity.clone()],
            supporting_progress_identities: vec![fixture
                .decision
                .progress_closed
                .identity()
                .as_str()
                .to_owned()],
        },
        activation: Activation::Triggered,
        participation: Participation::Known {
            required_role_instance_refs: vec!["role:buyer".into()],
            participating_role_instance_refs: vec!["role:buyer".into()],
        },
        assumptions: vec![Assumption {
            identity: "assumption:one".into(),
            state: AssumptionState::Met,
        }],
        observation_adequacy: Adequacy::Adequate,
        protocol_adequacy: ProtocolAdequacy::Demonstrated,
        claim: Claim {
            kind: ClaimKind::GlobalConformance,
            strength: Some(ClaimStrength::Proven),
            admitted_fragment_identity: Some("fragment:finite".into()),
            backend_identity: Some("backend:model-checker".into()),
        },
        decision_support: vec![DecisionSupport {
            oracle_identity: "oracle:one".into(),
            observation_identity,
            relation: DependenceRelation::Independent,
            predecessor: None,
            dependence_group: None,
        }],
        global_conformance: GlobalConformanceInput {
            state: GlobalConformanceClosure::Closed,
            required_premises: premises.clone(),
            premises,
        },
        limitations: vec![Limitation {
            code: "bound:finite".into(),
            affected_identity: "claim:one".into(),
        }],
        provenance: ProvenanceRecord {
            source_identity: "source:one".into(),
            profile_identity: "profile:one".into(),
            model_identity: "model:one".into(),
            protocol_identity: "protocol:one".into(),
            request_identity: "request:one".into(),
            binding_identity: "binding:one".into(),
            input_identity: "input:one".into(),
            backend_identity: "backend:model-checker".into(),
            producer_identity: "producer:one".into(),
            reproduction: ReproductionInputs::nothing_offered()
                .with_seed(Seed::new(0x5eed_0000_0000_0001))
                .with_toolchain(ToolchainIdentity::new(
                    b"rustc-1.98.0-x86_64-unknown-linux-gnu",
                    "fixture",
                ))
                .with_corpus(CorpusIdentity::new(b"corpus:9f2c1ad4", "fixture"))
                .record()
                .expect("reproduction identity"),
        },
        correction: None,
    }
}
