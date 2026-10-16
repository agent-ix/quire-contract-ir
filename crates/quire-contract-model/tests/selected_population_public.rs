// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (C) 2026 Agent-IX

use ix_trace_rs::trace;
use quire_contract_model::{
    selected_population::{
        select_population, CheckedSelectedPopulationOutcome as Outcome,
        CheckedSelectedPopulationSelector as Selector, CheckedSelectedPopulationSource as Source,
    },
    CheckedNodeId, CheckedOccurrence, CheckedOccurrenceRole, CheckedPackageEvidence,
    CheckedPackageReadLimits, CheckedPackageV2, CheckedPackageV2ReadResult, CheckedSemanticId,
    CheckedSourceRef, CheckedSourceRegion,
};
use serde_json::Value;
use std::{
    fs,
    path::{Path, PathBuf},
    process::Command,
    sync::atomic::{AtomicU64, Ordering},
};

const COMPLETE_VALUE_FEATURE: &str = "quire.value.complete/v1";
const NODE_DOMAIN: &str = "quire.checked-semantic-node/v1";
const PACKAGE_DOMAIN: &str = "quire.package.semantic/v2";
static NEXT_SCRATCH_ID: AtomicU64 = AtomicU64::new(0);

struct Scratch(PathBuf);

impl Drop for Scratch {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.0);
    }
}

#[derive(Clone, Copy)]
struct FixtureCase {
    file: &'static str,
    expected: &'static [u64],
}

const CASES: &[FixtureCase] = &[
    FixtureCase {
        file: "population-0.json",
        expected: &[0],
    },
    FixtureCase {
        file: "population-1.json",
        expected: &[1],
    },
    FixtureCase {
        file: "population-3.json",
        expected: &[3],
    },
    FixtureCase {
        file: "population-2-5.json",
        expected: &[2, 5],
    },
];

fn run_producer() -> Scratch {
    let qsl = std::env::var_os("QUIRE_SPEC_LANGUAGE_DIR")
        .map(PathBuf::from)
        .expect("selected-population conformance requires QUIRE_SPEC_LANGUAGE_DIR");
    let scratch_path = std::env::temp_dir().join(format!(
        "ir760-population-{}-{}",
        std::process::id(),
        NEXT_SCRATCH_ID.fetch_add(1, Ordering::Relaxed)
    ));
    let _ = fs::remove_dir_all(&scratch_path);
    fs::create_dir(&scratch_path).expect("create owned QSL producer scratch");
    let output = scratch_path.join("emitted");
    let target = std::env::var_os("QUIRE_SPEC_LANGUAGE_TARGET_DIR")
        .map(PathBuf::from)
        .unwrap_or_else(|| Path::new(env!("CARGO_MANIFEST_DIR")).join("../../target/qsl-producer"));
    let result = Command::new("cargo")
        .arg("run")
        .arg("--locked")
        .arg("--manifest-path")
        .arg(qsl.join("Cargo.toml"))
        .arg("--example")
        .arg("population_v2_fixtures")
        .arg("--features")
        .arg("population-fixtures")
        .arg("--")
        .arg(&output)
        .env("CARGO_TARGET_DIR", target)
        .output()
        .expect("run the QSL population fixture producer");
    assert!(
        result.status.success(),
        "QSL population fixture producer failed: {}",
        String::from_utf8_lossy(&result.stderr)
    );
    Scratch(scratch_path)
}

fn canonical(value: &Value) -> Vec<u8> {
    quire_canonical::to_vec(value, quire_canonical::Limits::new(1 << 30))
        .expect("QSL producer output is within canonical encoding limits")
}

fn read_package(path: &Path, domain_package: &[u8]) -> CheckedPackageV2 {
    read_package_with_limits(path, domain_package, CheckedPackageReadLimits::bounded())
}

fn read_package_with_limits(
    path: &Path,
    domain_package: &[u8],
    limits: CheckedPackageReadLimits,
) -> CheckedPackageV2 {
    let bytes = fs::read(path).expect("QSL producer emitted the case");
    let package: Value = serde_json::from_slice(&bytes).expect("QSL package JSON");
    let domain: Value = serde_json::from_slice(domain_package).expect("QSL domain package JSON");
    let domain_bytes = canonical(&domain);
    let mut evidence = CheckedPackageEvidence::new();
    for selection in package["lock"]["model_selections"]
        .as_array()
        .expect("QSL model selections")
    {
        let digest = selection["digest"].as_str().expect("selected model digest");
        evidence
            .insert_domain_package_document(digest, domain_bytes.clone())
            .expect("QSL domain model selection matches producer document");
    }
    evidence.support_feature(COMPLETE_VALUE_FEATURE);
    match CheckedPackageV2::read(&bytes, limits, &evidence) {
        CheckedPackageV2ReadResult::Admitted(package) => *package,
        other => panic!("QSL-emitted package must be admitted: {other:?}"),
    }
}

