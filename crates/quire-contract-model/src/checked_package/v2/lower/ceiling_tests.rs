use super::*;
use crate::checked_package::shared::CheckedPackageLimit;
use ix_trace_rs::trace;
use quire_canonical::LimitKind;
use serde_json::json;

fn node_id(fill: char) -> serde_json::Value {
    json!({
        "domain": "quire.checked-semantic-node/v1",
        "digest": fill.to_string().repeat(64),
    })
}

fn projection() -> super::super::CheckedNodeProjectionV2 {
    serde_json::from_value(json!({
        "node_id": node_id('a'),
        "schema_version": "quire.checked-semantic-graph/v2",
        "node_tag": "scalar_type",
        "semantic_form": "boolean",
        "semantic_type": node_id('a'),
        "dependencies": [],
        "body": {"term": "aggregate", "members": []},
    }))
    .expect("projection")
}

fn typed_id(fill: char) -> CheckedNodeId {
    serde_json::from_value(node_id(fill)).expect("node id")
}

fn sha256_of(text: &str) -> String {
    format!("{:x}", Sha256::digest(text.as_bytes()))
}

/// The lowered node preimage of [`projection`] with one dependency `b...`,
/// written out by hand with members in RFC 8785 order.
const LOWERED_NODE_TEXT: &str = concat!(
    r#"{"bounds":[],"claims":[],"dependencies":[{"digest":""#,
    "bbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbb",
    r#"","domain":"quire.checked-semantic-node/v1"}],"node":{"body":{"members":[],"#,
    r#""term":"aggregate"},"dependencies":[],"node_id":{"digest":""#,
    "aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa",
    r#"","domain":"quire.checked-semantic-node/v1"},"node_tag":"scalar_type","#,
    r#""schema_version":"quire.checked-semantic-graph/v2","semantic_form":"boolean","#,
    r#""semantic_type":{"digest":""#,
    "aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa",
    r#"","domain":"quire.checked-semantic-node/v1"}},"#,
    r#""version":"quire.contract-ir.lowered-node/v1"}"#,
);

/// The package preimage of an empty lowering, written out by hand.
const EMPTY_PACKAGE_TEXT: &str = concat!(
    r#"{"dependencies":[],"lowered":[],"source_package_id":{"algorithm":"sha256","digest":""#,
    "cccccccccccccccccccccccccccccccccccccccccccccccccccccccccccccccc",
    r#"","domain":"quire.package.semantic/v2"},"#,
    r#""version":"quire.contract-ir.contract-package/v1"}"#,
);

fn source_package_id() -> CheckedSemanticId {
    CheckedSemanticId {
        domain: "quire.package.semantic/v2".into(),
        algorithm: "sha256".into(),
        digest: "c".repeat(64).into_boxed_str(),
    }
}

/// The lowered node and lowered package preimages encode to expected byte
/// strings written out above, not computed by a call into the code under
/// test, and hash to the SHA-256 of those bytes.
///
/// Tracing: TC-048, FR-038-AC-89
#[trace("TC-048", "FR-038-AC-89")]
#[test]
fn tc_048_lowered_preimages_encode_to_the_expected_canonical_bytes() {
    let dependencies = [typed_id('b')];
    let node = LoweredNodePreimage {
        version: LOWERED_NODE_PREIMAGE,
        node: projection(),
        dependencies: &dependencies,
        bounds: &[],
        claims: &[],
    };
    let bytes = quire_canonical::to_vec(&node, Limits::new(1 << 20)).expect("encodes");
    assert_eq!(String::from_utf8(bytes).expect("utf-8"), LOWERED_NODE_TEXT);
    assert_eq!(
        identify_node(&node, 1 << 20).as_deref(),
        Ok(sha256_of(LOWERED_NODE_TEXT).as_str())
    );

    let source = source_package_id();
    let package = ContractPackagePreimage {
        version: CONTRACT_PACKAGE_VERSION,
        source_package_id: &source,
        lowered: &[],
        dependencies: &[],
    };
    let bytes = quire_canonical::to_vec(&package, Limits::new(1 << 20)).expect("encodes");
    assert_eq!(String::from_utf8(bytes).expect("utf-8"), EMPTY_PACKAGE_TEXT);
    let encoded = encode_package(&package, 1 << 20).expect("encodes");
    assert_eq!(
        encoded.canonical_bytes.as_ref(),
        EMPTY_PACKAGE_TEXT.as_bytes()
    );
    assert_eq!(
        encoded.package_id.digest.as_ref(),
        sha256_of(EMPTY_PACKAGE_TEXT)
    );
}

