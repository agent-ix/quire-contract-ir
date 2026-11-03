// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (C) 2026 Agent-IX

//! TC-978 monitor recipes are read from the selected QSpec checkout by reference.

use super::monitor_plan_fixtures::*;
use super::*;
use quire_contract_model::CheckedMonitorPlanRef;
use std::collections::BTreeSet;

fn admitted(wire: &Value) -> Box<CheckedPackageV2> {
    match read_package(wire).expect("QSpec selection evidence") {
        CheckedPackageV2ReadResult::Admitted(package) => package,
        other => panic!("monitor recipe must admit: {other:?}"),
    }
}
fn assert_reads(plan: CheckedMonitorPlanRef<'_>, facts: &Value) {
    assert_eq!(
        plan.activation_capacity(),
        facts["premises"]["activation_capacity"].to_string()
    );
    assert_eq!(
        u64::from(plan.counter_width_bits()),
        facts["premises"]["counter_width_bits"].as_u64().unwrap()
    );
    assert_eq!(
        plan.max_reading_gap(),
        facts["premises"]["max_reading_gap"].as_u64().unwrap()
    );
    assert_eq!(
        plan.counter(),
        facts["premises"]["counter"].as_str().unwrap()
    );
    assert_eq!(
        plan.buffers().map(|b| b.capacity()).collect::<Vec<_>>(),
        facts["buffers"]
            .as_array()
            .unwrap()
            .iter()
            .map(|b| b["capacity"].to_string())
            .collect::<Vec<_>>()
    );
    for (entry, name) in [
        (plan.init(), "init"),
        (plan.on_event(), "on_event"),
        (plan.on_watermark(), "on_watermark"),
        (plan.on_gap(), "on_gap"),
    ] {
        assert_eq!(
            entry.parameter_types().len(),
            facts["entries"][name]["params"].as_array().unwrap().len()
        );
        assert!(matches!(
            entry.body(),
            quire_contract_model::CheckedTerm::Aggregate { .. }
        ));
    }
}

fn assert_activation_status_type(wire: &Value, expected: &str) {
    let activation = wire["semantic_graph"]["nodes"]
        .as_array()
        .unwrap()
        .iter()
        .find(|node| {
            node["node_tag"] == "composite_type"
                && node["semantic_form"] == "record"
                && node["body"]["members"][2]["name"] == "status"
        })
        .expect("activation record");
    let status_id = &activation["body"]["members"][2]["value"]["target"];
    let status = &wire["semantic_graph"]["nodes"][position(wire, status_id)];
    match expected {
        "integer" => assert_eq!(status["semantic_form"], "integer"),
        "record{position:integer}" => {
            assert_eq!(status["semantic_form"], "record");
            let fields = status["body"]["members"].as_array().unwrap();
            assert_eq!(fields.len(), 1);
            assert_eq!(fields[0]["name"], "position");
            let position_type = &fields[0]["value"]["target"];
            assert_eq!(
                wire["semantic_graph"]["nodes"][position(wire, position_type)]["semantic_form"],
                "integer"
            );
        }
        "text" => assert_eq!(status["semantic_form"], "text"),
        other => panic!("unexpected status assertion type {other}"),
    }
}

#[trace(
    "TC-048",
    "FR-038-AC-421",
    "FR-038-AC-422",
    "FR-038-AC-423",
    "FR-038-AC-424",
    "FR-038-AC-425",
    "FR-038-AC-426",
    "FR-038-AC-427",
    "FR-038-AC-429",
    "FR-038-AC-433"
)]
#[test]
fn monitor_plan_published_reader_recipes_have_exact_outcomes() {
    let vectors = recipes();
    let mut status_cases = BTreeSet::new();
    let mut base_key = None;
    for case in vectors["cases"].as_array().unwrap() {
        let mut facts = vectors["base"].clone();
        patch(&mut facts, &case["set"]);
        let (wire, n) = materialize(&facts);
        if matches!(case["id"].as_str(), Some("MP-01" | "MP-81" | "MP-82")) {
            assert!(status_cases.insert(case["id"].as_str().unwrap()));
            assert_eq!(case["expected"], "admitted", "{}", case["id"]);
            assert_activation_status_type(
                &wire,
                facts["state"]["activation_status"].as_str().unwrap(),
            );
        }
        let result = read_package(&wire).expect("QSpec evidence");
        if case["expected"] == "admitted" {
            let CheckedPackageV2ReadResult::Admitted(package) = result else {
                panic!("{}: {result:?}", case["id"]);
            };
            let plan = package.monitor_plans().next().expect("one root");
            assert_eq!(package.monitor_plans().len(), 1);
            assert_reads(plan, &facts);
            assert!(std::ptr::eq(
                plan.structural_key(),
                package.structural_key(plan.node_id()).unwrap()
            ));
            for entry in [
                plan.init(),
                plan.on_event(),
                plan.on_watermark(),
                plan.on_gap(),
            ] {
                assert!(std::ptr::eq(
                    entry.structural_key(),
                    package.structural_key(entry.node_id()).unwrap()
                ));
            }
            if let Some(base) = base_key.as_ref() {
                assert_ne!(plan.structural_key(), base, "{} identity", case["id"]);
            } else {
                base_key = Some(plan.structural_key().clone());
            }
        } else {
            let CheckedPackageV2ReadResult::Refused(refusal) = result else {
                panic!("{}: {result:?}", case["id"]);
            };
            assert_eq!(
                code_word(refusal.code),
                case["code"],
                "{}: {refusal:?}",
                case["id"]
            );
            assert_eq!(
                refusal.cause.map(cause_word),
                case["cause"].as_str(),
                "{}: {refusal:?}",
                case["id"]
            );
            assert_eq!(
                refusal.loci[0].path.as_ref().unwrap().as_str(),
                case["locus"]
                    .as_str()
                    .unwrap()
                    .replace("{n}", &n.to_string()),
                "{}: {refusal:?}",
                case["id"]
            );
            assert_eq!(
                refusal.locus.as_ref(),
                Some(
                    &serde_json::from_value::<quire_contract_model::CheckedNodeId>(
                        wire["semantic_graph"]["nodes"][n]["node_id"].clone()
                    )
                    .unwrap()
                ),
                "{} structural root",
                case["id"]
            );
        }
    }
    assert_eq!(status_cases, BTreeSet::from(["MP-01", "MP-81", "MP-82"]));
    assert_eq!(
        vectors["gap_cases"]["calls"].as_array().unwrap().len(),
        3,
        "MP54..56 remain runtime controls"
    );
    assert_eq!(
        vectors["gap_cases"]["sequences"][0]["id"], "MP-57",
        "runtime baseline control remains downstream"
    );
    let mut execution_ids = vec![];
    for group in [
        "calls",
        "sequences",
        "history_cases",
        "reporting_cases",
        "overflow_cases",
        "position_cases",
    ] {
        execution_ids.extend(
            vectors["gap_cases"][group]
                .as_array()
                .expect("published execution controls")
                .iter()
                .map(|case| case["id"].as_str().unwrap()),
        );
    }
    execution_ids.sort_unstable();
    let expected = (54..=57)
        .chain(66..=73)
        .chain(std::iter::once(77))
        .map(|n| format!("MP-{n:02}"))
        .collect::<Vec<_>>();
    assert_eq!(
        execution_ids, expected,
        "execution controls are read by reference, never run or converted to admission verdicts"
    );
}