fn selector(
    package: &CheckedPackageV2,
    node: &quire_contract_model::CheckedSemanticNodeV2,
) -> Selector {
    let occurrence = node
        .occurrences
        .iter()
        .find(|occurrence| occurrence.role == CheckedOccurrenceRole::Type)
        .expect("population node has a Type occurrence")
        .clone();
    Selector {
        package_id: package.package_id().clone(),
        binding: node.node_id.clone(),
        occurrence,
    }
}

fn population_nodes(
    package: &CheckedPackageV2,
) -> Vec<&quire_contract_model::CheckedSemanticNodeV2> {
    package
        .graph()
        .nodes
        .iter()
        .filter(|node| {
            node.node_tag.as_ref() == "bounded_domain"
                && node.semantic_form.as_ref() == "model_population"
        })
        .collect()
}

fn assert_no_extent(outcome: Outcome, expected: impl FnOnce(&Outcome) -> bool) {
    assert!(expected(&outcome), "unexpected typed outcome: {outcome:?}");
    assert!(
        !matches!(outcome, Outcome::Finite(_)),
        "non-finite selection unexpectedly carried an extent: {outcome:?}"
    );
}

#[trace("FR-053-AC-3", "FR-053-AC-4", "FR-053-AC-5", "TC-453")]
#[test]
fn selected_population_is_bound_to_the_emitted_node_and_source_occurrence() {
    let scratch = run_producer();
    let domain_path = scratch.0.join("emitted/spine-model.semantic-ir.json");
    let domain = fs::read(&domain_path).expect("QSL producer wrote domain evidence");
    let mut paired = None;

    for case in CASES {
        let package = read_package(&scratch.0.join("emitted").join(case.file), &domain);
        let nodes = population_nodes(&package);
        assert_eq!(nodes.len(), case.expected.len(), "{}", case.file);
        let mut found = Vec::new();
        for node in nodes {
            let selected = match select_population(Source::Admitted {
                package: &package,
                selector: selector(&package, node),
            }) {
                Outcome::Finite(selected) => selected,
                other => panic!("exact QSL population selection must be finite: {other:?}"),
            };
            assert_eq!(selected.package_id, *package.package_id());
            assert_eq!(selected.binding, node.node_id);
            assert_eq!(selected.minimum, 0);
            assert_eq!(
                selected.provenance,
                package
                    .source_map()
                    .iter()
                    .find(|entry| {
                        entry.node_id == node.node_id
                            && entry.role == CheckedOccurrenceRole::Type
                            && entry.ordinal == selected.occurrence.ordinal
                    })
                    .expect("exact node-occurrence source map")
                    .regions
            );
            assert!(package.model_object_fields(&selected.object_type).is_ok());
            found.push(selected);
        }
        let mut actual: Vec<_> = found.iter().map(|population| population.maximum).collect();
        actual.sort_unstable();
        assert_eq!(actual, case.expected, "{}", case.file);
        if case.file == "population-2-5.json" {
            paired = Some(found);
        }
    }

    let mut paired = paired.expect("two-binding QSL fixture");
    paired.sort_by_key(|population| population.maximum);
    assert_eq!(paired.len(), 2);
    assert_eq!(
        paired
            .iter()
            .map(|population| population.maximum)
            .collect::<Vec<_>>(),
        vec![2, 5]
    );
    assert_ne!(paired[0].binding, paired[1].binding);
    assert_eq!(paired[0].object_type, paired[1].object_type);
    assert_ne!(
        (paired[0].binding.clone(), paired[0].occurrence.clone()),
        (paired[1].binding.clone(), paired[1].occurrence.clone())
    );

    let package = read_package(&scratch.0.join("emitted/population-2-5.json"), &domain);
    let mut raised_limits = CheckedPackageReadLimits::bounded();
    raised_limits.nodes *= 2;
    raised_limits.edges *= 2;
    raised_limits.occurrences *= 2;
    raised_limits.work *= 2;
    let raised = read_package_with_limits(
        &scratch.0.join("emitted/population-2-5.json"),
        &domain,
        raised_limits,
    );
    let original_finite = match select_population(Source::Admitted {
        package: &package,
        selector: selector(&package, population_nodes(&package)[0]),
    }) {
        Outcome::Finite(selected) => selected,
        other => panic!("original-limit selection must be finite: {other:?}"),
    };
    let raised_finite = match select_population(Source::Admitted {
        package: &raised,
        selector: selector(&raised, population_nodes(&raised)[0]),
    }) {
        Outcome::Finite(selected) => selected,
        other => panic!("raised-limit selection must be finite: {other:?}"),
    };
    assert_eq!(original_finite.maximum, raised_finite.maximum);
    assert_eq!(original_finite.object_type, raised_finite.object_type);
    assert_eq!(original_finite.provenance, raised_finite.provenance);
    assert_no_extent(select_population(Source::Unavailable), |outcome| {
        matches!(outcome, Outcome::Unavailable)
    });
    let population = population_nodes(&package)[0];
    let original = selector(&package, population);
    let mut foreign = original.clone();
    foreign.package_id.domain = "foreign".into();
    assert_no_extent(
        select_population(Source::Admitted {
            package: &package,
            selector: foreign,
        }),
        |outcome| matches!(outcome, Outcome::ForeignPackage(_)),
    );
    let mut stale = original.clone();
    stale.binding = CheckedNodeId {
        domain: NODE_DOMAIN.into(),
        digest: "f".repeat(64).into(),
    };
    assert_no_extent(
        select_population(Source::Admitted {
            package: &package,
            selector: stale,
        }),
        |outcome| matches!(outcome, Outcome::StaleBinding(_)),
    );
    let mut missing = original.clone();
    missing.occurrence = CheckedOccurrence {
        role: CheckedOccurrenceRole::Type,
        ordinal: missing.occurrence.ordinal + 999,
    };
    assert_no_extent(
        select_population(Source::Admitted {
            package: &package,
            selector: missing,
        }),
        |outcome| matches!(outcome, Outcome::MissingOccurrence(_)),
    );
    let non_population = package
        .graph()
        .nodes
        .iter()
        .find(|node| {
            node.node_tag.as_ref() != "bounded_domain"
                && node.occurrences.iter().any(|occurrence| {
                    occurrence.role == CheckedOccurrenceRole::Type
                        && package.source_map().iter().any(|entry| {
                            entry.node_id == node.node_id
                                && entry.role == CheckedOccurrenceRole::Type
                                && entry.ordinal == occurrence.ordinal
                        })
                })
        })
        .expect("QSL package contains a source-mapped non-population Type node");
    assert_no_extent(
        select_population(Source::Admitted {
            package: &package,
            selector: selector(&package, non_population),
        }),
        |outcome| matches!(outcome, Outcome::NotPopulation(_)),
    );
    let selected_model = CheckedSemanticId {
        domain: PACKAGE_DOMAIN.into(),
        algorithm: "sha256".into(),
        digest: "e".repeat(64).into(),
    };
    let population_id = population.node_id.clone();
    assert_no_extent(
        select_population(Source::Declaration {
            selected_model: selected_model.clone(),
            population: population_id.clone(),
        }),
        |outcome| matches!(outcome, Outcome::Unbounded { .. }),
    );
    let upstream_source = CheckedSourceRef {
        authority: "qsl".into(),
        identity: "test:unknown-closure".into(),
        digest_domain: "quire.source.bytes/v1".into(),
        digest: "0".repeat(64).into(),
    };
    let provenance = vec![CheckedSourceRegion {
        source: upstream_source.clone(),
        start: 0,
        end: 1,
    }];
    let open = select_population(Source::UnknownClosureOpenExtent {
        selected_model,
        population: population_id,
        source: upstream_source.clone(),
        provenance: provenance.clone(),
    });
    assert_no_extent(open.clone(), |outcome| {
        matches!(outcome, Outcome::UnknownClosureOpenExtent { .. })
    });
    assert_eq!(
        open,
        Outcome::UnknownClosureOpenExtent {
            selected_model: CheckedSemanticId {
                domain: PACKAGE_DOMAIN.into(),
                algorithm: "sha256".into(),
                digest: "e".repeat(64).into(),
            },
            population: population_nodes(&package)[0].node_id.clone(),
            source: upstream_source,
            provenance,
        }
    );
    assert_no_extent(
        select_population(Source::UnknownClosureUnresolvedObjectType {
            selected_model: CheckedSemanticId {
                domain: PACKAGE_DOMAIN.into(),
                algorithm: "sha256".into(),
                digest: "d".repeat(64).into(),
            },
            population: population_nodes(&package)[0].node_id.clone(),
            source: CheckedSourceRef {
                authority: "qsl".into(),
                identity: "test:unknown-object".into(),
                digest_domain: "quire.source.bytes/v1".into(),
                digest: "1".repeat(64).into(),
            },
            provenance: vec![],
        }),
        |outcome| matches!(outcome, Outcome::UnknownClosureUnresolvedObjectType { .. }),
    );
    assert_no_extent(
        select_population(Source::UnknownClosureUnresolvedSubtypeClosure {
            selected_model: CheckedSemanticId {
                domain: PACKAGE_DOMAIN.into(),
                algorithm: "sha256".into(),
                digest: "c".repeat(64).into(),
            },
            population: population_nodes(&package)[0].node_id.clone(),
            source: CheckedSourceRef {
                authority: "qsl".into(),
                identity: "test:unknown-subtype".into(),
                digest_domain: "quire.source.bytes/v1".into(),
                digest: "2".repeat(64).into(),
            },
            provenance: vec![],
        }),
        |outcome| {
            matches!(
                outcome,
                Outcome::UnknownClosureUnresolvedSubtypeClosure { .. }
            )
        },
    );
}

