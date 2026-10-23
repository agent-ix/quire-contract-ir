use super::*;
use crate::checked_package::shared::CheckedPackageLimit;
use ix_trace_rs::trace;
use quire_canonical::LimitKind;
use serde_json::json;
use sha2::{Digest, Sha256};

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

/// The lowered node preimage encodes to expected bytes written out above.
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
}

/// The seam of FR-038-AC-95's node step: a ceiling equal to the length of
/// the preimage's canonical bytes identifies the node, one byte lower does
/// not and reports the byte count the encoder needed, which is above the
/// ceiling.
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
        node('b', 'b', Some(&"x".repeat(20_000))),
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
    let mut package = CheckedPackageV2 {
        node_index: super::super::retained_node_index(&wire),
        source_index: super::super::retained_source_index(&wire),
        lowering_facts: Vec::new(),
        wire,
        terms: Vec::new(),
        kinds,
        bytes,
        models: Default::default(),
        direct_dependencies: Vec::new(),
        closure_index: Default::default(),
    };
    let index = package
        .graph()
        .nodes
        .iter()
        .enumerate()
        .map(|(position, node)| (&node.node_id, position))
        .collect();
    package.lowering_facts = package.scan_lowering_facts(&index);
    package
}

fn profile() -> CompleteLoweringProfileV2 {
    CompleteLoweringProfileV2 {
        reference: super::LoweringProfileRef {
            authority: "quire".into(),
            identity: "complete-v1".into(),
        },
        supported_tags: BTreeSet::from([CheckedNodeTag::ScalarType]),
        require_bounds: false,
        work_limit: u64::MAX,
    }
}

fn shared_closure_package(roots: usize, shared: usize) -> (CheckedPackageV2, Vec<CheckedNodeId>) {
    let mut package = package(u64::MAX);
    let id = |number: usize| {
        json!({
            "domain": "quire.checked-semantic-node/v1",
            "digest": format!("{number:064x}"),
        })
    };
    let nodes = (0..shared + roots)
        .map(|position| {
            let parent = if position == 0 {
                0
            } else if position < shared {
                position - 1
            } else {
                shared - 1
            };
            json!({
                "node_id": id(position),
                "schema_version": "quire.checked-semantic-graph/v2",
                "node_tag": "scalar_type",
                "semantic_form": "boolean",
                "semantic_type": id(parent),
                "dependencies": [],
                "occurrences": [],
                "body": {"term": "aggregate", "members": []},
            })
        })
        .collect::<Vec<_>>();
    package.wire.semantic_graph.nodes = serde_json::from_value(json!(nodes)).expect("nodes");
    package.kinds = vec![CheckedNodeKind::ScalarType(ScalarTypeForm::Boolean); shared + roots];
    BODY_WALKS.with(|walks| walks.set(0));
    let index = package
        .graph()
        .nodes
        .iter()
        .enumerate()
        .map(|(position, node)| (&node.node_id, position))
        .collect();
    package.lowering_facts = package.scan_lowering_facts(&index);
    let requests = (shared..shared + roots)
        .map(|position| serde_json::from_value(id(position)).expect("request"))
        .collect();
    (package, requests)
}

/// The same shared closure is parsed once per call, independent of the number
/// of requested leaves. Each request's work meter still charges its full closure.
///
/// Tracing: FR-038-AC-308
#[trace("TC-050", "FR-038-AC-308")]
#[test]
fn shared_closure_body_walks_scale_with_nodes_not_requests() {
    let mut measured = Vec::new();
    for roots in [2, 4, 8] {
        for shared in [4, 8, 16] {
            let (package, requests) = shared_closure_package(roots, shared);
            SHARED_CLOSURE_BUILDS.with(|builds| builds.set(0));
            let admission_walks = BODY_WALKS.with(std::cell::Cell::get);
            let start = std::time::Instant::now();
            let lowered = package.lower(&requests, &profile());
            let lower_elapsed_ns = start.elapsed().as_nanos();
            assert_eq!(lowered.records.len(), roots);
            let walks = BODY_WALKS.with(std::cell::Cell::get);
            let closure_builds = SHARED_CLOSURE_BUILDS.with(std::cell::Cell::get);
            assert_eq!(closure_builds, 1, "one shared SCC closure per call");
            println!(
                "{}",
                json!({
                    "stage": "after_retained_facts_and_lower",
                    "K": roots,
                    "R": shared,
                    "lowering_fact_scan_body_validations": admission_walks,
                    "additional_lower_body_validations": walks - admission_walks,
                    "shared_closure_builds": closure_builds,
                    "lower_elapsed_ns": lower_elapsed_ns.to_string(),
                })
            );
            for (request, batch_record) in requests.iter().zip(&lowered.records) {
                let single = package.lower(std::slice::from_ref(request), &profile());
                assert_eq!(batch_record, &single.records[0]);
            }
            let mut stopped = profile();
            stopped.work_limit = 4;
            let batch_stopped = package.lower(&requests, &stopped);
            for (request, batch_record) in requests.iter().zip(&batch_stopped.records) {
                let single = package.lower(std::slice::from_ref(request), &stopped);
                assert_eq!(batch_record, &single.records[0]);
            }
            measured.push((roots, shared, walks));
        }
    }
    for (roots, shared, walks) in measured {
        assert_eq!(walks, shared + roots, "roots={roots}, shared={shared}");
    }
}

