use super::super::model_members::tests::{
    differential_document, differential_documents, document, field, multiplicity, object_type,
    read, BYTES, ORDERS, RECORDED,
};
use super::super::model_members::{
    declaration_key, probe, read_semantic_ir, Budget, DeclarationForm, MemberKind, ModelFailure,
    ModelOwners,
};
use super::super::CheckedSemanticNodeV2;
use super::*;
use crate::checked_package::shared::CheckedPackageLimit;
use crate::checked_package::v2::{CheckedNodeKind, ScalarTypeForm};
use ix_trace_rs::trace;
use serde_json::{json, Value};

/// [`read_semantic_ir`] without the field tables, which the lock stage builds.
fn unbuilt(document: &Value) -> DomainModel {
    let mut meter = WorkMeter::new(u64::MAX);
    match read_semantic_ir(document, &mut Budget::new(&mut meter, 0, BYTES)) {
        Ok(model) => model,
        Err(_) => panic!("the document reads"),
    }
}

/// The work [`build_field_tables`] charges for `document` under no limit, and
/// the model it built the tables of.
fn built(document: &Value) -> (u64, DomainModel) {
    let mut models = [unbuilt(document)];
    let mut meter = WorkMeter::new(u64::MAX);
    build_field_tables(&mut models, &mut meter).expect("no limit is reached");
    let [model] = models;
    (meter.consumed(), model)
}

/// `count` object types `C1 .. Cn`, each declaring one field `f<k>` and
/// extending its predecessor.
fn chain_document(count: usize) -> Value {
    let node = |k: usize| format!("{ORDERS}C{k}");
    document(
        (1..=count)
            .map(|k| {
                let supertypes = if k == 1 { vec![] } else { vec![node(k - 1)] };
                let supertypes: Vec<&str> = supertypes.iter().map(String::as_str).collect();
                object_type(
                    &node(k),
                    &supertypes,
                    vec![field(
                        &node(k),
                        &format!("f{k}"),
                        "ix://quire/native/Integer",
                    )],
                )
            })
            .collect(),
    )
}

fn document_of(label: &str) -> Value {
    let (_, types) = differential_documents()
        .into_iter()
        .find(|(candidate, _)| *candidate == label)
        .expect("a recorded document");
    differential_document(&types)
}

fn exposed_identities(table: &FieldTable) -> Vec<&str> {
    table.exposed.iter().map(|entry| &*entry.identity).collect()
}

fn hidden_identities(table: &FieldTable) -> Vec<&str> {
    table.hidden.iter().map(AsRef::as_ref).collect()
}

fn table_for<'m>(model: &'m DomainModel, short: &str) -> &'m FieldTable {
    model
        .fields
        .of(&format!("{ORDERS}{short}"))
        .expect("a table for the type")
}

/// The table of each type, read for every field name, says what the
/// pre-change `resolve` answered (FR-038 items 4 to 6).
///
/// Trace: FR-038-AC-144
#[trace("TC-227", "FR-038-AC-144")]
#[test]
fn tc_227_tables_hold_the_fields_the_recorded_resolve_answers_expose() {
    let mut actual = Vec::new();
    for (label, types) in differential_documents() {
        let (_, model) = built(&differential_document(&types));
        let names: BTreeSet<&str> = types
            .iter()
            .flat_map(|(_, _, fields)| fields.iter().map(|(name, _)| *name))
            .collect();
        for (short, _, _) in &types {
            let table = table_for(&model, short);
            let answers: Vec<String> = names
                .iter()
                .filter_map(|name| match table.select(name) {
                    Selected::One(field) => Some(format!(
                        "{name}={}",
                        field.identity.trim_start_matches(ORDERS)
                    )),
                    Selected::Ambiguous => Some(format!("{name}=ambiguous")),
                    Selected::Absent => None,
                })
                .collect();
            actual.push(format!("{label}/{short}: {}", answers.join(" ")));
        }
    }
    assert_eq!(actual, RECORDED);
}

