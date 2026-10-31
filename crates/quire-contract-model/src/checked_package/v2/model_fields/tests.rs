use super::super::model_members::tests::{
    differential_document, differential_documents, document, field, multiplicity, object_type,
    read, BYTES, ORDERS, RECORDED,
};
use super::super::model_members::{
    declaration_id, probe, read_semantic_ir, Budget, DeclarationForm, MemberKind, ModelFailure,
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
    match read_semantic_ir(
        document,
        "checked",
        &mut Budget::new(&mut meter, 0, BYTES),
        &Arc::new(BTreeMap::new()),
    ) {
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

fn ordered_identities(table: &FieldTable) -> Vec<&str> {
    table
        .ordered_exposed
        .iter()
        .map(|slot| &*slot.entry.identity)
        .collect()
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

fn record_document(types: Vec<Value>) -> Value {
    let mut document = document(types);
    document["constructs"]
        .as_array_mut()
        .expect("constructs")
        .push(json!({
            "kind": {"module": "acme/orders", "name": "record"},
            "construct": {"meaning": "quire.meaning.model.record-value-type/v1"},
        }));
    document
}

fn record_type(short: &str, supertypes: &[&str], fields: Vec<Value>) -> Value {
    let mut record = object_type(&format!("{ORDERS}{short}"), supertypes, fields);
    record["kind"]["name"] = json!("record");
    record
}

/// Record fields retain the source order that an activation binder reads,
/// including reversed declarations and reversed supertype edges.
#[trace("TC-048", "FR-038-AC-325")]
#[test]
fn tc_048_record_field_table_preserves_source_order() {
    let base = format!("{ORDERS}Base");
    let mid = format!("{ORDERS}Mid");
    let leaf = format!("{ORDERS}Leaf");
    let p = format!("{ORDERS}P");
    let q = format!("{ORDERS}Q");
    let both = format!("{ORDERS}Both");
    let types = |base_fields: &[&str], supertypes: &[&str]| {
        record_document(vec![
            record_type(
                "Base",
                &[],
                base_fields
                    .iter()
                    .map(|name| field(&base, name, "ix://quire/native/Boolean"))
                    .collect(),
            ),
            record_type(
                "Mid",
                &[&base],
                vec![field(&mid, "c", "ix://quire/native/Integer")],
            ),
            record_type(
                "Leaf",
                &[&mid],
                vec![field(&leaf, "d", "ix://quire/native/Boolean")],
            ),
            record_type("P", &[], vec![field(&p, "p", "ix://quire/native/Boolean")]),
            record_type("Q", &[], vec![field(&q, "q", "ix://quire/native/Integer")]),
            record_type(
                "Both",
                supertypes,
                vec![field(&both, "r", "ix://quire/native/Boolean")],
            ),
        ])
    };
    let (_, model) = built(&types(&["b", "a"], &[&q, &p]));
    assert_eq!(
        ordered_identities(table_for(&model, "Leaf")),
        [
            format!("{base}/b"),
            format!("{base}/a"),
            format!("{mid}/c"),
            format!("{leaf}/d"),
        ]
    );
    assert_eq!(
        ordered_identities(table_for(&model, "Both")),
        [format!("{q}/q"), format!("{p}/p"), format!("{both}/r")]
    );
    assert_eq!(
        exposed_identities(table_for(&model, "Leaf")),
        [
            format!("{base}/a"),
            format!("{base}/b"),
            format!("{leaf}/d"),
            format!("{mid}/c"),
        ],
        "the existing identity-sorted lookup remains intact"
    );

    let (_, reversed) = built(&types(&["a", "b"], &[&p, &q]));
    assert_eq!(
        ordered_identities(table_for(&reversed, "Leaf")),
        [
            format!("{base}/a"),
            format!("{base}/b"),
            format!("{mid}/c"),
            format!("{leaf}/d"),
        ]
    );
    assert_eq!(
        ordered_identities(table_for(&reversed, "Both")),
        [format!("{p}/p"), format!("{q}/q"), format!("{both}/r")]
    );
}

/// A diamond contributes the shared field once, and a redefiner takes the
/// inherited position even when another branch reaches the hidden field.
#[trace("TC-048", "FR-038-AC-325")]
#[test]
fn tc_048_record_field_table_deduplicates_diamond_and_positions_redefinition() {
    let top = format!("{ORDERS}Top");
    let left = format!("{ORDERS}Left");
    let right = format!("{ORDERS}Right");
    let bottom = format!("{ORDERS}Bottom");
    let mut replacement = field(&left, "t", "ix://quire/native/Integer");
    replacement["redefines"] = json!(format!("{top}/t"));
    let document = record_document(vec![
        record_type(
            "Top",
            &[],
            vec![field(&top, "t", "ix://quire/native/Boolean")],
        ),
        record_type(
            "Left",
            &[&top],
            vec![replacement, field(&left, "l", "ix://quire/native/Boolean")],
        ),
        record_type(
            "Right",
            &[&top],
            vec![field(&right, "r", "ix://quire/native/Integer")],
        ),
        record_type(
            "Bottom",
            &[&left, &right],
            vec![field(&bottom, "z", "ix://quire/native/Boolean")],
        ),
    ]);
    let (_, model) = built(&document);
    let table = table_for(&model, "Bottom");
    assert_eq!(
        ordered_identities(table),
        [
            format!("{left}/t"),
            format!("{left}/l"),
            format!("{right}/r"),
            format!("{bottom}/z"),
        ]
    );
    assert_eq!(
        table.ordered_exposed[0].entry.member_type,
        Some(MemberType::Integer),
        "the replacement supplies the binder type at Top's inherited position"
    );
    assert_eq!(table.ambiguous, None);

    let mut reverse = document.clone();
    reverse["types"][3]["supertypes"] = json!([right, left]);
    let (_, reverse_model) = built(&reverse);
    assert_eq!(
        ordered_identities(table_for(&reverse_model, "Bottom")),
        [
            format!("{left}/t"),
            format!("{right}/r"),
            format!("{left}/l"),
            format!("{bottom}/z"),
        ],
        "the later supertype's redefiner takes Top's earlier position"
    );

    let base = format!("{ORDERS}Base");
    let child = format!("{ORDERS}Child");
    let mut replacement = field(&child, "a", "ix://quire/native/Integer");
    replacement["redefines"] = json!(format!("{base}/a"));
    let (_, direct) = built(&record_document(vec![
        record_type(
            "Base",
            &[],
            vec![
                field(&base, "b", "ix://quire/native/Boolean"),
                field(&base, "a", "ix://quire/native/Boolean"),
            ],
        ),
        record_type(
            "Child",
            &[&base],
            vec![replacement, field(&child, "c", "ix://quire/native/Boolean")],
        ),
    ]));
    assert_eq!(
        ordered_identities(table_for(&direct, "Child")),
        [
            format!("{base}/b"),
            format!("{child}/a"),
            format!("{child}/c")
        ],
        "a direct redefinition takes the hidden field's slot"
    );
}

/// Selected records use the effective field table, with redefinitions and
/// ambiguous exposed names decided the same way as object fields.
#[trace("TC-056", "FR-040-AC-15")]
#[test]
fn tc_056_record_fields_keep_value_types_and_effective_names() {
    let address = format!("{ORDERS}Address");
    let sub = format!("{ORDERS}SubAddress");
    let profile = format!("{ORDERS}Profile");
    let mut home = field(&profile, "home", &address);
    home["presence"] = json!("optional");
    let mut routes = field(&profile, "routes", &address);
    routes["multiplicity"] = multiplicity(0, Some(3));
    routes["multiplicity"]["ordered"] = json!(true);
    routes["multiplicity"]["unique"] = json!(false);
    let mut parent = field(&profile, "parent", &profile);
    parent["presence"] = json!("optional");
    let mut renamed = field(&sub, "street", "ix://quire/native/Integer");
    renamed["redefines"] = json!(format!("{address}/street"));
    let document = record_document(vec![
        record_type(
            "Address",
            &[],
            vec![field(&address, "street", "ix://quire/native/Integer")],
        ),
        record_type("SubAddress", &[&address], vec![renamed]),
        record_type("Profile", &[], vec![home, routes, parent]),
        record_type("SubProfile", &[&profile], vec![]),
    ]);
    let (_, model) = built(&document);
    let record_key = declaration_id(&model.identity, DeclarationForm::RecordValueType, &address);
    let profile_key = declaration_id(&model.identity, DeclarationForm::RecordValueType, &profile);
    assert_eq!(
        exposed_identities(table_for(&model, "SubAddress")),
        [format!("{sub}/street")]
    );
    assert!(matches!(
        table_for(&model, "Profile").select("home"),
        Selected::One(_)
    ));
    let inherited = match table_for(&model, "SubProfile").select("home") {
        Selected::One(field) => field,
        _ => panic!("the record subtype inherits home"),
    };
    assert_eq!(
        model.field_type(inherited),
        Some(MemberType::Option(Box::new(MemberType::Record(
            record_key.clone(),
        ))))
    );
    assert_eq!(
        model.field_type(&model.record_types[&*profile].fields[0]),
        Some(MemberType::Option(Box::new(MemberType::Record(
            record_key.clone()
        ))))
    );
    assert_eq!(
        model.field_type(&model.record_types[&*profile].fields[1]),
        Some(MemberType::Collection {
            kind: super::super::model_members::CollectionKind::Sequence,
            element: Box::new(MemberType::Record(record_key)),
            bounds: Some((0, 3)),
        })
    );
    assert_eq!(
        model.field_type(&model.record_types[&*profile].fields[2]),
        Some(MemberType::Option(Box::new(MemberType::Record(
            profile_key
        ))))
    );
}

/// Two record supertypes exposing the same field name stay ambiguous, and
/// building their tables uses the selected row's exact work budget.
#[trace("TC-056", "FR-040-AC-15", "FR-040-AC-16")]
#[test]
fn tc_056_record_field_ambiguity_and_table_limit() {
    let left = format!("{ORDERS}LeftRecord");
    let right = format!("{ORDERS}RightRecord");
    let document = record_document(vec![
        record_type(
            "LeftRecord",
            &[],
            vec![field(&left, "same", "ix://quire/native/Integer")],
        ),
        record_type(
            "RightRecord",
            &[],
            vec![field(&right, "same", "ix://quire/native/Integer")],
        ),
        record_type("BothRecords", &[&left, &right], vec![]),
    ]);
    let (exact, model) = built(&document);
    assert!(matches!(
        table_for(&model, "BothRecords").select("same"),
        Selected::Ambiguous
    ));
    let mut at_limit = [unbuilt(&document)];
    assert!(build_field_tables(&mut at_limit, &mut WorkMeter::new(exact)).is_ok());
    let mut below = [unbuilt(&document)];
    match build_field_tables(&mut below, &mut WorkMeter::new(exact - 1)) {
        Err(ValidationFailure::Incomplete(record)) => {
            assert_eq!(record.limit_kind, CheckedPackageLimit::Work);
            assert_eq!(
                record.path.as_ref().map(|path| path.as_str()),
                Some("/semantic_graph/nodes/0/owner")
            );
        }
        other => panic!("expected selected row work limit: {other:?}"),
    }
}

/// The table of each type, read for every field name, says what the
/// pre-change `resolve` answered (FR-038 items 4 to 6).
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
                Some("/semantic_graph/nodes/0/owner")
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
#[trace("TC-227", "FR-038-AC-144")]
#[test]
fn tc_227_a_field_entry_resolves_through_the_table_and_charges_its_entries() {
    let models = [read(&chain_document(4)).expect("reads")];
    let declaring = |short: &str| -> CheckedSemanticNodeV2 {
        let identity = format!("{ORDERS}{short}");
        serde_json::from_value(node_of(
            declaration_id(&models[0].identity, DeclarationForm::ObjectType, &identity),
            "model",
            "object_type",
            json!({"term":"aggregate","members":[]}),
            Some(json!({"kind":"model","identity":models[0].identity,"node":identity})),
        ))
        .expect("model declaration node")
    };
    let nodes: Vec<_> = ["C1", "C2", "C3", "C4"]
        .into_iter()
        .map(declaring)
        .collect();
    let owners = ModelOwners::new(&models, &nodes, |_| Ok(())).expect("owner index");
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

fn node_of(id: CheckedNodeId, tag: &str, form: &str, body: Value, owner: Option<Value>) -> Value {
    let mut node = json!({
        "node_id": id, "schema_version": "quire.checked-semantic-graph/v2",
        "node_tag": tag, "semantic_form": form, "semantic_type": id,
        "dependencies": [], "occurrences": [], "body": body,
    });
    if let Some(owner) = owner {
        node["owner"] = owner;
    }
    node
}

fn build_fixture(document: &Value, declared: &[(&str, DeclarationForm)]) -> Fixture {
    let model = read(document).expect("reads");
    let models = [model];
    let empty = json!({"term": "aggregate", "members": []});
    let mut nodes = Vec::new();
    for (node, form) in declared {
        let identity = format!("{ORDERS}{node}");
        nodes.push(node_of(
            declaration_id(&models[0].identity, *form, &identity),
            form.tag(),
            form.form(),
            empty.clone(),
            Some(json!({"kind":"model","identity":models[0].identity,"node":identity})),
        ));
    }
    let integer_id = MemberType::Integer.node_id("checked").expect("integer ID");
    let range = nodes.len();
    let mut range_id = MemberType::IntRange(super::super::model_members::IntegerBounds {
        lower: 0,
        upper: 1000,
    })
    .node_id("checked")
    .expect("range ID");
    // The Integer operand is the node immediately after this range node.
    // V2 carries its graph index, while the member type still describes the
    // same structural bounded domain.
    let operand = range_id.path.get_mut(3).expect("Integer operand");
    assert!(matches!(
        operand,
        super::super::CheckedNodePathSegment::Node(_)
    ));
    *operand = super::super::CheckedNodePathSegment::Index(
        u64::try_from(range + 1).expect("fixture graph index"),
    );
    let mut bounded = node_of(
        range_id,
        "bounded_domain",
        "integer_range",
        json!({"term":"aggregate","members":[
            {"term":"binding","name":"min","value":{"term":"literal","type":integer_id,"value_kind":"integer","value":"0"}},
            {"term":"binding","name":"max","value":{"term":"literal","type":integer_id,"value_kind":"integer","value":"1000"}},
        ]}),
        None,
    );
    bounded["semantic_type"] = json!(integer_id);
    nodes.push(bounded);
    nodes.push(node_of(integer_id, "scalar_type", "integer", empty, None));
    let artifact = |identity: &str| json!({"authority": "agent-ix", "identity": identity});
    let edition = json!({"role": "edition", "definition": artifact("edition")});
    let wire_value = json!({
        "contract_version": "quire.checked-package/v2",
        "name": "checked",
        "lock": {
            "sources": [],
            "edition": edition,
            "profile_selections": [],
            "definition_selections": [],
            "required_features": [],
            "dependency_selections": [],
        },
        "semantic_graph": {"graph_version": "quire.checked-semantic-graph/v2", "nodes": nodes},
        "source_map": [],
        "capability_report": [],
        "diagnostics": {"catalog": artifact("diagnostics"), "entries": []},
    });
    let raw: super::super::RawCheckedPackageWireV2 =
        serde_json::from_value(wire_value).expect("raw wire");
    let graph = super::super::CheckedSemanticGraphV2 {
        graph_version: raw.semantic_graph.graph_version,
        nodes: raw
            .semantic_graph
            .nodes
            .into_iter()
            .map(|node| {
                let body =
                    super::super::CheckedTerm::decode_admitted(&node.body).expect("fixture body");
                node.into_typed(body)
            })
            .collect(),
    };
    let wire = super::super::CheckedPackageWireV2 {
        contract_version: raw.contract_version,
        name: raw.name,
        lock: raw.lock,
        semantic_graph: graph,
        source_map: raw.source_map,
        refinement_step_rows: raw.refinement_step_rows,
        capability_report: raw.capability_report,
        diagnostics: raw.diagnostics,
    };
    let owners =
        ModelOwners::new(&models, &wire.semantic_graph.nodes, |_| Ok(())).expect("owner index");
    let index = owners.index();
    let [model] = models;
    let kinds = vec![CheckedNodeKind::ScalarType(ScalarTypeForm::Boolean); nodes_len(&wire)];
    let terms = wire
        .semantic_graph
        .nodes
        .iter()
        .map(|node| node.body.clone())
        .collect();
    Fixture {
        package: CheckedPackageV2 {
            node_index: super::super::retained_node_index(&wire),
            structural_keys: Vec::new(),
            source_index: super::super::retained_source_index(&wire),
            lowering_facts: Vec::new(),
            wire,
            terms,
            kinds,
            bytes: BYTES,
            models: RetainedModels::new(index, vec![model]),
            direct_dependencies: Vec::new(),
            closure_index: Default::default(),
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

/// Repeated field reads visit only their selected graph node, even when the
/// admitted package contains other model declarations.
#[trace("TC-227", "FR-038-AC-144")]
#[test]
fn repeated_model_field_reads_do_not_walk_the_whole_graph() {
    let mut measured = Vec::new();
    for models in [4, 8, 16, 64] {
        let names: Vec<String> = (0..models).map(|i| format!("R{i}")).collect();
        let types = names
            .iter()
            .map(|name| object_type(&format!("{ORDERS}{name}"), &[], vec![]))
            .collect();
        let declarations: Vec<_> = names
            .iter()
            .map(|name| (name.as_str(), DeclarationForm::ObjectType))
            .collect();
        let fixture = build_fixture(&document(types), &declarations);
        let selected = id_at(&fixture, models - 1);
        for requests in [2, 4, 8] {
            super::probe::reset_node_visits();
            crate::checked_package::shared::checked_node_comparisons::reset();
            for _ in 0..requests {
                fixture
                    .package
                    .model_object_fields(&selected)
                    .expect("fields");
            }
            let visits = super::probe::node_visits();
            let comparisons = crate::checked_package::shared::checked_node_comparisons::count();
            println!("model_fields K={requests} R={models} graph_candidates={visits} id_comparisons={comparisons}");
            if models == 64 {
                assert!(comparisons < requests * 64, "a full graph scan must fail");
            }
            measured.push((models, requests, visits));
        }
        if models == 64 {
            crate::checked_package::shared::checked_node_comparisons::reset();
            for _ in 0..8 {
                assert!(fixture
                    .package
                    .wire
                    .semantic_graph
                    .nodes
                    .iter()
                    .any(|candidate| candidate.node_id == selected));
            }
            assert!(
                crate::checked_package::shared::checked_node_comparisons::count() >= 8 * 64,
                "the retired full scan exceeds the bound"
            );
        }
    }
    for (models, requests, visits) in measured {
        assert_eq!(visits, requests, "models={models}, requests={requests}");
    }
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
        if let super::super::CheckedTerm::Aggregate { members } = &mut node.body {
            if let super::super::CheckedTerm::Binding { value, .. } = &mut members[1] {
                if let super::super::CheckedTerm::Literal { value, .. } = value.as_mut() {
                    *value = json!(upper);
                }
            }
        }
        assert_eq!(
            fixture.package.model_object_fields(&ledger),
            Ok(before.clone()),
            "bounds replaced with 0 and {upper}"
        );
    }
}

#[trace("FR-349-AC-2", "FR-038-AC-140")]
#[test]
fn admitted_model_fields_use_retained_declaration_term() {
    let mut fixture = build_fixture(&ledger(), &[("Ledger", DeclarationForm::ObjectType)]);
    let ledger = id_at(&fixture, 0);
    let expected = fixture
        .package
        .model_object_fields(&ledger)
        .expect("admitted fields");
    fixture.package.wire.semantic_graph.nodes[0].body = super::super::CheckedTerm::Aggregate {
        members: vec![super::super::CheckedTerm::Reference {
            target: ledger.clone(),
        }],
    };
    assert_eq!(fixture.package.model_object_fields(&ledger), Ok(expected));
}

/// `UnknownNode` and `NotModelObjectType` are distinct; the passed node's own
/// fixed members decide the second, by the one rule of step 2, so an unread
/// model node whose body is not `aggregate{[]}` returns the error and not the
/// fields.
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
        package: "checked".into(),
        path: vec![super::super::CheckedNodePathSegment::Name("absent".into())],
        ordinal: 0,
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
    // object type: a non-empty retained term, a `declaration`, a recursion
    // group, a `semantic_type` that is not itself.
    let mut wrong_term = fixture.package.clone();
    wrong_term.terms[1] = super::super::CheckedTerm::Reference {
        target: base.clone(),
    };
    assert_eq!(
        wrong_term.model_object_fields(&base),
        Err(CheckedModelFieldsError::NotModelObjectType)
    );
    let breaks: [fn(&mut CheckedSemanticNodeV2); 3] = [
        |node| {
            node.declaration = Some(super::super::CheckedDeclaration {
                qualified_name: vec!["Base".into()],
            });
        },
        |node| node.recursion_group = Some("group".into()),
        |node| {
            node.semantic_type = CheckedNodeId {
                package: "checked".into(),
                path: vec![super::super::CheckedNodePathSegment::Name("other".into())],
                ordinal: 0,
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
