// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (C) 2026 Agent-IX

//! FR-038 "Typed population of a reaches application": the `population` link of
//! a `quire.op.model.reaches` application, its reader check, its dependency
//! join and the `reaches_population` accessor.
//!
//! Every package is built here from the sibling builders and the authored
//! domain document; expected values come from those fixtures, never from the
//! accessor under test. Nothing of QSpec or QSL is copied in.

use super::*;
use quire_contract_model::selected_population::{
    select_population, CheckedSelectedPopulationOutcome as Outcome,
    CheckedSelectedPopulationSelector as Selector, CheckedSelectedPopulationSource as Source,
};
use quire_contract_model::{
    CheckedOccurrence, CheckedOccurrenceRole, CheckedPackageEvidence, CheckedReachesPopulationError,
};
use std::collections::BTreeSet;

const POPULATION_AT: &str = "/semantic_graph/nodes/6/body/operation/member/population";
const ILL_TYPED: (
    CheckedPackageRefusalCode,
    Option<CheckedPackageRefusalCause>,
) = (
    CheckedPackageRefusalCode::IllTyped,
    Some(CheckedPackageRefusalCause::OperatorIneligible),
);
const BAD_GRAPH: (
    CheckedPackageRefusalCode,
    Option<CheckedPackageRefusalCause>,
) = (CheckedPackageRefusalCode::InvalidSemanticGraph, None);
const INVALID_DERIVED_SHAPE: (
    CheckedPackageRefusalCode,
    Option<CheckedPackageRefusalCause>,
) = (
    CheckedPackageRefusalCode::InvalidPackage,
    Some(CheckedPackageRefusalCause::InvalidDerivedShape),
);

fn missing_dependency_at(scenario: &Scenario, missing: &str) -> String {
    let dependencies = scenario.package["semantic_graph"]["nodes"][CALL]["dependencies"]
        .as_array()
        .expect("application dependencies");
    let position = dependencies
        .iter()
        .position(|dependency| dependency["digest"] == missing)
        .expect("missing dependency is listed");
    format!("/semantic_graph/nodes/{CALL}/dependencies/{position}")
}

/// A package whose reaches application at [`CALL`] carries (or omits) a
/// `population` key over a graph that holds the nodes the tests add.
struct Scenario {
    package: Value,
    evidence: CheckedPackageEvidence,
}

impl Scenario {
    /// The unlinked reaches package of `relation` over operands typed at the
    /// two given object types (ORDER, then either subtype, are admitted).
    fn over(relation: Value, first: &str, second: &str) -> Self {
        let (package, evidence) = reaches_over(relation, "links", first, second);
        Self { package, evidence }
    }

    /// `PRIORITY_ORDER` and `PREFERRED_ORDER` operands: both subtypes of
    /// `ORDER`, so the graph holds a reference type for each of the three.
    fn standard() -> Self {
        Self::over(authored_relationship(), PRIORITY_ORDER, PREFERRED_ORDER)
    }

    fn nodes(&mut self) -> &mut Vec<Value> {
        self.package["semantic_graph"]["nodes"]
            .as_array_mut()
            .expect("nodes")
    }

    /// The reference-type node over the declaration of `identity`.
    fn reference_of(&mut self, identity: &str) -> String {
        let object = declaration_key_of(identity, IDENTITY);
        self.nodes()
            .iter()
            .find(|node| {
                node["semantic_form"] == "reference"
                    && node["body"]["members"][0]["target"]["digest"] == object.as_str()
            })
            .expect("a reference type over the declaration")["node_id"]["digest"]
            .as_str()
            .expect("key")
            .to_owned()
    }

    /// Adds a `set<reference<identity>>` and a `model_population` node over it
    /// with the given body `members` and `Type` ordinals; returns its key.
    fn add_population(&mut self, identity: &str, members: Vec<Value>, ordinals: &[u64]) -> String {
        let reference = self.reference_of(identity);
        let set_body = over_body(&reference);
        let set = structural_key("composite_type", "set", None, None, &set_body);
        if !self
            .nodes()
            .iter()
            .any(|node| node["node_id"] == node_id(&set))
        {
            self.nodes().push(wire_node(
                &set,
                "composite_type",
                "set",
                &set,
                &[&reference],
                set_body,
            ));
        }
        let body = json!({"term": "aggregate", "members": members});
        let key = structural_key(
            "bounded_domain",
            "model_population",
            Some(&set),
            None,
            &body,
        );
        let mut node = wire_node(
            &key,
            "bounded_domain",
            "model_population",
            &set,
            &[&set],
            body,
        );
        node["occurrences"] = json!(ordinals
            .iter()
            .map(|ordinal| json!({"role": "type", "ordinal": ordinal}))
            .collect::<Vec<_>>());
        self.nodes().push(node);
        key
    }