/// A table holds the hidden entries too, and a cycle's members share one
/// table: a redefiner of a type that is its own ancestor hides itself and its
/// target, so neither is exposed.
///
/// Trace: FR-038-AC-144
#[trace("TC-227", "FR-038-AC-144")]
#[test]
fn tc_227_a_redefiner_in_a_cycle_hides_itself_and_its_target() {
    let (_, model) = built(&document_of("self_cycle_redefinition"));
    let table = table_for(&model, "A");
    assert!(table.exposed.is_empty());
    assert_eq!(
        hidden_identities(table),
        [
            "ix://acme/orders/A/f".to_owned(),
            "ix://acme/orders/A/g".to_owned()
        ]
    );
    let (_, model) = built(&document_of("redefining_chain"));
    let table = table_for(&model, "C3");
    assert_eq!(
        exposed_identities(table),
        ["ix://acme/orders/C0/g", "ix://acme/orders/C2/f"]
    );
    assert_eq!(
        hidden_identities(table),
        ["ix://acme/orders/C0/f", "ix://acme/orders/C1/f"]
    );
    assert_eq!(table.entries(), 4);
}

/// The charge of a chain of N types with one field each is
/// `N (N + 1) / 2 + N - 1`: 13 for four types and 43 for eight; a table is
/// built once per type from its supertype's, with no ancestor walk.
///
/// Trace: FR-038-AC-144
#[trace("TC-227", "FR-038-AC-144")]
#[test]
fn tc_227_a_chain_charges_the_sum_of_its_tables() {
    for (count, charge) in [(4_usize, 13_u64), (8, 43)] {
        let walks = probe::ancestor_walks();
        let built_before = super::probe::components_built();
        let (charged, model) = built(&chain_document(count));
        assert_eq!(charged, charge, "a chain of {count}");
        assert_eq!(
            super::probe::components_built() - built_before,
            u64::try_from(count).expect("count"),
            "one table per type"
        );
        assert_eq!(probe::ancestor_walks(), walks, "no ancestor walk");
        // The deepest type's table holds one entry per type of the chain.
        assert_eq!(table_for(&model, &format!("C{count}")).entries(), count);
    }
    // The smallest limit that builds the tables is the charge, and one below
    // it is `incomplete` at the selection's row.
    let mut model = [unbuilt(&chain_document(4))];
    assert!(build_field_tables(&mut model, &mut WorkMeter::new(13)).is_ok());
    let mut model = [unbuilt(&chain_document(4))];
    match build_field_tables(&mut model, &mut WorkMeter::new(12)) {
        Err(ValidationFailure::Incomplete(record)) => {
            assert_eq!(record.limit_kind, CheckedPackageLimit::Work);
            assert_eq!(
                record.path.map(|path| path.to_string()).as_deref(),
                Some("/lock/model_selections/0")
            );
        }
        other => panic!("expected an incomplete read, got {other:?}"),
    }
    // The arithmetic of the larger chains the owner decision names.
    for (count, charge) in [(1000_u64, 501_499_u64), (1500, 1_127_249)] {
        assert_eq!(count * (count + 1) / 2 + count - 1, charge);
    }
}

/// A chain of 1000 and a chain of 1500 types are charged the sums the cost
/// paragraph states, measured over the built tables.
///
/// Trace: FR-038-AC-144
#[trace("TC-227", "FR-038-AC-144")]
#[test]
fn tc_227_the_long_chains_charge_what_the_cost_paragraph_states() {
    assert_eq!(built(&chain_document(1000)).0, 501_499);
    assert_eq!(built(&chain_document(1500)).0, 1_127_249);
}

