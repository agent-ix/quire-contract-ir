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

fn toml_code_before_comment(line: &str) -> String {
    let mut code = String::new();
    let mut in_basic_string = false;
    let mut in_literal_string = false;
    let mut escaped = false;
    for character in line.chars() {
        if escaped {
            code.push(character);
            escaped = false;
            continue;
        }
        match character {
            '\\' if in_basic_string => {
                code.push(character);
                escaped = true;
            }
            '"' if !in_literal_string => {
                code.push(character);
                in_basic_string = !in_basic_string;
            }
            '\'' if !in_basic_string => {
                code.push(character);
                in_literal_string = !in_literal_string;
            }
            '#' if !in_basic_string && !in_literal_string => break,
            _ => code.push(character),
        }
    }
    code
}

fn production_dependency_violations(manifest: &str) -> Vec<String> {
    let forbidden = ["quire", "quire-rs", "quoin"];
    let mut in_production_dependencies = false;
    let mut violations = Vec::new();

    for raw_line in manifest.lines() {
        let code = toml_code_before_comment(raw_line);
        let line = code.trim();
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
            // A dotted key names the dependency in its first segment, so
            // `quoin.version = "1"` and `quoin.git = "..."` must be rejected
            // exactly like the bare `quoin = "1"` form.
            let dependency = key.split('.').next().unwrap_or(key).trim_matches('"');
            if forbidden.contains(&dependency) {
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

const OBSOLETE_PRESCRIPTIONS: [&str; 6] = [
    "The common evidence envelope identity is `quire.derivation-evidence/v1`.",
    "### PGM-01-R08 — common derivation and evidence envelope",
    "When actual deployment differs from the primary class, the evidence envelope shall declare the deployed role",
    "Candidate evidence shall be immutable, revision-scoped, content-addressed, and retained with a manifest",
    "- the common PGM-01 evidence envelope is used for generated and analysis artifacts;",
    "# FR-008: Validate the common evidence envelope",
];

/// Strips the quotation regions of a review artifact: Markdown blockquote lines
/// and fenced code blocks. A review must be able to cite the policy it removed.
///
/// An unterminated fence is not a quotation. Its lines are restored, so a
/// stray opening fence cannot exempt the remainder of a document.
fn without_quotations(document: &str) -> String {
    let mut prescriptive = String::new();
    let mut fenced = String::new();
    let mut in_fence = false;
    for line in document.lines() {
        let trimmed = line.trim_start();
        if trimmed.starts_with("```") || trimmed.starts_with("~~~") {
            if in_fence {
                fenced.clear();
            }
            in_fence = !in_fence;
            continue;
        }
        if in_fence {
            fenced.push_str(line);
            fenced.push('\n');
            continue;
        }
        if trimmed.starts_with('>') {
            continue;
        }
        prescriptive.push_str(line);
        prescriptive.push('\n');
    }
    if in_fence {
        prescriptive.push_str(&fenced);
    }
    prescriptive
}

/// Reports obsolete prescriptions carried as live policy by a campaign document.
///
/// `reviews/**` is the one exception, and only inside a quotation: a blockquote
/// line or a fenced code block is a citation of removed policy. Everywhere else
/// — and elsewhere in a review artifact — the prescription is rejected however
/// it is written. See `CONTRIBUTING.md`, "Quoting removed campaign policy".
fn campaign_prescription_violations(relative: &str, document: &str) -> Vec<String> {
    let quoting_allowed = relative.starts_with("reviews/");
    let inspected = if quoting_allowed {
        without_quotations(document)
    } else {
        document.to_owned()
    };
    let inspected = normalized(&inspected);
    OBSOLETE_PRESCRIPTIONS
        .iter()
        .filter(|obsolete| inspected.contains(&normalized(obsolete)))
        .map(|obsolete| (*obsolete).to_owned())
        .collect()
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
        "[dependencies]\nassurance={git=\"https://example.invalid/repo#rev\",package=\"quoin\"}",
        "[dependencies]\nquoin.version = \"1\"",
        "[dependencies]\nquoin.git = \"https://example.invalid/quoin.git\"",
        "[build-dependencies]\nquire-rs.workspace = true",
        "[target.'cfg(unix)'.dependencies]\n'quire'.version = \"1\"",
    ] {
        assert!(
            !production_dependency_violations(mutation).is_empty(),
            "dependency mutation escaped inspection: {mutation}"
        );
    }
    assert_eq!(
        production_dependency_violations("[dependencies]\nserde=\"1\" # package=\"quoin\""),
        Vec::<String>::new(),
        "a TOML comment must not create a false dependency"
    );
    assert_eq!(
        production_dependency_violations(
            "[dependencies]\nserde.version = \"1\"\nserde.features = [\"derive\"]"
        ),
        Vec::<String>::new(),
        "a non-Quoin dotted dependency key must not be reported"
    );
}

/// Reads a body retained beside the disposition receipt and asserts that its
/// bytes still hash to the digest the receipt recorded.
///
/// This proves fixture integrity only: it shows the retained bytes are the
/// bytes that were inspected at `observedAt`. It does not re-observe GitHub.
fn retained_body(receipt: &serde_json::Value, node: &serde_json::Value, label: &str) -> String {
    let relative = node["bodyPath"]
        .as_str()
        .unwrap_or_else(|| panic!("{label} must record a retained bodyPath"));
    let path = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("tests/fixtures")
        .join(relative);
    let bytes = fs::read(&path)
        .unwrap_or_else(|error| panic!("retained body {relative} is unreadable: {error}"));
    let actual = format!("{:x}", Sha256::digest(&bytes));
    let expected = node["bodySha256"]
        .as_str()
        .unwrap_or_else(|| panic!("{label} must record a bodySha256"));
    assert_eq!(
        actual, expected,
        "retained body {relative} no longer matches the receipt digest"
    );
    assert!(is_lower_sha256(expected));
    assert!(
        receipt["serialization"]
            .as_str()
            .is_some_and(|text| text.starts_with("SHA-256 of UTF-8 body bytes")),
        "the receipt must state the digest serialization it retains"
    );
    String::from_utf8(bytes).unwrap_or_else(|error| panic!("{relative} must be UTF-8: {error}"))
}

/// Asserts the receipt's marker fields against the retained body bytes, so the
/// markers are enforced rather than decorative.
fn assert_markers(node: &serde_json::Value, body: &str, label: &str) {
    let body = normalized(body);
    let required = node["requiredMarkers"]
        .as_array()
        .unwrap_or_else(|| panic!("{label} must record requiredMarkers"));
    assert!(!required.is_empty(), "{label} requiredMarkers must be set");
    for marker in required {
        let marker = marker.as_str().expect("markers are text");
        assert!(
            body.contains(&normalized(marker)),
            "{label} retained body is missing required marker: {marker}"
        );
    }
    for marker in node["absentMarkers"].as_array().unwrap_or(&Vec::new()) {
        let marker = marker.as_str().expect("markers are text");
        assert!(
            !body.contains(&normalized(marker)),
            "{label} retained body still contains absent marker: {marker}"
        );
    }
}

/// Tracing: TC-026.
/// FR-021-AC-3.
///
/// Scope: this test is an offline fixture-integrity check. It proves that the
/// retained bodies are the bytes recorded by the receipt and that the receipt's
/// markers hold in those bytes. It does not query GitHub and is not a live
/// GitHub oracle; the correspondence between these bytes and live repository
/// state was established once, by live inspection at `observedAt`.
#[test]
fn tc_026_binds_the_retained_campaign_disposition_bytes() {
    let receipt: serde_json::Value =
        serde_json::from_str(DISPOSITION_RECEIPT).expect("disposition receipt must be JSON");

    let proves = receipt["proves"]
        .as_array()
        .expect("receipt must state what it proves offline");
    let does_not_prove = receipt["doesNotProve"]
        .as_array()
        .expect("receipt must state what it does not prove offline");
    assert!(proves.iter().all(|claim| claim
        .as_str()
        .is_some_and(|claim| claim.starts_with("Offline:"))));
    assert!(does_not_prove.iter().any(|claim| claim
        .as_str()
        .is_some_and(|claim| claim.contains("not a live GitHub oracle"))));
    assert!(receipt["limitations"]
        .as_array()
        .expect("receipt must record limitations")
        .iter()
        .any(|claim| claim.as_str().is_some_and(|claim| claim
            .contains("do not re-establish that those bytes are the live GitHub state"))));
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
            "2026-09-01T17:03:52Z",
            "https://github.com/agent-ix/quire-contract-ir/issues/1",
            "f668c2395ffcaf1e7d8586b5e9b67609dc17bc3f0d22dbea4c32c07859ea56a1",
        ),
        (
            7,
            "open",
            "Inventory post-release Quire/Quoin catalog and adapter opportunities",
            "2026-09-01T17:03:54Z",
            "https://github.com/agent-ix/quire-contract-ir/issues/7",
            "a0ce9ded6604bc510fdb9060e39691bdd235845d095e9039636f389459c27c09",
        ),
    ];
    for (number, state, title, updated_at, url, digest) in expected {
        let issue = issues
            .iter()
            .find(|issue| issue["number"] == number)
            .unwrap_or_else(|| panic!("missing issue #{number} receipt"));
        assert_eq!(issue["state"], state);
        assert_eq!(issue["title"], title);
        assert_eq!(issue["updatedAt"], updated_at);
        assert_eq!(issue["url"], url);
        assert_eq!(issue["bodySha256"], digest);
        let label = format!("issue #{number}");
        let body = retained_body(&receipt, issue, &label);
        assert_markers(issue, &body, &label);
        assert!(
            !issue["absentMarkers"].as_array().unwrap().is_empty(),
            "{label} must record at least one absent marker"
        );
    }

    let issue20 = issues
        .iter()
        .find(|issue| issue["number"] == 20)
        .expect("missing issue #20 receipt");
    assert_eq!(issue20["state"], "closed");
    assert_eq!(issue20["stateReason"], "not_planned");
    assert_eq!(
        issue20["title"],
        "PGM-02: Repeatable assurance tooling — converge eight per-repo script trees onto one tested, reproducible component"
    );
    assert_eq!(
        issue20["url"],
        "https://github.com/agent-ix/quire-contract-ir/issues/20"
    );
    assert_eq!(issue20["updatedAt"], "2026-09-01T17:04:02Z");
    assert_eq!(issue20["closedAt"], "2026-09-01T17:04:02Z");
    assert_eq!(issue20["closureComment"]["id"], 5_497_534_831_u64);
    assert_eq!(issue20["closureComment"]["author"], "kreneskyp");
    assert_eq!(
        issue20["closureComment"]["createdAt"],
        "2026-09-01T17:03:55Z"
    );
    assert_eq!(
        issue20["closureComment"]["url"],
        "https://github.com/agent-ix/quire-contract-ir/issues/20#issuecomment-5497534831"
    );
    let comment_digest = issue20["closureComment"]["bodySha256"]
        .as_str()
        .expect("closure comment digest must be text");
    assert_eq!(
        comment_digest,
        "481f2e028177b3103d4d18d5b01cb70d3821c7452b957cb1fcd7fe90122dc874"
    );
    let comment = &issue20["closureComment"];
    let comment_body = retained_body(&receipt, comment, "issue #20 closure comment");
    assert_markers(comment, &comment_body, "issue #20 closure comment");
    assert_eq!(comment["requiredMarkers"].as_array().unwrap().len(), 3);

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
        let relative = repository_relative(root, &path);
        assert_eq!(
            campaign_prescription_violations(&relative, &document),
            Vec::<String>::new(),
            "{relative} retains an obsolete prescription as live policy"
        );
    }

    // The quoting rule is normative in CONTRIBUTING.md and enforced here.
    assert!(CONTRIBUTING.contains("## Quoting removed campaign policy"));
    assert!(normalized(CONTRIBUTING)
        .contains("In `reviews/**` only, a removed prescription may appear inside a quotation"));
    assert!(normalized(CONTRIBUTING).contains("Quoting does not exempt governed campaign content."));

    let obsolete = OBSOLETE_PRESCRIPTIONS[0];
    let blockquote = format!("# SR-999 review\n\nThe removed text was:\n\n> {obsolete}\n");
    let fenced = format!("# SR-999 review\n\nThe removed text was:\n\n```text\n{obsolete}\n```\n");
    let unquoted = format!("# SR-999 review\n\n{obsolete}\n");
    for (relative, document) in [
        ("reviews/SR-999-example.md", &blockquote),
        ("reviews/SR-999-example.md", &fenced),
    ] {
        assert_eq!(
            campaign_prescription_violations(relative, document),
            Vec::<String>::new(),
            "a review artifact must be able to quote a removed prescription"
        );
    }
    let unterminated_fence = format!("# SR-999 review\n\n```text\n{obsolete}\n");
    for (relative, document) in [
        ("reviews/SR-999-example.md", &unquoted),
        ("reviews/SR-999-example.md", &unterminated_fence),
        ("spec/program/PGM-01-governance.md", &blockquote),
        ("spec/program/PGM-01-governance.md", &unquoted),
        ("README.md", &unquoted),
    ] {
        assert!(
            !campaign_prescription_violations(relative, document).is_empty(),
            "{relative} must still reject an active obsolete prescription"
        );
    }

    assert!(README.contains("Quire and Quoin are non-executing"));
    assert!(CONTRIBUTING.contains("not runtime dependencies or a shared producer runner"));
    assert!(RECONCILIATION.contains("The eight migration issues are not part of this gate"));
}
