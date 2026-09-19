use ix_trace_rs::trace;
use quire_contract_ir::kani::{
    replay_counterexample, replay_with_native_runtime, CounterexamplePacket, FiniteInput,
    FiniteObject, KaniOutcome, KaniOutcomeKind, PopulationCompleteness, ProfileSelection,
    ResourceBounds, Witness, WitnessBinding, WitnessCheck, WitnessValue, WitnessValueType, PROFILE,
};
use quire_contract_model_owner as ir;
use quire_spec_language::checking::{check, CheckBindings, CheckLimits, ClauseBinding};
use quire_spec_language::formal_source::FormalSource;
use quire_spec_language::model_source::{self, ModelSourceLimits};
use quire_spec_language::native_model::{ModelLimits, NativeModel};
use quire_spec_language::package::{NativePackage, PackageLimits};
use quire_spec_language::runtime::{
    ArtifactLimits, ExecutionLimits, ExecutionSelection, FieldBinding, ModelBinding, ObjectEntry,
    ObjectIdentity, ObservationSelection, Population, RuntimeInput, Snapshot, SnapshotDraft,
    ValueId, ValueNode,
};
use quire_spec_language::{link_native, parse, Limits, LinkLimits, Source, SourceIdentity};

fn symbol(name: &str) -> ir::SymbolName {
    ir::SymbolName::new(name).expect("test symbol")
}

fn owner() -> ir::RequirementRef {
    ir::RequirementRef::parse("example/kani-replay", "NativeReplay", 1).expect("test owner")
}

fn native_model() -> NativeModel {
    let source = Source::read(
        SourceIdentity {
            identity: "test:kani-replay-model".into(),
            revision: "1".into(),
        },
        "native-rule-model.json",
        include_bytes!("fixtures/native-rule-model.json"),
        Limits::default().source_bytes,
    )
    .expect("read native model fixture");
    let formal = FormalSource::new(
        source,
        ir::SourceIdentity::new(
            ir::SourceDocumentId::new("KaniReplayModel").expect("test source"),
            ir::SourceRevision::new(1).expect("test revision"),
        ),
    );
    model_source::read(formal, model_source::FORMAT, ModelSourceLimits::default())
        .expect("lower native model fixture")
        .admit(ModelLimits::default())
        .expect("admit native model fixture")
}

fn object(model: &NativeModel) -> ObjectIdentity {
    ObjectIdentity {
        model: model.environment().owner().clone(),
        record: symbol("Node"),
        universe: symbol("nodes"),
        key: "self".into(),
    }
}

fn native_request(model: &NativeModel) -> (RuntimeInput, ExecutionSelection) {
    let snapshot = Snapshot::new(
        SourceIdentity {
            identity: "test:kani-replay-state".into(),
            revision: "1".into(),
        },
        SnapshotDraft {
            observation: ir::StateObservation::Current,
            models: vec![ModelBinding {
                model: model.environment().owner().clone(),
                digest: model.digest(),
            }],
            populations: vec![Population {
                model: model.environment().owner().clone(),
                record: symbol("Node"),
                universe: symbol("nodes"),
                complete: true,
                objects: vec![ObjectEntry {
                    key: "self".into(),
                    fields: vec![
                        ("n", 0),
                        ("signed", 1),
                        ("den", 0),
                        ("wide", 1),
                        ("count", 1),
                        ("distance", 0),
                        ("duration", 0),
                        ("parent", 2),
                        ("peer", 3),
                        ("items", 4),
                    ]
                    .into_iter()
                    .map(|(name, value)| FieldBinding {
                        name: symbol(name),
                        value: ValueId::new(value),
                    })
                    .collect(),
                }],
            }],
            values: Vec::new(),
            arena: vec![
                ValueNode::Integer { value: 1 },
                ValueNode::Integer { value: 0 },
                ValueNode::Absent,
                ValueNode::Reference {
                    identity: object(model),
                },
                ValueNode::Sequence { values: Vec::new() },
            ],
        },
        ArtifactLimits::default(),
    )
    .expect("valid native snapshot");
    let selection = ExecutionSelection {
        requirement: owner(),
        clause: ir::ClauseId::new("native_replay").expect("test clause"),
        observation: ObservationSelection::Current {
            snapshot: snapshot.reference(),
            self_object: object(model),
        },
    };
    (
        RuntimeInput {
            snapshots: vec![snapshot],
            invocations: Vec::new(),
        },
        selection,
    )
}