/// A cycle is charged once per component: a type that extends itself with one
/// field charges 2, two types that extend each other with one field each
/// charge 4 and share one table holding both fields, and a type outside the
/// cycle that extends it is charged the entries it copies.
///
/// Trace: FR-038-AC-144
#[trace("TC-227", "FR-038-AC-144")]
#[test]
fn tc_227_a_cycle_is_charged_and_built_once_per_component() {
    let before = super::probe::components_built();
    let (charged, model) = built(&document_of("self_cycle"));
    assert_eq!(charged, 2);
    assert_eq!(
        exposed_identities(table_for(&model, "A")),
        ["ix://acme/orders/A/f"]
    );
    assert_eq!(super::probe::components_built() - before, 1);

    let before = super::probe::components_built();
    let (charged, model) = built(&document_of("two_cycle"));
    assert_eq!(charged, 4);
    assert_eq!(
        super::probe::components_built() - before,
        1,
        "per component"
    );
    assert!(
        std::ptr::eq(table_for(&model, "A"), table_for(&model, "B")),
        "one shared table"
    );
    assert_eq!(
        exposed_identities(table_for(&model, "A")),
        ["ix://acme/orders/A/a", "ix://acme/orders/B/b"]
    );

    // `C extends A` declares `c`: one own field, one edge, and the one entry
    // copied from the cycle's table, on top of the cycle's 2.
    let (charged, model) = built(&document_of("extends_cycle"));
    assert_eq!(charged, 2 + 3);
    assert_eq!(
        exposed_identities(table_for(&model, "C")),
        ["ix://acme/orders/A/f", "ix://acme/orders/C/c"]
    );
}

/// An ambiguous name is retained in the table, not refused, and is charged
/// as any other table.
///
/// Trace: FR-038-AC-139, FR-038-AC-144
#[trace("TC-227", "FR-038-AC-139", "FR-038-AC-144")]
#[test]
fn tc_227_an_ambiguous_table_is_retained_and_charged_like_any_other() {
    let (charged, model) = built(&document_of("ambiguity"));
    // T1 and T2: one field each; T3: one field, two edges, two copied entries.
    assert_eq!(charged, 1 + 1 + (1 + 2 + 2));
    assert_eq!(table_for(&model, "T3").ambiguous.as_deref(), Some("x"));
    assert_eq!(table_for(&model, "T1").ambiguous, None);
}

/// A field resolution charges one unit per exposed or hidden entry of the
/// table it selects from, without the ancestor walk; an operation keeps the
/// walk.
///
/// Trace: FR-038-AC-144
#[trace("TC-227", "FR-038-AC-144")]
#[test]
fn tc_227_a_field_read_charges_the_table_and_an_operation_keeps_the_walk() {
    let mut chain = chain_document(4);
    // `total`, an operation of the root type.
    let root = format!("{ORDERS}C1");
    chain["types"][0]["operations"] = json!([{
        "identity": format!("{root}/total"),
        "params": [],
        "returns": {"typeRef": "ix://quire/native/Integer", "multiplicity": multiplicity(1, Some(1))},
    }]);
    let model = read(&chain).expect("reads");
    let charged = |node: &str, kind: MemberKind, name: &str| {
        let walks = probe::ancestor_walks();
        let mut meter = WorkMeter::new(u64::MAX);
        let resolved = model.resolve(
            &format!("{ORDERS}{node}"),
            kind,
            name,
            &mut Budget::new(&mut meter, 0, BYTES),
        );
        assert!(resolved.is_ok(), "{node}.{name} resolves");
        (meter.consumed(), probe::ancestor_walks() - walks)
    };
    // The four entries of the deepest type's table, and no walk.
    assert_eq!(charged("C4", MemberKind::Field, "f4"), (4, 0));
    assert_eq!(charged("C4", MemberKind::Field, "f1"), (4, 0));
    assert_eq!(charged("C1", MemberKind::Field, "f1"), (1, 0));
    // Three ancestor edges, and each of the four owners' members: a field
    // apiece and the root's operation.
    assert_eq!(charged("C4", MemberKind::Operation, "total"), (3 + 5, 1));
    // The first unit is refused under a limit of 3: `incomplete` at the row.
    let mut meter = WorkMeter::new(3);
    let Err(ModelFailure::Limit(_)) = model.resolve(
        &format!("{ORDERS}C4"),
        MemberKind::Field,
        "f4",
        &mut Budget::new(&mut meter, 0, BYTES),
    ) else {
        panic!("a table of four entries is not charged 3");
    };
}

