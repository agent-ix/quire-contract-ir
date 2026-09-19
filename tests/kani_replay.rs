use std::collections::BTreeMap;

use ix_trace_rs::trace;
use quire_contract_ir::kani::{
    replay_counterexample, replay_with_native_runtime, CounterexamplePacket, FiniteInput,
    FiniteObject, KaniOutcome, KaniOutcomeKind, NativeReplayAgreement, PopulationCompleteness,
    ProfileSelection, ReplayAgreement, ReplaySource, ResourceBounds, Witness, WitnessBinding,
    WitnessCheck, WitnessValue, WitnessValueType, PROFILE,
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
        // backend transcript: the `Input` arm with no assignments. `Witness`
        // parsing/decoding is covered separately below against a real
        // captured playback block.
        source: ReplaySource::Input(BTreeMap::new()),
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
#[trace("TC-042")]
#[test]
fn tc_042_counterexample_replay_agrees_or_is_non_success() {
    let agreement = replay_counterexample(packet(), |_| {
        KaniOutcome::counterexample("clause", "native")
    })
    .expect("same counterexample");
    let ReplayAgreement::Input(agreement) = agreement else {
        panic!("packet() carries source: ReplaySource::Input(..); the agreement must settle the Input arm, never the Witness arm");
    };
    assert_eq!(agreement.native().kind, KaniOutcomeKind::Counterexample);
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

#[trace("TC-042")]
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
    let NativeReplayAgreement::Input(agreement) = agreement else {
        panic!("packet() carries source: ReplaySource::Input(..); the agreement must settle the Input arm, never the Witness arm");
    };
    assert_eq!(agreement.native().truth(), Some(false));
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

#[trace("TC-221", "FR-031-AC-4")]
#[test]
fn tc_042_witness_parses_real_playback_block_with_multiline_check_text() {
    let witness = Witness::parse("clause", "kani-bounded/1.0.0", real_playback_block())
        .expect("a real falsified concrete-playback block parses");
    assert_eq!(
        witness
            .harness_symbol()
            .expect("harness symbol re-derives from transcript"),
        "kob_..._module::kob_..._proof",
        "harness symbol must come from the backtick-quoted `Test generated for harness` text"
    );
    assert_eq!(
        witness
            .check()
            .expect("check kind re-derives from transcript"),
        WitnessCheck::Assertion
    );
    let check_text = witness
        .check_text()
        .expect("check text re-derives from transcript");
    assert_eq!(
        check_text,
        "|post_state: &i64| (*post_state >= 0_i64 && *post_state <= 1000_i64) &&\n\
         oracle_fr_200_1_balance_never_grows_id_6aad...(balance_pre,\n\
         *post_state)",
        "the `Check for` text spans multiple lines and must be captured whole"
    );
    assert!(
        check_text.contains('\n'),
        "a line-based extractor that stopped at the first newline would fail this"
    );
    assert_eq!(
        witness
            .concrete_values()
            .expect("concrete values parse from the retained transcript"),
        vec![vec![8, 0, 0, 0, 0, 0, 0, 0], vec![224, 3, 0, 0, 0, 0, 0, 0]],
        "one untyped byte vector per kani::any() call, in declaration order"
    );
}

#[trace("TC-221", "FR-031-AC-4")]
#[test]
fn tc_042_witness_cover_playback_is_refused() {
    let refusal = Witness::parse("clause", "kani-bounded/1.0.0", cover_playback_block())
        .expect_err("a cover playback witnesses reachability, not falsity");
    assert_eq!(refusal.kind, KaniOutcomeKind::Refused);
    assert_eq!(refusal.code, "kani_witness_cover_refused");
}

#[trace("TC-221", "FR-031-AC-4")]
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

#[trace("TC-221", "FR-031-AC-4")]
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

#[trace("TC-221", "FR-031-AC-4")]
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

#[trace("TC-221", "FR-031-AC-4")]
#[test]
fn tc_042_witness_decode_refuses_comment_disagreement() {
    // `transcript` is private and `Witness::parse` (directly, or through
    // `Deserialize`; see the module doc) is the only admission path, so
    // disagreement can only be introduced in the transcript's own text
    // before admission, the way a corrupted or hand-edited packet would:
    // keep the `// 8` comment but change the bytes it names. Parsing does
    // not itself cross-check a comment against its bytes (only `decode`
    // does), so this is structurally well-formed and is admitted.
    let corrupted_transcript = real_playback_block().replace(
        "vec![8, 0, 0, 0, 0, 0, 0, 0]",
        "vec![9, 0, 0, 0, 0, 0, 0, 0]",
    );
    let witness: Witness =
        serde_json::from_value(serde_json::json!({ "transcript": corrupted_transcript }))
            .expect("a structurally well-formed transcript is admitted even though its own comment disagrees with its bytes");
    let refusal = witness
        .decode(&balance_schema())
        .expect_err("a decoded value that disagrees with Kani's own comment must refuse");
    assert_eq!(refusal.kind, KaniOutcomeKind::InvalidInput);
    assert_eq!(refusal.code, "kani_witness_comment_mismatch");
}

#[trace("TC-221", "FR-031-AC-4")]
#[test]
fn tc_042_witness_deserializing_cover_playback_is_refused() {
    // `Witness::parse` is the only admission path, including through
    // `Deserialize` (see the module doc): a cover witness never backs a
    // counterexample, so it is refused at admission — there is no longer a
    // way to construct a `Witness` carrying a cover transcript at all, so
    // `CounterexamplePacket` can no longer even be built with one.
    let result: Result<Witness, _> = serde_json::from_value(serde_json::json!({
        "transcript": cover_playback_block().trim(),
    }));
    result.expect_err(
        "a cover playback witnesses reachability, not falsity, and must be refused at admission",
    );
}

// ---------------------------------------------------------------------------
// Reviewer-reported findings on PR #139 (Refs #137). Each test below
// reproduces the reviewer's exact scenario.
// ---------------------------------------------------------------------------

/// A playback block generated for an `unwinding assertion` check: a
/// bound-exhaustion artifact Kani reports, not a falsified contract.
fn unwinding_assertion_playback_block() -> &'static str {
    "/// Test generated for harness `kob_..._module::kob_..._proof`\n\
     ///\n\
     /// Check for `unwinding assertion`: \"unwinding assertion loop 0\"\n\
     \n\
     #[test]\n\
     fn kani_concrete_playback_kob_..._proof_1() {\n\
     let concrete_vals: Vec<Vec<u8>> = vec![\n\
     // 1\n\
     vec![1],\n\
     ];\n\
     kani::concrete_playback_run(concrete_vals, kob_..._proof);\n\
     }\n"
}

// F1: `WitnessCheck::Other` must not be accepted as a witness of falsity.
#[trace("TC-221", "FR-031-AC-4")]
#[test]
fn tc_042_witness_refuses_unwinding_assertion_check_kind() {
    let refusal = Witness::parse(
        "clause",
        "kani-bounded/1.0.0",
        unwinding_assertion_playback_block(),
    )
    .expect_err(
        "an unwinding-assertion failure is a bound-exhaustion artifact, not a counterexample",
    );
    assert_eq!(refusal.kind, KaniOutcomeKind::Refused);
    assert_eq!(refusal.code, "kani_witness_check_kind_refused");
    assert_ne!(
        refusal.code, "kani_witness_cover_refused",
        "an unwinding-assertion refusal must not be conflated with the cover refusal"
    );
    // F7: `context` must carry the caller-supplied context (here the profile
    // revision passed to `Witness::parse`), the same as every other refusal
    // in this module — not the other check's kind text, which would make
    // `source_id`/`context` mean different things across refusals with the
    // cause already named by `code` alone.
    assert_eq!(refusal.context, "kani-bounded/1.0.0");
}

// F2: multi-block transcripts must be delimited and the assertion block
// selected, rather than the first `Check for` / `let concrete_vals` in the
// whole text.
#[trace("TC-221", "FR-031-AC-4")]
#[test]
fn tc_042_witness_selects_assertion_block_when_cover_precedes_it() {
    let transcript = format!("{}\n{}", cover_playback_block(), real_playback_block());
    let witness = Witness::parse("clause", "kani-bounded/1.0.0", &transcript).expect(
        "a cover block followed by a falsified assertion block is an ordinary run; \
         the genuine falsification must not be discarded as a cover refusal",
    );
    assert_eq!(
        witness
            .check()
            .expect("check kind re-derives from transcript"),
        WitnessCheck::Assertion
    );
    assert_eq!(
        witness
            .harness_symbol()
            .expect("harness symbol re-derives from transcript"),
        "kob_..._module::kob_..._proof"
    );
}

// F1: a `Witness` deserialized from a raw, un-selected multi-block
// transcript (the shape a wire packet arrives in) must still narrow to the
// real assertion block for every derived fact, including `concrete_values`
// — not read concrete values out of whichever block happens to appear first
// in the raw text. `Deserialize` now routes through `Witness::parse` (see
// the module doc), the same admission path direct construction used to
// bypass; there is no longer any way to hold an un-narrowed transcript.
#[trace("TC-221", "FR-031-AC-4")]
#[test]
fn tc_042_witness_deserialize_narrows_concrete_values_to_selected_block() {
    let multi_block_transcript = format!("{}\n{}", cover_playback_block(), real_playback_block());
    let witness: Witness =
        serde_json::from_value(serde_json::json!({ "transcript": multi_block_transcript }))
            .expect("a cover block followed by a falsified assertion block is an ordinary run");
    assert_eq!(
        witness
            .check()
            .expect("re-derivation must select the assertion block, not the leading cover block"),
        WitnessCheck::Assertion
    );
    assert_eq!(
        witness
            .concrete_values()
            .expect("concrete values must come from the selected assertion block"),
        vec![vec![8, 0, 0, 0, 0, 0, 0, 0], vec![224, 3, 0, 0, 0, 0, 0, 0]],
        "the cover block's own single-byte entry must not leak into the result"
    );
}

#[trace("TC-221", "FR-031-AC-4")]
#[test]
fn tc_042_witness_refuses_multiple_assertion_blocks() {
    let transcript = format!("{}\n{}", real_playback_block(), real_playback_block());
    let refusal = Witness::parse("clause", "kani-bounded/1.0.0", &transcript).expect_err(
        "two assertion blocks in one transcript must be refused, not silently narrowed to the first",
    );
    assert_eq!(refusal.kind, KaniOutcomeKind::Refused);
    assert_eq!(
        refusal.code, "kani_witness_multiple_assertions_refused",
        "F6: the exact cause code, not merely a code distinct from the cover refusal"
    );
}

// F3: `transcript` must be the validated source of truth on the shipped
// replay path, not merely a field `decode` can cross-check in tests nobody
// calls from `replay_counterexample`.
//
// F5 (PR #139 review): a transcript can look well-formed and still be
// wrong. `Witness::parse` is now the only admission path, including through
// `Deserialize` (see the module doc), so a hand-written or corrupted packet
// can no longer carry a `Witness` that disagrees with its own transcript at
// all: three of the four cases below are refused at admission, before a
// `CounterexamplePacket` can even be built. The fourth — bytes that
// structurally parse but contradict their own `//` comment — is admitted
// (parsing does not cross-check comments against bytes) and is caught only
// once `replay_counterexample` validates the packet, exactly as before.
#[trace("TC-221", "FR-031-AC-4")]
#[test]
fn tc_042_replay_counterexample_refuses_witness_with_untrustworthy_transcript() {
    let refused_at_admission: Vec<(&str, String)> = vec![
        (
            "empty transcript: cannot possibly reproduce any concrete values",
            String::new(),
        ),
        (
            "verbatim cover playback block: witnesses reachability, not falsity (F1)",
            cover_playback_block().trim().to_owned(),
        ),
        (
            "not a playback block at all",
            "this is not a kani playback block".to_owned(),
        ),
    ];
    for (description, transcript) in refused_at_admission {
        let result: Result<Witness, _> =
            serde_json::from_value(serde_json::json!({ "transcript": transcript }));
        assert!(result.is_err(), "must refuse at admission: {description}");
    }

    let comment_mismatch_transcript = real_playback_block().replace(
        "vec![8, 0, 0, 0, 0, 0, 0, 0]",
        "vec![9, 0, 0, 0, 0, 0, 0, 0]",
    );
    let witness: Witness =
        serde_json::from_value(serde_json::json!({ "transcript": comment_mismatch_transcript }))
            .expect("a structurally well-formed transcript is admitted even though its own comment disagrees with its bytes");
    let mut with_untrustworthy_transcript = packet();
    with_untrustworthy_transcript.source = ReplaySource::Witness(witness);
    let refusal = replay_counterexample(with_untrustworthy_transcript, |_| {
        KaniOutcome::counterexample("clause", "native")
    })
    .expect_err("bytes contradicting their own comment must refuse at replay");
    assert_eq!(refusal.kind, KaniOutcomeKind::InvalidInput);
    assert_eq!(refusal.code, "kani_replay_witness_invalid");
}

// F6: a concrete value with no `//` comment must be refused, not silently
// dropped from the parsed entries.
fn missing_first_comment_playback_block() -> &'static str {
    "/// Test generated for harness `kob_..._module::kob_..._proof`\n\
     ///\n\
     /// Check for `assertion`: \"oracle_holds()\"\n\
     \n\
     #[test]\n\
     fn kani_concrete_playback_kob_..._proof_1() {\n\
     let concrete_vals: Vec<Vec<u8>> = vec![\n\
     vec![1, 0, 0, 0, 0, 0, 0, 0],\n\
     // 2\n\
     vec![2, 0, 0, 0, 0, 0, 0, 0],\n\
     ];\n\
     kani::concrete_playback_run(concrete_vals, kob_..._proof);\n\
     }\n"
}

