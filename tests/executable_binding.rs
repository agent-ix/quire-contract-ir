//! Synthetic derived projections assembled from shared corpus wire forms.
//! These tests qualify the public binder, not a source-language frontend.
use quire_contract_ir::{BoundPackage, DiagnosticCode, EXECUTABLE_PROJECTION_FORMAT};
use serde_json::{json, Value};
use sha2::{Digest, Sha256};

fn projection() -> Value {
    let mut package: Value = serde_json::from_str(include_str!(
        "../corpus/contract-v0.1/inputs/package-constructs.json"
    ))
    .unwrap();
    // Use the shared package's clause kinds and source identities, with literal
    // metadata matching this synthetic constant expression population.
    let mut bindings = Vec::new();
    for requirement in package["requirements"].as_array_mut().unwrap() {
        let owner = json!({"package":"agent-ix/conformance",
            "requirement":requirement["id"], "revision":requirement["revision"]});
        for clause in requirement["clauses"].as_array_mut().unwrap() {
            clause["body"] = json!({"node":"literal"});
            if clause["kind"] == "information" {
                continue;
            }
            let mut expression: Value = serde_json::from_str(include_str!(
                "../corpus/contract-v0.1/inputs/expression-boolean-literal.json"
            ))
            .unwrap();
            expression["owner"] = owner.clone();
            expression["execution_point"] = clause["anchor"].clone();
            expression["clause_root"] = json!(true);
            bindings.push(json!({"clause":{"requirement":owner,"clause":clause["id"]},
                "expression":expression}));
        }
    }
    json!({"format":EXECUTABLE_PROJECTION_FORMAT,"package":package,"bindings":bindings})
}

fn decode(value: &Value) -> Result<BoundPackage, Vec<quire_contract_ir::Diagnostic>> {
    BoundPackage::from_json_bytes(&serde_json::to_vec(value).unwrap())
}

/// TC-035
/// FR-023-AC-1
/// FR-023-AC-5
#[test]
fn tc035_public_consumer_preserves_complete_population() {
    let value = projection();
    let bound = decode(&value).unwrap();
    // This authored fixture census is independent of the binding array's length.
    assert_eq!(bound.clauses().len(), 5);
    assert_eq!(bound.informational().len(), 1);
    assert_eq!(
        bound
            .clauses()
            .iter()
            .map(|clause| clause.identity().clause().as_str())
            .collect::<Vec<_>>(),
        ["a_assert", "b_case", "d_inv", "e_post", "f_pre"]
    );
    assert_eq!(
        bound.clauses().len(),
        value["bindings"].as_array().unwrap().len()
    );
    assert!(!bound.informational().is_empty());
    assert!(bound
        .clauses()
        .windows(2)
        .all(|pair| pair[0].identity() < pair[1].identity()));
    for clause in bound.clauses() {
        let original = bound
            .package()
            .resolve_clause(clause.identity(), None)
            .unwrap();
        assert_eq!(clause.kind(), original.kind());
        assert_eq!(clause.anchor(), original.anchor().unwrap());
        assert_eq!(clause.source(), original.source());
        assert_eq!(
            clause.environment().owner(),
            clause.identity().requirement()
        );
        assert_eq!(
            clause.expression().value_type(),
            &quire_contract_ir::ValueType::Boolean
        );
        assert_ne!(clause.declaration_digest(), clause.expression_digest());
    }
}

/// TC-035
/// FR-023-AC-2
#[test]
fn tc035_refuses_invalid_binding_population_and_context() {
    let original = projection();
    let mut cases = Vec::new();
    let mut missing = original.clone();
    missing["bindings"].as_array_mut().unwrap().pop();
    cases.push(missing);
    let mut duplicate = original.clone();
    let first = duplicate["bindings"][0].clone();
    duplicate["bindings"].as_array_mut().unwrap().push(first);
    cases.push(duplicate);
    for (pointer, replacement) in [
        (
            "/bindings/0/clause/requirement/package",
            json!("foreign/package"),
        ),
        ("/bindings/0/clause/requirement/revision", json!(999)),
        ("/bindings/0/clause/clause", json!("absent")),
        ("/bindings/0/expression/owner/requirement", json!("foreign")),
        (
            "/bindings/0/expression/execution_point",
            json!({"kind":"pre","operation":"other"}),
        ),
        ("/bindings/0/expression/clause_root", json!(false)),
        (
            "/bindings/0/expression/expected_type",
            json!({"kind":"text"}),
        ),
        (
            "/bindings/0/expression/expression",
            json!({"node":"text_literal","value":"wrong",
            "source":original["bindings"][0]["expression"]["expression"]["source"]}),
        ),
    ] {
        let mut mutated = original.clone();
        *mutated.pointer_mut(pointer).unwrap() = replacement;
        cases.push(mutated);
    }
    let bound = decode(&original).unwrap();
    let mut informational = original.clone();
    informational["bindings"][0]["clause"] =
        serde_json::to_value(&bound.informational()[0]).unwrap();
    cases.push(informational);
    for (index, invalid) in cases.iter().enumerate() {
        assert!(decode(invalid).is_err(), "invalid case {index} accepted");
    }
}