fn refusal(wire: &Value, code: Code, cause: Option<Cause>, path: &str, root: usize) {
    let result = read_package(wire).expect("selected QSpec evidence");
    let CheckedPackageV2ReadResult::Refused(refusal) = result else {
        panic!("expected {path}: {result:?}");
    };
    assert_eq!((refusal.code, refusal.cause), (code, cause), "{refusal:?}");
    assert_eq!(
        refusal.loci[0].path.as_ref().unwrap().as_str(),
        path,
        "{refusal:?}"
    );
    assert_eq!(
        refusal.locus.as_ref(),
        Some(
            &serde_json::from_value::<quire_contract_model::CheckedNodeId>(
                wire["semantic_graph"]["nodes"][root]["node_id"].clone()
            )
            .unwrap()
        ),
        "root locus"
    );
}
fn value_mut(wire: &mut Value, root: usize, slot: usize) -> &mut Value {
    &mut wire["semantic_graph"]["nodes"][root]["body"]["members"][slot]["value"]
}
fn binding_path(root: usize, slot: usize) -> String {
    format!("/semantic_graph/nodes/{root}/body/members/{slot}")
}

#[trace(
    "TC-048",
    "FR-038-AC-422",
    "FR-038-AC-423",
    "FR-038-AC-424",
    "FR-038-AC-425",
    "FR-038-AC-428",
    "FR-038-AC-430"
)]
#[test]
fn monitor_plan_adjacent_stages_and_premises_keep_first_refusal() {
    let vectors = recipes();
    // Each pair has two independently observable defects. Removing the first
    // exposes the second, including declaration and expression checks.
    for (a, b) in [
        ("MP-04", "MP-07"),
        ("MP-07", "MP-10"),
        ("MP-10", "MP-18"),
        ("MP-18", "MP-23"),
        ("MP-18", "MP-20"),
        ("MP-20", "MP-21"),
        ("MP-21", "MP-22"),
        ("MP-22", "MP-38"),
        ("MP-38", "MP-34"),
        ("MP-34", "MP-59"),
        ("MP-23", "MP-26"),
    ] {
        let case = |id: &str| {
            vectors["cases"]
                .as_array()
                .unwrap()
                .iter()
                .find(|c| c["id"] == id)
                .unwrap()
        };
        let mut facts = vectors["base"].clone();
        patch(&mut facts, &case(a)["set"]);
        patch(&mut facts, &case(b)["set"]);
        let (wire, root) = materialize(&facts);
        let expected = case(a);
        refusal(
            &wire,
            serde_json::from_value::<Code>(expected["code"].clone()).unwrap(),
            expected["cause"]
                .as_str()
                .map(|c| serde_json::from_value::<Cause>(json!(c)).unwrap()),
            &expected["locus"]
                .as_str()
                .unwrap()
                .replace("{n}", &root.to_string()),
            root,
        );
        let mut only_second = vectors["base"].clone();
        patch(&mut only_second, &case(b)["set"]);
        let (wire, root) = materialize(&only_second);
        let expected = case(b);
        refusal(
            &wire,
            serde_json::from_value::<Code>(expected["code"].clone()).unwrap(),
            expected["cause"]
                .as_str()
                .map(|c| serde_json::from_value::<Cause>(json!(c)).unwrap()),
            &expected["locus"]
                .as_str()
                .unwrap()
                .replace("{n}", &root.to_string()),
            root,
        );
    }
    let (mut wire, root) = materialize(&vectors["base"]);
    value_mut(&mut wire, root, 8)["value"] = json!("0");
    wire["semantic_graph"]["nodes"][root]["dependencies"] = json!([]);
    refusal(
        &wire,
        Code::InvalidSemanticGraph,
        None,
        &format!("/semantic_graph/nodes/{root}/dependencies"),
        root,
    );
    let (mut wire, root) = materialize(&vectors["base"]);
    value_mut(&mut wire, root, 8)["value"] = json!("0");
    let entry = position(
        &wire,
        &wire["semantic_graph"]["nodes"][root]["body"]["members"][5]["value"]["target"],
    );
    let expression = position(
        &wire,
        &wire["semantic_graph"]["nodes"][entry]["body"]["members"][1]["value"]["target"],
    );
    wire["semantic_graph"]["nodes"][expression]["body"]["operation"]["identity"] =
        json!("quire.op.no_such_operation");
    let result = read_package(&wire).unwrap();
    let CheckedPackageV2ReadResult::Refused(refused) = result else {
        panic!("expression defect must precede plan: {result:?}")
    };
    assert_eq!(
        (refused.code, refused.cause),
        (Code::InvalidPackage, Some(Cause::UnknownOperation))
    );
    assert_eq!(
        refused.loci[0].path.as_ref().unwrap().as_str(),
        format!("/semantic_graph/nodes/{expression}/body/operation/identity")
    );
}