/// A frame entry that names a field and an abstraction relation's field entry
/// resolve through `ModelOwners::resolve_member`, which charges the table of
/// the owner's type: four units on the deepest type of a chain of four, one on
/// the root, and no ancestor walk.
///
/// Trace: FR-038-AC-144
#[trace("TC-227", "FR-038-AC-144")]
#[test]
fn tc_227_a_field_entry_resolves_through_the_table_and_charges_its_entries() {
    let models = [read(&chain_document(4)).expect("reads")];
    let owners = ModelOwners::new(&models, |_| Ok(())).expect("keys");
    let declaring = |short: &str| -> CheckedSemanticNodeV2 {
        let key = declaration_key(
            &models[0].identity,
            DeclarationForm::ObjectType,
            &format!("{ORDERS}{short}"),
            BYTES,
        )
        .expect("a key");
        let id = json!({"domain": "quire.checked-semantic-node/v1", "digest": key});
        serde_json::from_value(json!({
            "node_id": id, "schema_version": "quire.checked-semantic-graph/v2",
            "node_tag": "model", "semantic_form": "object_type", "semantic_type": id,
            "dependencies": [], "occurrences": [],
            "body": {"term": "aggregate", "members": []},
        }))
        .expect("a node")
    };
    for (short, name, charge) in [("C4", "f2", 4), ("C1", "f1", 1)] {
        let walks = probe::ancestor_walks();
        let mut meter = WorkMeter::new(u64::MAX);
        let resolved = owners
            .resolve_member(&declaring(short), MemberKind::Field, name, &mut meter)
            .expect("no limit is reached");
        assert!(resolved.is_ok(), "{short}.{name} resolves");
        assert_eq!(meter.consumed(), charge, "{short}.{name}");
        assert_eq!(probe::ancestor_walks(), walks, "no walk for a field");
    }
}

/// A hand-built admitted package over `document`: one model declaration node
/// per object type, systems interface and relationship the document declares,
/// and an `Int[0, 1000]` bounded domain a read of a field would name.
struct Fixture {
    package: CheckedPackageV2,
    /// The position of the bounded domain node.
    range: usize,
}

fn node_of(key: &str, tag: &str, form: &str, body: Value) -> Value {
    let id = json!({"domain": "quire.checked-semantic-node/v1", "digest": key});
    json!({
        "node_id": id,
        "schema_version": "quire.checked-semantic-graph/v2",
        "node_tag": tag,
        "semantic_form": form,
        "semantic_type": id,
        "dependencies": [],
        "occurrences": [],
        "body": body,
    })
}

fn build_fixture(document: &Value, declared: &[(&str, DeclarationForm)]) -> Fixture {
    let model = read(document).expect("reads");
    let models = [model];
    let owners = ModelOwners::new(&models, |_| Ok(())).expect("keys");
    let index = owners.index();
    let [model] = models;
    let key_of = |node: &str, form| {
        super::super::model_members::declaration_key(&model.identity, form, node, BYTES)
            .expect("a key")
    };
    let empty = json!({"term": "aggregate", "members": []});
    let mut nodes = Vec::new();
    for (node, form) in declared {
        nodes.push(node_of(
            &key_of(&format!("{ORDERS}{node}"), *form),
            form.tag(),
            form.form(),
            empty.clone(),
        ));
    }
    let integer = "1".repeat(64);
    let range = nodes.len();
    let mut bounded = node_of(
        &"2".repeat(64),
        "bounded_domain",
        "integer_range",
        json!({"term": "aggregate", "members": [
            {"term": "binding", "name": "min", "value": {"term": "literal", "type": {"domain": "quire.checked-semantic-node/v1", "digest": integer}, "value_kind": "integer", "value": "0"}},
            {"term": "binding", "name": "max", "value": {"term": "literal", "type": {"domain": "quire.checked-semantic-node/v1", "digest": integer}, "value_kind": "integer", "value": "1000"}},
        ]}),
    );
    bounded["semantic_type"] =
        json!({"domain": "quire.checked-semantic-node/v1", "digest": integer});
    nodes.push(bounded);
    nodes.push(node_of(&integer, "scalar_type", "integer", empty));
    let artifact = |identity: &str| json!({"authority": "agent-ix", "identity": identity});
    let edition = json!({"role": "edition", "definition": artifact("edition")});
    let wire: super::super::CheckedPackageWireV2 = serde_json::from_value(json!({
        "contract_version": "quire.checked-package/v2",
        "identity_preimage": {
            "version": "quire.checked-package-id/v2",
            "edition": edition,
            "profile_selections": [],
            "definition_selections": [],
            "model_selections": [],
            "required_features": [],
            "dependency_selections": [],
            "identity_projection": [],
        },
        "package_id": {
            "domain": "quire.package.semantic/v2",
            "algorithm": "sha256",
            "digest": "c".repeat(64),
        },
        "lock": {
            "sources": [],
            "edition": edition,
            "profile_selections": [],
            "definition_selections": [],
            "model_selections": [],
            "required_features": [],
            "dependency_selections": [],
        },
        "semantic_graph": {"graph_version": "quire.checked-semantic-graph/v2", "nodes": nodes},
        "source_map": [],
        "capability_report": [],
        "diagnostics": {"catalog": artifact("diagnostics"), "entries": []},
    }))
    .expect("wire");
    let kinds = vec![CheckedNodeKind::ScalarType(ScalarTypeForm::Boolean); nodes_len(&wire)];
    Fixture {
        package: CheckedPackageV2 {
            wire,
            kinds,
            bytes: BYTES,
            models: RetainedModels::new(index, vec![model]),
        },
        range,
    }
}