/// TC-035
/// FR-023-AC-3
#[test]
fn tc035_identity_is_order_independent_and_semantically_sensitive() {
    let original = projection();
    let digest = decode(&original).unwrap().digest();
    let mut reordered = original.clone();
    reordered["bindings"].as_array_mut().unwrap().reverse();
    assert_eq!(digest, decode(&reordered).unwrap().digest());
    reordered["bindings"][0]["expression"]["expression"]["value"] = json!(false);
    assert_ne!(digest, decode(&reordered).unwrap().digest());
}

/// TC-035
/// FR-023-AC-4
/// FR-023-AC-5
#[test]
fn tc035_strict_wire_boundary() {
    let original = projection();
    for pointer in [
        "",
        "/package",
        "/bindings/0",
        "/bindings/0/clause/requirement",
        "/bindings/0/expression/owner",
        "/bindings/0/expression/expression/source/start/source",
    ] {
        let mut invalid = original.clone();
        invalid
            .pointer_mut(pointer)
            .unwrap()
            .as_object_mut()
            .unwrap()
            .insert("unknown".into(), json!(true));
        assert!(
            decode(&invalid).is_err(),
            "unknown member at {pointer} accepted"
        );
    }
    let mut wrong = original.clone();
    wrong["format"] = json!("future");
    assert!(decode(&wrong).is_err());
    let mut bytes = serde_json::to_vec(&original).unwrap();
    bytes.extend_from_slice(b" {}");
    assert!(BoundPackage::from_json_bytes(&bytes).is_err());
    assert!(BoundPackage::from_json_bytes(&[0xff]).is_err());
    let huge = vec![b' '; quire_contract_ir::MAX_CONFORMANCE_FILE_BYTES as usize + 1];
    assert_eq!(
        BoundPackage::from_json_bytes(&huge).unwrap_err()[0].code,
        DiagnosticCode::SemanticInputTooLarge
    );
    let deep = format!("{}0{}", "[".repeat(577), "]".repeat(577));
    assert!(BoundPackage::from_json_bytes(deep.as_bytes()).is_err());
    let mut collection = original.clone();
    collection["bindings"] = Value::Array(vec![original["bindings"][0].clone(); 10001]);
    assert_eq!(
        decode(&collection).unwrap_err()[0].code,
        DiagnosticCode::SemanticInputTooLarge
    );
}

fn duplicate_member_json(value: &Value, path: &[&str], member: &str) -> String {
    if path.is_empty() {
        assert!(value.as_object().unwrap().contains_key(member));
        let object = serde_json::to_string(value).unwrap();
        // Put the conflicting member first; a last-value-wins Value decoder
        // would silently restore the healthy original member and accept it.
        return format!(
            "{{{}:null,{}",
            serde_json::to_string(member).unwrap(),
            &object[1..]
        );
    }
    match value {
        Value::Object(fields) => {
            assert!(fields.contains_key(path[0]));
            let entries = fields
                .iter()
                .map(|(key, child)| {
                    let contents = if key == path[0] {
                        duplicate_member_json(child, &path[1..], member)
                    } else {
                        serde_json::to_string(child).unwrap()
                    };
                    format!("{}:{contents}", serde_json::to_string(key).unwrap())
                })
                .collect::<Vec<_>>()
                .join(",");
            format!("{{{entries}}}")
        }
        Value::Array(items) => {
            let selected: usize = path[0].parse().unwrap();
            assert!(selected < items.len());
            let entries = items
                .iter()
                .enumerate()
                .map(|(index, child)| {
                    if index == selected {
                        duplicate_member_json(child, &path[1..], member)
                    } else {
                        serde_json::to_string(child).unwrap()
                    }
                })
                .collect::<Vec<_>>()
                .join(",");
            format!("[{entries}]")
        }
        _ => panic!("duplicate injection path does not reach an object"),
    }
}