#[trace("TC-221", "FR-031-AC-4")]
#[test]
fn tc_042_witness_refuses_concrete_value_with_no_comment() {
    let refusal = Witness::parse(
        "clause",
        "kani-bounded/1.0.0",
        missing_first_comment_playback_block(),
    )
    .expect_err(
        "the first value has no `//` comment; driving the loop off comments silently drops it \
         instead of refusing, later misdiagnosed downstream as an arity mismatch",
    );
    assert_eq!(refusal.kind, KaniOutcomeKind::InvalidInput);
    assert_eq!(
        refusal.code, "kani_witness_comment_missing",
        "F6: the exact cause code, naming that a value had no preceding comment"
    );
}

// F7: an earlier `Check for `cover`` substring appearing before the real
// check line (e.g. embedded in caller-controlled contract text appended to
// the harness doc line) must not force a false refusal. A backtick inside
// the assertion's own quoted text must still be handled correctly.
fn tricky_backtick_playback_block() -> &'static str {
    "/// Test generated for harness `kob_..._module::kob_..._proof` that checks contract for \
     `note: Check for `cover`: \"decoy\" appended`\n\
     ///\n\
     /// Check for `assertion`: \"oracle_holds(`x`) && true\"\n\
     \n\
     #[test]\n\
     fn kani_concrete_playback_kob_..._proof_1() {\n\
     let concrete_vals: Vec<Vec<u8>> = vec![\n\
     // 1\n\
     vec![1],\n\
     ];\n\
     kani::concrete_playback_run(concrete_vals, kob_..._proof);\n\
     }\n"
}