fn nodes_len(wire: &super::super::CheckedPackageWireV2) -> usize {
    wire.semantic_graph.nodes.len()
}

fn id_at(fixture: &Fixture, position: usize) -> CheckedNodeId {
    fixture
        .package
        .wire
        .semantic_graph
        .nodes
        .get(position)
        .expect("a node")
        .node_id
        .clone()
}

/// Declared types for [`fixture`]'s documents: an object type with an
/// `Int[0, 1000]` field `balance` and an inherited `audit`.
fn ledger() -> Value {
    let digit = json!({
        "identity": format!("{ORDERS}Thousand"), "displayName": "Thousand",
        "kind": {"module": "acme/orders", "name": "digit"}, "roles": [], "extensions": [],
        "unknownPolicy": "reject", "scalar": "integer",
        "constraints": [
            {"keyword": "min", "operands": {"value": 0}},
            {"keyword": "max", "operands": {"value": 1000}},
        ],
    });
    let thousand = format!("{ORDERS}Thousand");
    let base = format!("{ORDERS}Base");
    let ledger = format!("{ORDERS}Ledger");
    let mut base_type = object_type(&base, &[], vec![field(&base, "audit", &thousand)]);
    base_type["relationships"] = json!([{
        "identity": format!("{ORDERS}relationship/Base-owns-Base"),
        "category": "structural", "composite": false, "direction": "source-to-target",
        "sourceEnd": {
            "type": base, "role": "owns",
            "multiplicity": multiplicity(0, Some(1)),
        },
        "targetEnd": {"type": base, "multiplicity": multiplicity(0, Some(1))},
        "origin": {"source": {
            "sourceIdentity": base, "path": "models/Base.md",
            "startLine": 1, "startColumn": 1,
        }},
    }]);
    document(vec![
        digit,
        base_type,
        object_type(
            &ledger,
            &[&base],
            vec![field(&ledger, "balance", &thousand)],
        ),
    ])
}

/// The accessor reads the table and the passed node's own fixed members, and
/// no other node body: replacing the body bounds of the `Int[0, 1000]` node a
/// read would name, after admission, changes nothing, and the accessor
/// depends on no body of any other node.
///
/// Trace: FR-038-AC-140
#[trace("TC-227", "FR-038-AC-140")]
#[test]
fn tc_227_the_accessor_does_not_read_a_node_body_it_was_not_passed() {
    let mut fixture = build_fixture(&ledger(), &[("Ledger", DeclarationForm::ObjectType)]);
    let ledger = id_at(&fixture, 0);
    let expected = |fields: &CheckedModelObjectFields| {
        fields
            .fields()
            .iter()
            .map(|field| (field.name().to_owned(), field.member_type().cloned()))
            .collect::<Vec<_>>()
    };
    let before = fixture
        .package
        .model_object_fields(&ledger)
        .expect("fields");
    let range = Some(CheckedMemberType::IntRange {
        lower: 0,
        upper: 1000,
    });
    assert_eq!(
        expected(&before),
        [
            ("audit".to_owned(), range.clone()),
            ("balance".to_owned(), range)
        ]
    );
    for upper in ["10", "5000"] {
        let position = fixture.range;
        let node = fixture
            .package
            .wire
            .semantic_graph
            .nodes
            .get_mut(position)
            .expect("the bounded domain node");
        if let Some(members) = node.body["members"].as_array_mut() {
            members[1]["value"]["value"] = json!(upper);
        }
        assert_eq!(
            fixture.package.model_object_fields(&ledger),
            Ok(before.clone()),
            "bounds replaced with 0 and {upper}"
        );
    }
}

