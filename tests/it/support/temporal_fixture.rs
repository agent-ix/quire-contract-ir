//! Real event-position observation authority for FR-026 tests.

use quire_observation::authority::{
    self, AuthoritySelection, Context, History, Limits, OpenClosed, SubjectSelection,
    TemporalBoundary,
};
use quire_observation::{
    admit, AdmissionOutcome, AdmissionRequest, AdmittedRecord, Anchor, ClockRange, Digest,
    Identity, Member, ObservationBinding, PackageSelection, ProducerSelection,
    QualifiedObservation, ResourceLimits, ScopeKind, ScopeSelection, Subject, SubjectKind,
    ValueState, Visibility, NATIVE_LINKED_PACKAGE_FORMAT, PRODUCER_INTERFACE_VERSION,
};

use super::result_fixture::Fixture;

pub struct TemporalAuthority {
    qualified: Box<QualifiedObservation>,
    owner: AuthoritySelection,
    subject: SubjectSelection,
    pub position: authority::position::View,
    pub clock: authority::clock::View,
    pub capture: authority::capture::View,
    pub progress: authority::progress::View,
    pub closure: authority::closure::View,
    pub completeness: authority::completeness::View,
    pub observation_identity: String,
}

impl TemporalAuthority {
    pub fn availability(&self, result_identity: &str) -> authority::availability::View {
        self.availability_for(&[result_identity])
    }

    pub fn availability_for(&self, result_identities: &[&str]) -> authority::availability::View {
        let context = Context::new(
            History::batch(&self.qualified),
            &self.owner,
            &self.subject,
            1,
            None,
        );
        let required: Vec<Identity> = result_identities.iter().map(|value| id(*value)).collect();
        let selection = authority::availability::Selection::new(
            required.clone(),
            required,
            authority::availability::DependencyState::Available,
            authority::availability::DependencyState::Available,
        );
        let document = authority::availability::derive(context, &selection, Limits::owner_max())
            .expect("availability document");
        authority::availability::read(document.bytes(), context, &selection, Limits::owner_max())
            .expect("availability reader")
    }
}

pub fn event_position(fixture: &Fixture, tag: &str, state: OpenClosed) -> TemporalAuthority {
    authority(fixture, tag, state, ClockFixture::EventPosition)
}

pub fn fixed_sample(fixture: &Fixture, tag: &str, state: OpenClosed) -> TemporalAuthority {
    authority(fixture, tag, state, ClockFixture::FixedSample)
}

pub fn timestamped_event(fixture: &Fixture, tag: &str, state: OpenClosed) -> TemporalAuthority {
    authority(fixture, tag, state, ClockFixture::TimestampedEvent)
}

#[derive(Clone, Copy)]
enum ClockFixture {
    EventPosition,
    FixedSample,
    TimestampedEvent,
}

