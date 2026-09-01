use std::{
    collections::BTreeSet,
    fs,
    path::{Path, PathBuf},
};

use sha2::{Digest, Sha256};

const POLICY: &str = include_str!("../spec/program/PGM-01-governance.md");
const RECONCILIATION: &str = include_str!("../spec/program/STD-002-shared-assurance-governance.md");
const README: &str = include_str!("../README.md");
const CONTRIBUTING: &str = include_str!("../CONTRIBUTING.md");
const HISTORICAL_LOCK: &str = include_str!("fixtures/historical-pgm01-files.sha256");
const DISPOSITION_RECEIPT: &str = include_str!("fixtures/campaign-issue-dispositions-v1.json");

fn normalized(document: &str) -> String {
    document.split_whitespace().collect::<Vec<_>>().join(" ")
}

fn markdown_table_after<'a>(document: &'a str, marker: &str) -> Vec<Vec<&'a str>> {
    document
        .split_once(marker)
        .unwrap_or_else(|| panic!("missing table marker: {marker}"))
        .1
        .lines()
        .skip_while(|line| !line.starts_with('|'))
        .take_while(|line| line.starts_with('|'))
        .skip(2)
        .map(|line| {
            line.trim_matches('|')
                .split('|')
                .map(str::trim)
                .collect::<Vec<_>>()
        })
        .collect()
}

fn inventory_after(document: &str, marker: &str) -> BTreeSet<String> {
    document
        .split_once(marker)
        .unwrap_or_else(|| panic!("missing inventory marker: {marker}"))
        .1
        .lines()
        .skip_while(|line| line.is_empty())
        .take_while(|line| line.starts_with("- `"))
        .map(|line| {
            line.trim_start_matches("- `")
                .trim_end_matches('`')
                .to_owned()
        })
        .collect()
}

fn collect_files(path: &Path, files: &mut Vec<PathBuf>) {
    let mut entries = fs::read_dir(path)
        .unwrap_or_else(|error| panic!("cannot enumerate {}: {error}", path.display()))
        .map(|entry| entry.expect("directory entry must remain readable"))
        .collect::<Vec<_>>();
    entries.sort_by_key(|entry| entry.file_name());
    for entry in entries {
        let name = entry.file_name();
        if matches!(name.to_str(), Some(".git" | "target" | ".worktrees")) {
            continue;
        }
        let file_type = entry.file_type().expect("file type must remain readable");
        if file_type.is_dir() {
            collect_files(&entry.path(), files);
        } else if file_type.is_file() {
            files.push(entry.path());
        }
    }
}

fn repository_relative(root: &Path, path: &Path) -> String {
    path.strip_prefix(root)
        .expect("collected path stays below repository root")
        .components()
        .map(|component| component.as_os_str().to_string_lossy())
        .collect::<Vec<_>>()
        .join("/")
}

fn production_dependency_violations(manifest: &str) -> Vec<String> {
    let forbidden = ["quire", "quire-rs", "quoin"];
    let mut in_production_dependencies = false;
    let mut violations = Vec::new();

    for raw_line in manifest.lines() {
        let line = raw_line.split('#').next().unwrap_or_default().trim();
        if line.is_empty() {
            continue;
        }
        let compact = line
            .chars()
            .filter(|character| !character.is_whitespace())
            .collect::<String>()
            .replace('\'', "\"")
            .to_ascii_lowercase();

        if compact.starts_with('[') && compact.ends_with(']') {
            let header = compact.trim_matches(['[', ']']);
            in_production_dependencies = !header.contains("dev-dependencies")
                && (header == "dependencies"
                    || header == "build-dependencies"
                    || header == "workspace.dependencies"
                    || header.ends_with(".dependencies")
                    || header.ends_with(".build-dependencies")
                    || header.starts_with("dependencies.")
                    || header.starts_with("build-dependencies.")
                    || header.contains(".dependencies.")
                    || header.contains(".build-dependencies."));
            if in_production_dependencies {
                for marker in ["dependencies.", "build-dependencies."] {
                    if let Some((_, dependency)) = header.rsplit_once(marker) {
                        let dependency = dependency.trim_matches('"');
                        if forbidden.contains(&dependency) {
                            violations.push(line.to_owned());
                        }
                    }
                }
            }
            continue;
        }

        if !in_production_dependencies {
            continue;
        }
        if let Some((key, _)) = compact.split_once('=') {
            if forbidden.contains(&key.trim_matches('"')) {
                violations.push(line.to_owned());
                continue;
            }
        }
        if forbidden
            .iter()
            .any(|name| compact.contains(&format!("package=\"{name}\"")))
        {
            violations.push(line.to_owned());
        }
    }

    violations
}

