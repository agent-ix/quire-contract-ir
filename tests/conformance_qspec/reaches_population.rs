// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (C) 2026 Agent-IX

//! The published QSpec reaches-population descriptors through the IR reader.

use super::*;

fn node_id(digest: &str) -> Value {
    json!({"domain": "quire.checked-semantic-node/v1", "digest": digest})
}

fn node_index(package: &Value, tag: &str, form: &str) -> usize {
    package["semantic_graph"]["nodes"]
        .as_array()
        .expect("nodes")
        .iter()
        .position(|node| node["node_tag"] == tag && node["semantic_form"] == form)
        .unwrap_or_else(|| panic!("missing {tag}/{form} node"))
}

fn append_node(package: &mut Value, node: Value) -> Value {
    let id = node["node_id"].clone();
    let mut source = package["source_map"][0].clone();
    source["node_id"] = id.clone();
    source["role"] = node["occurrences"][0]["role"].clone();
    source["ordinal"] = node["occurrences"][0]["ordinal"].clone();
    package["semantic_graph"]["nodes"]
        .as_array_mut()
        .expect("nodes")
        .push(node);
    package["source_map"]
        .as_array_mut()
        .expect("source map")
        .push(source);
    id
}

fn structural_id(node: &Value) -> Value {
    let preimage = json!({
        "version": "quire.structural-node/v1",
        "node_tag": node["node_tag"],
        "semantic_form": node["semantic_form"],
        "semantic_type": if node["semantic_type"] == node["node_id"] {
            Value::Null
        } else {
            node["semantic_type"].clone()
        },
        "declaration": node.get("declaration"),
        "recursion": node.get("recursion_group"),
        "body": node["body"],
        "owner": node.get("owner"),
    });
    // Optional owner is absent from anonymous structural preimages.
    let mut preimage = preimage;
    if node.get("owner").is_none() {
        preimage.as_object_mut().expect("preimage").remove("owner");
    }
    node_id(&canonical_digest(&preimage))
}

fn append_rekeyed(package: &mut Value, mut node: Value) -> Value {
    let self_typed = node["semantic_type"] == node["node_id"];
    let id = structural_id(&node);
    node["node_id"] = id.clone();
    if self_typed {
        node["semantic_type"] = id.clone();
    }
    if package["semantic_graph"]["nodes"]
        .as_array()
        .expect("nodes")
        .iter()
        .any(|existing| existing["node_id"] == id)
    {
        return id;
    }
    append_node(package, node)
}

fn append_population(package: &mut Value, object: &Value, max: Option<&str>, extra: bool) -> Value {
    let integer = package["semantic_graph"]["nodes"][node_index(package, "scalar_type", "integer")]
        ["node_id"]
        .clone();
    let mut reference = package["semantic_graph"]["nodes"]
        [node_index(package, "composite_type", "reference")]
    .clone();
    reference["body"]["members"][0]["target"] = object.clone();
    reference["dependencies"] = json!([object]);
    let reference = append_rekeyed(package, reference);

    let mut set =
        package["semantic_graph"]["nodes"][node_index(package, "composite_type", "set")].clone();
    set["body"]["members"][0]["target"] = reference.clone();
    set["dependencies"] = json!([reference]);
    let set = append_rekeyed(package, set);

    let mut population = package["semantic_graph"]["nodes"]
        [node_index(package, "bounded_domain", "integer_range")]
    .clone();
    population["semantic_form"] = json!("model_population");
    population["semantic_type"] = set.clone();
    let mut members = Vec::new();
    if let Some(max) = max {
        members.push(json!({"name":"max", "term":"binding", "value":{
            "term":"literal", "type":integer, "value_kind":"integer", "value":max
        }}));
    }
    if extra {
        members.push(json!({"name":"other", "term":"binding", "value":{
            "term":"literal", "type":integer, "value_kind":"integer", "value":"1"
        }}));
    }
    population["body"] = json!({"term":"aggregate", "members": members});
    population["dependencies"] = json!([set]);
    append_rekeyed(package, population)
}

