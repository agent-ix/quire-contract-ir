// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (C) 2026 Agent-IX

//! Published QSpec source-byte integrity vectors, read by reference.

use super::{
    cause_word, code_word, fixtures_dir, proposal_dir_of, read_json,
    read_package_with_sources_override, SourceSupply, DOMAIN_DOCUMENT,
};
use ix_trace_rs::trace;
use quire_contract_model::{
    CheckedArtifactLocator, CheckedPackageReadLimits, CheckedPackageV2ReadResult,
};
use serde_json::Value;
use sha2::{Digest, Sha256};

fn apply_patch(package: &mut Value, patch: &Value) {
    let path = patch["path"].as_str().expect("published patch pointer");
    let (parent, member) = path.rsplit_once('/').expect("published patch parent");
    let target = package.pointer_mut(parent).expect("published patch target");
    let operation = patch["op"].as_str().expect("published patch operation");
    match target {
        Value::Object(object) => match operation {
            "add" | "replace" => {
                object.insert(member.to_owned(), patch["value"].clone());
            }
            "remove" => {
                object.remove(member).expect("published removal target");
            }
            _ => panic!("unsupported published operation {operation}"),
        },
        Value::Array(items) => {
            let index: usize = member.parse().expect("published array index");
            match operation {
                "add" => items.insert(index, patch["value"].clone()),
                "replace" => items[index] = patch["value"].clone(),
                "remove" => {
                    items.remove(index);
                }
                _ => panic!("unsupported published operation {operation}"),
            }
        }
        _ => panic!("published patch parent is not a container: {parent}"),
    }
}

#[trace(
    "TC-233",
    "FR-038-AC-437",
    "FR-038-AC-438",
    "FR-038-AC-439",
    "FR-038-AC-440",
    "FR-038-AC-441",
    "FR-038-AC-442",
    "FR-019-AC-25"
)]
#[test]
fn published_source_integrity_vectors_follow_raw_supplied_bytes() {
    let fixtures = fixtures_dir();
    let proposal = proposal_dir_of(&fixtures);
    let vectors = read_json(&proposal.join("source-integrity-vectors.json"))
        .expect("published source integrity vectors");
    let base = vectors["base"].as_str().expect("published base path");
    assert_eq!(base, "fixtures/positive-two-owners-a.json");
    let base = read_json(&proposal.join(base)).expect("published base package");
    let document = proposal.join(DOMAIN_DOCUMENT);
    let positives = vectors["positive"].as_array().expect("positive vectors");
    let refusals = vectors["refusals"].as_array().expect("refusal vectors");
    assert_eq!((positives.len(), refusals.len()), (2, 16));
    for row in positives.iter().chain(refusals) {
        let id = row["id"].as_str().expect("vector id");
        let mut package = base.clone();
        for patch in row["patch"].as_array().expect("published patches") {
            apply_patch(&mut package, patch);
        }
        let supplied = row["supplied"].as_array().expect("supplied source pairs");
        let result = read_package_with_sources_override(
            &package,
            id,
            &document,
            true,
            CheckedPackageReadLimits::bounded(),
            None,
            SourceSupply {
                selected: Some(supplied),
                flip_byte: row.get("flip_byte"),
                ..SourceSupply::default()
            },
        )
        .expect("production reader with published evidence");
        if row["expected"]["admitted"] == true {
            assert!(
                matches!(result, CheckedPackageV2ReadResult::Admitted(_)),
                "{id}: {result:?}"
            );
            continue;
        }
        let CheckedPackageV2ReadResult::Refused(refusal) = result else {
            panic!("{id}: expected refusal, got {result:?}");
        };
        assert_eq!(
            code_word(refusal.code),
            row["expected"]["code"],
            "{id}: code"
        );
        assert_eq!(
            refusal.cause.map(cause_word),
            row["expected"]["cause"].as_str(),
            "{id}: cause"
        );
        assert_eq!(
            refusal.loci[0]
                .path
                .as_ref()
                .expect("refusal pointer")
                .to_string(),
            row["expected"]["locus"],
            "{id}: pointer"
        );
        let expected_pair = if row["expected"]["cause"] == "source-integrity" {
            let pointer = row["expected"]["locus"].as_str().expect("source pointer");
            let source = package
                .pointer(pointer)
                .expect("source at expected pointer");
            Some(CheckedArtifactLocator {
                authority: source["authority"]
                    .as_str()
                    .expect("source authority")
                    .into(),
                identity: source["identity"].as_str().expect("source identity").into(),
            })
        } else {
            None
        };
        assert_eq!(refusal.source_identity, expected_pair, "{id}: source pair");
    }
}

