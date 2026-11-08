// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (C) 2026 Agent-IX

//! FR-361/TC-471 over a genuine QSL tick-monitor plan root: QSL emits the plan and
//! admits its order and progress authority; IR binds the authority row to the
//! selected closure of the same emission.

use std::{env, fs, path::PathBuf};

use ix_trace_rs::trace;
use qsl_analyze::monitor_root::{
    emit_plan_root, AdmittedPlanAuthority, PlanAuthoritySelection, PlanAuthorityValidationCaps,
    PlanRoot, PlanRootRequest,
};
use qsl_foundation::SourceIdentity;
use qsl_replay::spine::SpineLimits;
use qsl_replay::{MonitorTarget, TimingObligationSettings};
use qsl_semantics::check::TimeSourceKind;
use qsl_semantics::library::LibraryName;
use quire_contract_model::{
    read_checked_package, CheckedMonitorClock, CheckedNodeId, CheckedOccurrence,
    CheckedOccurrenceRole, CheckedPackageDispatchResult, CheckedPackageReadLimits,
    CheckedPackageV2, CheckedRational, NativeJoinBudget, NativeJoinLocus, NativeJoinLocusSource,
    SelectedMonitorAuthorityComponent, SelectedMonitorAuthorityErrorCode,
    SelectedMonitorAuthorityRow, SelectedMonitorClosureRows, SelectedMonitorTimeSource,
    SelectedMonitorTypeClosure, SelectedMonitorTypeClosureError,
    SelectedMonitorTypeClosureErrorCode,
};
use quire_exact::{Cancel, Integer, Rational};
use quire_observation::authority::{self, clock, progress, SubjectSelection};
use serde_json::{json, Value};

#[path = "support/observation.rs"]
mod observation;

const LIMITS: CheckedPackageReadLimits = CheckedPackageReadLimits {
    bytes: 1 << 26,
    nodes: 1_000_000,
    edges: 10_000_000,
    occurrences: 10_000_000,
    diagnostics: 10_000,
    work: 1_000_000_000,
};

const FORMULA: &str =
    "always (holds(c.versionNumber = 2) implies eventually[0 ms, 3 ms] holds(c.versionNumber = 3))";

fn ratio(numerator: i64, denominator: i64) -> Rational {
    Rational::new(Integer::from(numerator), Integer::from(denominator)).expect("nonzero")
}

fn target() -> MonitorTarget {
    MonitorTarget {
        counter: "counter".to_owned(),
        unit: "us".to_owned(),
        width_bits: 12,
        max_reading_gap: 3600,
        uncertainty: ratio(0, 1),
        max_event_rate: Some(ratio(2, 1000)),
        max_activations: 8,
    }
}

fn domain_document(qsl: &std::path::Path) -> Vec<u8> {
    let mut document: Value = serde_json::from_slice(
        &fs::read(qsl.join("examples/config-version/model.semantic-ir.json"))
            .expect("owner domain package by reference"),
    )
    .expect("domain package JSON");
    let frame = &mut document["types"][1]["operations"][0]["frame"];
    let field = frame["modifies"][0].clone();
    *frame = json!({"modifies": [], "modifies_receiver": [field], "creates": [], "deletes": []});
    document.to_string().into_bytes()
}

fn authored(time_source: &str) -> String {
    format!(
        "language \"ix:native\" edition \"1-draft\";\n\
         profile v = \"quire.value.complete/v1\";\n\
         profile t = \"quire.temporal.timed/v1\";\n\
         profile tc = \"quire.temporal.complete/v1\";\n\
         model M = \"example/config-version\";\n\
         dimension Time;\n\
         unit s : Time = rational(1, 1);\n\
         unit ms : Time = rational(1, 1000) * s;\n\
         unit us : Time = rational(1, 1000000) * s;\n\
         {time_source}\n\
         temporal C using t over (c: M::ConfigVersion) clock \"model-time\" on origin {{ {FORMULA} }}\n"
    )
}

