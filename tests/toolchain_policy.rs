use std::{fs, path::Path};

use ix_trace_rs::trace;

const RUST_VERSION: &str = "1.98.1";

fn root() -> &'static Path {
    Path::new(env!("CARGO_MANIFEST_DIR"))
}

fn read(relative: &str) -> String {
    fs::read_to_string(root().join(relative))
        .unwrap_or_else(|error| panic!("cannot read {relative}: {error}"))
}

#[derive(Clone)]
struct PolicySurface {
    cargo: String,
    toolchain: String,
    clippy: String,
    makefile: String,
    workflow: String,
    component_assurance: String,
    change_assurance: String,
    assurance_chain: String,
    decision: String,
}

impl PolicySurface {
    fn current() -> Self {
        Self {
            cargo: read("Cargo.toml"),
            toolchain: read("rust-toolchain.toml"),
            clippy: read("clippy.toml"),
            makefile: read("Makefile"),
            workflow: read(".github/workflows/ci.yml"),
            component_assurance: read("spec/assurance/CAC-001-semantic-validator.md"),
            change_assurance: read("assurance/change-assurance.json"),
            assurance_chain: read("scripts/assurance_chain.py"),
            decision: read("spec/decisions/ADR-0055-qualified-rust-baseline.md"),
        }
    }

    fn failures(&self) -> Vec<&'static str> {
        let mut failures = Vec::new();
        if !self
            .cargo
            .contains(&format!("rust-version = \"{RUST_VERSION}\""))
        {
            failures.push("Cargo supported minimum");
        }
        if !self
            .toolchain
            .contains(&format!("channel = \"{RUST_VERSION}\""))
            || self.toolchain.contains("channel = \"stable\"")
        {
            failures.push("repository qualification toolchain");
        }
        if !self.clippy.contains(&format!("msrv = \"{RUST_VERSION}\"")) {
            failures.push("Clippy supported minimum");
        }
        for declaration in [
            format!("SUPPORTED_RUST_MINIMUM := {RUST_VERSION}"),
            format!("QUALIFICATION_RUST := {RUST_VERSION}"),
        ] {
            if !self.makefile.contains(&declaration) {
                failures.push("Make compiler declaration");
            }
        }
        let hosted_toolchains: Vec<&str> = self
            .workflow
            .lines()
            .filter_map(|line| line.trim().strip_prefix("toolchain: "))
            .collect();
        if hosted_toolchains.is_empty()
            || hosted_toolchains
                .iter()
                .any(|toolchain| *toolchain != RUST_VERSION)
        {
            failures.push("hosted compiler declaration");
        }
        let mutable_action = self
            .workflow
            .lines()
            .map(str::trim)
            .map(|line| line.strip_prefix("- ").unwrap_or(line))
            .filter_map(|line| line.strip_prefix("uses: "))
            .filter_map(|action| action.split_once('@').map(|(_, revision)| revision))
            .map(|revision| revision.split_whitespace().next().unwrap_or_default())
            .any(|revision| {
                revision.len() != 40 || !revision.bytes().all(|byte| byte.is_ascii_hexdigit())
            });
        if mutable_action {
            failures.push("hosted action pin");
        }
        if !self
            .component_assurance
            .contains("supported-rust-minimum=1.98.1; qualification-rust=1.98.1")
        {
            failures.push("component-assurance compiler declarations");
        }
        if !self.change_assurance.contains("\"1.98.1\"") {
            failures.push("change-assurance compiler declaration");
        }
        if !self.assurance_chain.contains("tool_version=\"1.98.1\"") {
            failures.push("assurance scenario compiler identity");
        }
        let decision = self
            .decision
            .split_whitespace()
            .collect::<Vec<_>>()
            .join(" ");
        if !self.decision.contains("status: accepted") {
            failures.push("accepted decision state");
        }
        if !decision.contains("within 7 calendar days") {
            failures.push("stable-release response window");
        }
        if !decision.contains("30 days or the next affected-tool release") {
            failures.push("older-toolchain hold expiry");
        }
        failures
    }
}