#[trace("TC-233", "FR-038-AC-439")]
#[test]
fn paired_source_faults_follow_site_then_array_order() {
    let fixtures = fixtures_dir();
    let proposal = proposal_dir_of(&fixtures);
    let vectors = read_json(&proposal.join("source-integrity-vectors.json"))
        .expect("published source integrity vectors");
    let base = read_json(&proposal.join(vectors["base"].as_str().expect("base")))
        .expect("published base package");
    let rows = vectors["refusals"].as_array().expect("refusal vectors");
    let patch_of = |id: &str| {
        &rows
            .iter()
            .find(|row| row["id"] == id)
            .expect("named published vector")["patch"]
    };
    let read = |package: &Value| {
        let document = proposal.join(DOMAIN_DOCUMENT);
        read_package_with_sources_override(
            package,
            "paired source faults",
            &document,
            true,
            CheckedPackageReadLimits::bounded(),
            None,
            SourceSupply::default(),
        )
        .expect("production read")
    };
    for (ids, pointer) in [
        (["SI-R9", "SI-R10"], "/source_map/0/regions/0/source"),
        (
            ["SI-R10", "SI-R11"],
            "/refinement_step_rows/0/region/source",
        ),
    ] {
        let mut package = base.clone();
        for id in ids {
            for patch in patch_of(id).as_array().expect("published patches") {
                apply_patch(&mut package, patch);
            }
        }
        let result = read(&package);
        let CheckedPackageV2ReadResult::Refused(refusal) = result else {
            panic!("{ids:?}: expected refusal, got {result:?}");
        };
        assert_eq!(code_word(refusal.code), "invalid_package", "{ids:?}");
        assert_eq!(
            refusal.cause.map(cause_word),
            Some("source-integrity"),
            "{ids:?}"
        );
        assert_eq!(
            refusal.loci[0].path.as_ref().unwrap().as_str(),
            pointer,
            "{ids:?}"
        );
        let source = package.pointer(pointer).expect("failing source reference");
        assert_eq!(
            refusal.source_identity,
            Some(CheckedArtifactLocator {
                authority: source["authority"].as_str().unwrap().into(),
                identity: source["identity"].as_str().unwrap().into(),
            }),
            "{ids:?}: source pair"
        );
    }

    let mut package = base;
    for patch in patch_of("SI-R11").as_array().expect("published patches") {
        apply_patch(&mut package, patch);
    }
    let valid = package["lock"]["sources"][0].clone();
    let bad = package["diagnostics"]["entries"][0]["loci"][0].clone();
    package["diagnostics"]["entries"][0]["loci"] =
        serde_json::json!([{"source": valid, "start": 0, "end": 1}, bad]);
    let result = read(&package);
    let CheckedPackageV2ReadResult::Refused(refusal) = result else {
        panic!("diagnostic locus order: expected refusal, got {result:?}");
    };
    assert_eq!(refusal.cause.map(cause_word), Some("source-integrity"));
    assert_eq!(
        refusal.loci[0].path.as_ref().unwrap().as_str(),
        "/diagnostics/entries/0/loci/1/source"
    );
    assert_eq!(
        refusal.source_identity,
        Some(CheckedArtifactLocator {
            authority: "agent-ix".into(),
            identity: "example-a".into(),
        })
    );

    let mut later = package["diagnostics"]["entries"][0].clone();
    later["loci"] = serde_json::json!([package["diagnostics"]["entries"][0]["loci"][1].clone()]);
    package["diagnostics"]["entries"]
        .as_array_mut()
        .expect("diagnostic entries")
        .push(later);
    let result = read(&package);
    let CheckedPackageV2ReadResult::Refused(refusal) = result else {
        panic!("diagnostic entry order: expected refusal, got {result:?}");
    };
    assert_eq!(code_word(refusal.code), "invalid_package");
    assert_eq!(refusal.cause.map(cause_word), Some("source-integrity"));
    assert_eq!(
        refusal.loci[0].path.as_ref().unwrap().as_str(),
        "/diagnostics/entries/0/loci/1/source"
    );
    assert_eq!(
        refusal.source_identity,
        Some(CheckedArtifactLocator {
            authority: "agent-ix".into(),
            identity: "example-a".into(),
        })
    );
}

fn set_source_claims(value: &mut Value, claim: &str) {
    match value {
        Value::Object(object) => {
            if let Some(member) = object.get_mut("source_integrity") {
                *member = Value::String(claim.to_owned());
            }
            for child in object.values_mut() {
                set_source_claims(child, claim);
            }
        }
        Value::Array(items) => {
            for child in items {
                set_source_claims(child, claim);
            }
        }
        _ => {}
    }
}