/// TC-035
/// FR-023-AC-4
/// FR-023-AC-5
#[test]
fn tc035_recursive_duplicate_members_cannot_be_erased_by_value_decoding() {
    let original = minimal_projection(1);
    assert!(decode(&original).is_ok());
    for (path, member) in [
        ("", "format"),
        ("", "bindings"),
        ("package", "id"),
        ("package/schema_version", "major"),
        ("bindings/0", "expression"),
        ("bindings/0/clause", "clause"),
        ("bindings/0/clause/requirement", "package"),
        ("bindings/0/clause/requirement", "revision"),
        ("bindings/0/expression", "owner"),
        ("bindings/0/expression", "clause_root"),
        ("bindings/0/expression/expression", "node"),
        (
            "bindings/0/expression/expression/source/start/source",
            "document",
        ),
    ] {
        let segments: Vec<_> = path.split('/').filter(|part| !part.is_empty()).collect();
        let bytes = duplicate_member_json(&original, &segments, member);
        // The adversary really is indistinguishable to last-value-wins decoding.
        assert_eq!(serde_json::from_str::<Value>(&bytes).unwrap(), original);
        let errors = BoundPackage::from_json_bytes(bytes.as_bytes())
            .expect_err(&format!("duplicate {path}/{member} was erased"));
        assert!(
            errors
                .iter()
                .any(|error| error.code == DiagnosticCode::InvalidWireFormat),
            "duplicate {path}/{member}: {errors:?}"
        );
    }
    // JSON names are compared after escape decoding, not as raw source bytes.
    let escaped = duplicate_member_json(&original, &["package"], "id").replacen(
        r#""id":null"#,
        r#""\u0069d":null"#,
        1,
    );
    assert!(escaped.contains(r#""\u0069d":null"#));
    assert_eq!(serde_json::from_str::<Value>(&escaped).unwrap(), original);
    assert!(BoundPackage::from_json_bytes(escaped.as_bytes())
        .unwrap_err()
        .iter()
        .any(|error| error.code == DiagnosticCode::InvalidWireFormat));
}

fn minimal_projection(count: usize) -> Value {
    let mut value = projection();
    value["bindings"].as_array_mut().unwrap().truncate(count);
    let identities: Vec<_> = value["bindings"]
        .as_array()
        .unwrap()
        .iter()
        .map(|binding| binding["clause"]["clause"].clone())
        .collect();
    value["package"]["requirements"][0]["clauses"]
        .as_array_mut()
        .unwrap()
        .retain(|clause| identities.contains(&clause["id"]));
    for binding in value["bindings"].as_array_mut().unwrap() {
        for collection in ["types", "values", "functions"] {
            binding["expression"][collection] = json!([]);
        }
    }
    value
}

fn declarations(value: &mut Value, count: usize, option_layers: usize) {
    for binding in value["bindings"].as_array_mut().unwrap() {
        let source = binding["expression"]["expression"]["source"].clone();
        let mut value_type = json!({"kind":"boolean"});
        for _ in 0..option_layers {
            value_type = json!({"kind":"option", "value":value_type});
        }
        binding["expression"]["values"] = Value::Array(
            (0..count)
                .map(|index| {
                    json!({"name":format!("v{index}"), "kind":"input", "value_type":value_type,
                "source":source})
                })
                .collect(),
        );
    }
}

/// TC-035
/// FR-023-AC-2
/// FR-023-AC-3
#[test]
fn tc035_dependency_agreement_has_a_real_positive_and_independent_negatives() {
    let mut valid = minimal_projection(1);
    declarations(&mut valid, 1, 0);
    let expression = &mut valid["bindings"][0]["expression"];
    expression["expression"] = json!({"node":"value_reference", "name":"v0",
        "observation":"current", "source":expression["expression"]["source"]});
    let identity = json!({"requirement":valid["bindings"][0]["clause"]["requirement"],
        "kind":"input", "path":["v0"], "observation":"current"});
    valid["package"]["requirements"][0]["clauses"][0]["body"] =
        json!({"node":"reference", "identity":identity});
    let bound = decode(&valid).unwrap();
    assert_eq!(bound.clauses()[0].expression().dependencies().len(), 1);
    assert_eq!(
        serde_json::to_value(bound.clauses()[0].expression().dependencies()).unwrap(),
        json!([identity])
    );
    let mut missing_metadata = valid.clone();
    missing_metadata["package"]["requirements"][0]["clauses"][0]["body"] =
        json!({"node":"literal"});
    let mut wrong_metadata = valid.clone();
    wrong_metadata["package"]["requirements"][0]["clauses"][0]["body"]["identity"]["path"] =
        json!(["other"]);
    let mut missing_expression_dependency = valid;
    missing_expression_dependency["bindings"][0]["expression"]["expression"] =
        minimal_projection(1)["bindings"][0]["expression"]["expression"].clone();
    for invalid in [
        missing_metadata,
        wrong_metadata,
        missing_expression_dependency,
    ] {
        let errors = decode(&invalid).unwrap_err();
        assert!(
            errors
                .iter()
                .any(|error| error.code == DiagnosticCode::MalformedReference
                    && error.message.contains("dependencies disagree")),
            "{errors:?}"
        );
    }
}

/// TC-035
/// FR-023-AC-1
/// FR-023-AC-3
#[test]
fn tc035_declaration_semantics_change_identity_without_changing_expression() {
    let mut original = minimal_projection(1);
    declarations(&mut original, 1, 0);
    let before = decode(&original).unwrap();
    original["bindings"][0]["expression"]["values"][0]["value_type"] = json!({"kind":"text"});
    let after = decode(&original).unwrap();
    assert_ne!(before.digest(), after.digest());
    assert_ne!(
        before.clauses()[0].declaration_digest(),
        after.clauses()[0].declaration_digest()
    );
    assert_eq!(
        before.clauses()[0].expression_digest(),
        after.clauses()[0].expression_digest()
    );
    assert_eq!(before.package(), after.package());
}

/// TC-035
/// FR-023-AC-3
#[test]
fn tc035_bound_digest_matches_an_independently_written_canonical_envelope() {
    use quire_contract_ir::CanonicalProfile;
    let bound = decode(&projection()).unwrap();
    let quote = |text: &str| serde_json::to_string(text).unwrap();
    let mut expected_clauses: Vec<_> = bound.clauses().iter().collect();
    expected_clauses.sort_by_key(|clause| clause.identity());
    let entries = expected_clauses
        .iter()
        .map(|clause| {
            let owner = clause.identity().requirement();
            let declaration = clause
                .environment()
                .canonical_declaration(CanonicalProfile::V1)
                .unwrap()
                .digest();
            let expression = clause
                .expression()
                .canonical_expression(CanonicalProfile::V1)
                .unwrap()
                .digest();
            assert_eq!(clause.declaration_digest(), declaration);
            assert_eq!(clause.expression_digest(), expression);
            // Write the exact lexical field order, not serde_json::Value map order
            // and not the binder's private canonical-envelope helper.
            format!(
                concat!(
                    "{{\"clause\":{{\"clause\":{},\"requirement\":{{",
                    "\"package\":{},\"requirement\":{},\"revision\":{}}}}},",
                    "\"declaration\":\"{}\",\"expression\":\"{}\"}}"
                ),
                quote(clause.identity().clause().as_str()),
                quote(owner.package().as_str()),
                quote(owner.requirement().as_str()),
                owner.revision().get(),
                declaration,
                expression
            )
        })
        .collect::<Vec<_>>()
        .join(",");
    let package = bound
        .package()
        .canonical_package(CanonicalProfile::V1)
        .unwrap()
        .digest();
    let expected = format!(
        concat!(
            "{{\"bindings\":[{}],\"canonical_profile\":\"quire.contract.canonical-json/v1\",",
            "\"package\":\"{}\",\"profile\":\"quire.contract.bound-identity/v1\"}}"
        ),
        entries, package
    );
    // Parsing checks the independently authored braces without deriving the expected bytes.
    assert!(serde_json::from_str::<Value>(&expected).is_ok());
    assert_eq!(
        bound.digest().to_string(),
        format!("{:x}", Sha256::digest(expected.as_bytes()))
    );
}

/// TC-035
/// FR-023-AC-1
/// FR-023-AC-2
#[test]
fn tc035_informational_and_empty_packages_do_not_invent_executable_clauses() {
    let mut information = projection();
    information["bindings"] = json!([]);
    information["package"]["requirements"][0]["clauses"]
        .as_array_mut()
        .unwrap()
        .retain(|clause| clause["kind"] == "information");
    let bound = decode(&information).unwrap();
    assert!(bound.clauses().is_empty());
    assert_eq!(bound.informational().len(), 1);
    let mut empty = information;
    empty["package"]["requirements"] = json!([]);
    let bound = decode(&empty).unwrap();
    assert!(bound.clauses().is_empty());
    assert!(bound.informational().is_empty());
    // Empty *bindings* on an executable package are a missing population, not information.
    let mut missing = minimal_projection(1);
    missing["bindings"] = json!([]);
    assert!(decode(&missing)
        .unwrap_err()
        .iter()
        .any(|error| error.code == DiagnosticCode::OrphanedClauseReference));
}

/// TC-035
/// FR-023-AC-4
#[test]
fn tc035_semantic_budget_is_aggregate_and_individual_nodes_are_bounded() {
    // Each declaration contributes a declaration and its Boolean type; each
    // expression adds its expected type and literal node. These populations
    // are authored independently of the binder's internal counter.
    let mut single = minimal_projection(1);
    declarations(&mut single, 6500, 0);
    assert!(decode(&single).is_ok());
    let mut under = minimal_projection(2);
    declarations(&mut under, 6000, 0);
    assert!(decode(&under).is_ok());
    let mut aggregate = minimal_projection(2);
    declarations(&mut aggregate, 6500, 0);
    let encoded = serde_json::to_vec(&aggregate).unwrap();
    assert!(encoded.len() < quire_contract_ir::MAX_CONFORMANCE_FILE_BYTES as usize);
    let errors = decode(&aggregate).unwrap_err();
    assert!(
        errors
            .iter()
            .any(|error| error.code == DiagnosticCode::SemanticInputTooLarge
                && error.message.contains("aggregate")),
        "{errors:?}"
    );
    let mut individual = minimal_projection(1);
    declarations(&mut individual, 9000, 1);
    assert!(decode(&individual)
        .unwrap_err()
        .iter()
        .any(|error| error.code == DiagnosticCode::SemanticInputTooLarge));
}

/// TC-035
/// FR-023-AC-4
#[test]
fn tc035_expression_node_limit_is_not_replaced_by_the_larger_semantic_budget() {
    for (leaves, accepted) in [(5000, true), (5001, false)] {
        let mut value = minimal_projection(1);
        let literal = value["bindings"][0]["expression"]["expression"].clone();
        let source = literal["source"].clone();
        let mut level = vec![literal; leaves];
        // A shallow balanced expression has exactly 2*leaves-1 nodes: below the
        // 25000 semantic budget and collection limit, straddling only 10000 AST nodes.
        while level.len() > 1 {
            let mut next = Vec::new();
            let mut expressions = level.into_iter();
            while let Some(left) = expressions.next() {
                next.push(if let Some(right) = expressions.next() {
                    json!({"node":"boolean", "operator":"short_circuit_and", "left":left,
                        "right":right, "source":source})
                } else {
                    left
                });
            }
            level = next;
        }
        value["bindings"][0]["expression"]["expression"] = level.pop().unwrap();
        let bytes = serde_json::to_vec(&value).unwrap();
        assert!(bytes.len() < quire_contract_ir::MAX_CONFORMANCE_FILE_BYTES as usize);
        let outcome = BoundPackage::from_json_bytes(&bytes);
        if accepted {
            assert!(outcome.is_ok(), "9999 AST nodes refused: {outcome:?}");
        } else {
            assert!(outcome
                .unwrap_err()
                .iter()
                .any(|error| error.code == DiagnosticCode::ExpressionTooLarge));
        }
    }
}

/// TC-035
/// FR-023-AC-5
#[test]
fn tc035_normative_schema_is_checked_independently_of_binder() {
    let schema: Value = serde_json::from_str(include_str!(
        "../schemas/contract-executable-projection-v1.schema.json"
    ))
    .unwrap();
    let package: Value = serde_json::from_str(include_str!(
        "../schemas/contract-package-reference-v1.schema.json"
    ))
    .unwrap();
    let conformance: Value = serde_json::from_str(include_str!(
        "../schemas/contract-conformance-manifest-v1.schema.json"
    ))
    .unwrap();
    let validator = jsonschema::JSONSchema::options()
        .with_draft(jsonschema::Draft::Draft7)
        .with_document(package["$id"].as_str().unwrap().to_owned(), package.clone())
        .with_document(
            conformance["$id"].as_str().unwrap().to_owned(),
            conformance.clone(),
        )
        .compile(&schema)
        .unwrap();
    let original = projection();
    assert!(validator.is_valid(&original));
    assert!(decode(&original).is_ok());
    for pointer in [
        "/format",
        "/package/id",
        "/bindings/0/clause/clause",
        "/bindings/0/expression/expression/node",
        "/bindings/0/expression/expected_type/kind",
    ] {
        let mut invalid = original.clone();
        *invalid.pointer_mut(pointer).unwrap() = json!(17);
        assert!(
            !validator.is_valid(&invalid),
            "schema accepted wrong type at {pointer}"
        );
        assert!(decode(&invalid).is_err());
    }
    for member in ["format", "package", "bindings"] {
        let mut invalid = original.clone();
        invalid.as_object_mut().unwrap().remove(member);
        assert!(
            !validator.is_valid(&invalid),
            "schema accepted missing {member}"
        );
        assert!(decode(&invalid).is_err());
    }
}

/// TC-035
/// FR-023-AC-4
#[test]
fn tc035_depth_limits_do_not_abort_the_process() {
    const CHILD: &str = "QUIRE_BINDING_DEPTH_CONTROL";
    if let Ok(control) = std::env::var(CHILD) {
        if control.starts_with("wire:") {
            let depth: usize = control.split_once(':').unwrap().1.parse().unwrap();
            let bytes = format!("{}0{}", "[".repeat(depth), "]".repeat(depth));
            let errors = BoundPackage::from_json_bytes(bytes.as_bytes()).unwrap_err();
            assert_eq!(
                errors[0].code,
                if depth == 576 {
                    DiagnosticCode::InvalidWireFormat
                } else {
                    DiagnosticCode::SemanticInputTooLarge
                }
            );
            return;
        }
        let mut value = minimal_projection(1);
        let source = value["bindings"][0]["expression"]["expression"]["source"].clone();
        let (kind, layers) = control.split_once(':').unwrap();
        let layers: usize = layers.parse().unwrap();
        let mut nested = match kind {
            "expression" => json!({"node":"boolean_literal","value":true,"source":source}),
            "body" => json!({"node":"literal"}),
            _ => json!({"kind":"boolean"}),
        };
        for _ in 0..layers {
            nested = match kind {
                "expression" => json!({"node":"boolean_not","operand":nested,"source":source}),
                "body" => json!({"node":"composite","children":[nested]}),
                _ => json!({"kind":"option","value":nested}),
            };
        }
        if kind == "expression" {
            value["bindings"][0]["expression"]["expression"] = nested;
        } else if kind == "body" {
            value["package"]["requirements"][0]["clauses"][0]["body"] = nested;
        } else {
            declarations(&mut value, 1, 0);
            value["bindings"][0]["expression"]["values"][0]["value_type"] = nested;
        }
        let bytes = serde_json::to_vec(&value).unwrap();
        eprintln!("constructed bounded-depth input; entering public binder");
        let outcome = BoundPackage::from_json_bytes(&bytes);
        eprintln!("public binder returned normally");
        if layers == 255 {
            assert!(outcome.is_ok(), "accepted semantic depth 256: {outcome:?}");
        } else {
            assert!(outcome
                .unwrap_err()
                .iter()
                .any(|error| error.code == DiagnosticCode::SemanticInputTooLarge));
        }
        return;
    }
    for control in [
        "expression:255",
        "expression:256",
        "type:255",
        "type:256",
        "body:255",
        "body:256",
        "wire:576",
        "wire:577",
    ] {
        let output = std::process::Command::new(std::env::current_exe().unwrap())
            .args([
                "--exact",
                "tc035_depth_limits_do_not_abort_the_process",
                "--nocapture",
            ])
            .env(CHILD, control)
            .output()
            .unwrap();
        assert!(
            output.status.success(),
            "{control} aborted or misclassified: {}\n{}",
            String::from_utf8_lossy(&output.stdout),
            String::from_utf8_lossy(&output.stderr)
        );
    }
}