/// Independent record lowering keeps one shared closure calculation while preserving each
/// request's one-item result.
#[trace("TC-050", "FR-038-AC-309")]
#[test]
fn independently_lowered_records_share_one_closure_and_equal_single_requests() {
    let (package, requests) = shared_closure_package(4, 8);
    BODY_WALKS.with(|walks| walks.set(0));
    SHARED_CLOSURE_BUILDS.with(|builds| builds.set(0));
    LOWERING_GRAPH_INDEX_BUILDS.with(|builds| builds.set(0));
    LOWERING_SOURCE_INDEX_BUILDS.with(|builds| builds.set(0));
    let records = package.lower_independent_records(&requests, &profile());
    assert_eq!(records.len(), requests.len());
    assert_eq!(
        BODY_WALKS.with(std::cell::Cell::get),
        0,
        "admission retained body facts"
    );
    assert_eq!(SHARED_CLOSURE_BUILDS.with(std::cell::Cell::get), 1);
    assert_eq!(LOWERING_GRAPH_INDEX_BUILDS.with(std::cell::Cell::get), 1);
    assert_eq!(LOWERING_SOURCE_INDEX_BUILDS.with(std::cell::Cell::get), 1);
    for (request, record) in requests.iter().zip(records) {
        let single = package.lower(std::slice::from_ref(request), &profile());
        assert_eq!(record, single.records[0]);
    }
}

/// The grouped package may exceed the retained byte ceiling even when each one-item
/// package fits; independent records keep their separate byte outcomes.
#[trace("TC-048", "FR-038-AC-309")]
#[test]
fn independently_lowered_records_do_not_inherit_a_union_package_byte_failure() {
    let requests = requested();
    let unlimited = package(u64::MAX);
    let single_max = requests
        .iter()
        .map(|request| {
            unlimited
                .lower(std::slice::from_ref(request), &profile())
                .package
                .canonical_bytes()
                .expect("one-item package bytes")
                .len() as u64
        })
        .max()
        .expect("requested items");
    let union_len = unlimited
        .lower(&requests, &profile())
        .package
        .canonical_bytes()
        .expect("union package bytes")
        .len() as u64;
    assert!(union_len > single_max);

    let limited = package(single_max);
    let grouped = limited.lower(&requests, &profile());
    assert!(grouped.records.iter().all(|record| matches!(
        record,
        CompleteLoweringRecordV2::Failed {
            limit_kind: CheckedPackageLimit::Bytes,
            ..
        }
    )));
    let expected = requests
        .iter()
        .map(|request| {
            limited
                .lower(std::slice::from_ref(request), &profile())
                .records
                .into_iter()
                .next()
                .expect("one record")
        })
        .collect::<Vec<_>>();
    let actual = limited.lower_independent_records(&requests, &profile());
    assert_eq!(actual, expected);
    assert!(actual
        .iter()
        .any(|record| matches!(record, CompleteLoweringRecordV2::Lowered { .. })));
}