#[trace(
    "TC-048",
    "FR-038-AC-426",
    "FR-038-AC-428",
    "FR-038-AC-429",
    "FR-038-AC-430"
)]
#[test]
fn monitor_plan_exact_reads_allow_unbounded_capacities_and_opaque_status() {
    let vectors = recipes();
    let (mut wire, root) = materialize(&vectors["base"]);
    let large = "184467440737095516160000000000000000000000000000000000001";
    value_mut(&mut wire, root, 16)["value"] = json!(large);
    value_mut(&mut wire, root, 11)["members"][0]["members"][1]["value"]["value"] = json!(large);
    value_mut(&mut wire, root, 8)["value"] = json!("64");
    value_mut(&mut wire, root, 9)["value"] = json!(u64::MAX.to_string());
    value_mut(&mut wire, root, 10)["value"] = json!(format!("{large}/2"));
    value_mut(&mut wire, root, 14)["value"] = json!("0");
    let package = admitted(&wire);
    let plan = package.monitor_plans().next().unwrap();
    assert_eq!(plan.counter_width_bits(), 64);
    assert_eq!(plan.max_reading_gap(), u64::MAX);
    assert_eq!(plan.activation_capacity(), large);
    assert_eq!(plan.buffers().next().unwrap().capacity(), large);
    assert_eq!(plan.max_event_rate().numerator.as_ref(), large);
    assert_eq!(plan.max_event_rate().denominator.as_ref(), "2");
    assert_eq!(plan.uncertainty().numerator.as_ref(), "0");
    assert_eq!(plan.uncertainty().denominator.as_ref(), "1");
    assert_eq!(
        plan.on_gap().parameter_types().collect::<Vec<_>>(),
        vec![
            plan.state(),
            package
                .graph()
                .nodes
                .iter()
                .find(|n| n.semantic_form.as_ref() == "integer")
                .map(|n| &n.node_id)
                .unwrap()
        ]
    );
    assert_eq!(plan.init().result_type(), plan.state());
    assert_eq!(plan.body(), package.term(plan.node_id()).unwrap());
    for (slot, id) in [
        (0, plan.clause()),
        (1, plan.state()),
        (2, plan.inputs()),
        (3, plan.scope()),
        (4, plan.results()),
        (13, plan.tick_unit()),
    ] {
        assert_eq!(
            serde_json::to_value(id).unwrap(),
            wire["semantic_graph"]["nodes"][root]["body"]["members"][slot]["value"]["target"]
        );
    }
    // Status has no plan-specific type. An ordinary different admitted type
    // remains usable without executing or inferring any activation status.
    let activation = wire["semantic_graph"]["nodes"]
        .as_array()
        .unwrap()
        .iter()
        .position(|n| {
            n["node_tag"] == "composite_type"
                && n["semantic_form"] == "record"
                && n["body"]["members"][2]["name"] == "status"
        })
        .unwrap();
    let integer = find(&wire, "integer");
    wire["semantic_graph"]["nodes"][activation]["body"]["members"][2]["value"] =
        reference(&integer);
    finish(&mut wire);
    assert_eq!(admitted(&wire).monitor_plans().len(), 1);
    for raw in ["+1", "01", "1/1", "0/1", "2/4", "1/0", "1/-2"] {
        let (mut wire, root) = materialize(&vectors["base"]);
        value_mut(&mut wire, root, 14)["value"] = json!(raw);
        refusal(
            &wire,
            Code::InvalidPackage,
            Some(Cause::InvalidValue),
            &format!("{}/value", binding_path(root, 14)),
            root,
        );
    }
}

