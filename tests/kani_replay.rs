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
    assert!(
        !agreement.witness_backed,
        "packet() carries witness: None; the agreement must record that"
    );
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
    assert!(
        !agreement.witness_backed,
        "packet() carries witness: None; the agreement must record that"
    );
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

#[trace("TC-042", "FR-031-AC-4")]
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

#[trace("TC-042", "FR-031-AC-4")]
#[test]
fn tc_042_witness_cover_playback_is_refused() {
    let refusal = Witness::parse("clause", "kani-bounded/1.0.0", cover_playback_block())
        .expect_err("a cover playback witnesses reachability, not falsity");
    assert_eq!(refusal.kind, KaniOutcomeKind::Refused);
    assert_eq!(refusal.code, "kani_witness_cover_refused");
}

#[trace("TC-042", "FR-031-AC-4")]
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

#[trace("TC-042", "FR-031-AC-4")]
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

#[trace("TC-042", "FR-031-AC-4")]
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

#[trace("TC-042", "FR-031-AC-4")]
#[test]
fn tc_042_witness_decode_refuses_comment_disagreement() {
    let mut witness = Witness::parse("clause", "kani-bounded/1.0.0", real_playback_block())
        .expect("a real falsified concrete-playback block parses");
    // `transcript` is the single source of truth for the concrete bytes
    // (F3), so disagreement can only be introduced by editing it directly,
    // the way a corrupted or hand-edited packet would: keep the `// 8`
    // comment but change the bytes it names.
    witness.transcript = witness.transcript.replace(
        "vec![8, 0, 0, 0, 0, 0, 0, 0]",
        "vec![9, 0, 0, 0, 0, 0, 0, 0]",
    );
    let refusal = witness
        .decode(&balance_schema())
        .expect_err("a decoded value that disagrees with Kani's own comment must refuse");
    assert_eq!(refusal.kind, KaniOutcomeKind::InvalidInput);
    assert_eq!(refusal.code, "kani_witness_comment_mismatch");
}

#[trace("TC-042", "FR-031-AC-4")]
#[test]
fn tc_042_counterexample_packet_refuses_cover_witness() {
    let mut with_cover = packet();
    with_cover.witness = Some(Witness {
        transcript: cover_playback_block().trim().into(),
    });
    let refusal = replay_counterexample(with_cover, |_| {
        KaniOutcome::counterexample("clause", "native")
    })
    .expect_err("a cover witness never backs a counterexample packet");
    assert_eq!(refusal.kind, KaniOutcomeKind::InvalidInput);
    assert_eq!(refusal.code, "kani_replay_witness_invalid");
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
#[trace("TC-042", "FR-031-AC-4")]
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
#[trace("TC-042", "FR-031-AC-4")]
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

// F1: a `Witness` built directly from a raw, un-selected multi-block
// transcript (the shape a `Deserialize`d packet arrives in, since
// `Deserialize` bypasses `Witness::parse`'s block selection entirely) must
// still narrow to the real assertion block for every derived fact,
// including `concrete_values` — not read concrete values out of whichever
// block happens to appear first in the raw text.
#[trace("TC-042", "FR-031-AC-4")]
#[test]
fn tc_042_witness_direct_construction_narrows_concrete_values_to_selected_block() {
    let multi_block_transcript = format!("{}\n{}", cover_playback_block(), real_playback_block());
    let witness = Witness {
        transcript: multi_block_transcript,
    };
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

#[trace("TC-042", "FR-031-AC-4")]
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
// wrong. `Witness` no longer has a `check`/`harness_symbol`/`check_text`
// field a hand-written packet could set to disagree with `transcript` (F1)
// — every one of these cases is only reachable by constructing a `Witness`
// directly from a `transcript` string, the same shape a `Deserialize`d
// packet arrives in.
#[trace("TC-042", "FR-031-AC-4")]
#[test]
fn tc_042_replay_counterexample_refuses_witness_with_untrustworthy_transcript() {
    let cases: Vec<(&str, String)> = vec![
        (
            "empty transcript: cannot possibly reproduce any concrete values",
            String::new(),
        ),
        (
            "verbatim cover playback block: witnesses reachability, not falsity (F1)",
            cover_playback_block().trim().to_owned(),
        ),
        (
            "bytes contradict their own `//` decoded-value comment",
            real_playback_block().replace(
                "vec![8, 0, 0, 0, 0, 0, 0, 0]",
                "vec![9, 0, 0, 0, 0, 0, 0, 0]",
            ),
        ),
        (
            "not a playback block at all",
            "this is not a kani playback block".to_owned(),
        ),
    ];

    for (description, transcript) in cases {
        let mut with_untrustworthy_transcript = packet();
        with_untrustworthy_transcript.witness = Some(Witness { transcript });
        let result = replay_counterexample(with_untrustworthy_transcript, |_| {
            KaniOutcome::counterexample("clause", "native")
        });
        let refusal = match result {
            Err(refusal) => refusal,
            Ok(_) => panic!("must refuse: {description}"),
        };
        assert_eq!(
            refusal.kind,
            KaniOutcomeKind::InvalidInput,
            "case: {description}"
        );
        assert_eq!(
            refusal.code, "kani_replay_witness_invalid",
            "case: {description}"
        );
    }
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

#[trace("TC-042", "FR-031-AC-4")]
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

#[trace("TC-042", "FR-031-AC-4")]
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

#[trace("TC-042", "FR-031-AC-4")]
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

#[trace("TC-042", "FR-031-AC-4")]
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

// F5: a `None` witness must not earn the same unqualified agreement as one
// reproduced together with its evaluated witness; `ReplayAgreement` and
// `NativeReplayAgreement` must carry a field that distinguishes the two. The
// only previously covered `Some` case was the cover refusal, leaving the
// `Some(Assertion)` accept path untested.
#[trace("TC-042", "FR-031-AC-4")]
#[test]
fn tc_042_replay_counterexample_marks_agreement_witness_backed_for_assertion_witness() {
    let mut with_witness = packet();
    with_witness.witness = Some(
        Witness::parse("clause", "kani-bounded/1.0.0", real_playback_block())
            .expect("a real falsified concrete-playback block parses"),
    );
    let agreement = replay_counterexample(with_witness, |_| {
        KaniOutcome::counterexample("clause", "native")
    })
    .expect("a Some(Assertion) witness must reproduce alongside its native agreement");
    assert!(
        agreement.witness_backed,
        "the agreement must record that it was reproduced together with its evaluated witness"
    );
}

#[trace("TC-042", "FR-031-AC-4")]
#[test]
fn tc_042_replay_counterexample_marks_agreement_not_witness_backed_for_none() {
    let agreement = replay_counterexample(packet(), |_| {
        KaniOutcome::counterexample("clause", "native")
    })
    .expect("a witness-free packet is a real, honestly modeled state");
    assert!(
        !agreement.witness_backed,
        "a None witness must not earn the same unqualified agreement as an evaluated one"
    );
}

// F4 (PR #139 review): the new wire types must deny unknown fields, matching
// the repo's ~20 other `#[serde(deny_unknown_fields)]` sites, so a stray or
// misspelled field in a hand-authored or corrupted packet is refused at
// deserialization rather than silently ignored.
#[trace("TC-042", "FR-031-AC-4")]
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