/// `UnknownNode` and `NotModelObjectType` are distinct; the passed node's own
/// fixed members decide the second, by the one rule of step 2, so an unread
/// model node whose body is not `aggregate{[]}` returns the error and not the
/// fields.
///
/// Trace: FR-038-AC-139, FR-038-AC-143
#[trace("TC-227", "FR-038-AC-139", "FR-038-AC-143")]
#[test]
fn tc_227_the_accessor_names_what_is_wrong_with_the_node() {
    let mut types = vec![
        ("Ledger", DeclarationForm::ObjectType),
        ("Base", DeclarationForm::ObjectType),
    ];
    types.push(("Ledger", DeclarationForm::SystemsInterface));
    types.push(("relationship/Base-owns-Base", DeclarationForm::Relationship));
    let mut fixture = build_fixture(&ledger(), &types);
    // The first node is Ledger's object type declaration node; the second is
    // Base's; the third is a node keyed as Ledger's systems interface, which
    // no selected declaration is; the fourth is Base's relationship.
    let (ledger, base, interface, relationship) = (
        id_at(&fixture, 0),
        id_at(&fixture, 1),
        id_at(&fixture, 2),
        id_at(&fixture, 3),
    );
    let package = &fixture.package;
    assert!(package.model_object_fields(&ledger).is_ok());
    assert_eq!(
        package.model_object_fields(&interface),
        Err(CheckedModelFieldsError::NotModelObjectType),
        "a node keyed as no selected declaration is no object type"
    );
    assert_eq!(
        package.model_object_fields(&relationship),
        Err(CheckedModelFieldsError::NotModelObjectType),
        "a relationship declaration node is no object type"
    );
    let unknown = CheckedNodeId {
        domain: NODE_DOMAIN.into(),
        digest: "9".repeat(64).into(),
    };
    assert_eq!(
        package.model_object_fields(&unknown),
        Err(CheckedModelFieldsError::UnknownNode)
    );
    // The bounded domain node and the scalar node are no model node at all.
    for position in [fixture.range, fixture.range + 1] {
        assert_eq!(
            package.model_object_fields(&id_at(&fixture, position)),
            Err(CheckedModelFieldsError::NotModelObjectType)
        );
    }
    // Each fixed member of step 2, broken in turn, makes Base's node no model
    // object type: a non-empty body, a `declaration`, a recursion group, a
    // `semantic_type` that is not itself.
    let breaks: [fn(&mut CheckedSemanticNodeV2); 4] = [
        |node| node.body = json!({"term": "aggregate", "members": [{"term": "x"}]}),
        |node| {
            node.declaration = Some(super::super::CheckedDeclaration {
                qualified_name: vec!["Base".into()],
            });
        },
        |node| node.recursion_group = Some("group".into()),
        |node| {
            node.semantic_type = CheckedNodeId {
                domain: NODE_DOMAIN.into(),
                digest: "8".repeat(64).into(),
            }
        },
    ];
    for broken in breaks {
        let mut package = fixture.package.clone();
        if let Some(node) = package.wire.semantic_graph.nodes.get_mut(1) {
            broken(node);
        }
        assert_eq!(
            package.model_object_fields(&base),
            Err(CheckedModelFieldsError::NotModelObjectType)
        );
        // The other object type's node is not affected.
        assert!(package.model_object_fields(&ledger).is_ok());
    }
    fixture.package.wire.semantic_graph.nodes.clear();
    assert_eq!(
        fixture.package.model_object_fields(&ledger),
        Err(CheckedModelFieldsError::UnknownNode)
    );
}