fn authority(
    fixture: &Fixture,
    tag: &str,
    state: OpenClosed,
    family: ClockFixture,
) -> TemporalAuthority {
    let clock_identity = fixture
        .temporal()
        .clock()
        .and_then(|(_, clock)| {
            match (family, clock) {
                (
                    ClockFixture::EventPosition,
                    quire_spec_language::protocol_artifact::v2::wire::ClockConfiguration::EventPosition {
                        sequence_authority,
                    },
                ) => Some(sequence_authority.as_str()),
                (
                    ClockFixture::FixedSample,
                    quire_spec_language::protocol_artifact::v2::wire::ClockConfiguration::FixedSample {
                        ..
                    },
                ) => Some("sample-clock"),
                (
                    ClockFixture::TimestampedEvent,
                    quire_spec_language::protocol_artifact::v2::wire::ClockConfiguration::TimestampedEvent {
                        ..
                    },
                ) => Some("timestamp-clock"),
                _ => None,
            }
        })
        .expect("selected temporal clock family");
    let trigger = match fixture
        .temporal()
        .activation()
        .expect("temporal activation")
    {
        quire_spec_language::protocol_artifact::wire::Activation::Origin { anchor } => anchor,
        quire_spec_language::protocol_artifact::wire::Activation::Each { trigger, .. } => trigger,
    };
    let trigger_identity = format!("{}:{}", trigger.declaration, trigger.index);
    let qualified = qualified_event(tag, clock_identity, &trigger_identity, family);
    let owner = owner(tag);
    let subject = subject(tag, &qualified);
    let context = Context::new(History::batch(&qualified), &owner, &subject, 1, None);
    let observation_identity = qualified.records()[0].identity.as_str().to_owned();
    let clock_selection = authority::clock::Selection::new(id(clock_identity), id("1"));
    let clock_document = authority::clock::derive(context, &clock_selection, Limits::owner_max())
        .expect("clock document");
    let clock = authority::clock::read(
        clock_document.bytes(),
        context,
        &clock_selection,
        Limits::owner_max(),
    )
    .expect("clock reader");
    let position_selection = authority::position::Selection::new(
        id(format!("ledger:{tag}")),
        id(clock_identity),
        id("1"),
        vec![authority::position::Position::new(
            0,
            id(&observation_identity),
        )],
    );
    let position_document =
        authority::position::derive(context, &position_selection, Limits::owner_max())
            .expect("position document");
    let position = authority::position::read(
        position_document.bytes(),
        context,
        &position_selection,
        Limits::owner_max(),
    )
    .expect("position reader");
    let record = &qualified.records()[0];
    let anchor = match family {
        ClockFixture::EventPosition => Anchor::EventPosition(0),
        ClockFixture::FixedSample => Anchor::FixedSample {
            index: 0,
            epoch_nanos: 0,
            period_nanos: 500_000_000,
        },
        ClockFixture::TimestampedEvent => Anchor::TimestampNanos(0),
    };
    let capture_selection = authority::capture::Selection::new(
        id(&trigger_identity),
        anchor,
        vec![authority::capture::Binding::new(
            id(format!("input:{tag}")),
            record.binding_identity.clone(),
            record.identity.clone(),
        )],
    );
    let capture_document =
        authority::capture::derive(context, &capture_selection, Limits::owner_max())
            .expect("capture document");
    let capture = authority::capture::read(
        capture_document.bytes(),
        context,
        &capture_selection,
        Limits::owner_max(),
    )
    .expect("capture reader");
    let boundary = match family {
        ClockFixture::EventPosition => TemporalBoundary::EventPosition {
            lower: 0,
            upper_inclusive: 0,
            carrier_end_exclusive: 1,
            watermark: u64::from(state == OpenClosed::Closed),
        },
        ClockFixture::FixedSample => TemporalBoundary::FixedSample {
            lower: 0,
            upper_inclusive: 0,
            carrier_end_exclusive: 1,
            watermark: u64::from(state == OpenClosed::Closed),
        },
        ClockFixture::TimestampedEvent => TemporalBoundary::TimestampedEvent {
            lower_nanos: 0,
            upper_inclusive_nanos: 0,
            carrier_end_exclusive_nanos: 1,
            watermark_nanos: i128::from(state == OpenClosed::Closed),
        },
    };
    let progress_selection = authority::progress::Selection::new(
        authority::clock::Selection::new(id(clock_identity), id("1")),
        vec![id(format!("source:{tag}"))],
        boundary,
        state,
        id(&trigger_identity),
        authority::observation::CutoffSelection::new(
            id(clock_identity),
            id("1"),
            40,
            authority::observation::CutoffRule::IngestionTimeAtOrBefore,
            id("1"),
        ),
        id(format!("restoration:{tag}")),
    );
    let progress_document =
        authority::progress::derive(context, &progress_selection, Limits::owner_max())
            .expect("progress document");
    let progress = authority::progress::read(
        progress_document.bytes(),
        context,
        &progress_selection,
        Limits::owner_max(),
    )
    .expect("progress reader");
    let closure_selection = authority::closure::Selection::new(
        id(clock_identity),
        id("1"),
        vec![id(format!("source:{tag}"))],
        boundary,
        state,
    );
    let closure_document =
        authority::closure::derive(context, &closure_selection, Limits::owner_max())
            .expect("closure document");
    let closure = authority::closure::read(
        closure_document.bytes(),
        context,
        &closure_selection,
        Limits::owner_max(),
    )
    .expect("closure reader");
    let completeness_selection = authority::completeness::Selection::new(
        id(format!("boundary:{tag}")),
        vec![authority::completeness::Fact::new(
            qualified.scope().members[0].object_identity.clone(),
            Some(record.identity.clone()),
            authority::completeness::FactStatus::Available,
        )],
    );
    let completeness_document =
        authority::completeness::derive(context, &completeness_selection, Limits::owner_max())
            .expect("completeness document");
    let completeness = authority::completeness::read(
        completeness_document.bytes(),
        context,
        &completeness_selection,
        Limits::owner_max(),
    )
    .expect("completeness reader");
    TemporalAuthority {
        qualified,
        owner,
        subject,
        position,
        clock,
        capture,
        progress,
        closure,
        completeness,
        observation_identity,
    }
}