fn is_lower_sha256(value: &str) -> bool {
    value.len() == 64
        && value
            .bytes()
            .all(|byte| byte.is_ascii_digit() || (b'a'..=b'f').contains(&byte))
}

/// Tracing: TC-023.
/// FR-008-AC-4.
/// FR-021-AC-1.
#[test]
fn tc_023_assigns_each_shared_responsibility_exactly_once() {
    let rows = markdown_table_after(POLICY, "The shared responsibility assignment is exact:");
    let actual = rows.iter().map(|row| (row[0], row[1])).collect::<Vec<_>>();
    let expected = vec![
        (
            "Static verification definitions, obligations, relations, and locators",
            "Quire",
        ),
        (
            "Verification execution",
            "Contract or temporal domain producer",
        ),
        (
            "Structured domain result and diagnostic",
            "Originating contract or temporal producer",
        ),
        (
            "Evidence intake, retained bytes, integrity, audit, and report views",
            "Quoin",
        ),
        (
            "Human approval, rejection, revision, and workflow state",
            "ix-flow",
        ),
        (
            "Cross-repository campaign policy and release order",
            "PGM-01",
        ),
    ];
    assert_eq!(actual, expected);
}

/// Tracing: TC-024.
/// FR-009-AC-5.
#[test]
fn tc_024_locks_historical_bytes_and_requires_lossy_read_only_mapping() {
    let root = Path::new(env!("CARGO_MANIFEST_DIR"));
    let mut locked = BTreeSet::new();
    for line in HISTORICAL_LOCK.lines().filter(|line| !line.is_empty()) {
        let (expected, relative) = line
            .split_once("  ")
            .expect("historical lock rows use sha256sum format");
        assert!(
            locked.insert(relative.to_owned()),
            "duplicate historical path: {relative}"
        );
        let bytes = fs::read(root.join(relative)).unwrap_or_else(|error| {
            panic!("locked historical input {relative} is unreadable: {error}")
        });
        let actual = format!("{:x}", Sha256::digest(bytes));
        assert_eq!(actual, expected, "historical bytes changed: {relative}");
    }
    let mut historical_paths = Vec::new();
    collect_files(&root.join("evidence"), &mut historical_paths);
    let mut expected = historical_paths
        .iter()
        .map(|path| repository_relative(root, path))
        .collect::<BTreeSet<_>>();
    expected.extend(
        [
            "schemas/derivation-evidence-envelope-v1.schema.json",
            "schemas/evidence-correction-v1.schema.json",
            "schemas/pgm01-evidence-v1.schema.json",
        ]
        .map(str::to_owned),
    );
    assert_eq!(
        locked, expected,
        "historical lock must cover every evidence file and PGM-01 schema"
    );

    let policy = normalized(POLICY);
    let reconciliation = normalized(RECONCILIATION);
    assert!(policy.contains("explicit read-only compatibility mapping"));
    assert!(policy.contains("never rewrite them into a newer schema"));
    assert!(policy.contains("treat an absent historical field as known"));
    assert!(reconciliation.contains("explicit read-only, lossy mapping"));
    assert!(reconciliation.contains("does not mutate source bytes, synthesize missing"));
}