#[trace("TC-048", "FR-038-AC-424", "FR-038-AC-430")]
#[test]
fn monitor_plan_state_layout_rejects_order_extra_and_wrong_activation_types() {
    let vectors = recipes();
    for mutation in 0..7 {
        let (mut wire, root) = materialize(&vectors["base"]);
        let state = position(
            &wire,
            &wire["semantic_graph"]["nodes"][root]["body"]["members"][1]["value"]["target"],
        );
        let activation = wire["semantic_graph"]["nodes"]
            .as_array()
            .unwrap()
            .iter()
            .position(|n| {
                n["node_tag"] == "composite_type"
                    && n["semantic_form"] == "record"
                    && n["body"]["members"][2]["name"] == "status"
            })
            .unwrap();
        let text = find(&wire, "text");
        match mutation {
            0 => wire["semantic_graph"]["nodes"][state]["body"]["members"]
                .as_array_mut()
                .unwrap()
                .swap(0, 1),
            1 => wire["semantic_graph"]["nodes"][state]["body"]["members"]
                .as_array_mut()
                .unwrap()
                .push(binding("extra", reference(&text))),
            2 => {
                wire["semantic_graph"]["nodes"][state]["body"]["members"][4]["value"] =
                    reference(&text)
            }
            3 => wire["semantic_graph"]["nodes"][activation]["body"]["members"]
                .as_array_mut()
                .unwrap()
                .swap(0, 1),
            4 | 5 => {
                wire["semantic_graph"]["nodes"][activation]["body"]["members"][mutation - 4]
                    ["value"] = reference(&text)
            }
            6 => wire["semantic_graph"]["nodes"][activation]["body"]["members"]
                .as_array_mut()
                .unwrap()
                .push(binding("extra", reference(&text))),
            _ => unreachable!(),
        }
        finish(&mut wire);
        refusal(
            &wire,
            Code::InvalidSemanticGraph,
            None,
            &binding_path(root, 1),
            root,
        );
    }
}

fn reverse_indexes(value: &mut Value, count: usize) {
    match value {
        Value::Object(object) if object.len() == 1 && object.contains_key("node") => {
            let n = usize::try_from(object["node"].as_u64().unwrap()).unwrap();
            object.insert("node".to_owned(), json!(count - 1 - n));
        }
        Value::Object(object) => {
            for member in object.values_mut() {
                reverse_indexes(member, count);
            }
        }
        Value::Array(items) => {
            for item in items {
                reverse_indexes(item, count);
            }
        }
        _ => {}
    }
}
#[trace("TC-048", "FR-038-AC-426", "FR-038-AC-428", "FR-038-AC-429")]
#[test]
fn monitor_plan_resolved_root_and_four_entry_keys_survive_graph_permutation() {
    let vectors = recipes();
    let (wire, _) = materialize(&vectors["base"]);
    let package = admitted(&wire);
    let plan = package.monitor_plans().next().unwrap();
    let mut permuted = wire.clone();
    let count = permuted["semantic_graph"]["nodes"]
        .as_array()
        .unwrap()
        .len();
    permuted["semantic_graph"]["nodes"]
        .as_array_mut()
        .unwrap()
        .reverse();
    reverse_indexes(&mut permuted, count);
    let package2 = admitted(&permuted);
    let other = package2.monitor_plans().next().unwrap();
    assert_ne!(plan.node_id(), other.node_id(), "wire selectors differ");
    assert_eq!(plan.structural_key(), other.structural_key());
    for (a, b) in [
        (plan.init(), other.init()),
        (plan.on_event(), other.on_event()),
        (plan.on_watermark(), other.on_watermark()),
        (plan.on_gap(), other.on_gap()),
    ] {
        assert_eq!(a.structural_key(), b.structural_key());
    }
    assert_eq!(
        plan.buffers()
            .map(|b| package.structural_key(b.subformula()).unwrap())
            .collect::<Vec<_>>(),
        other
            .buffers()
            .map(|b| package2.structural_key(b.subformula()).unwrap())
            .collect::<Vec<_>>()
    );
}

fn at_work(wire: &Value, work: u64) -> CheckedPackageV2ReadResult {
    read_package_with_document_and_extensions_at_limit(
        wire,
        "monitor plan work",
        &proposal_dir_of(&fixtures_dir()).join(DOMAIN_DOCUMENT),
        true,
        CheckedPackageReadLimits {
            work,
            ..CheckedPackageReadLimits::bounded()
        },
    )
    .unwrap()
}
#[trace("TC-048", "FR-038-AC-425", "FR-038-AC-426")]
#[test]
fn monitor_plan_exact_work_limit_admits_and_one_less_has_no_partial_package() {
    let vectors = recipes();
    let (wire, _) = materialize(&vectors["base"]);
    let mut low = 0;
    let mut high = CheckedPackageReadLimits::bounded().work;
    while low < high {
        let middle = low + (high - low) / 2;
        match at_work(&wire, middle) {
            CheckedPackageV2ReadResult::Admitted(_) => high = middle,
            CheckedPackageV2ReadResult::Incomplete(_) => low = middle + 1,
            other => panic!("valid package cannot refuse under work limit: {other:?}"),
        }
    }
    assert!(low > 0);
    assert_eq!(
        match at_work(&wire, low) {
            CheckedPackageV2ReadResult::Admitted(p) => p.monitor_plans().len(),
            other => panic!("exact limit: {other:?}"),
        },
        1
    );
    let CheckedPackageV2ReadResult::Incomplete(incomplete) = at_work(&wire, low - 1) else {
        panic!("one less must be incomplete")
    };
    assert_eq!(
        incomplete.limit_kind,
        quire_contract_model::CheckedPackageLimit::Work
    );
    assert_eq!(incomplete.limit, low - 1);
    assert!(incomplete.consumed > incomplete.limit);
}