/// The seam of FR-038-AC-95's node step: a ceiling equal to the length of
/// the preimage's canonical bytes identifies the node, one byte lower does
/// not and reports the byte count the encoder needed, which is above the
/// ceiling.
///
/// Tracing: TC-048, FR-038-AC-95
#[trace("TC-048", "FR-038-AC-95")]
#[test]
fn tc_048_a_lowered_node_is_identified_at_the_ceiling_and_not_one_byte_under_it() {
    let dependencies = [typed_id('b')];
    let preimage = LoweredNodePreimage {
        version: LOWERED_NODE_PREIMAGE,
        node: projection(),
        dependencies: &dependencies,
        bounds: &[],
        claims: &[],
    };
    let length = LOWERED_NODE_TEXT.len() as u64;
    assert_eq!(
        identify_node(&preimage, length).map(|digest| digest.len()),
        Ok(64)
    );
    // One byte under: `consumed` is the `required` of encoding the same
    // preimage under that limit, above the limit and at most the length.
    let required = identify_node(&preimage, length - 1).expect_err("one byte under refuses");
    assert_eq!(required, required_of(&preimage, length - 1));
    assert!(required > length - 1 && required <= length);
    // At a limit of one byte the encoder refuses at the opening brace.
    let required = identify_node(&preimage, 1).expect_err("one byte refuses");
    assert_eq!(required, required_of(&preimage, 1));
    assert!(required > 1);
}

/// The `required` of the canonical-bytes refusal of encoding `value` under
/// `limit`: the oracle a `failed` record's `consumed` is compared with.
fn required_of<T: Encode>(value: &T, limit: u64) -> u64 {
    match quire_canonical::to_vec(value, Limits::new(limit)) {
        Err(Error::Limit(refusal)) if refusal.kind == LimitKind::CanonicalBytes => refusal.required,
        other => panic!("expected the canonical-bytes refusal, got {other:?}"),
    }
}

/// A ceiling roughly mid-way through the encoding that lies two or more bytes
/// inside a string value, not at a byte the encoder writes alone and not inside
/// an escape run: the middle of the dependency's 64-character digest.
///
/// Tracing: TC-048, FR-038-AC-95
#[trace("TC-048", "FR-038-AC-95")]
#[test]
fn tc_048_a_refusal_inside_a_string_value_records_the_encoders_required_count() {
    let dependencies = [typed_id('b')];
    let preimage = LoweredNodePreimage {
        version: LOWERED_NODE_PREIMAGE,
        node: projection(),
        dependencies: &dependencies,
        bounds: &[],
        claims: &[],
    };
    let length = LOWERED_NODE_TEXT.len() as u64;
    // The encoder counts bytes in the order the preimage writes them, not in
    // canonical member order, so the ceiling is chosen against that count: the
    // one nearest the middle of the encoding at which the refused write is a
    // run of two or more bytes past the limit, which is a string's characters
    // and not a quote, colon, comma or brace written alone. The fixture holds
    // no escape.
    let ceiling = (1..length)
        .filter(|ceiling| required_of(&preimage, *ceiling) >= ceiling + 2)
        .min_by_key(|ceiling| ceiling.abs_diff(length / 2))
        .expect("a ceiling inside a string value");
    assert!(!LOWERED_NODE_TEXT.contains('\\'));
    assert!(
        ceiling > length / 4 && ceiling < length * 3 / 4,
        "mid-way: {ceiling} of {length}"
    );
    let required = identify_node(&preimage, ceiling).expect_err("refuses");
    assert_eq!(required, required_of(&preimage, ceiling));
    assert!(required > ceiling);
    assert_ne!(required, ceiling + 1, "not the invented `limit + 1`");
    assert_ne!(required, length, "not the full length");
    assert!(required < length);
}

/// A preimage built in memory holding an integer past 2^53 (which no admitted
/// package holds, so it skips the reader) is refused with a refusal that is not
/// the canonical-bytes limit: `consumed` is `limit + 1`, saturating at
/// `u64::MAX`.
///
/// Tracing: TC-048, FR-038-AC-95
#[trace("TC-048", "FR-038-AC-95")]
#[test]
fn tc_048_a_refusal_that_is_not_the_byte_limit_records_one_above_the_limit() {
    let mut node = projection();
    node.body = json!({"term": "literal", "value": 9_007_199_254_740_993_i64});
    let preimage = LoweredNodePreimage {
        version: LOWERED_NODE_PREIMAGE,
        node,
        dependencies: &[],
        bounds: &[],
        claims: &[],
    };
    // The encoding reaches the number under this ceiling and is refused for it.
    let ceiling = 1 << 20;
    assert!(matches!(
        quire_canonical::to_vec(&preimage, Limits::new(ceiling)),
        Err(error) if !matches!(error, Error::Limit(_))
    ));
    assert_eq!(identify_node(&preimage, ceiling), Err(ceiling + 1));
    // At `u64::MAX` there is no greater count: `consumed` equals the limit.
    assert_eq!(identify_node(&preimage, u64::MAX), Err(u64::MAX));
}