fn qualified_event(
    tag: &str,
    clock_identity: &str,
    trigger_identity: &str,
    family: ClockFixture,
) -> Box<QualifiedObservation> {
    let binding_identity = id(format!("binding:{tag}"));
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
            identity: binding_identity.clone(),
            source_identity: id(format!("source:{tag}")),
            schema_identity: id(format!("schema:{tag}")),
            signal_identity: id(format!("signal:{tag}")),
            trigger_identity: id(trigger_identity),
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
            clock_identity: id(clock_identity),
            clock_revision: id("1"),
            membership_complete: true,
            closure_identity: Some(id(format!("closure:{tag}"))),
            closure_digest: Some(digest(5)),
            kind: ScopeKind::Snapshot {
                snapshot_identity: id(format!("snapshot:{tag}")),
            },
            range: match family {
                ClockFixture::EventPosition => ClockRange::EventPosition {
                    start: 0,
                    end_exclusive: 1,
                },
                ClockFixture::FixedSample => ClockRange::FixedSample {
                    start: 0,
                    end_exclusive: 1,
                    epoch_nanos: 0,
                    period_nanos: 500_000_000,
                },
                ClockFixture::TimestampedEvent => ClockRange::Timestamp {
                    start_nanos: 0,
                    end_nanos: 1,
                },
            },
            members: vec![Member {
                object_identity: id(format!("member:{tag}")),
                record_identity: id("unsealed-record"),
                anchor: match family {
                    ClockFixture::EventPosition => Anchor::EventPosition(0),
                    ClockFixture::FixedSample => Anchor::FixedSample {
                        index: 0,
                        epoch_nanos: 0,
                        period_nanos: 500_000_000,
                    },
                    ClockFixture::TimestampedEvent => Anchor::TimestampNanos(0),
                },
            }],
        },
        records: vec![AdmittedRecord {
            identity: id("unsealed-record"),
            binding_identity,
            source_identity: id(format!("source:{tag}")),
            schema_identity: id(format!("schema:{tag}")),
            subject: record_subject,
            signal_identity: id(format!("signal:{tag}")),
            trigger_identity: id(trigger_identity),
            unit: id("unit"),
            value: ValueState::Present {
                value_type: id("boolean"),
                canonical_value: "true".to_owned(),
            },
            visibility: Visibility::External,
            anchor: match family {
                ClockFixture::EventPosition => Anchor::EventPosition(0),
                ClockFixture::FixedSample => Anchor::FixedSample {
                    index: 0,
                    epoch_nanos: 0,
                    period_nanos: 500_000_000,
                },
                ClockFixture::TimestampedEvent => Anchor::TimestampNanos(0),
            },
            event_time_nanos: 0,
            ingestion_time_nanos: 1,
            causal_relationship_identity: None,
            clock_identity: id(clock_identity),
            clock_revision: id("1"),
            clock_uncertainty_nanos: 0,
        }],
        limits: ResourceLimits {
            max_records: 1,
            max_members: 1,
            max_relationships: 0,
            max_required_relationships: 0,
        },
    };
    let record_identity =
        authority::observation::record_identity(&request.records[0], Limits::owner_max())
            .expect("record identity");
    request.records[0].identity = record_identity.clone();
    request.scope.members[0].record_identity = record_identity;
    authority::population::assign_request_identities(&mut request, Limits::owner_max())
        .expect("population identities");
    match admit(request) {
        AdmissionOutcome::Available { observation } => observation,
        other => panic!("event qualification failed: {other:?}"),
    }
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
        scope_identity: id(format!("snapshot:{tag}")),
        population_identity: qualified.scope().population_identity.clone(),
    }
}

fn id(value: impl Into<String>) -> Identity {
    Identity::new(value)
}

fn digest(value: u8) -> Digest {
    Digest::new([value; 32])
}