fn root_of(qsl: &std::path::Path, time_source: &str) -> PlanRoot {
    let document = domain_document(qsl);
    let packages = qsl_semantics::model::intake::package_input([document.as_slice()]).unwrap();
    let source = SourceIdentity::new("agent-ix", "monitor-root");
    let name = LibraryName::new("fixture").unwrap();
    let text = authored(time_source);
    emit_plan_root(
        &PlanRootRequest {
            source: &source,
            path: "monitor.native",
            bytes: text.as_bytes(),
            packages: &packages,
            package_name: &name,
            limits: SpineLimits::default(),
            clause: "C",
            target: &target(),
            settings: &TimingObligationSettings::default(),
        },
        &Cancel::new(),
    )
    .expect("genuine QSL plan root")
}

fn node(key: quire_exact::NodeKey) -> progress::NodeIdentity {
    progress::NodeIdentity::from_key(key, authority::Limits::owner_max()).unwrap()
}

fn row(root: &PlanRoot) -> progress::AssociationRow {
    let source = &root.checked_unit().package().graph().time_sources()[0];
    let claim = root.plan().subject();
    progress::AssociationRow {
        model: source.model.clone(),
        model_node: node(source.model_node.clone()),
        claim: progress::ClaimOccurrence::new(
            node(claim.clause_node().clone()),
            claim.occurrence().origin(),
        ),
        source: match &source.source {
            TimeSourceKind::Dense => progress::TimeSource::Dense,
            TimeSourceKind::Tick { operation, period } => progress::TimeSource::Tick {
                operation: operation.clone(),
                operation_node: node(source.tick_operation_anchor.as_ref().unwrap().clone()),
                period_numerator: period.numerator().to_string(),
                period_denominator: period.denominator().to_string(),
            },
        },
        unit: source.unit.clone(),
        unit_node: node(source.unit_node.clone()),
        time_dimension: node(source.time_dimension.clone()),
        observation_source: observation::source().as_str().to_owned(),
        observation_clock: "clock:monitor-events".to_owned(),
        observation_revision: "1".to_owned(),
    }
}

fn caps() -> PlanAuthorityValidationCaps {
    PlanAuthorityValidationCaps {
        read: LIMITS,
        order: quire_canonical::Limits::new(1 << 20),
    }
}

/// A genuine authority and the independently derived values it must carry.
struct Emitted {
    authority: AdmittedPlanAuthority,
    expected_order: Vec<u8>,
    expected_progress: Vec<u8>,
}

fn emit(qsl: &std::path::Path, time_source: &str) -> Emitted {
    let root = root_of(qsl, time_source);
    let (progress, subject) = observation::admitted(Some(vec![row(&root)]));
    let expected_progress = progress.association_canonical_bytes().unwrap().to_vec();
    let expected_order = root
        .model_step_order(caps().order)
        .unwrap()
        .bytes()
        .to_vec();
    let source = root.checked_unit().source().identity().clone();
    let claim = root.plan().subject().clone();
    let observation_clock: clock::Selection = observation::clock();
    let observation_source = observation::source();
    let selection_subject: &SubjectSelection = &subject;
    let authority = root
        .admit_authority(
            PlanAuthoritySelection {
                source: &source,
                claim: &claim,
                observation_subject: selection_subject,
                observation_clock: &observation_clock,
                observation_source: &observation_source,
            },
            Some(progress),
            caps(),
            &mut NativeJoinBudget::new(100_000_000, 100_000_000)
                .with_read_storage_limit(1 << 28),
            NativeJoinLocus {
                source: NativeJoinLocusSource::ReachedRow(0),
                form: "qsl_authority_selection",
            },
        )
        .expect("QSL admits the same-emission authority");
    Emitted {
        authority,
        expected_order,
        expected_progress,
    }
}

/// The IR row a driver builds from QSL's authority, changing no byte or value.
fn ir_row(authority: &AdmittedPlanAuthority) -> SelectedMonitorAuthorityRow<'_> {
    let package = authority.admitted_package();
    let clause = package.monitor_plans().next().unwrap().clause().clone();
    let claim = authority.root().plan().subject();
    SelectedMonitorAuthorityRow::try_new(
        authority.admission_binding(),
        clause,
        CheckedOccurrence {
            role: CheckedOccurrenceRole::Claim,
            ordinal: u64::from(claim.occurrence().origin().ordinal()),
        },
        match authority.checked_tick_period() {
            Some(period) => SelectedMonitorTimeSource::Tick {
                period: period.clone(),
            },
            None => SelectedMonitorTimeSource::Dense,
        },
        authority.admitted_order(),
        authority.progress_context(),
    )
    .expect("a genuine authority builds a row")
}