#[trace("TC-048", "FR-038-AC-421", "FR-038-AC-427")]
#[test]
fn monitor_plan_mp32_uses_existing_flatten_and_sequence_operations() {
    let vectors = recipes();
    assert_eq!(vectors["append"]["id"], "MP-32");
    assert_eq!(
        vectors["append"]["operations"],
        json!([
            "quire.op.collection.flatten",
            "quire.op.collection.sequence"
        ])
    );
    let (mut wire, _) = materialize(&vectors["base"]);
    let integer = find(&wire, "integer");
    let sequence = type_node(
        &mut wire,
        "sequence",
        aggregate(vec![reference(&integer)]),
        "append sequence",
    );
    let nested = type_node(
        &mut wire,
        "sequence",
        aggregate(vec![reference(&sequence)]),
        "append nested",
    );
    let x = constant(&mut wire, &integer, "append x");
    let op = |identity: &str, result: &Value, args: Vec<Value>| json!({"term":"application","operator":"collection","operation":{"identity":identity,"laws":[],"leaves":[],"member":null,"mode":null},"result_type":result,"arguments":args});
    let s = append(
        &mut wire,
        "expression",
        "collection",
        Some(&sequence),
        op("quire.op.collection.sequence", &sequence, vec![]),
        "append s",
    );
    let singleton = append(
        &mut wire,
        "expression",
        "collection",
        Some(&sequence),
        op(
            "quire.op.collection.sequence",
            &sequence,
            vec![reference(&x)],
        ),
        "append singleton",
    );
    let pairs = append(
        &mut wire,
        "expression",
        "collection",
        Some(&nested),
        op(
            "quire.op.collection.sequence",
            &nested,
            vec![reference(&s), reference(&singleton)],
        ),
        "append pairs",
    );
    let flattened = append(
        &mut wire,
        "expression",
        "collection",
        Some(&sequence),
        op(
            "quire.op.collection.flatten",
            &sequence,
            vec![reference(&pairs)],
        ),
        "append flattened",
    );
    finish(&mut wire);
    let package = admitted(&wire);
    assert!(matches!(
        package.term(&serde_json::from_value(flattened).unwrap()),
        Some(quire_contract_model::CheckedTerm::Application { .. })
    ));
}

#[trace(
    "TC-048",
    "FR-038-AC-425",
    "FR-038-AC-426",
    "FR-038-AC-428",
    "FR-038-AC-429",
    "FR-038-AC-430"
)]
#[test]
fn monitor_plan_root_order_and_content_changes_use_selected_keys() {
    let vectors = recipes();
    let (mut wire, root) = materialize(&vectors["base"]);
    let original = admitted(&wire);
    let original_plan = original.monitor_plans().next().unwrap();
    let entry = position(
        &wire,
        &wire["semantic_graph"]["nodes"][root]["body"]["members"][15]["value"]["target"],
    );
    let expression = position(
        &wire,
        &wire["semantic_graph"]["nodes"][entry]["body"]["members"][1]["value"]["target"],
    );
    wire["semantic_graph"]["nodes"][expression]["body"]["arguments"][0]["value"] = json!(false);
    rekey(&mut wire, expression);
    rekey(&mut wire, entry);
    rekey(&mut wire, root);
    finish(&mut wire);
    let changed = admitted(&wire);
    let changed_plan = changed.monitor_plans().next().unwrap();
    assert_ne!(
        original_plan.on_gap().structural_key(),
        changed_plan.on_gap().structural_key()
    );
    assert_ne!(
        original_plan.structural_key(),
        changed_plan.structural_key()
    );
    assert_eq!(
        original_plan.counter(),
        changed_plan.counter(),
        "same labels do not select a program"
    );
    assert_eq!(original_plan.tick_unit(), changed_plan.tick_unit());
    assert_eq!(
        original_plan.init().structural_key(),
        changed_plan.init().structural_key()
    );
    // Put the structurally earlier root last in the wire array.
    let mut second = wire["semantic_graph"]["nodes"][root].clone();
    second["body"]["members"][12]["value"]["value"] = json!("aaa-counter");
    second["node_id"]["ordinal"] = json!(1);
    let later = wire["semantic_graph"]["nodes"].as_array().unwrap().len();
    wire["semantic_graph"]["nodes"]
        .as_array_mut()
        .unwrap()
        .push(second);
    rekey(&mut wire, later);
    finish(&mut wire);
    let both = admitted(&wire);
    let plans = both.monitor_plans().collect::<Vec<_>>();
    assert_eq!(plans.len(), 2);
    assert_eq!(plans[0].counter(), "aaa-counter");
    assert_eq!(plans[1].counter(), "ticks");
    assert!(plans[0].structural_key() < plans[1].structural_key());
    value_mut(&mut wire, later, 9)["value"] = json!("4294967296");
    value_mut(&mut wire, root, 8)["value"] = json!("0");
    refusal(
        &wire,
        Code::InvalidPackage,
        Some(Cause::InvalidValue),
        &format!("{}/value", binding_path(later, 9)),
        later,
    );
}