#[trace(
    "TC-036",
    "NFR-005-AC-1",
    "NFR-005-AC-4",
    "NFR-005-AC-5",
    "NFR-005-AC-6",
    "NFR-002-AC-2"
)]
#[test]
fn tc_036_exact_supported_and_qualification_rust_policy_agree() {
    let policy = PolicySurface::current();
    assert_eq!(policy.failures(), Vec::<&str>::new());

    let mut mutations = Vec::new();

    let mut mutated = policy.clone();
    mutated.cargo = mutated
        .cargo
        .replace("rust-version = \"1.98.1\"", "rust-version = \"1.75\"");
    mutations.push(("inherited Rust 1.75 minimum", mutated));

    let mut mutated = policy.clone();
    mutated.clippy = mutated.clippy.replace("1.98.1", "1.85");
    mutations.push(("stale Rust 1.85 Clippy minimum", mutated));

    let mut mutated = policy.clone();
    mutated.toolchain = mutated.toolchain.replace("1.98.1", "stable");
    mutations.push(("floating stable qualification compiler", mutated));

    let mut mutated = policy.clone();
    mutated.cargo = mutated.cargo.replace("rust-version = \"1.98.1\"\n", "");
    mutations.push(("absent supported minimum", mutated));

    let mut mutated = policy.clone();
    mutated.makefile = mutated
        .makefile
        .replace("QUALIFICATION_RUST := 1.98.1", "QUALIFICATION_RUST := 1.85");
    mutations.push(("inconsistent supported and qualification versions", mutated));

    let mut mutated = policy.clone();
    mutated.workflow = mutated
        .workflow
        .replacen("toolchain: 1.98.1", "toolchain: stable", 1);
    mutations.push(("floating hosted compiler", mutated));

    let mut mutated = policy.clone();
    mutated.workflow = mutated.workflow.replacen(
        "actions/checkout@11d5960a326750d5838078e36cf38b85af677262",
        "actions/checkout@v4",
        1,
    );
    mutations.push(("mutable hosted action", mutated));

    let mut mutated = policy.clone();
    mutated.component_assurance = mutated
        .component_assurance
        .replace("qualification-rust=1.98.1", "qualification-rust=1.85");
    mutations.push(("inconsistent assurance version", mutated));

    let mut mutated = policy.clone();
    mutated.change_assurance = mutated.change_assurance.replace("\"1.98.1\"", "\"1.75\"");
    mutations.push(("stale sealed assurance input", mutated));

    let mut mutated = policy.clone();
    mutated.assurance_chain = mutated
        .assurance_chain
        .replace("tool_version=\"1.98.1\"", "tool_version=\"1.75.0\"");
    mutations.push(("stale generated attestation identity", mutated));

    let mut mutated = policy.clone();
    mutated.decision = mutated
        .decision
        .replace("status: accepted", "status: proposed");
    mutations.push(("unaccepted decision", mutated));

    let mut mutated = policy.clone();
    mutated.decision = mutated.decision.replace(
        "within 7 calendar\ndays",
        "without a bounded response window",
    );
    mutations.push(("missing stable-release response window", mutated));

    let mut mutated = policy.clone();
    mutated.decision = mutated.decision.replace(
        "30 days or the next\naffected-tool release",
        "an unbounded interval",
    );
    mutations.push(("unbounded older-toolchain hold", mutated));

    for (name, mutation) in mutations {
        assert!(
            !mutation.failures().is_empty(),
            "policy mutation escaped detection: {name}"
        );
    }
}

#[trace("TC-037", "NFR-005-AC-2", "NFR-005-AC-3", "NFR-005-AC-6")]
#[test]
fn tc_037_release_gate_executes_exact_compiler_and_required_tools() {
    let makefile = read("Makefile");
    for operation in [
        "$(RUSTUP) run $(SUPPORTED_RUST_MINIMUM) $(CARGO) check --locked --all-targets",
        "$(RUSTUP) run $(QUALIFICATION_RUST) $(CARGO) test --locked --all-targets",
        "$(CARGO) fmt --all -- --check",
        "$(CARGO) clippy --locked --all-targets -- -D warnings",
        "$(CARGO) build --locked --release",
        "$(CARGO) deny check",
        "$(CARGO) audit",
        "bash scripts/check_unsafe_comments.sh",
    ] {
        assert!(
            makefile.contains(operation),
            "release gate lacks {operation}"
        );
    }

    let ci = makefile
        .lines()
        .find(|line| line.starts_with("ci:"))
        .expect("Makefile must declare the composite ci target");
    for target in [
        "fmt-check",
        "lint",
        "test",
        "supported-rust",
        "qualification-rust",
        "deny",
        "cargo-audit",
        "audit-unsafe",
    ] {
        assert!(ci.split_whitespace().any(|word| word == target));
    }
}