fn project<'a>(
    authority: &'a AdmittedPlanAuthority,
    package: &'a CheckedPackageV2,
    row: &'a SelectedMonitorAuthorityRow<'a>,
    budget: &mut NativeJoinBudget,
) -> Result<SelectedMonitorTypeClosure<'a>, SelectedMonitorTypeClosureError> {
    let root = authority.root();
    let emission = root.emission();
    let mut prepare = NativeJoinBudget::new(100_000_000, 100_000_000);
    let identities = Box::leak(Box::new(
        emission
            .identity_rows(package, &mut prepare)
            .expect("same-emission identity rows"),
    ));
    let equality = Box::leak(Box::new(
        emission
            .equality_witness(package, &mut prepare)
            .expect("same-emission equality rows"),
    ));
    let enums: &'static _ = Vec::leak(
        emission
            .selected_source_enum_rows(&mut prepare)
            .expect("source enum adapter"),
    );
    let shapes: &'static _ = Vec::leak(
        emission
            .selected_ir_model_enum_shape_rows(&mut prepare)
            .expect("model enum adapter"),
    );
    let collections: &'static _ = Vec::leak(
        emission
            .selected_source_collection_rows()
            .iter()
            .enumerate()
            .map(|(index, collection)| {
                collection
                    .to_ir(
                        &mut prepare,
                        NativeJoinLocus {
                            source: NativeJoinLocusSource::AdapterRow(index),
                            form: "monitor_source_collection_row",
                        },
                    )
                    .expect("collection converts")
            })
            .collect(),
    );
    package
        .monitor_plans()
        .next()
        .expect("one plan root")
        .project_selected_type_closure(
            SelectedMonitorClosureRows {
                declarations: identities.declarations(),
                fields: identities.fields(),
                model_tuples: emission.selected_tuple_declaration_rows(),
                source_units: emission.selected_source_unit_rows(),
                model_enum_declarations: emission.selected_model_enum_declaration_rows(),
                model_enum_members: emission.selected_model_enum_member_rows(),
                model_enum_shapes: shapes,
                source_records: identities.source_records(),
                source_tuples: emission.selected_source_tuple_rows(),
                source_unions: identities.source_unions(),
                source_enums: enums,
                source_collections: collections,
                equality_applications: equality.applications(),
                equality_facts: equality.facts(),
                authority: row,
            },
            budget,
        )
}

fn big() -> NativeJoinBudget {
    NativeJoinBudget::new(1_000_000_000, 1_000_000_000)
}

fn code(error: &SelectedMonitorTypeClosureError) -> SelectedMonitorAuthorityErrorCode {
    match error.code {
        SelectedMonitorTypeClosureErrorCode::Authority(code) => code,
        other => panic!("expected an authority refusal, got {other:?}"),
    }
}

fn qsl_dir() -> PathBuf {
    env::var_os("QUIRE_SPEC_LANGUAGE_DIR")
        .map(PathBuf::from)
        .expect("the lane reads the owner's domain package from QUIRE_SPEC_LANGUAGE_DIR")
}