/// Tracing: TC-025.
/// FR-009-AC-6.
/// FR-021-AC-2.
#[test]
fn tc_025_preserves_domain_ownership_nonexecution_and_runtime_independence() {
    let policy = normalized(POLICY);
    for phrase in [
        "Domain repositories still own their producers and structured results",
        "Quire and Quoin are explicitly non-executing",
        "Published runtime and domain crates shall not acquire runtime dependencies on Quire or Quoin",
        "Quoin owns retention, integrity checks, audit, and report views",
        "ix-flow owns any human decision",
    ] {
        assert!(policy.contains(phrase), "missing ownership rule: {phrase}");
    }

    let root = Path::new(env!("CARGO_MANIFEST_DIR"));
    let mut files = Vec::new();
    collect_files(root, &mut files);
    let manifests = files
        .iter()
        .filter(|path| path.file_name().and_then(|name| name.to_str()) == Some("Cargo.toml"))
        .collect::<Vec<_>>();
    assert!(
        !manifests.is_empty(),
        "at least one Cargo manifest is inspected"
    );
    for path in manifests {
        let manifest = fs::read_to_string(path)
            .unwrap_or_else(|error| panic!("cannot read {}: {error}", path.display()));
        assert_eq!(
            production_dependency_violations(&manifest),
            Vec::<String>::new(),
            "{} links a forbidden Quire/Quoin production dependency",
            repository_relative(root, path)
        );
    }

    for mutation in [
        "[dependencies]\nassurance={version=\"1\",package=\"quoin\"}",
        "[dependencies.quoin]\nversion=\"1\"",
        "[build-dependencies]\nassurance = { package = 'quire' }",
        "[target.'cfg(unix)'.dependencies.assurance]\npackage=\"quire-rs\"",
        "[workspace.dependencies]\nassurance={package=\"quoin\"}",
    ] {
        assert!(
            !production_dependency_violations(mutation).is_empty(),
            "dependency mutation escaped inspection: {mutation}"
        );
    }
}

/// Tracing: TC-026.
/// FR-021-AC-3.
#[test]
fn tc_026_authenticates_the_inspected_campaign_issue_dispositions() {
    let receipt: serde_json::Value =
        serde_json::from_str(DISPOSITION_RECEIPT).expect("disposition receipt must be JSON");
    assert_eq!(
        receipt["schemaVersion"],
        "quire.campaign-issue-dispositions/v1"
    );
    assert_eq!(receipt["repository"], "agent-ix/quire-contract-ir");
    assert_eq!(receipt["observedAt"], "2026-09-01T17:04:02Z");
    let issues = receipt["issues"]
        .as_array()
        .expect("issues must be an array");
    assert_eq!(issues.len(), 3);

    let expected = [
        (
            1,
            "open",
            "EPIC: Contract-derived verification and temporal assurance v0.1",
            "f668c2395ffcaf1e7d8586b5e9b67609dc17bc3f0d22dbea4c32c07859ea56a1",
        ),
        (
            7,
            "open",
            "Inventory post-release Quire/Quoin catalog and adapter opportunities",
            "a0ce9ded6604bc510fdb9060e39691bdd235845d095e9039636f389459c27c09",
        ),
    ];
    for (number, state, title, digest) in expected {
        let issue = issues
            .iter()
            .find(|issue| issue["number"] == number)
            .unwrap_or_else(|| panic!("missing issue #{number} receipt"));
        assert_eq!(issue["state"], state);
        assert_eq!(issue["title"], title);
        assert_eq!(issue["bodySha256"], digest);
        assert!(is_lower_sha256(digest));
        assert!(!issue["requiredMarkers"].as_array().unwrap().is_empty());
        assert!(!issue["absentMarkers"].as_array().unwrap().is_empty());
    }

    let issue20 = issues
        .iter()
        .find(|issue| issue["number"] == 20)
        .expect("missing issue #20 receipt");
    assert_eq!(issue20["state"], "closed");
    assert_eq!(issue20["stateReason"], "not_planned");
    assert_eq!(issue20["closedAt"], "2026-09-01T17:04:02Z");
    assert_eq!(issue20["closureComment"]["id"], 5_497_534_831_u64);
    assert_eq!(issue20["closureComment"]["author"], "kreneskyp");
    let comment_digest = issue20["closureComment"]["bodySha256"]
        .as_str()
        .expect("closure comment digest must be text");
    assert_eq!(
        comment_digest,
        "481f2e028177b3103d4d18d5b01cb70d3821c7452b957cb1fcd7fe90122dc874"
    );
    assert!(is_lower_sha256(comment_digest));

    for link in [
        "https://github.com/agent-ix/quire-contract-ir/issues/1",
        "https://github.com/agent-ix/quire-contract-ir/issues/7",
        "https://github.com/agent-ix/quire-contract-ir/issues/20",
        "https://github.com/agent-ix/engineering-assurance/issues/7",
    ] {
        assert!(
            RECONCILIATION.contains(link),
            "missing disposition link: {link}"
        );
    }
}