#[trace("TC-048", "FR-038-AC-425", "FR-038-AC-428")]
#[test]
fn monitor_plan_rational_work_denial_stays_incomplete() {
    let vectors = recipes();
    let (mut wire, root) = materialize(&vectors["base"]);
    value_mut(&mut wire, root, 10)["value"] =
        json!("123456789012345678901234567890123456789/123456789012345678901234567890123456788");
    value_mut(&mut wire, root, 12)["value"] = json!("");
    let mut low = 0;
    let mut high = CheckedPackageReadLimits::bounded().work;
    while low < high {
        let middle = low + (high - low) / 2;
        match at_work(&wire, middle) {
            CheckedPackageV2ReadResult::Refused(refused) => {
                assert_eq!(
                    (refused.code, refused.cause),
                    (Code::InvalidPackage, Some(Cause::InvalidValue))
                );
                high = middle;
            }
            CheckedPackageV2ReadResult::Incomplete(_) => low = middle + 1,
            other => panic!("counter is invalid after exact rate checking: {other:?}"),
        }
    }
    let CheckedPackageV2ReadResult::Incomplete(incomplete) = at_work(&wire, low - 1) else {
        panic!("rational denial stays incomplete")
    };
    assert_eq!(
        incomplete.limit_kind,
        quire_contract_model::CheckedPackageLimit::Work
    );
    assert_eq!(
        incomplete.path.as_ref().unwrap().as_str(),
        format!("{}/value", binding_path(root, 10))
    );
    refusal(
        &wire,
        Code::InvalidPackage,
        Some(Cause::InvalidValue),
        &format!("{}/value", binding_path(root, 12)),
        root,
    );
}

#[trace("TC-048", "FR-038-AC-422", "FR-038-AC-425")]
#[test]
fn monitor_plan_owner_source_map_and_wire_defects_keep_precedence() {
    let vectors = recipes();
    let (mut wire, root) = materialize(&vectors["base"]);
    value_mut(&mut wire, root, 8)["value"] = json!("0");
    wire["semantic_graph"]["nodes"][root]["owner"] = wire["semantic_graph"]["nodes"]
        .as_array()
        .unwrap()
        .iter()
        .find(|n| n["owner"]["kind"] == "source")
        .unwrap()["owner"]
        .clone();
    let CheckedPackageV2ReadResult::Refused(refused) = read_package(&wire).unwrap() else {
        panic!("root owner refuses before premises")
    };
    assert_eq!(refused.code, Code::MalformedWire);
    assert_eq!(
        refused.loci[0].path.as_ref().unwrap().as_str(),
        format!("/semantic_graph/nodes/{root}/owner")
    );
    wire["semantic_graph"]["nodes"][root]
        .as_object_mut()
        .unwrap()
        .remove("owner");
    let row = wire["source_map"]
        .as_array()
        .unwrap()
        .iter()
        .position(|r| r["node_id"] == wire["semantic_graph"]["nodes"][root]["node_id"])
        .unwrap();
    wire["source_map"][row]["role"] = json!("expression");
    let CheckedPackageV2ReadResult::Refused(refused) = read_package(&wire).unwrap() else {
        panic!("source map refuses before premises")
    };
    assert_eq!(refused.code, Code::InvalidSourceMap);
    assert!(refused.loci[0]
        .path
        .as_ref()
        .unwrap()
        .as_str()
        .starts_with("/source_map"));
    wire["source_map"][row]["role"] = json!("generated");
    wire["semantic_graph"]["nodes"][root]["body"]["members"][0]["value"]["bad_member"] =
        json!(true);
    let CheckedPackageV2ReadResult::Refused(refused) = read_package(&wire).unwrap() else {
        panic!("wire shape refuses before premises")
    };
    assert_eq!(refused.code, Code::InvalidSemanticGraph);
    assert_eq!(
        refused.loci[0].path.as_ref().unwrap().as_str(),
        format!("{}/value", binding_path(root, 0))
    );
}

#[trace("TC-048", "FR-038-AC-428")]
#[test]
fn monitor_plan_tick_unit_uses_the_admitted_model_dimension_identity() {
    let vectors = recipes();
    let (mut wire, root) = materialize(&vectors["base"]);
    let dimension = json!({"package":"acme/orders","path":["scalar_type","dimension","ix://acme/orders/dimension/Time"],"ordinal":0});
    let unit = json!({"package":"acme/orders","path":["scalar_type","unit","ix://acme/orders/unit/tick"],"ordinal":0});
    for (form, id, ty, deps) in [
        ("dimension", dimension.clone(), dimension.clone(), vec![]),
        (
            "unit",
            unit.clone(),
            dimension.clone(),
            vec![dimension.clone()],
        ),
    ] {
        wire["semantic_graph"]["nodes"].as_array_mut().unwrap().push(json!({
            "schema_version":"quire.checked-semantic-graph/v2","node_tag":"scalar_type","semantic_form":form,
            "node_id":id,"semantic_type":ty,"dependencies":deps,"body":aggregate(vec![]),"occurrences":[{"role":"generated","ordinal":0}],
            "owner":{"kind":"model","identity":"acme/orders","node":format!("ix://acme/orders/{form}/{}",if form=="unit" {"tick"} else {"Time"})}
        }));
    }
    *value_mut(&mut wire, root, 13) = reference(&unit);
    rekey(&mut wire, root);
    finish(&mut wire);
    let document_path = proposal_dir_of(&fixtures_dir()).join(DOMAIN_DOCUMENT);
    let mut document = read_json(&document_path).unwrap();
    document["dimensions"] = json!([{"identity":"ix://acme/orders/dimension/Time","name":"Time","base":true,"terms":[]}]);
    document["units"] = json!([{"identity":"ix://acme/orders/unit/tick","name":"tick","dimension":"ix://acme/orders/dimension/Time","scale":"1","offset":"0"}]);
    let read = |document: &Value| {
        read_package_with_document_override(
            &wire,
            "selected model tick unit",
            &document_path,
            true,
            CheckedPackageReadLimits::bounded(),
            Some(&fixture_support::canonical(document)),
        )
        .unwrap()
    };
    let CheckedPackageV2ReadResult::Admitted(package) = read(&document) else {
        panic!("selected model Time unit admits")
    };
    let plan = package.monitor_plans().next().unwrap();
    assert_eq!(serde_json::to_value(plan.tick_unit()).unwrap(), unit);
    document["dimensions"][0]["name"] = json!("Length");
    let CheckedPackageV2ReadResult::Refused(refused) = read(&document) else {
        panic!("a selected non-Time dimension is not a tick unit")
    };
    assert_eq!(
        (refused.code, refused.cause),
        (Code::InvalidSemanticGraph, None)
    );
    assert_eq!(
        refused.loci[0].path.as_ref().unwrap().as_str(),
        binding_path(root, 13)
    );
}