    /// Sets the `population` of the application at [`CALL`] (`None` removes the
    /// key) and lists `listed` among its dependencies besides the operands and
    /// the relationship, then re-keys and settles the package.
    fn link(&mut self, population: Option<Value>, listed: Option<&str>) {
        let call = CALL;
        link_application(&mut self.nodes()[call], population, listed);
        self.settle();
    }

    /// Adds a copy of the application at [`CALL`] with its operands swapped
    /// (or not) naming `population`; returns the new application's id.
    fn add_application(&mut self, swapped: bool, population: Option<&str>) -> CheckedNodeId {
        let mut node = self.nodes()[CALL].clone();
        if swapped {
            node["body"]["arguments"]
                .as_array_mut()
                .expect("arguments")
                .reverse();
        }
        link_application(&mut node, population.map(node_id), population);
        let id = typed_id(node["node_id"]["digest"].as_str().expect("key"));
        self.nodes().push(node);
        self.settle();
        id
    }

    /// Source map for every occurrence, then the package identity.
    fn settle(&mut self) {
        rebuild_source_map(&mut self.package);
        let nodes = self.package["semantic_graph"]["nodes"]
            .as_array()
            .expect("nodes")
            .clone();
        let map = self.package["source_map"].as_array_mut().expect("map");
        for node in nodes {
            let first = map
                .iter()
                .find(|entry| entry["node_id"] == node["node_id"])
                .expect("the node's first entry")
                .clone();
            for occurrence in &node["occurrences"].as_array().expect("occurrences")[1..] {
                let mut entry = first.clone();
                entry["role"] = occurrence["role"].clone();
                entry["ordinal"] = occurrence["ordinal"].clone();
                map.push(entry);
            }
        }
        refresh_identity(&mut self.package);
    }

    fn read(&self) -> CheckedPackageV2ReadResult {
        read(&self.package, &self.evidence)
    }

    fn admitted(&self) -> CheckedPackageV2 {
        admitted(&self.package, &self.evidence)
    }

    fn assert_refused(
        &self,
        expected: (
            CheckedPackageRefusalCode,
            Option<CheckedPackageRefusalCause>,
        ),
        at: &str,
    ) {
        let refusal = refused(&self.package, &self.evidence);
        assert_eq!(
            (refusal.code, refusal.cause),
            expected,
            "code and cause at {at}"
        );
        assert_eq!(
            refusal.path.as_ref().map(ToString::to_string).as_deref(),
            Some(at)
        );
    }
}

/// Sets or removes the `population` of `application`, re-lists its
/// dependencies (operands, relationship declaration and `listed`) and re-keys it.
fn link_application(application: &mut Value, population: Option<Value>, listed: Option<&str>) {
    let member = application["body"]["operation"]["member"]
        .as_object_mut()
        .expect("member");
    match population {
        Some(value) => {
            member.insert("population".into(), value);
        }
        None => {
            member.remove("population");
        }
    }
    let mut dependencies: BTreeSet<String> = application["body"]["arguments"]
        .as_array()
        .expect("arguments")
        .iter()
        .map(|argument| {
            argument["target"]["digest"]
                .as_str()
                .expect("key")
                .to_owned()
        })
        .collect();
    dependencies.insert(
        application["body"]["operation"]["member"]["declaration"]["digest"]
            .as_str()
            .expect("declaration")
            .to_owned(),
    );
    dependencies.extend(listed.map(str::to_owned));
    application["dependencies"] = json!(dependencies
        .iter()
        .map(|key| node_id(key))
        .collect::<Vec<_>>());
    application["node_id"] = node_id(&application_node_key(
        "expression",
        "reachability",
        application["semantic_type"]["digest"]
            .as_str()
            .expect("Boolean"),
        &application["body"],
    ));
}

fn max_of(value: &str) -> Value {
    json!({"term": "binding", "name": "max", "value": literal(&INTEGER, "integer", value)})
}

/// An admitted package whose reaches application names a new population over
/// `identity` with `max` and `ordinals`. Returns it and the population's key.
fn linked(identity: &str, max: &str, ordinals: &[u64]) -> (Scenario, String) {
    let mut scenario = Scenario::standard();
    let population = scenario.add_population(identity, vec![max_of(max)], ordinals);
    scenario.link(Some(node_id(&population)), Some(&population));
    (scenario, population)
}

fn reaches_id(package: &CheckedPackageV2) -> CheckedNodeId {
    application_of(package, "quire.op.model.reaches")
}