/// The package step: a ceiling equal to the length admits, one byte under
/// refuses with the byte count the encoder needed.
///
/// Tracing: TC-048, FR-038-AC-95
#[trace("TC-048", "FR-038-AC-95")]
#[test]
fn tc_048_a_package_is_encoded_at_the_ceiling_and_not_one_byte_under_it() {
    let source = source_package_id();
    let preimage = ContractPackagePreimage {
        version: CONTRACT_PACKAGE_VERSION,
        source_package_id: &source,
        lowered: &[],
        dependencies: &[],
    };
    let length = EMPTY_PACKAGE_TEXT.len() as u64;
    let encoded = encode_package(&preimage, length).expect("a ceiling equal to the length");
    assert_eq!(encoded.canonical_bytes.len() as u64, length);
    let required = encode_package(&preimage, length - 1).expect_err("one byte under refuses");
    assert!(required > length - 1);
}

/// An admitted package of three scalar nodes, built without the reader: the
/// lowering reads only the graph, so nothing here needs a valid key. Node `x`
/// carries a long declaration name, so its preimage is longer than the whole
/// package of `y` alone; `z` is typed at `a`, so lowering it reaches `a` as a
/// dependency node.
fn package(bytes: u64) -> CheckedPackageV2 {
    let artifact = |identity: &str| json!({"authority": "agent-ix", "identity": identity});
    let source = json!({
        "authority": "agent-ix",
        "identity": "source",
        "digest_domain": "quire.source.bytes/v1",
        "digest": "1".repeat(64),
    });
    let node = |fill: char, semantic_type: char, declaration: Option<&str>| {
        let mut node = json!({
            "node_id": node_id(fill),
            "schema_version": "quire.checked-semantic-graph/v2",
            "node_tag": "scalar_type",
            "semantic_form": "boolean",
            "semantic_type": node_id(semantic_type),
            "dependencies": [],
            "occurrences": [],
            "body": {"term": "aggregate", "members": []},
        });
        if let Some(name) = declaration {
            node["declaration"] = json!({"qualified_name": [name]});
        }
        node
    };
    let edition = json!({"role": "edition", "definition": artifact("edition")});
    let nodes = json!([
        node('a', 'a', None),
        node('b', 'b', Some(&"x".repeat(2000))),
        node('c', 'c', None),
        node('d', 'a', None),
    ]);
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
            "sources": [source],
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
    let kinds = vec![CheckedNodeKind::ScalarType(ScalarTypeForm::Boolean); 4];
    CheckedPackageV2 { wire, kinds, bytes }
}

fn profile() -> CompleteLoweringProfileV2 {
    CompleteLoweringProfileV2 {
        supported_tags: BTreeSet::from([CheckedNodeTag::ScalarType]),
        require_bounds: false,
        work_limit: u64::MAX,
    }
}

/// `x`, `y` and `z` of [`package`]: a long node, a short one, and one that
/// reaches a dependency node.
fn requested() -> [CheckedNodeId; 3] {
    [typed_id('b'), typed_id('c'), typed_id('d')]
}

fn lowered_ids(records: &[CompleteLoweringRecordV2]) -> Vec<&CheckedNodeId> {
    records
        .iter()
        .filter_map(|record| match record {
            CompleteLoweringRecordV2::Lowered { node } => Some(&node.node.node_id),
            _ => None,
        })
        .collect()
}