#[trace("TC-048", "FR-038-AC-423", "FR-038-AC-425")]
#[test]
fn monitor_plan_entry_owner_exception_cannot_admit_a_model_descriptor() {
    let vectors = recipes();
    for mutation in 0..6 {
        let (mut wire, root) = materialize(&vectors["base"]);
        let entry = position(
            &wire,
            &wire["semantic_graph"]["nodes"][root]["body"]["members"][6]["value"]["target"],
        );
        let text = find(&wire, "text");
        match mutation {
            0 => {
                wire["semantic_graph"]["nodes"][entry]["body"]["members"]
                    .as_array_mut()
                    .unwrap()
                    .remove(0);
            }
            1 => {
                wire["semantic_graph"]["nodes"][entry]["body"]["members"][0]["value"] =
                    literal(&text, "text", json!("parameters"))
            }
            2 => wire["semantic_graph"]["nodes"][entry]["body"]["members"]
                .as_array_mut()
                .unwrap()
                .push(binding("clause", literal(&text, "text", json!("body")))),
            3 => wire["semantic_graph"]["nodes"][entry]["body"]["members"]
                .as_array_mut()
                .unwrap()
                .swap(0, 1),
            4 => {
                wire["semantic_graph"]["nodes"][entry]["body"]["members"][0]["value"]["members"]
                    [0] = reference(&text)
            }
            5 => {
                let expression = position(
                    &wire,
                    &wire["semantic_graph"]["nodes"][entry]["body"]["members"][1]["value"]
                        ["target"],
                );
                wire["semantic_graph"]["nodes"][entry]["body"]["members"][1]["value"] =
                    wire["semantic_graph"]["nodes"][expression]["body"]["arguments"][1].clone();
            }
            _ => unreachable!(),
        }
        finish(&mut wire);
        wire["semantic_graph"]["nodes"][root]["body"]["members"][11]["name"] =
            json!("misplaced_buffers");
        refusal(
            &wire,
            Code::InvalidSemanticGraph,
            None,
            &binding_path(root, 11),
            root,
        );
        wire["semantic_graph"]["nodes"][root]["body"]["members"][11]["name"] = json!("buffers");
        refusal(
            &wire,
            Code::IllTyped,
            Some(Cause::TypeMismatch),
            &binding_path(root, 6),
            root,
        );
        // The same malformed, unowned node outside the entry selection still
        // requires its ordinary owner. A body marker cannot grant admission.
        let entry_value =
            wire["semantic_graph"]["nodes"][root]["body"]["members"][5]["value"].clone();
        wire["semantic_graph"]["nodes"][root]["body"]["members"][6]["value"] = entry_value;
        finish(&mut wire);
        let CheckedPackageV2ReadResult::Refused(refused) = read_package(&wire).unwrap() else {
            panic!("unreferenced descriptor requires owner")
        };
        assert_eq!(refused.code, Code::MalformedWire);
        assert_eq!(
            refused.loci[0].path.as_ref().unwrap().as_str(),
            format!("/semantic_graph/nodes/{entry}")
        );
    }
    for form in ["predicate", "recursive_function"] {
        let (mut wire, root) = materialize(&vectors["base"]);
        let mut entry = wire["semantic_graph"]["nodes"]
            .as_array()
            .unwrap()
            .iter()
            .find(|n| {
                n["node_tag"] == "function"
                    && n["body"]["members"][0]["name"] == "parameters"
                    && n["body"]["members"][0]["value"]["members"]
                        .as_array()
                        .unwrap()
                        .is_empty()
            })
            .unwrap()
            .clone();
        entry["node_id"]["ordinal"] = json!(99);
        entry["semantic_form"] = json!(form);
        let n = wire["semantic_graph"]["nodes"].as_array().unwrap().len();
        wire["semantic_graph"]["nodes"]
            .as_array_mut()
            .unwrap()
            .push(entry);
        finish(&mut wire);
        value_mut(&mut wire, root, 8)["value"] = json!("0");
        let CheckedPackageV2ReadResult::Refused(refused) = read_package(&wire).unwrap() else {
            panic!("unrelated function form retains owner policy")
        };
        assert_eq!(refused.code, Code::MalformedWire);
        assert_eq!(
            refused.loci[0].path.as_ref().unwrap().as_str(),
            format!("/semantic_graph/nodes/{n}")
        );
    }
}