#[trace("TC-048", "FR-038-AC-269")]
#[test]
fn tc_048_reaches_population_returns_the_least_ordinal_type_occurrence() {
    let (scenario, population) = linked(ORDER, "3", &[4, 1, 7]);
    let package = scenario.admitted();
    let selector = package
        .reaches_population(&reaches_id(&package))
        .expect("an admitted link")
        .expect("a selector");
    let occurrence = CheckedOccurrence {
        role: CheckedOccurrenceRole::Type,
        ordinal: 1,
    };
    assert_eq!(
        selector,
        Selector {
            package_id: package.package_id().clone(),
            binding: typed_id(&population),
            occurrence: occurrence.clone(),
        }
    );
    let expected = select_population(Source::Admitted {
        package: &package,
        selector: selector.clone(),
    });
    let identity = package.package_id().clone();
    let sources = package.source_map().to_vec();
    for requests in [2, 4, 8] {
        for _ in 0..requests {
            assert_eq!(
                select_population(Source::Admitted {
                    package: &package,
                    selector: selector.clone(),
                }),
                expected
            );
        }
    }
    assert_eq!(package.package_id(), &identity);
    assert_eq!(package.source_map(), sources);
    match select_population(Source::Admitted {
        package: &package,
        selector,
    }) {
        Outcome::Finite(finite) => {
            assert_eq!(finite.maximum, 3);
            assert_eq!(finite.minimum, 0);
            assert_eq!(finite.object_type, order_declaration_id());
            assert_eq!(finite.binding, typed_id(&population));
            assert_eq!(finite.occurrence, occurrence);
        }
        other => panic!("expected a finite population, got {other:?}"),
    }
}

#[trace("TC-048", "FR-038-AC-270")]
#[test]
fn tc_048_reaches_population_is_none_without_a_key_and_distinguishes_populations() {
    let mut scenario = Scenario::standard();
    let small = scenario.add_population(ORDER, vec![max_of("3")], &[0]);
    let large = scenario.add_population(ORDER, vec![max_of("5")], &[0]);
    scenario.link(Some(node_id(&small)), Some(&small));
    let swapped_same = scenario.add_application(true, Some(&small));
    let other_population = scenario.add_application(false, Some(&large));
    let no_key = scenario.add_application(true, None);
    let package = scenario.admitted();
    let first = package
        .reaches_population(&reaches_id(&package))
        .expect("first")
        .expect("a selector");
    assert_eq!(first.binding, typed_id(&small));
    assert_eq!(
        package
            .reaches_population(&swapped_same)
            .expect("same")
            .as_ref(),
        Some(&first)
    );
    let other = package
        .reaches_population(&other_population)
        .expect("other")
        .expect("a selector");
    assert_eq!(other.binding, typed_id(&large));
    assert_ne!(other.binding, first.binding);
    // The graph holds a population of the receiver end's declared type, and the
    // application that names none still has none.
    assert_eq!(package.reaches_population(&no_key), Ok(None));
}

#[trace("TC-048", "FR-038-AC-271")]
#[test]
fn tc_048_reaches_population_refuses_every_node_that_is_not_a_reaches_application() {
    let refusal = Err(CheckedReachesPopulationError::NotReachesApplication);
    let (scenario, population) = linked(ORDER, "3", &[0]);
    let package = scenario.admitted();
    assert_eq!(
        package.reaches_population(&typed_id(&"f".repeat(64))),
        Err(CheckedReachesPopulationError::UnknownNode)
    );
    for (label, node) in [
        ("relationship", relationship_id()),
        ("population", typed_id(&population)),
        ("object declaration", order_declaration_id()),
    ] {
        assert_eq!(package.reaches_population(&node), refusal, "{label}");
    }

    let (navigate, evidence) = relationship_package(authored_relationship(), "links");
    let package = admitted(&navigate, &evidence);
    let node = application_of(&package, "quire.op.model.navigate");
    assert_eq!(package.reaches_population(&node), refusal, "navigate");

    let package = crate::checked_package_v2_model_fields::integer_add_control();
    let add = application_of(&package, "quire.op.integer.add");
    assert_eq!(package.reaches_population(&add), refusal, "integer.add");
    let literal = package
        .graph()
        .nodes
        .iter()
        .find(|node| node.node_tag.as_ref() == "value" && node.semantic_form.as_ref() == "literal")
        .expect("a literal node")
        .node_id
        .clone();
    assert_eq!(package.reaches_population(&literal), refusal, "literal");
}