#[trace("TC-471", "FR-361-AC-1", "FR-361-AC-4", "FR-361-AC-6")]
#[test]
fn genuine_qsl_authority_binds_dense_and_tick_emissions() {
    let qsl = qsl_dir();
    let qsl = qsl.as_path();
    for (label, declaration, period) in [
        (
            "tick",
            "time M tick M::ConfigVersion::attemptUpdate period 2 ms unit ms;",
            Some(("2", "1")),
        ),
        ("dense", "time M dense unit ms;", None),
    ] {
        let emitted = emit(qsl, declaration);
        let authority = &emitted.authority;
        let package = authority.admitted_package();
        let row = ir_row(authority);
        let mut budget = big();
        let closure = project(authority, package, &row, &mut budget)
            .unwrap_or_else(|error| panic!("{label}: healthy emission binds: {error}"));
        let receipt = closure.authority();
        let plan = package.monitor_plans().next().unwrap();
        assert_eq!(receipt.clause(), plan.clause(), "{label}");
        assert_eq!(
            receipt.order_value(),
            emitted.expected_order.as_slice(),
            "{label}"
        );
        assert_eq!(
            receipt.progress_association(),
            emitted.expected_progress.as_slice(),
            "{label}"
        );
        assert_eq!(
            receipt.occurrence(),
            receipt.clock().occurrence(),
            "{label}"
        );
        assert_eq!(
            receipt.occurrence().role,
            CheckedOccurrenceRole::Claim,
            "{label}"
        );
        assert_eq!(receipt.clock().clause(), plan.clause(), "{label}");
        assert!(
            matches!(
                receipt.clock().clock(),
                CheckedMonitorClock::ModelTime { .. }
            ),
            "{label}"
        );
        assert_eq!(
            authority
                .checked_tick_period()
                .map(|p| (&*p.numerator, &*p.denominator)),
            period
        );

        // The exact measured work and storage allowance succeeds; one unit
        // below either refuses at a charge, with no closure.
        let spent = budget.consumed();
        let exact = &mut NativeJoinBudget::new(spent.work, spent.storage);
        assert!(
            project(authority, package, &row, exact).is_ok(),
            "{label}: exact allowance"
        );
        let below = project(
            authority,
            package,
            &row,
            &mut NativeJoinBudget::new(spent.work - 1, spent.storage),
        )
        .expect_err("one unit of work below the allowance");
        assert_eq!(
            below.code,
            SelectedMonitorTypeClosureErrorCode::WorkLimit,
            "{label}"
        );
        let below = project(
            authority,
            package,
            &row,
            &mut NativeJoinBudget::new(spent.work, spent.storage - 1),
        )
        .expect_err("one unit of storage below the allowance");
        assert_eq!(
            below.code,
            SelectedMonitorTypeClosureErrorCode::StorageLimit,
            "{label}"
        );
    }
}