fn native_package<'a>(models: &'a [NativeModel]) -> NativePackage<'a> {
    let model = &models[0];
    let text = format!(
        "language \"ix:native\" edition \"0-draft\";\nprofile \"state-finite/0-draft\";\nmodel M = \"{}\" version \"{}\" digest \"{}\";\ninvariant Replay on M::Node at current {{ false }}\n",
        model.environment().owner().package().as_str(),
        model.environment().owner().revision().get(),
        model.digest(),
    );
    let unit = parse(
        SourceIdentity {
            identity: "test:kani-replay-rule".into(),
            revision: "1".into(),
        },
        "kani-replay.native",
        text.as_bytes(),
        Limits::default(),
    )
    .expect("parse native false rule");
    let source = FormalSource::new(
        unit.source().clone(),
        ir::SourceIdentity::new(
            ir::SourceDocumentId::new("KaniReplayRule").expect("test source"),
            ir::SourceRevision::new(1).expect("test revision"),
        ),
    );
    let linked = link_native(unit, models, LinkLimits::default()).expect("link native rule");
    let checked = check(
        linked,
        CheckBindings {
            source,
            clauses: vec![ClauseBinding {
                name: "Replay".into(),
                requirement: owner(),
                clause: ir::ClauseId::new("native_replay").expect("test clause"),
                execution_point: ir::ExecutionPoint::Handler {
                    name: ir::AnchorName::new("validate").expect("test anchor"),
                },
            }],
        },
        CheckLimits::default(),
    )
    .expect("check native rule");
    NativePackage::new(checked, PackageLimits::default()).expect("package native rule")
}
fn packet() -> CounterexamplePacket {
    CounterexamplePacket {
        profile_revision: "kani-bounded/1.0.0".into(),
        // This synthetic packet never ran Kani, so it honestly retains no
        // backend transcript; `Witness` parsing/decoding is covered
        // separately below against a real captured playback block.
        witness: None,
        input: FiniteInput {
            model_id: "model".into(),
            source_id: "clause".into(),
            profile: ProfileSelection {
                profile: PROFILE.into(),
                revision: "kani-bounded/1.0.0".into(),
                executable_digest: "sha256:kani".into(),
                options_digest: "sha256:options".into(),
                abi_revision: "kani-abi/1".into(),
            },
            completeness: PopulationCompleteness::Complete,
            bounds: ResourceBounds {
                max_objects: 1,
                max_references: 0,
                max_input_bytes: 10,
            },
            input_bytes: 1,
            objects: vec![FiniteObject {
                identity: "self".into(),
                type_id: "Node".into(),
                snapshot_id: "s".into(),
            }],
            references: vec![],
        },
    }
}
#[trace("TC-042", "FR-031-AC-3")]
#[test]
fn tc_042_counterexample_replay_agrees_or_is_non_success() {
    let agreement = replay_counterexample(packet(), |_| {
        KaniOutcome::counterexample("clause", "native")
    })
    .expect("same counterexample");
    assert_eq!(agreement.native.kind, KaniOutcomeKind::Counterexample);
    let disagreement = replay_counterexample(packet(), |_| KaniOutcome::proved("clause", "native"))
        .expect_err("proof is not replay agreement");
    assert_eq!(disagreement.kind, KaniOutcomeKind::Inconclusive);
    assert_eq!(disagreement.boolean_claim(), None);

    let mut invalid = packet();
    invalid.profile_revision = "different-profile".into();
    let invalid =
        replay_counterexample(invalid, |_| KaniOutcome::counterexample("clause", "native"))
            .expect_err("profile identity must bind the retained input");
    assert_eq!(invalid.kind, KaniOutcomeKind::InvalidInput);
}

#[trace("TC-042", "FR-031-AC-3")]
#[test]
fn tc_042_counterexample_replays_through_native_runtime_execute() {
    let models = vec![native_model()];
    let package = native_package(&models);
    let (input, selection) = native_request(&models[0]);
    let report = quire_spec_language::runtime::execute(
        &package,
        input,
        selection,
        ExecutionLimits::default(),
        || false,
    );
    assert_eq!(report.truth(), Some(false), "{:#?}", report.outcome());
    let agreement = replay_with_native_runtime(
        packet(),
        &package,
        |_| native_request(&models[0]),
        ExecutionLimits::default(),
    )
    .expect("the independently executed false predicate agrees with the counterexample");
    assert_eq!(agreement.native.truth(), Some(false));
}

/// A real `kani::concrete_playback_run` block captured from a falsified
/// two-argument `i64` contract check, copied verbatim.
fn real_playback_block() -> &'static str {
    include_str!("fixtures/kani-concrete-playback.txt")
}

/// A cover playback in the same shape Kani emits, copied down to a single
/// binding: cover checks witness reachability, never falsity.
fn cover_playback_block() -> &'static str {
    "/// Test generated for harness `kob_..._module::kob_..._cover_proof`\n\
     ///\n\
     /// Check for `cover`: \"cover_marker\"\n\
     \n\
     #[test]\n\
     fn kani_concrete_playback_kob_..._cover_proof_1() {\n\
     let concrete_vals: Vec<Vec<u8>> = vec![\n\
     // 1\n\
     vec![1],\n\
     ];\n\
     kani::concrete_playback_run(concrete_vals, kob_..._cover_proof);\n\
     }\n"
}

fn balance_schema() -> Vec<WitnessBinding> {
    vec![
        WitnessBinding {
            identifier: "balance_pre".into(),
            value_type: WitnessValueType::I64,
        },
        WitnessBinding {
            identifier: "post_state".into(),
            value_type: WitnessValueType::I64,
        },
    ]
}