#[trace("FR-053-AC-3", "FR-053-AC-4", "FR-053-AC-5", "TC-453")]
#[test]
fn selected_population_tracks_authoritative_variants_and_retargeted_package() {
    let scratch = run_producer();
    let emitted = scratch.0.join("emitted");
    let domain = fs::read(emitted.join("spine-model.semantic-ir.json"))
        .expect("QSL producer wrote domain evidence");
    let same_type = read_package(&emitted.join("population-variant-same-type.json"), &domain);
    let changed_max = read_package(
        &emitted.join("population-variant-changed-max.json"),
        &domain,
    );
    let retargeted = read_package(&emitted.join("population-variant-retargeted.json"), &domain);

    let selected = |package: &CheckedPackageV2| {
        let mut populations: Vec<_> = population_nodes(package)
            .into_iter()
            .map(|node| {
                match select_population(Source::Admitted {
                    package,
                    selector: selector(package, node),
                }) {
                    Outcome::Finite(population) => population,
                    other => panic!("QSL-produced population must be finite: {other:?}"),
                }
            })
            .collect();
        populations.sort_by_key(|population| population.maximum);
        populations
    };

    let before = selected(&same_type);
    assert_eq!(
        before
            .iter()
            .map(|population| population.maximum)
            .collect::<Vec<_>>(),
        vec![0, 3, 5]
    );
    assert_eq!(before[0].object_type, before[2].object_type);
    assert_ne!(before[0].binding, before[2].binding);
    assert_ne!(before[0].object_type, before[1].object_type);

    let changed = selected(&changed_max);
    assert_ne!(same_type.package_id(), changed_max.package_id());
    assert_eq!(
        changed
            .iter()
            .map(|population| population.maximum)
            .collect::<Vec<_>>(),
        vec![0, 3, 6]
    );
    for (old, new) in before.iter().take(2).zip(changed.iter().take(2)) {
        assert_eq!(old.maximum, new.maximum);
        assert_eq!(old.binding, new.binding);
    }
    assert_ne!(before[2].binding, changed[2].binding);

    let retargeted_selections = selected(&retargeted);
    assert_ne!(same_type.package_id(), retargeted.package_id());
    assert_eq!(
        retargeted_selections
            .iter()
            .map(|population| population.maximum)
            .collect::<Vec<_>>(),
        vec![0, 3, 5]
    );
    for (original, after_retarget) in before.iter().zip(retargeted_selections.iter()) {
        assert_eq!(original.binding, after_retarget.binding);
        assert_eq!(original.maximum, after_retarget.maximum);
        assert_eq!(original.object_type, after_retarget.object_type);
        assert_eq!(original.provenance, after_retarget.provenance);
    }
}