#[trace("TC-233", "FR-038-AC-441")]
#[test]
fn source_byte_and_visit_limits_charge_references_exactly() {
    let fixtures = fixtures_dir();
    let proposal = proposal_dir_of(&fixtures);
    let mut package = read_json(&fixtures.join("positive-two-owners-a.json"))
        .expect("published positive package");
    let source_len = serde_json::to_vec(&package).unwrap().len();
    let source = vec![b'x'; source_len];
    set_source_claims(&mut package, &format!("{:x}", Sha256::digest(&source)));
    assert_eq!(serde_json::to_vec(&package).unwrap().len(), source_len);
    let document = proposal.join(DOMAIN_DOCUMENT);
    let read = |limits, bytes: &[u8]| {
        read_package_with_sources_override(
            &package,
            "source budget",
            &document,
            true,
            limits,
            None,
            SourceSupply {
                override_bytes: Some(("agent-ix", "example-a", bytes)),
                ..SourceSupply::default()
            },
        )
        .expect("production read")
    };
    let mut limits = CheckedPackageReadLimits::bounded();
    limits.bytes = u64::try_from(source_len).unwrap();
    assert!(matches!(
        read(limits, &source),
        CheckedPackageV2ReadResult::Admitted(_)
    ));
    let mut over = source.clone();
    over.push(b'x');
    let CheckedPackageV2ReadResult::Incomplete(stop) = read(limits, &over) else {
        panic!("first source byte past ceiling must be incomplete");
    };
    assert_eq!(
        stop.limit_kind,
        quire_contract_model::CheckedPackageLimit::Bytes
    );
    assert_eq!(stop.consumed, limits.bytes + 1);
    assert_eq!(stop.path, None);
    limits.work = 1;
    let CheckedPackageV2ReadResult::Incomplete(stop) = read(limits, &source) else {
        panic!("second source reference must exceed one visit");
    };
    assert_eq!(
        stop.limit_kind,
        quire_contract_model::CheckedPackageLimit::Work
    );
    assert_eq!(
        stop.path.as_ref().map(|path| path.as_str()),
        Some("/source_map/0/regions/0/source")
    );
}

#[trace("TC-233", "FR-038-AC-439")]
#[test]
fn strict_and_canonical_wire_checks_precede_integrity() {
    let package =
        read_json(&fixtures_dir().join("positive-two-owners-a.json")).expect("published package");
    let pretty = serde_json::to_vec_pretty(&package).unwrap();
    let read = |bytes: &[u8]| {
        quire_contract_model::CheckedPackageV2::read(
            bytes,
            CheckedPackageReadLimits::bounded(),
            &quire_contract_model::CheckedPackageEvidence::new(),
        )
    };
    let CheckedPackageV2ReadResult::Refused(refusal) = read(&pretty) else {
        panic!("noncanonical bytes must refuse before source lookup");
    };
    assert_ne!(refusal.cause.map(cause_word), Some("source-integrity"));
    let canonical = serde_json::to_string(&package).unwrap();
    let duplicate = canonical.replacen('{', "{\"name\":\"duplicate\",", 1);
    let CheckedPackageV2ReadResult::Refused(refusal) = read(duplicate.as_bytes()) else {
        panic!("duplicate JSON member must refuse before source lookup");
    };
    assert_ne!(refusal.cause.map(cause_word), Some("source-integrity"));
}

#[trace("TC-233", "FR-038-AC-437")]
#[test]
fn converting_a_published_crlf_source_changes_its_integrity() {
    let fixtures = fixtures_dir();
    let proposal = proposal_dir_of(&fixtures);
    let package = read_json(&fixtures.join("positive-two-owners-a.json"))
        .expect("published positive package");
    let original = std::fs::read(proposal.join("source-documents/example-a.qsl"))
        .expect("published source bytes");
    assert!(original.windows(2).any(|window| window == b"\r\n"));
    let converted = original
        .windows(2)
        .position(|window| window == b"\r\n")
        .map(|position| {
            let mut bytes = original.clone();
            bytes.remove(position);
            bytes
        })
        .expect("published CRLF");
    let result = read_package_with_sources_override(
        &package,
        "changed line ending",
        &proposal.join(DOMAIN_DOCUMENT),
        true,
        CheckedPackageReadLimits::bounded(),
        None,
        SourceSupply {
            override_bytes: Some(("agent-ix", "example-a", &converted)),
            ..SourceSupply::default()
        },
    )
    .expect("production read");
    let CheckedPackageV2ReadResult::Refused(refusal) = result else {
        panic!("line-ending conversion must refuse: {result:?}");
    };
    assert_eq!(refusal.cause.map(cause_word), Some("source-integrity"));
    assert_eq!(
        refusal.loci[0].path.as_ref().unwrap().as_str(),
        "/lock/sources/0"
    );
}