/// Tracing: FR-038-AC-308
#[trace("TC-050", "FR-038-AC-308")]
#[test]
fn grouped_recursive_shared_closure_keeps_independent_records() {
    let (mut package, requests) = shared_closure_package(3, 4);
    package.wire.semantic_graph.nodes[0].semantic_type =
        serde_json::from_value(serde_json::json!({
            "domain": "quire.checked-semantic-node/v1",
            "digest": format!("{:064x}", 1),
        }))
        .expect("cycle target");
    package.wire.semantic_graph.nodes[0].recursion_group = Some("shared".into());
    package.wire.semantic_graph.nodes[1].recursion_group = Some("shared".into());
    let index = package
        .graph()
        .nodes
        .iter()
        .enumerate()
        .map(|(position, node)| (&node.node_id, position))
        .collect();
    package.lowering_facts = package.scan_lowering_facts(&index);
    assert_eq!(
        package.lowering_facts[0].component,
        package.lowering_facts[1].component
    );
    let batch = package.lower(&requests, &profile());
    assert_eq!(batch.records.len(), 3);
    for (request, record) in requests.iter().zip(&batch.records) {
        let single = package.lower(std::slice::from_ref(request), &profile());
        assert_eq!(record, &single.records[0]);
    }
    let mut stopped = profile();
    stopped.work_limit = 4;
    let batch_stopped = package.lower(&requests, &stopped);
    for (request, record) in requests.iter().zip(&batch_stopped.records) {
        let single = package.lower(std::slice::from_ref(request), &stopped);
        assert_eq!(record, &single.records[0]);
    }
}

#[trace("TC-050", "FR-038-AC-308")]
#[test]
fn lowering_components_exclude_admission_only_extension_edges() {
    let mut facts = vec![
        LoweringNodeFacts {
            terms: Ok(1),
            successors: vec![(0, EdgeRole::Typing)],
            component: usize::MAX,
        },
        LoweringNodeFacts {
            terms: Ok(1),
            successors: vec![(0, EdgeRole::Typing)],
            component: usize::MAX,
        },
    ];
    // Admission may also see 0 -> 1 as an extension target. Lowering omits
    // it, so a request for 0 must not inherit 1 through an admission SCC.
    assign_components(&mut facts);
    assert_ne!(facts[0].component, facts[1].component);
    let mut closure = SccClosureIndex::new(&facts);
    assert_eq!(closure.reachable(facts[0].component), BTreeSet::from([0]));
    assert_eq!(
        closure.reachable(facts[1].component),
        BTreeSet::from([0, 1])
    );
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

/// The emitter's graph input order is not part of semantic identity: keyed
/// nodes are ordered by their identities before the four-member projection is
/// hashed. Request order remains fixed in the accounting records.
#[trace("TC-047", "FR-035-AC-5")]
#[test]
fn tc_047_equivalent_graph_construction_orders_have_one_package_identity() {
    let source = package(u64::MAX);
    let selected = profile();
    let result = source.lower(&requested(), &selected);
    let source_maps = super::NodeSourceMaps::new(&source.wire.source_map);
    let encode = |nodes: &[CompleteContractNodeV2]| {
        super::super::contract_package::ContractPackageWireV1::from_lowering(
            &source,
            &selected,
            &result.records,
            nodes,
            result.package.dependencies(),
            &source_maps,
            u64::MAX,
        )
        .expect("package encodes")
    };
    let original = result.package.lowered();
    let mut reversed = original.to_vec();
    reversed.reverse();
    assert_ne!(original, reversed, "the graph inputs differ in order");
    let (first, first_bytes) = encode(original);
    let (second, second_bytes) = encode(&reversed);
    assert_eq!(first.identity(), second.identity());
    assert_eq!(first_bytes, second_bytes);
}

/// A lowered package over the ceiling returns nothing of the call: every
/// requested record `failed` for the `bytes` limit with the ceiling as `limit`
/// and a `consumed` above it, no lowered node, no dependency node, no bytes and
/// no id; at the ceiling it lowers. The retained limit is the ceiling `lower`
/// encodes under.
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
    // The encoder reports the first failed byte charge.
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
                assert!(*consumed > length - 1 && *consumed <= length);
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
#[trace("TC-048", "FR-038-AC-95")]
#[test]
fn tc_048_a_node_over_the_ceiling_fails_alone_and_leaves_its_siblings_unchanged() {
    let [x, y, z] = requested();
    // The ceiling the package of `y` and `z` alone needs; `x`'s preimage, with
    // its long declaration name, is longer.
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
    // Reserve room for `x`'s failed accounting record in the package. The
    // refused write of `x` is still a run of characters past the limit, not
    // one token byte, so `required` is not `limit + 1`.
    assert!(siblings_length < 10_000);
    let ceiling = (10_000..)
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
    assert_eq!(result.package.lowered(), without.package.lowered());
    assert_eq!(
        result.package.dependencies(),
        without.package.dependencies()
    );
    assert_eq!(without.package, siblings_only.package);
    // The failed `x` record belongs to this call's six-field wire and thus
    // changes its accounting and semantic identity without changing siblings.
    assert_ne!(result.package.package_id(), without.package.package_id());
}

/// A work-budget failure still names the `work` limit.
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