fn candidate(base: &Value, case: &Value) -> Value {
    let mut package = base.clone();
    let application = package["semantic_graph"]["nodes"]
        .as_array()
        .expect("nodes")
        .iter()
        .position(|node| node["body"]["operation"]["identity"] == "quire.op.model.reaches")
        .expect("reaches application");
    let descriptor = &case["population"];
    let link = if descriptor.is_null() {
        None
    } else if descriptor["absent"] == true {
        Some(node_id(&"f".repeat(64)))
    } else if let Some(owner) = descriptor["model_population_over"].as_str() {
        let order_node = package["semantic_graph"]["nodes"]
            .as_array()
            .expect("nodes")
            .iter()
            .find(|node| {
                node["node_tag"] == "model" && node["owner"]["node"] == "ix://acme/orders/Order"
            })
            .expect("Order declaration")
            .clone();
        let order = order_node["node_id"].clone();
        let object = if owner == "ix://acme/orders/Order" {
            order
        } else {
            let mut other = order_node;
            other["owner"]["node"] = json!(owner);
            append_rekeyed(&mut package, other)
        };
        Some(append_population(
            &mut package,
            &object,
            descriptor["max"].as_str(),
            descriptor["extra_member"] == true,
        ))
    } else {
        Some(
            package["semantic_graph"]["nodes"][node_index(
                &package,
                descriptor["node_tag"].as_str().expect("tag"),
                descriptor["semantic_form"].as_str().expect("form"),
            )]["node_id"]
                .clone(),
        )
    };
    if let Some(link) = link {
        let node = &mut package["semantic_graph"]["nodes"][application];
        node["body"]["operation"]["member"]["population"] = link.clone();
        if case["dependencies"] != "without-population" {
            let mut dependencies = node["dependencies"].as_array().expect("deps").clone();
            dependencies.push(link);
            dependencies.sort_by(|a, b| a["digest"].as_str().cmp(&b["digest"].as_str()));
            dependencies.dedup();
            node["dependencies"] = json!(dependencies);
        }
        let old = node["node_id"]["digest"].as_str().expect("id").to_owned();
        let new = canonical_digest(&union_value_preimage(node));
        rewrite_digest(&mut package, &old, &new);
    }
    package["package_id"]["digest"] =
        json!(derived_with(&package, None).expect("package identity"));
    package
}

#[trace(
    "TC-048",
    "FR-038-AC-269",
    "FR-038-AC-272",
    "FR-038-AC-273",
    "FR-038-AC-274",
    "FR-038-AC-276"
)]
#[test]
fn tc_048_qspec_reaches_population_cases_match_the_reader() {
    let dir = fixtures_dir();
    let adverse = read_json(&dir.join(ADVERSE)).expect("QSpec adverse fixture");
    let rows = mutations(&adverse, "reaches_population_cases").expect("published cases");
    assert_eq!(rows.len(), 7);
    let mut seen = std::collections::BTreeSet::new();
    for row in rows {
        let id = text_of(row, "id").expect("case id");
        assert!(seen.insert(id), "duplicate case {id}");
        let base = text_of(row, "base").expect("base");
        let package = read_json(&dir.join(format!("{base}.json"))).expect("published base");
        let package = candidate(&package, row);
        let got = reading_of(&package).unwrap_or_else(|why| panic!("{id}: {why}"));
        let expected = if row["outcome"] == "admitted" {
            "admitted".to_owned()
        } else if let Some(cause) = row["expected_cause"].as_str() {
            format!(
                "refused:{}/{cause}",
                text_of(row, "expected_code").expect("code")
            )
        } else {
            format!("refused:{}", text_of(row, "expected_code").expect("code"))
        };
        assert_eq!(got, expected, "{id}: {:?}", read_package(&package));
    }
}

#[trace("TC-048", "FR-038-AC-343")]
#[test]
fn tc_048_qspec_unbounded_population_shapes_match_the_reader() {
    let dir = fixtures_dir();
    let adverse = read_json(&dir.join(ADVERSE)).expect("QSpec adverse fixture");
    let rows = mutations(&adverse, "unbounded_population_cases").expect("published cases");
    assert_eq!(rows.len(), 5);
    let mut seen = std::collections::BTreeSet::new();
    for row in rows {
        let id = text_of(row, "id").expect("case id");
        assert!(seen.insert(id), "duplicate case {id}");
        let base = text_of(row, "base").expect("base");
        let package = read_json(&dir.join(format!("{base}.json"))).expect("published base");
        let descriptor = json!({
            "model_population_over": "ix://acme/orders/Order",
            "max": row["max"],
            "extra_member": row["extra_member"] == true,
        });
        let case = json!({"population": descriptor});
        let package = candidate(&package, &case);
        let population_position = node_index(&package, "bounded_domain", "model_population");
        let population_id =
            package["semantic_graph"]["nodes"][population_position]["node_id"].clone();
        let result = read_package(&package).unwrap_or_else(|why| panic!("{id}: {why}"));
        if row["outcome"] == "admitted" {
            assert!(
                matches!(result, CheckedPackageV2ReadResult::Admitted(_)),
                "{id}: {result:?}"
            );
        } else {
            let CheckedPackageV2ReadResult::Refused(refusal) = result else {
                panic!("{id}: expected refusal, got {result:?}");
            };
            assert_eq!(
                code_word(refusal.code),
                text_of(row, "expected_code").expect("code"),
                "{id}"
            );
            assert_eq!(
                refusal.cause.map(cause_word),
                row["expected_cause"].as_str(),
                "{id}"
            );
            assert_eq!(
                refusal.path.as_ref().map(ToString::to_string).as_deref(),
                Some(format!("/semantic_graph/nodes/{population_position}/node_id").as_str()),
                "{id}"
            );
            assert_eq!(
                refusal.locus.as_ref().map(|locus| locus.digest.as_ref()),
                population_id["digest"].as_str(),
                "{id}"
            );
        }
    }
}