#[trace("TC-048", "FR-038-AC-272")]
#[test]
fn tc_048_a_population_that_is_not_a_listed_node_id_is_refused_ill_typed() {
    let absent = "f".repeat(64);
    for (label, value, listed) in [
        ("number", json!(7), None),
        ("object", json!({"digest": 1}), None),
        ("null", Value::Null, None),
        ("text", json!("not a node id"), None),
        (
            "node id with no node",
            node_id(&absent),
            Some(absent.as_str()),
        ),
    ] {
        let mut scenario = Scenario::standard();
        scenario.link(Some(value), listed);
        let refusal = refused(&scenario.package, &scenario.evidence);
        let expected = if listed.is_some() {
            BAD_GRAPH
        } else {
            ILL_TYPED
        };
        let at = if listed.is_some() {
            missing_dependency_at(&scenario, &absent)
        } else {
            POPULATION_AT.to_owned()
        };
        assert_eq!((refusal.code, refusal.cause), expected, "{label}");
        assert_ne!(
            refusal.code,
            CheckedPackageRefusalCode::MissingDeclaration,
            "{label}"
        );
        scenario.assert_refused(expected, &at);
    }
}

#[trace("TC-048", "FR-038-AC-273")]
#[test]
fn tc_048_a_population_that_is_not_a_population_with_a_type_occurrence_is_refused() {
    let mut scenario = Scenario::standard();
    let reference = scenario.reference_of(ORDER);
    scenario.link(Some(node_id(&reference)), Some(&reference));
    scenario.assert_refused(ILL_TYPED, POPULATION_AT);

    let mut scenario = Scenario::standard();
    scenario.link(Some(node_id(&INTEGER)), Some(&INTEGER));
    scenario.assert_refused(ILL_TYPED, POPULATION_AT);

    // A `collection_bounds` node over `Set<Reference<Order>>` has a readable
    // `max` and an object type, and is not a population.
    let mut scenario = Scenario::standard();
    let reference = scenario.reference_of(ORDER);
    let bounds = bounded_collection(scenario.nodes(), &reference, "set", 0, 3);
    scenario.link(Some(node_id(&bounds)), Some(&bounds));
    scenario.assert_refused(ILL_TYPED, POPULATION_AT);

    let mut scenario = Scenario::standard();
    let population = scenario.add_population(ORDER, vec![max_of("3")], &[0]);
    let node = scenario
        .nodes()
        .iter_mut()
        .find(|node| node["node_id"] == node_id(&population))
        .expect("the population");
    node["occurrences"] = json!([{"role": "expression", "ordinal": 0}]);
    scenario.link(Some(node_id(&population)), Some(&population));
    scenario.assert_refused(ILL_TYPED, POPULATION_AT);
}

#[trace("TC-048", "FR-038-AC-274")]
#[test]
fn tc_048_a_population_of_another_object_type_than_the_receiver_end_is_refused() {
    for (label, identity) in [("subtype", PRIORITY_ORDER), ("sibling", PREFERRED_ORDER)] {
        let (scenario, _) = linked(identity, "3", &[0]);
        let refusal = refused(&scenario.package, &scenario.evidence);
        assert_eq!((refusal.code, refusal.cause), ILL_TYPED, "{label}");
        scenario.assert_refused(ILL_TYPED, POPULATION_AT);
    }
    let (scenario, _) = linked(ORDER, "3", &[0]);
    assert!(matches!(
        scenario.read(),
        CheckedPackageV2ReadResult::Admitted(_)
    ));
}

/// A navigate package whose application carries `population`.
fn navigating(population: Value, listed: Option<&str>, extra: Vec<Value>) -> Scenario {
    let (mut package, evidence) = relationship_package(authored_relationship(), "links");
    let nodes = package["semantic_graph"]["nodes"]
        .as_array_mut()
        .expect("nodes");
    nodes.extend(extra);
    let call = &mut nodes[CALL];
    call["body"]["operation"]["member"]["population"] = population;
    let mut dependencies: BTreeSet<String> = call["dependencies"]
        .as_array()
        .expect("dependencies")
        .iter()
        .map(|id| id["digest"].as_str().expect("key").to_owned())
        .collect();
    dependencies.extend(listed.map(str::to_owned));
    call["dependencies"] = json!(dependencies
        .iter()
        .map(|key| node_id(key))
        .collect::<Vec<_>>());
    call["node_id"] = node_id(&application_node_key(
        "expression",
        "query",
        call["semantic_type"]["digest"].as_str().expect("result"),
        &call["body"],
    ));
    let mut scenario = Scenario { package, evidence };
    scenario.settle();
    scenario
}

