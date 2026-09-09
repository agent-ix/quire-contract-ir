---
id: NFR-005
title: "Qualify the exact supported Rust toolchain"
type: NFR
quality_attribute: portability
relationships:
  - target: ix://agent-ix/quire-contract-ir/ADR-0055
    type: depends_on
  - target: ix://agent-ix/quire-contract-ir/FR-019
    type: constrains
---
# NFR-005: Qualify the exact supported Rust toolchain

## Statement

The crate shall declare and qualify exact Rust 1.98.1 as its initial supported
minimum and candidate qualification compiler. An older compiler shall not be
retained merely because a scaffold or another repository declared it.

## Scope

Cargo metadata, toolchain and Clippy configuration, local orchestration, hosted
checks, specifications, assurance inputs, all library/binary/test targets, and
the exact required compiler-adjacent tools and embedded targets.

## Measurement and Evaluation

| Metric | Target | Threshold | Method |
|---|---|---|---|
| Supported minimum declarations | exact `1.98.1` everywhere | any stale or floating declaration fails | Inspection |
| Qualification compiler declarations | exact `1.98.1` everywhere | any stale or floating declaration fails | Inspection |
| Rust targets | all declared targets compile/test as applicable | any compiler/target incompatibility fails | Test |
| Required tools | each required operation executes | launch/version-only evidence or genuine incompatibility fails | Test |
| Formatting migration | formatted by 1.98.1 | unchecked output fails; changed formatting is permitted | Test |
| Unsupported older floor | none without evidence | inherited pin or assertion fails | Inspection |
| Stable-release review latency | compatibility run within 7 calendar days; disposition before the next candidate | late or absent disposition blocks qualification | Inspection |

## Acceptance Criteria

| ID | Criteria | Verification |
|---|---|---|
| NFR-005-AC-1 | `Cargo.toml`, `rust-toolchain.toml`, `clippy.toml`, Make targets, hosted workflow, requirements and assurance inputs distinguish supported minimum from qualification compiler and pin both to exact 1.98.1. | Inspection (TC-036) |
| NFR-005-AC-2 | Locked all-target tests, warning-denied Clippy, rustfmt, release build, unsafe audit and declared target builds execute under 1.98.1; formatting changes and remediated lint findings are accepted migration output. | Test (TC-037) |
| NFR-005-AC-3 | Required cargo-deny/audit and any repository-specific compiler tools execute their real operation under the candidate environment; domain findings are reported separately from launch/protocol/compiler incompatibility. | Test (TC-037) |
| NFR-005-AC-4 | Any proposal for an older supported minimum identifies a concrete consumer/target/tool constraint, reproduces the exact incompatibility, bounds its impact, records an expiry/review condition, and has owner approval. No such exception is currently recorded. | Inspection (TC-036) |
| NFR-005-AC-5 | Within 7 calendar days after a newer stable Rust release, the real compatibility matrix is executed and its exact-version disposition is recorded. A later candidate is blocked until that disposition; an older hold expires no later than 30 days or the next affected-tool release, whichever occurs first. | Inspection (TC-036) |
| NFR-005-AC-6 | Every new Rust requirement test imports `ix_trace_rs::trace` and uses a bare `#[trace("TC-036", "NFR-005-AC-...")]` or `#[trace("TC-037", "NFR-005-AC-...")]` attribute for its exact criterion. | Test (TC-036, TC-037) |

## Known non-compatibility findings

The `idna 0.4.0` and `time 0.3.36` vulnerabilities observed during
specification review were supply-chain findings, not Rust 1.98.1
incompatibilities. The implementation candidate updates their locked
resolutions within the existing dependency constraints. Downstream
shared-assurance matrix disagreement for `ix-flow` remains separate and shall
not cause a compiler reversion.

## Verification

TC-036 inspects every version-bearing repository surface and executes the exact
version/reevaluation policy. TC-037 executes the compiler, targets, and required
tools. Their reports separate compiler/tool incompatibility from formatting
changes, lint remediation, dependency vulnerabilities, and shared-assurance
acceptance state.