#[trace("TC-221", "FR-031-AC-4")]
#[test]
fn tc_042_witness_tricky_backtick_anchors_to_the_real_check_line() {
    let witness = Witness::parse(
        "clause",
        "kani-bounded/1.0.0",
        tricky_backtick_playback_block(),
    )
    .expect(
        "the earlier `Check for `cover`` text embedded in the harness doc line must not be \
             mistaken for the real check line",
    );
    assert_eq!(
        witness
            .check()
            .expect("check kind re-derives from transcript"),
        WitnessCheck::Assertion
    );
    assert_eq!(
        witness
            .check_text()
            .expect("check text re-derives from transcript"),
        "oracle_holds(`x`) && true",
        "a backtick inside the assertion's own quoted text must not disturb extraction"
    );
    assert_eq!(
        witness
            .harness_symbol()
            .expect("harness symbol re-derives from transcript"),
        "kob_..._module::kob_..._proof"
    );
}

// F8 (PR #139 review): AC-4 says a decoded value is never inferred. A boolean
// byte outside `{0, 1}` is not a value Kani's concrete playback could have
// produced for a `bool`, so it must be refused by name rather than decoded as
// `true` from "any nonzero byte".
fn boolean_out_of_range_playback_block() -> &'static str {
    "/// Test generated for harness `kob_..._module::kob_..._proof`\n\
     ///\n\
     /// Check for `assertion`: \"oracle_holds()\"\n\
     \n\
     #[test]\n\
     fn kani_concrete_playback_kob_..._proof_1() {\n\
     let concrete_vals: Vec<Vec<u8>> = vec![\n\
     // 2\n\
     vec![2],\n\
     ];\n\
     kani::concrete_playback_run(concrete_vals, kob_..._proof);\n\
     }\n"
}