#[trace("TC-048", "FR-038-AC-275")]
#[test]
fn tc_048_a_navigate_application_never_carries_a_population() {
    let absent = "f".repeat(64);
    let (probe, _) = relationship_package(authored_relationship(), "links");
    let reference = probe["semantic_graph"]["nodes"][3]["node_id"]["digest"]
        .as_str()
        .expect("reference")
        .to_owned();
    let set_body = over_body(&reference);
    let set = structural_key("composite_type", "set", None, None, &set_body);
    let set_node = wire_node(&set, "composite_type", "set", &set, &[&reference], set_body);
    let body = json!({"term": "aggregate", "members": [max_of("3")]});
    let valid = structural_key(
        "bounded_domain",
        "model_population",
        Some(&set),
        None,
        &body,
    );
    let valid_node = wire_node(
        &valid,
        "bounded_domain",
        "model_population",
        &set,
        &[&set],
        body,
    );
    let bounds = bounds_body("0", "9");
    let range = structural_key(
        "bounded_domain",
        "integer_range",
        Some(&INTEGER),
        None,
        &bounds,
    );
    let range_node = wire_node(
        &range,
        "bounded_domain",
        "integer_range",
        &INTEGER,
        &[&INTEGER],
        bounds,
    );
    for (label, population, extra) in [
        ("absent node", node_id(&absent), vec![]),
        ("integer range", node_id(&range), vec![range_node]),
        (
            "valid population",
            node_id(&valid),
            vec![set_node, valid_node],
        ),
    ] {
        let key = population["digest"].as_str().expect("key").to_owned();
        let listed = navigating(population.clone(), Some(&key), extra.clone());
        let refusal = refused(&listed.package, &listed.evidence);
        let expected = if label == "absent node" {
            BAD_GRAPH
        } else {
            ILL_TYPED
        };
        let at = if label == "absent node" {
            missing_dependency_at(&listed, &absent)
        } else {
            POPULATION_AT.to_owned()
        };
        assert_eq!((refusal.code, refusal.cause), expected, "{label}");
        listed.assert_refused(expected, &at);

        let unlisted = navigating(population, None, extra);
        let refusal = refused(&unlisted.package, &unlisted.evidence);
        assert_eq!((refusal.code, refusal.cause), BAD_GRAPH, "{label} unlisted");
        assert_eq!(
            refusal.path.as_ref().map(ToString::to_string).as_deref(),
            Some("/semantic_graph/nodes/6/dependencies"),
            "{label} unlisted"
        );
    }
}

/// Adds `population` to the member of the application at `at` of a control
/// package and settles it; the join lists nothing for it.
fn with_member_population(mut package: Value, at: usize, population: Value) -> Value {
    package["semantic_graph"]["nodes"][at]["body"]["operation"]["member"]["population"] =
        population;
    settle(&mut package);
    package
}

/// The position of the first application of `operation`.
fn position_of(package: &Value, operation: &str) -> usize {
    package["semantic_graph"]["nodes"]
        .as_array()
        .expect("nodes")
        .iter()
        .position(|node| node["body"]["operation"]["identity"] == operation)
        .expect("an application of the operation")
}

#[trace("TC-048", "FR-038-AC-275")]
#[test]
fn tc_048_other_operations_keep_their_present_outcome_for_a_population_key() {
    let key = json!(node_id(&"a".repeat(64)));
    let (field, evidence) = crate::checked_package_v2_model_fields::record_project_control();
    let at = position_of(&field, "quire.op.record.project");
    let field = with_member_population(field, at, key.clone());
    assert!(matches!(
        read(&field, &evidence),
        CheckedPackageV2ReadResult::Admitted(_)
    ));

    let (operation, evidence) = model_member_control("quire.op.model.dispatch_call");
    let at = position_of(&operation, "quire.op.model.dispatch_call");
    let operation = with_member_population(operation, at, key.clone());
    assert!(matches!(
        read(&operation, &evidence),
        CheckedPackageV2ReadResult::Admitted(_)
    ));

    for (label, identity) in [
        ("position", "quire.op.tuple.project"),
        ("element", "quire.op.collection.index"),
    ] {
        let package = crate::checked_package_v2_catalogue_differential::admitted_fixture(identity)
            .expect("catalogue member control");
        let evidence = evidence_for(&package);
        assert!(
            matches!(
                read(&package, &evidence),
                CheckedPackageV2ReadResult::Admitted(_)
            ),
            "{label} control"
        );
        let at = position_of(&package, identity);
        let with_key = with_member_population(package, at, key.clone());
        assert!(
            matches!(
                read(&with_key, &evidence),
                CheckedPackageV2ReadResult::Admitted(_)
            ),
            "{label} keeps admitting a population key"
        );
    }

    // A closed member kind refuses the key exactly as it refuses any other
    // unknown key.
    let (quantity, at) = crate::checked_package_v2_operand_families::quantity_sum_control();
    let (state, clause) = crate::checked_package_v2_frame_entries::state_operand_fixture();
    for (label, package, at) in [
        ("type_argument", quantity, at),
        ("state_clause", state, clause),
    ] {
        let evidence = evidence_for(&package);
        let with_key = with_member_population(package.clone(), at, key.clone());
        let mut stray = package;
        stray["semantic_graph"]["nodes"][at]["body"]["operation"]["member"]["stray"] = key.clone();
        settle(&mut stray);
        let (a, b) = (refused(&with_key, &evidence), refused(&stray, &evidence));
        assert_eq!((a.code, a.cause), (b.code, b.cause), "{label}");
    }
}