#[trace("TC-042", "FR-031-AC-3")]
#[test]
fn tc_042_witness_parses_real_playback_block_with_multiline_check_text() {
    let witness = Witness::parse("clause", "kani-bounded/1.0.0", real_playback_block())
        .expect("a real falsified concrete-playback block parses");
    assert_eq!(
        witness.harness_symbol, "kob_..._module::kob_..._proof",
        "harness symbol must come from the backtick-quoted `Test generated for harness` text"
    );
    assert_eq!(witness.check, WitnessCheck::Assertion);
    assert_eq!(
        witness.check_text,
        "|post_state: &i64| (*post_state >= 0_i64 && *post_state <= 1000_i64) &&\n\
         oracle_fr_200_1_balance_never_grows_id_6aad...(balance_pre,\n\
         *post_state)",
        "the `Check for` text spans multiple lines and must be captured whole"
    );
    assert!(
        witness.check_text.contains('\n'),
        "a line-based extractor that stopped at the first newline would fail this"
    );
    assert_eq!(
        witness.concrete_values,
        vec![vec![8, 0, 0, 0, 0, 0, 0, 0], vec![224, 3, 0, 0, 0, 0, 0, 0]],
        "one untyped byte vector per kani::any() call, in declaration order"
    );
}

#[trace("TC-042", "FR-031-AC-3")]
#[test]
fn tc_042_witness_cover_playback_is_refused() {
    let refusal = Witness::parse("clause", "kani-bounded/1.0.0", cover_playback_block())
        .expect_err("a cover playback witnesses reachability, not falsity");
    assert_eq!(refusal.kind, KaniOutcomeKind::Refused);
    assert_eq!(refusal.code, "kani_witness_cover_refused");
}

#[trace("TC-042", "FR-031-AC-3")]
#[test]
fn tc_042_witness_decode_round_trips_declared_schema() {
    let witness = Witness::parse("clause", "kani-bounded/1.0.0", real_playback_block())
        .expect("a real falsified concrete-playback block parses");
    let decoded = witness
        .decode(&balance_schema())
        .expect("8 and 992 decode against a matching i64 schema");
    assert_eq!(
        decoded,
        vec![
            ("balance_pre".to_string(), WitnessValue::Integer(8)),
            ("post_state".to_string(), WitnessValue::Integer(992)),
        ]
    );
}

#[trace("TC-042", "FR-031-AC-3")]
#[test]
fn tc_042_witness_decode_refuses_arity_mismatch() {
    let witness = Witness::parse("clause", "kani-bounded/1.0.0", real_playback_block())
        .expect("a real falsified concrete-playback block parses");
    let schema = vec![balance_schema().remove(0)];
    let refusal = witness
        .decode(&schema)
        .expect_err("two concrete values against a one-binding schema must refuse, not truncate");
    assert_eq!(refusal.kind, KaniOutcomeKind::InvalidInput);
    assert_eq!(refusal.code, "kani_witness_arity_mismatch");
}

#[trace("TC-042", "FR-031-AC-3")]
#[test]
fn tc_042_witness_decode_refuses_width_mismatch() {
    let witness = Witness::parse("clause", "kani-bounded/1.0.0", real_playback_block())
        .expect("a real falsified concrete-playback block parses");
    let mut schema = balance_schema();
    schema[0].value_type = WitnessValueType::Boolean;
    let refusal = witness.decode(&schema).expect_err(
        "an 8-byte concrete value against a 1-byte boolean binding must refuse, not truncate",
    );
    assert_eq!(refusal.kind, KaniOutcomeKind::InvalidInput);
    assert_eq!(refusal.code, "kani_witness_width_mismatch");
}

#[trace("TC-042", "FR-031-AC-3")]
#[test]
fn tc_042_witness_decode_refuses_comment_disagreement() {
    let mut witness = Witness::parse("clause", "kani-bounded/1.0.0", real_playback_block())
        .expect("a real falsified concrete-playback block parses");
    // The retained transcript still says `// 8`; disagree with it without
    // touching the transcript, the way a corrupted or hand-edited packet
    // would.
    witness.concrete_values[0] = vec![9, 9, 9, 9, 9, 9, 9, 9];
    let refusal = witness
        .decode(&balance_schema())
        .expect_err("a decoded value that disagrees with Kani's own comment must refuse");
    assert_eq!(refusal.kind, KaniOutcomeKind::InvalidInput);
    assert_eq!(refusal.code, "kani_witness_comment_mismatch");
}

#[trace("TC-042", "FR-031-AC-3")]
#[test]
fn tc_042_counterexample_packet_refuses_cover_witness() {
    let mut with_cover = packet();
    with_cover.witness = Some(Witness {
        harness_symbol: "kob_..._module::kob_..._cover_proof".into(),
        check: WitnessCheck::Cover,
        check_text: "cover_marker".into(),
        concrete_values: vec![vec![1]],
        transcript: cover_playback_block().trim().into(),
    });
    let refusal = replay_counterexample(with_cover, |_| {
        KaniOutcome::counterexample("clause", "native")
    })
    .expect_err("a cover witness never backs a counterexample packet");
    assert_eq!(refusal.kind, KaniOutcomeKind::InvalidInput);
    assert_eq!(refusal.code, "kani_replay_witness_invalid");
}