/// A lowered package over the ceiling returns nothing of the call: every
/// requested record `failed` for the `bytes` limit with the ceiling as `limit`
/// and a `consumed` above it, no lowered node, no dependency node, no bytes and
/// no id; at the ceiling it lowers. The retained limit is the ceiling `lower`
/// encodes under.
///
/// Tracing: TC-048, FR-038-AC-95
#[trace("TC-048", "FR-038-AC-95")]
#[test]
fn tc_048_a_package_over_the_ceiling_fails_every_record_and_returns_no_package() {
    let whole = package(u64::MAX).lower(&requested(), &profile());
    assert_eq!(lowered_ids(&whole.records).len(), 3);
    assert_eq!(whole.package.lowered().len(), 3);
    assert_eq!(whole.package.dependencies().len(), 1, "z reaches a");
    let length = whole.package.canonical_bytes().expect("bytes").len() as u64;

    // The retained limit equal to the package length lowers it.
    let at = package(length).lower(&requested(), &profile());
    assert_eq!(at, whole);

    // One byte under: every record `failed` for `bytes`, and no package.
    let under = package(length - 1).lower(&requested(), &profile());
    assert_eq!(under.records.len(), 3);
    // `consumed` is the `required` of encoding the same package under that
    // limit, not an invented count.
    let preimage = ContractPackagePreimage {
        version: CONTRACT_PACKAGE_VERSION,
        source_package_id: whole.package.source_package_id(),
        lowered: whole.package.lowered(),
        dependencies: whole.package.dependencies(),
    };
    let required = required_of(&preimage, length - 1);
    assert!(required > length - 1 && required <= length);
    for (record, request) in under.records.iter().zip(requested()) {
        match record {
            CompleteLoweringRecordV2::Failed {
                node_id,
                limit_kind,
                limit,
                consumed,
            } => {
                assert_eq!(*node_id, request);
                assert_eq!(*limit_kind, CheckedPackageLimit::Bytes);
                assert_eq!(*limit, length - 1);
                assert_eq!(*consumed, required);
            }
            other => panic!("expected a failed record, got {other:?}"),
        }
    }
    assert!(under.package.lowered().is_empty());
    assert!(under.package.dependencies().is_empty());
    assert_eq!(under.package.canonical_bytes(), None);
    assert_eq!(under.package.package_id(), None);
    assert_eq!(
        under.package.source_package_id(),
        package(u64::MAX).package_id()
    );
}

/// A node over the ceiling fails only its own record, for `bytes`, and its
/// siblings are the records the same call returns with its request removed.
///
/// Tracing: TC-048, FR-038-AC-95
#[trace("TC-048", "FR-038-AC-95")]
#[test]
fn tc_048_a_node_over_the_ceiling_fails_alone_and_leaves_its_siblings_unchanged() {
    let [x, y, z] = requested();
    // The ceiling the package of `y` and `z` alone needs; `x`'s preimage, with
    // its 2000-byte declaration name, is longer.
    let siblings_only = package(u64::MAX).lower(&[y.clone(), z.clone()], &profile());
    let siblings_length = siblings_only
        .package
        .canonical_bytes()
        .expect("bytes")
        .len() as u64;
    // `x`'s own preimage (it is self-typed, so its closure is empty).
    let packaged = package(u64::MAX);
    let preimage = LoweredNodePreimage {
        version: LOWERED_NODE_PREIMAGE,
        node: (&packaged.wire.semantic_graph.nodes[1]).into(),
        dependencies: &[],
        bounds: &[],
        claims: &[],
    };
    // The smallest ceiling the siblings' package fits under at which the
    // refused write of `x` is a run of two or more bytes past the limit (a
    // string's characters, not a token written alone), so `required` is not
    // `limit + 1`.
    let ceiling = (siblings_length..)
        .find(|ceiling| required_of(&preimage, *ceiling) >= ceiling + 2)
        .expect("a ceiling inside a string value");
    let result = package(ceiling).lower(&[x.clone(), y.clone(), z.clone()], &profile());
    match &result.records[0] {
        CompleteLoweringRecordV2::Failed {
            node_id,
            limit_kind,
            limit,
            consumed,
        } => {
            assert_eq!(*node_id, x);
            assert_eq!(*limit_kind, CheckedPackageLimit::Bytes);
            assert_eq!(*limit, ceiling);
            // `consumed` is the `required` of encoding `x`'s own preimage
            // under that ceiling, not an invented `limit + 1`.
            assert_eq!(*consumed, required_of(&preimage, ceiling));
            assert_ne!(*consumed, ceiling + 1);
            assert!(consumed > limit);
        }
        other => panic!("expected `x` failed, got {other:?}"),
    }
    let without = package(ceiling).lower(&[y.clone(), z.clone()], &profile());
    assert_eq!(result.records[1..], without.records[..]);
    assert_eq!(lowered_ids(&result.records), [&y, &z]);
    assert_eq!(result.package, without.package);
    assert_eq!(result.package, siblings_only.package);
}

/// A work-budget failure still names the `work` limit.
///
/// Tracing: TC-048, FR-038-AC-95
#[trace("TC-048", "FR-038-AC-95")]
#[test]
fn tc_048_a_work_failure_still_names_the_work_limit() {
    let mut tight = profile();
    tight.work_limit = 0;
    let result = package(u64::MAX).lower(&requested()[..1], &tight);
    assert!(matches!(
        &result.records[..],
        [CompleteLoweringRecordV2::Failed {
            limit_kind: CheckedPackageLimit::Work,
            limit: 0,
            consumed: 1,
            ..
        }]
    ));
}