#[trace("TC-048", "FR-038-AC-276")]
#[test]
fn tc_048_the_dependency_join_lists_exactly_the_named_population() {
    let (scenario, population) = linked(ORDER, "3", &[0]);
    let package = scenario.admitted();
    let application = package
        .graph()
        .nodes
        .iter()
        .find(|node| node.node_id == reaches_id(&package))
        .expect("application");
    let listed: Vec<String> = application
        .dependencies
        .iter()
        .map(|id| id.digest.to_string())
        .collect();
    let mut sorted = listed.clone();
    sorted.sort();
    sorted.dedup();
    assert_eq!(listed, sorted, "unique and digest-ascending");
    assert!(listed.contains(&population));

    // The population id omitted from the list.
    let mut omitted = Scenario::standard();
    let population = omitted.add_population(ORDER, vec![max_of("3")], &[0]);
    omitted.link(Some(node_id(&population)), None);
    omitted.assert_refused(BAD_GRAPH, "/semantic_graph/nodes/6/dependencies");

    // A population id listed by an application that names none.
    let mut extra = Scenario::standard();
    let population = extra.add_population(ORDER, vec![max_of("3")], &[0]);
    extra.link(None, Some(&population));
    extra.assert_refused(BAD_GRAPH, "/semantic_graph/nodes/6/dependencies");

    // An application without the key keeps the list it has today.
    let mut unkeyed = Scenario::standard();
    unkeyed.link(None, None);
    let before: Vec<Value> = unkeyed.package["semantic_graph"]["nodes"][CALL]["dependencies"]
        .as_array()
        .expect("deps")
        .clone();
    let (original, _) = reaches_over(
        authored_relationship(),
        "links",
        PRIORITY_ORDER,
        PREFERRED_ORDER,
    );
    assert_eq!(
        before,
        original["semantic_graph"]["nodes"][CALL]["dependencies"]
            .as_array()
            .expect("deps")
            .clone()
    );
    assert!(matches!(
        unkeyed.read(),
        CheckedPackageV2ReadResult::Admitted(_)
    ));
}

/// The least `work` limit that admits the package.
fn least_admitting_work(scenario: &Scenario) -> u64 {
    let bytes = canonical(&scenario.package);
    let read_at = |work| {
        let mut limits = CheckedPackageReadLimits::bounded();
        limits.work = work;
        CheckedPackageV2::read(&bytes, limits, &scenario.evidence)
    };
    let (mut insufficient, mut sufficient) = (0, CheckedPackageReadLimits::bounded().work);
    while insufficient + 1 < sufficient {
        let middle = insufficient + (sufficient - insufficient) / 2;
        match read_at(middle) {
            CheckedPackageV2ReadResult::Admitted(_) => sufficient = middle,
            CheckedPackageV2ReadResult::Incomplete(_) => insufficient = middle,
            other => panic!("expected work boundary, got {other:?}"),
        }
    }
    sufficient
}

#[trace("TC-048", "FR-038-AC-277")]
#[test]
fn tc_048_reaches_population_is_repeatable_and_charges_no_work() {
    let (scenario, _) = linked(ORDER, "3", &[2, 0]);
    let first = scenario.admitted();
    let second = scenario.admitted();
    let application = reaches_id(&first);
    let clone = first.clone();
    let result = first.reaches_population(&application);
    assert!(matches!(result, Ok(Some(_))));
    assert_eq!(first.reaches_population(&application), result);
    assert_eq!(clone.reaches_population(&application), result);
    assert_eq!(second.reaches_population(&application), result);
    assert_eq!(first, clone);
    assert_eq!(first, second);
    // A package without the key is charged exactly what it was before the key.
    let (package, evidence) =
        relationship_operation_package(authored_relationship(), "links", true);
    let plain = Scenario { package, evidence };
    assert_eq!(least_admitting_work(&plain), REACHES_ADMISSION_WORK);
}