#[trace("TC-221", "FR-031-AC-4")]
#[test]
fn tc_042_witness_decode_refuses_out_of_range_boolean_byte() {
    let witness = Witness::parse(
        "clause",
        "kani-bounded/1.0.0",
        boolean_out_of_range_playback_block(),
    )
    .expect("a well-formed playback block with an out-of-range boolean byte still parses");
    let schema = vec![WitnessBinding {
        identifier: "flag".into(),
        value_type: WitnessValueType::Boolean,
    }];
    let refusal = witness
        .decode(&schema)
        .expect_err("a boolean byte of 2 must never be inferred as `true`");
    assert_eq!(refusal.kind, KaniOutcomeKind::InvalidInput);
    assert_eq!(refusal.code, "kani_witness_boolean_byte_invalid");
}

// F9: a zero-argument harness (no `kani::any()` calls) is valid and has zero
// bindings; it must not be refused as if its concrete values were missing.
fn zero_argument_playback_block() -> &'static str {
    "/// Test generated for harness `kob_..._module::kob_..._proof`\n\
     ///\n\
     /// Check for `assertion`: \"always_false()\"\n\
     \n\
     #[test]\n\
     fn kani_concrete_playback_kob_..._proof_1() {\n\
     let concrete_vals: Vec<Vec<u8>> = vec![];\n\
     kani::concrete_playback_run(concrete_vals, kob_..._proof);\n\
     }\n"
}