/// An ambiguous object type returns the error naming the smallest ambiguous
/// name for the whole call; a clone and a second call return equal results; a
/// chain of 200 types and 100 redefinitions of one field returns its fields
/// without a panic and without resolving anything.
///
/// Trace: FR-038-AC-139, FR-038-AC-143
#[trace("TC-227", "FR-038-AC-139", "FR-038-AC-143")]
#[test]
fn tc_227_the_accessor_is_pure_total_and_does_not_resolve() {
    let ambiguous = document_of("ambiguity");
    let fixture = build_fixture(
        &ambiguous,
        &[
            ("T3", DeclarationForm::ObjectType),
            ("T1", DeclarationForm::ObjectType),
        ],
    );
    let (t3, t1) = (id_at(&fixture, 0), id_at(&fixture, 1));
    assert_eq!(
        fixture.package.model_object_fields(&t3),
        Err(CheckedModelFieldsError::AmbiguousField("x".into()))
    );
    let one = fixture.package.model_object_fields(&t1).expect("fields");
    assert_eq!(one.fields().len(), 1);
    assert_eq!(fixture.package.clone().model_object_fields(&t1), Ok(one));

    // 200 types: `R1` declares `f`, `R2` to `R101` each redefine their
    // predecessor's `f` (100 redefinitions), `R102` to `R200` each declare a
    // field of their own.
    let node = |k: usize| format!("{ORDERS}R{k}");
    let types: Vec<Value> = (1..=200)
        .map(|k| {
            let supertypes = if k == 1 { vec![] } else { vec![node(k - 1)] };
            let supertypes: Vec<&str> = supertypes.iter().map(String::as_str).collect();
            let declared = if k <= 101 {
                let mut declared = field(&node(k), "f", "ix://quire/native/Integer");
                if k > 1 {
                    declared["redefines"] = json!(format!("{}/f", node(k - 1)));
                }
                declared
            } else {
                field(&node(k), &format!("g{k}"), "ix://quire/native/Integer")
            };
            object_type(&node(k), &supertypes, vec![declared])
        })
        .collect();
    let deep = build_fixture(&document(types), &[("R200", DeclarationForm::ObjectType)]);
    let (walks, built_before) = (probe::ancestor_walks(), super::probe::components_built());
    let fields = deep
        .package
        .model_object_fields(&id_at(&deep, 0))
        .expect("the chain's fields");
    assert_eq!(fields.fields().len(), 1 + 99);
    assert!(fields.field("f").is_some() && fields.field("g200").is_some());
    assert!(fields.field("g101").is_none());
    assert_eq!(
        probe::ancestor_walks(),
        walks,
        "the call walks no ancestors"
    );
    assert_eq!(
        super::probe::components_built(),
        built_before,
        "the call builds no table"
    );
}

/// The retained tables and declaration index take no part in the package's
/// equality: two packages of one admitted content, one with its tables and one
/// without, are equal, though only one returns fields.
///
/// Trace: FR-038-AC-143
#[trace("TC-227", "FR-038-AC-143")]
#[test]
fn tc_227_the_retained_tables_take_no_part_in_equality() {
    let with = build_fixture(&ledger(), &[("Ledger", DeclarationForm::ObjectType)]).package;
    let mut without = with.clone();
    without.models = RetainedModels::default();
    let ledger = with
        .wire
        .semantic_graph
        .nodes
        .first()
        .expect("a node")
        .node_id
        .clone();
    assert!(with.model_object_fields(&ledger).is_ok());
    assert_eq!(
        without.model_object_fields(&ledger),
        Err(CheckedModelFieldsError::NotModelObjectType)
    );
    assert_eq!(with, without);
}

/// The component search follows a chain of any length on the heap: a chain of
/// 20,000 object types is searched on a thread of 256 KiB, which a recursion
/// over the chain does not survive.
///
/// Trace: FR-038-AC-143
#[trace("TC-227", "FR-038-AC-143")]
#[test]
fn tc_227_the_component_search_needs_no_call_stack() {
    let model = unbuilt(&chain_document(20_000));
    let found = std::thread::Builder::new()
        .stack_size(256 * 1024)
        .spawn(move || components(&model).len())
        .expect("a thread")
        .join()
        .expect("the search finishes on a small stack");
    assert_eq!(found, 20_000);
}