#[trace("TC-048", "FR-038-AC-278")]
#[test]
fn tc_048_reaches_population_api_is_a_closed_error_and_exports_no_selected_population_item() {
    let (scenario, _) = linked(ORDER, "3", &[0]);
    let package = scenario.admitted();
    let application = reaches_id(&package);
    let result: Result<Option<Selector>, CheckedReachesPopulationError> =
        package.reaches_population(&application);
    assert!(matches!(result, Ok(Some(_))));
    let describe = |error: CheckedReachesPopulationError| match error {
        CheckedReachesPopulationError::UnknownNode => "unknown",
        CheckedReachesPopulationError::NotReachesApplication => "not reaches",
        CheckedReachesPopulationError::UnresolvedPopulation => "unresolved",
    };
    assert_eq!(
        package
            .reaches_population(&typed_id(&"f".repeat(64)))
            .map_err(describe),
        Err("unknown")
    );

    let root = include_str!("../../../crates/quire-contract-model/src/lib.rs");
    let reexports: String = root
        .split("pub use ")
        .skip(1)
        .map(|item| item.split_once(';').expect("a use item ends").0)
        .collect::<Vec<_>>()
        .join(" ");
    assert!(!reexports.contains("selected_population"));
    assert!(!reexports.contains("CheckedSelectedPopulationSelector"));
    assert!(!include_str!("../../../src/lib.rs").contains("selected_population"));
}

#[trace("TC-048", "FR-038-AC-279")]
#[test]
fn tc_048_a_population_whose_max_selection_cannot_read_is_refused() {
    let not_a_binding = json!({"term": "literal", "name": "max", "type": node_id(&INTEGER),
        "value_kind": "integer", "value": "3"});
    let not_a_literal = json!({"term": "binding", "name": "max", "value": reference(&INTEGER)});
    for (label, members) in [
        ("missing", vec![]),
        ("two max members", vec![max_of("3"), max_of("5")]),
        ("not a literal", vec![not_a_literal]),
        ("negative", vec![max_of("-1")]),
        ("above u64", vec![max_of("18446744073709551616")]),
    ] {
        let mut scenario = Scenario::standard();
        let population = scenario.add_population(ORDER, members, &[0]);
        scenario.link(Some(node_id(&population)), Some(&population));
        let refusal = refused(&scenario.package, &scenario.evidence);
        let derived = matches!(label, "two max members" | "not a literal" | "negative");
        let expected = if derived {
            INVALID_DERIVED_SHAPE
        } else {
            ILL_TYPED
        };
        let at = if derived {
            let position = scenario
                .nodes()
                .iter()
                .position(|node| node["node_id"] == node_id(&population))
                .expect("population");
            format!("/semantic_graph/nodes/{position}/node_id")
        } else {
            POPULATION_AT.to_owned()
        };
        assert_eq!((refusal.code, refusal.cause), expected, "{label}");
        scenario.assert_refused(expected, &at);
    }
    // A `max` member that is not a `binding` is not a valid body term at all, so
    // the graph is refused at the member before any population check runs.
    let mut scenario = Scenario::standard();
    let population = scenario.add_population(ORDER, vec![not_a_binding], &[0]);
    scenario.link(Some(node_id(&population)), Some(&population));
    let at = scenario
        .nodes()
        .iter()
        .position(|node| node["node_id"] == node_id(&population))
        .expect("the population");
    scenario.assert_refused(
        BAD_GRAPH,
        &format!("/semantic_graph/nodes/{at}/body/members/0"),
    );

    for max in [0_u64, 3, u64::MAX] {
        let (scenario, population) = linked(ORDER, &max.to_string(), &[0]);
        let package = scenario.admitted();
        let selector = package
            .reaches_population(&reaches_id(&package))
            .expect("an admitted link")
            .expect("a selector");
        match select_population(Source::Admitted {
            package: &package,
            selector,
        }) {
            Outcome::Finite(finite) => {
                assert_eq!(finite.maximum, max);
                assert_eq!(finite.binding, typed_id(&population));
            }
            other => panic!("expected a finite population for {max}, got {other:?}"),
        }
    }
}