#[trace("TC-221", "FR-031-AC-4")]
#[test]
fn tc_042_witness_parses_zero_argument_harness() {
    let witness = Witness::parse(
        "clause",
        "kani-bounded/1.0.0",
        zero_argument_playback_block(),
    )
    .expect("a harness declaring no kani::any() calls is valid and has zero bindings");
    let decoded = witness
        .decode(&[])
        .expect("decoding a zero-binding witness against an empty schema must succeed");
    assert!(decoded.is_empty());
}

// F5: an `Input`-arm packet must not settle the same agreement type as one
// reproduced together with its evaluated witness; `ReplayAgreement` and
// `NativeReplayAgreement` are sums of the two distinct arm result types
// (AD-016 "Replay result"), so the two can never be confused — the arm
// itself is the fact, not a separate field. The only previously covered
// `Witness`-arm case was the cover refusal, leaving the accepted assertion
// witness path untested.
//
// Deliberately untraced: FR-031-AC-4 covers witness parsing, typing, and
// arity/width/comment refusal only — it says nothing about replay arms,
// `ReplaySource`, or which agreement type a replay settles. Arm separation
// is governed by AD-016 "Replay ownership" / "Replay source", which has no
// FR/AC of its own yet; authoring one is spec work, not this test's job.
#[test]
fn tc_042_replay_counterexample_settles_witness_arm_for_assertion_witness() {
    let mut with_witness = packet();
    with_witness.source = ReplaySource::Witness(
        Witness::parse("clause", "kani-bounded/1.0.0", real_playback_block())
            .expect("a real falsified concrete-playback block parses"),
    );
    let agreement = replay_counterexample(with_witness, |_| {
        KaniOutcome::counterexample("clause", "native")
    })
    .expect("a Witness-arm packet must reproduce alongside its native agreement");
    assert!(
        matches!(agreement, ReplayAgreement::Witness(_)),
        "an accepted assertion witness must settle the Witness arm (reproduced-with-evaluated-witness)"
    );
}

// Deliberately untraced: see the comment above
// `tc_042_replay_counterexample_settles_witness_arm_for_assertion_witness` —
// FR-031-AC-4 does not cover replay-arm settlement; AD-016 "Replay ownership"
// / "Replay source" governs it, with no FR/AC of its own yet.
#[test]
fn tc_042_replay_counterexample_settles_input_arm_for_witness_free_packet() {
    let agreement = replay_counterexample(packet(), |_| {
        KaniOutcome::counterexample("clause", "native")
    })
    .expect("an Input-arm packet is a real, honestly modeled corpus counterexample");
    assert!(
        matches!(agreement, ReplayAgreement::Input(_)),
        "a witness-free packet must settle the Input arm (reproduced-without-witness), never the Witness arm"
    );
}