#[trace("TC-471", "FR-361-AC-2", "FR-361-AC-6")]
#[test]
fn mismatched_emissions_refuse_with_typed_authority_errors() {
    let qsl = qsl_dir();
    let qsl = qsl.as_path();
    let emitted = emit(
        qsl,
        "time M tick M::ConfigVersion::attemptUpdate period 2 ms unit ms;",
    );
    let authority = &emitted.authority;
    let package = authority.admitted_package();
    let plan = package.monitor_plans().next().unwrap();
    let genuine = ir_row(authority);
    let claim = CheckedOccurrence {
        role: CheckedOccurrenceRole::Claim,
        ordinal: u64::from(
            authority
                .root()
                .plan()
                .subject()
                .occurrence()
                .origin()
                .ordinal(),
        ),
    };
    let with =
        |clause: CheckedNodeId, occurrence: CheckedOccurrence, time: SelectedMonitorTimeSource| {
            SelectedMonitorAuthorityRow::try_new(
                authority.admission_binding(),
                clause,
                occurrence,
                time,
                authority.admitted_order(),
                authority.progress_context(),
            )
            .unwrap()
        };
    let tick = |numerator: &str, denominator: &str| SelectedMonitorTimeSource::Tick {
        period: CheckedRational {
            numerator: numerator.into(),
            denominator: denominator.into(),
        },
    };

    // An equal, unreduced period is the same exact period.
    let unreduced = with(plan.clause().clone(), claim.clone(), tick("4", "2"));
    assert!(project(authority, package, &unreduced, &mut big()).is_ok());

    // A second admission of the same plan root's bytes is a foreign emission.
    let independent = match read_checked_package(
        authority.root().emission().package().bytes(),
        LIMITS,
        authority.root().emission().evidence(),
    ) {
        CheckedPackageDispatchResult::AdmittedV2(package) => package,
        other => panic!("independent strict read: {other:?}"),
    };
    let error = project(authority, &independent, &genuine, &mut big())
        .expect_err("equal bytes of another admission are foreign");
    assert_eq!(
        code(&error),
        SelectedMonitorAuthorityErrorCode::ForeignEmission
    );
    assert_eq!(
        error.at.as_deref(),
        Some(independent.monitor_plans().next().unwrap().node_id())
    );
    assert!(error.work > 0);

    let other_clause = with(plan.state().clone(), claim.clone(), tick("2", "1"));
    let error = project(authority, package, &other_clause, &mut big()).expect_err("clause");
    assert_eq!(
        code(&error),
        SelectedMonitorAuthorityErrorCode::ClauseMismatch
    );
    assert_eq!(error.at.as_deref(), Some(plan.node_id()));

    let absent = CheckedOccurrence {
        role: CheckedOccurrenceRole::Claim,
        ordinal: claim.ordinal + 7,
    };
    let other_occurrence = with(plan.clause().clone(), absent.clone(), tick("2", "1"));
    let error = project(authority, package, &other_occurrence, &mut big()).expect_err("ordinal");
    assert_eq!(
        code(&error),
        SelectedMonitorAuthorityErrorCode::OccurrenceMismatch
    );
    assert_eq!(error.at.as_deref(), Some(plan.clause()));
    assert_eq!(error.occurrence, Some(absent));

    for (label, time) in [
        (
            "dense row against a tick source",
            SelectedMonitorTimeSource::Dense,
        ),
        ("different exact period", tick("3", "1")),
        ("different denominator", tick("2", "3")),
        ("zero denominator", tick("0", "0")),
        ("invalid numerator", tick("not-a-number", "1")),
    ] {
        let wrong = with(plan.clause().clone(), claim.clone(), time);
        let error = project(authority, package, &wrong, &mut big()).expect_err(label);
        assert_eq!(
            code(&error),
            SelectedMonitorAuthorityErrorCode::TimeSourceMismatch,
            "{label}"
        );
        assert_eq!(error.at.as_deref(), Some(plan.clause()), "{label}");
        assert_eq!(error.occurrence, Some(claim.clone()), "{label}");
        assert!(
            matches!(
                error.denied_locus,
                None | Some(NativeJoinLocus {
                    source: NativeJoinLocusSource::MonitorSeed(_),
                    ..
                })
            ),
            "{label}: {:?}",
            error.denied_locus
        );
    }

    // The foreign check precedes the clause check.
    let foreign_and_wrong = with(plan.state().clone(), claim.clone(), tick("2", "1"));
    let error = project(authority, &independent, &foreign_and_wrong, &mut big()).unwrap_err();
    assert_eq!(
        code(&error),
        SelectedMonitorAuthorityErrorCode::ForeignEmission
    );

    // The dense emission's authority refuses the tick emission's package.
    let dense = emit(qsl, "time M dense unit ms;");
    let dense_row = ir_row(&dense.authority);
    let error = project(authority, package, &dense_row, &mut big()).unwrap_err();
    assert_eq!(
        code(&error),
        SelectedMonitorAuthorityErrorCode::ForeignEmission
    );

    // try_new refuses, in order, a non-claim role, an empty order and an empty association.
    let build = |role, order: &[u8], progress: &[u8]| {
        SelectedMonitorAuthorityRow::try_new(
            authority.admission_binding(),
            plan.clause().clone(),
            CheckedOccurrence { role, ordinal: 0 },
            SelectedMonitorTimeSource::Dense,
            order,
            progress,
        )
        .expect_err("refused row")
        .component
    };
    assert_eq!(
        build(CheckedOccurrenceRole::Expression, b"", b""),
        SelectedMonitorAuthorityComponent::Occurrence
    );
    assert_eq!(
        build(CheckedOccurrenceRole::Claim, b"", b""),
        SelectedMonitorAuthorityComponent::OrderValue
    );
    assert_eq!(
        build(CheckedOccurrenceRole::Claim, b"o", b""),
        SelectedMonitorAuthorityComponent::ProgressAssociation
    );
}