/// Tracing: TC-027.
/// FR-021-AC-4.
#[test]
fn tc_027_separates_preserved_cases_from_rejected_architecture() {
    let accepted_adversarial = inventory_after(RECONCILIATION, "Accepted adversarial cases:");
    let accepted_domain = inventory_after(RECONCILIATION, "Accepted domain cases:");
    let rejected = inventory_after(RECONCILIATION, "Rejected architecture:");

    assert_eq!(accepted_adversarial.len(), 9);
    assert_eq!(accepted_domain.len(), 4);
    assert_eq!(rejected.len(), 7);
    assert!(accepted_adversarial.is_disjoint(&rejected));
    assert!(accepted_domain.is_disjoint(&rejected));
    for required in [
        "checksum-reseal-tamper",
        "failed-versus-unavailable",
        "mutation-must-turn-domain-oracle-red",
    ] {
        assert!(accepted_adversarial.contains(required));
    }
    for required in [
        "contract-validation-and-canonicalization",
        "property-proof-and-vacuity-results",
        "smt-nonconclusive-results",
        "temporal-parse-rewrite-and-evaluation-results",
    ] {
        assert!(accepted_domain.contains(required));
    }
    for required in [
        "generic-command-executor",
        "central-execution-profile",
        "aggregate-overall-verdict",
        "evidence-authority-index",
        "repository-adoption-command",
        "parallel-result-family",
        "retention-layout-or-store",
    ] {
        assert!(rejected.contains(required));
    }
}

/// Tracing: TC-028.
/// FR-021-AC-5.
#[test]
fn tc_028_removes_conflicting_campaign_prescriptions() {
    let obsolete_prescriptions = [
        "The common evidence envelope identity is `quire.derivation-evidence/v1`.",
        "### PGM-01-R08 — common derivation and evidence envelope",
        "When actual deployment differs from the primary class, the evidence envelope shall declare the deployed role",
        "Candidate evidence shall be immutable, revision-scoped, content-addressed, and retained with a manifest",
        "- the common PGM-01 evidence envelope is used for generated and analysis artifacts;",
        "# FR-008: Validate the common evidence envelope",
    ];
    let root = Path::new(env!("CARGO_MANIFEST_DIR"));
    let mut documents = vec![root.join("README.md"), root.join("CONTRIBUTING.md")];
    for directory in ["spec", "plan", "docs", "reviews"] {
        let path = root.join(directory);
        if path.is_dir() {
            let mut files = Vec::new();
            collect_files(&path, &mut files);
            documents.extend(
                files
                    .into_iter()
                    .filter(|file| file.extension().and_then(|value| value.to_str()) == Some("md")),
            );
        }
    }
    assert!(
        documents.len() > 20,
        "campaign document census is unexpectedly small"
    );
    for path in documents {
        let document = fs::read_to_string(&path)
            .unwrap_or_else(|error| panic!("cannot read {}: {error}", path.display()));
        let document = normalized(&document);
        for obsolete in obsolete_prescriptions {
            assert!(
                !document.contains(obsolete),
                "{} retains obsolete prescription: {obsolete}",
                repository_relative(root, &path)
            );
        }
    }
    assert!(README.contains("Quire and Quoin are non-executing"));
    assert!(CONTRIBUTING.contains("not runtime dependencies or a shared producer runner"));
    assert!(RECONCILIATION.contains("The eight migration issues are not part of this gate"));
}