#[trace("TC-048", "FR-038-AC-280")]
#[test]
fn tc_048_the_population_check_follows_every_earlier_check() {
    let absent = "f".repeat(64);
    // A missing listed dependency is checked before operation operands.
    let mut earlier = Scenario::over(authored_relationship(), ORDER, CUSTOMER);
    earlier.link(Some(node_id(&absent)), Some(&absent));
    let at = missing_dependency_at(&earlier, &absent);
    earlier.assert_refused(BAD_GRAPH, &at);

    // The `result_type` check is the member check just before the population
    // check, so a result type that is not Boolean is refused at `result_type`.
    let mut result = Scenario::standard();
    result.nodes()[CALL]["body"]["result_type"] = node_id(&INTEGER);
    result.nodes()[CALL]["semantic_type"] = node_id(&INTEGER);
    result.link(Some(node_id(&absent)), Some(&absent));
    let at = missing_dependency_at(&result, &absent);
    result.assert_refused(BAD_GRAPH, &at);

    // A present link can reach mode and leaf checking; a missing listed node
    // is refused at the semantic-edge stage first.
    let mode = json!({"kind": "rounding", "value": "exact"});
    let leaf = json!({"path": [], "laws": [], "mode": null});
    for (_label, field, value, cause, at) in [
        (
            "mode",
            "mode",
            mode,
            CheckedPackageRefusalCause::OperationModeMismatch,
            "/semantic_graph/nodes/6/body/operation/mode",
        ),
        (
            "leaf",
            "leaves",
            json!([leaf]),
            CheckedPackageRefusalCause::OperationLawMismatch,
            "/semantic_graph/nodes/6/body/operation/leaves/0",
        ),
    ] {
        let expected = (CheckedPackageRefusalCode::InvalidPackage, Some(cause));
        for population in [Some(node_id(&absent)), None] {
            let mut scenario = Scenario::standard();
            scenario.nodes()[CALL]["body"]["operation"][field] = value.clone();
            scenario.link(
                population.clone(),
                population.is_some().then_some(absent.as_str()),
            );
            let actual = if population.is_some() {
                BAD_GRAPH
            } else {
                expected
            };
            let pointer = if population.is_some() {
                missing_dependency_at(&scenario, &absent)
            } else {
                at.to_owned()
            };
            scenario.assert_refused(actual, &pointer);
        }
        // Unlisted, the join refuses first.
        let mut unlisted = Scenario::standard();
        unlisted.nodes()[CALL]["body"]["operation"][field] = value;
        unlisted.link(Some(node_id(&absent)), None);
        unlisted.assert_refused(BAD_GRAPH, "/semantic_graph/nodes/6/dependencies");
    }
}

/// The selected population, its provenance and the exact admitting work boundary.
#[trace("TC-048", "FR-038-AC-277")]
#[test]
fn selected_population_result_provenance_and_work_boundary_hold() {
    let (scenario, population) = linked(ORDER, "3", &[2, 0]);
    let bytes = canonical(&scenario.package);
    let package = scenario.admitted();
    let selected = reaches_id(&package);
    let result = package
        .reaches_population(&selected)
        .expect("reaches result")
        .expect("selected population");
    assert_eq!(result.package_id, *package.package_id());
    assert_eq!(result.binding, typed_id(&population));
    assert_eq!(
        result.occurrence,
        CheckedOccurrence {
            role: CheckedOccurrenceRole::Type,
            ordinal: 0
        }
    );
    let missing = Selector {
        package_id: package.package_id().clone(),
        binding: typed_id(&population),
        occurrence: CheckedOccurrence {
            role: CheckedOccurrenceRole::Type,
            ordinal: 99,
        },
    };
    let missing_result = select_population(Source::Admitted {
        package: &package,
        selector: missing.clone(),
    });
    assert_eq!(missing_result, Outcome::MissingOccurrence(missing));
    let Outcome::Finite(finite) = select_population(Source::Admitted {
        package: &package,
        selector: result,
    }) else {
        panic!("finite selected population")
    };
    assert_eq!(finite.package_id, *package.package_id());
    assert_eq!(finite.binding, typed_id(&population));
    assert_eq!(
        finite.occurrence,
        CheckedOccurrence {
            role: CheckedOccurrenceRole::Type,
            ordinal: 0
        }
    );
    assert_eq!(finite.object_type, order_declaration_id());
    assert_eq!((finite.minimum, finite.maximum), (0, 3));
    let authored_sources: Vec<quire_contract_model::CheckedSourceMapEntry> =
        serde_json::from_value(scenario.package["source_map"].clone()).expect("authored sources");
    assert_eq!(package.source_map(), authored_sources);
    let selected_source = authored_sources
        .iter()
        .find(|source| {
            source.node_id == finite.binding
                && source.role == CheckedOccurrenceRole::Type
                && source.ordinal == 0
        })
        .expect("selected authored source");
    assert_eq!(finite.provenance, selected_source.regions);
    let high = least_admitting_work(&scenario);
    let mut limits = CheckedPackageReadLimits::bounded();
    limits.work = high;
    assert!(matches!(
        CheckedPackageV2::read(&bytes, limits, &scenario.evidence),
        CheckedPackageV2ReadResult::Admitted(_)
    ));
    limits.work = high - 1;
    assert!(matches!(
        CheckedPackageV2::read(&bytes, limits, &scenario.evidence),
        CheckedPackageV2ReadResult::Incomplete(_)
    ));
}