#[trace("TC-048", "FR-038-AC-433")]
#[test]
fn monitor_plan_event_position_old_shape_is_refused() {
    let vectors = recipes();
    let case = vectors["cases"]
        .as_array()
        .unwrap()
        .iter()
        .find(|c| c["id"] == "MP-74")
        .unwrap();
    let mut facts = vectors["base"].clone();
    patch(&mut facts, &case["set"]);
    let (wire, root) = materialize(&facts);
    refusal(
        &wire,
        Code::InvalidSemanticGraph,
        None,
        &binding_path(root, 1),
        root,
    );
}

#[trace("TC-048", "FR-038-AC-433", "FR-038-AC-434")]
#[test]
fn monitor_plan_genuine_measure_entry_keeps_signature_and_closed_body_checks() {
    let vectors = recipes();
    let (mut wire, root) = materialize(&vectors["base"]);
    let entry = position(
        &wire,
        &wire["semantic_graph"]["nodes"][root]["body"]["members"][15]["value"]["target"],
    );
    let before = admitted(&wire);
    let measure = append_measure(&mut wire, entry);
    let after = admitted(&wire);
    let plan = after.monitor_plans().next().unwrap();
    assert_eq!(plan.on_gap().parameter_types().len(), 2);
    assert_ne!(
        before
            .monitor_plans()
            .next()
            .unwrap()
            .on_gap()
            .structural_key(),
        plan.on_gap().structural_key()
    );
    assert_ne!(
        before.monitor_plans().next().unwrap().structural_key(),
        plan.structural_key()
    );
    assert_eq!(
        serde_json::to_value(plan.on_gap().body()).unwrap()["members"][2]["value"],
        reference(&measure)
    );
    for mutation in 0..5 {
        let mut invalid = wire.clone();
        let integer = find(&invalid, "integer");
        match mutation {
            0 => {
                invalid["semantic_graph"]["nodes"][entry]["body"]["members"][2]["value"] =
                    literal(&integer, "integer", json!("0"))
            }
            1 => {
                let duplicate =
                    invalid["semantic_graph"]["nodes"][entry]["body"]["members"][2].clone();
                invalid["semantic_graph"]["nodes"][entry]["body"]["members"]
                    .as_array_mut()
                    .unwrap()
                    .push(duplicate);
            }
            2 => invalid["semantic_graph"]["nodes"][entry]["body"]["members"]
                .as_array_mut()
                .unwrap()
                .push(binding("extra", reference(&integer))),
            3 => invalid["semantic_graph"]["nodes"][entry]["body"]["members"]
                .as_array_mut()
                .unwrap()
                .swap(1, 2),
            4 => {
                invalid["semantic_graph"]["nodes"][entry]["body"]["members"][2]["name"] =
                    json!("body")
            }
            _ => unreachable!(),
        }
        finish(&mut invalid);
        refusal(
            &invalid,
            Code::IllTyped,
            Some(Cause::TypeMismatch),
            &binding_path(root, 15),
            root,
        );
    }
    let mut invalid = wire.clone();
    invalid["semantic_graph"]["nodes"][entry]["body"]["members"][2]["value"]["unexpected"] =
        json!(true);
    let CheckedPackageV2ReadResult::Refused(refused) = read_package(&invalid).unwrap() else {
        panic!("malformed measure reference retains ordinary wire precedence")
    };
    assert_eq!(
        (refused.code, refused.cause),
        (Code::InvalidSemanticGraph, None)
    );
    assert_eq!(
        refused.loci[0].path.as_ref().unwrap().as_str(),
        format!("/semantic_graph/nodes/{entry}/body/members/2/value")
    );
}

#[trace("TC-048", "FR-038-AC-433")]
#[test]
fn monitor_plan_event_position_is_the_exact_final_integer_field() {
    let vectors = recipes();
    let (wire, root) = materialize(&vectors["base"]);
    let package = admitted(&wire);
    let plan = package.monitor_plans().next().unwrap();
    let body = package.term(plan.state()).unwrap();
    let CheckedTerm::Aggregate { members } = body else {
        panic!("State record")
    };
    let CheckedTerm::Binding { name, value } = members.last().unwrap() else {
        panic!("final field")
    };
    assert_eq!(name.as_ref(), "event_position");
    let CheckedTerm::Reference { target } = value.as_ref() else {
        panic!("integer reference")
    };
    assert_eq!(
        serde_json::to_value(target).unwrap(),
        find(&wire, "integer")
    );
    for mutation in 0..3 {
        let mut invalid = wire.clone();
        let state = position(
            &invalid,
            &invalid["semantic_graph"]["nodes"][root]["body"]["members"][1]["value"]["target"],
        );
        let fields = invalid["semantic_graph"]["nodes"][state]["body"]["members"]
            .as_array_mut()
            .unwrap();
        match mutation {
            0 => fields.push(fields.last().unwrap().clone()),
            1 => fields.last_mut().unwrap()["name"] = json!("other_position"),
            2 => {
                let last = fields.len() - 1;
                fields.swap(0, last);
            }
            _ => unreachable!(),
        }
        finish(&mut invalid);
        refusal(
            &invalid,
            Code::InvalidSemanticGraph,
            None,
            &binding_path(root, 1),
            root,
        );
    }
}