// Mutation charge: an `Input`-arm packet replayed against a stub executor
// that agrees is structurally incapable of producing a `WitnessReplayAgreement`
// — `InputReplayAgreement` carries no `Witness` field anywhere, so there is
// no value it could ever hold that a backend-evidence verdict could be built
// from (AD-016 "Replay ownership": "Its only construction path takes an
// agreeing `Witness`-arm result").
//
// Deliberately untraced: see the comment above
// `tc_042_replay_counterexample_settles_witness_arm_for_assertion_witness` —
// FR-031-AC-4 does not cover replay-arm settlement; AD-016 "Replay ownership"
// / "Replay source" governs it, with no FR/AC of its own yet.
#[test]
fn tc_042_input_arm_replay_cannot_settle_the_witness_arm() {
    let mut corpus_counterexample = packet();
    let mut assignments = BTreeMap::new();
    assignments.insert("balance_pre".to_string(), WitnessValue::Integer(8));
    assignments.insert("post_state".to_string(), WitnessValue::Integer(992));
    corpus_counterexample.source = ReplaySource::Input(assignments);

    // A stub executor that agrees unconditionally, for any input.
    let agreement = replay_counterexample(corpus_counterexample, |_| {
        KaniOutcome::counterexample("clause", "native")
    })
    .expect("an Input-arm packet with a stub executor that agrees must settle");
    match agreement {
        ReplayAgreement::Input(input_agreement) => {
            assert_eq!(
                input_agreement.native().kind,
                KaniOutcomeKind::Counterexample
            );
        }
        ReplayAgreement::Witness(_) => {
            panic!("an Input-arm packet must never settle the Witness arm")
        }
    }
}

// F4 (PR #139 review): the new wire types must deny unknown fields, matching
// the repo's ~20 other `#[serde(deny_unknown_fields)]` sites, so a stray or
// misspelled field in a hand-authored or corrupted packet is refused at
// deserialization rather than silently ignored.
#[trace("TC-221", "FR-031-AC-4")]
#[test]
fn tc_042_wire_types_deny_unknown_fields() {
    fn assert_denies_unknown_field<T>(mut value: serde_json::Value, type_name: &str)
    where
        T: serde::de::DeserializeOwned,
    {
        // Sanity: the unmodified value must still deserialize, or this test
        // would trivially "pass" against a type that rejects everything.
        assert!(
            serde_json::from_value::<T>(value.clone()).is_ok(),
            "{type_name}: the unmodified value must deserialize"
        );
        value
            .as_object_mut()
            .unwrap_or_else(|| panic!("{type_name}: fixture must serialize to a JSON object"))
            .insert("unexpected_field".to_owned(), serde_json::json!(true));
        assert!(
            serde_json::from_value::<T>(value).is_err(),
            "{type_name}: an unknown field must be refused, not silently ignored"
        );
    }

    let witness = Witness::parse("clause", "kani-bounded/1.0.0", real_playback_block())
        .expect("a real falsified concrete-playback block parses");
    assert_denies_unknown_field::<Witness>(
        serde_json::to_value(&witness).expect("Witness serializes"),
        "Witness",
    );
    assert_denies_unknown_field::<WitnessBinding>(
        serde_json::to_value(&balance_schema()[0]).expect("WitnessBinding serializes"),
        "WitnessBinding",
    );
    assert_denies_unknown_field::<WitnessValue>(
        serde_json::to_value(WitnessValue::Boolean(true)).expect("WitnessValue serializes"),
        "WitnessValue",
    );
    assert_denies_unknown_field::<CounterexamplePacket>(
        serde_json::to_value(packet()).expect("CounterexamplePacket serializes"),
        "CounterexamplePacket",
    );
}

// PR #156 review (finding 4): the `Input` arm was the only arm round-tripped
// above, so the externally tagged `ReplaySource::Witness(_)` wrapping
// `Witness`'s hand-written `Deserialize` was never exercised inside a real
// packet. A bad `rename_all` or a tagging interaction would have broken
// every real packet on the wire with the suite green.
//
// Deliberately untraced: FR-031-AC-4 covers witness parsing, typing, and
// arity/width/comment refusal only, not wire round-tripping of
// `CounterexamplePacket` or `ReplaySource`'s enum tagging; no other
// acceptance criterion covers it yet.
#[test]
fn tc_042_witness_arm_packet_round_trips_through_serde() {
    let mut with_witness = packet();
    with_witness.source = ReplaySource::Witness(
        Witness::parse("clause", "kani-bounded/1.0.0", real_playback_block())
            .expect("a real falsified concrete-playback block parses"),
    );
    let wire = serde_json::to_value(&with_witness).expect("Witness-arm packet serializes");
    let round_tripped: CounterexamplePacket =
        serde_json::from_value(wire).expect("Witness-arm packet deserializes");
    assert_eq!(
        round_tripped, with_witness,
        "a Witness-arm packet must round-trip through serde byte-identically"
    );
}
