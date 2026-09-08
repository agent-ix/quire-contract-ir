---
id: ADR-0055
title: "Proposed Rust 1.98.1 qualification and compatibility baseline"
type: ADR
status: proposed
owner: kreneskyp
relationships:
  - target: ix://agent-ix/quire-contract-ir/PGM-01
    type: depends_on
---
# ADR-0055: Proposed Rust 1.98.1 qualification and compatibility baseline

## Status

**Proposed; owner decision absent.** This records the correction requested
during issue #54 inspection. The repository's Rust 1.75 pin originated in its
initial scaffold and has no recorded consumer, target, or tool justification.
Existing use is evidence of propagation, not evidence that the decision is
correct.

## Decision proposed for approval

Use exact Rust **1.98.1** as both the initial supported minimum and the exact
qualification toolchain for this pre-release crate. Pin the same version in
Cargo metadata, the repository toolchain file, Clippy configuration, local
orchestration, hosted checks, specifications, and assurance inputs.

Keep the two concepts separately named even while their values are equal:

- `supported-rust-minimum` is the oldest compiler the crate promises to support;
- `qualification-rust` is the exact compiler used to produce candidate evidence.

An older minimum may be proposed later only with a concrete consumer need and
passing evidence on that exact compiler. Holding either value below current
stable requires an identified incompatibility in a required compiler target or
tool, an impact assessment, an expiry/review condition, and owner approval.
An inherited pin, formatting output change, or newly reported Clippy diagnostic
is not an incompatibility.

A new stable Rust release triggers the compatibility matrix within 7 calendar
days. The exact pin does not float during a candidate run, but no later candidate
may proceed until the newer release is adopted or the owner records the concrete
incompatibility and a time-bounded hold. A hold expires after 30 days or the next
affected-tool release, whichever comes first, and must then be re-evaluated.

Rustfmt output from 1.98.1 becomes the repository's formatting result. The
migration may reformat existing code. Required tools must be executed, not only
version-queried; a tool's successful execution that reports a real domain
finding is compatible behavior, not a compatibility failure.

## Preliminary compatibility observations

These are reproducible design inputs, not retained qualification evidence or an
owner acceptance. Environment: Linux x86_64 WSL2, Rust 1.98.1
`48a229ceaefd4985c50990b14116b6d856af0985`, LLVM 22.1.8.

| Repository revision | Executed operation | Observation |
|---|---|---|
| quire-contract-ir `decc99a430a0894de489102dbab04e83d1fb804f` | `cargo +1.98.1 test --locked --all-targets`; `cargo +1.98.1 clippy --locked --all-targets -- -D warnings` | pass |
| quire-contract-codegen `240fad84a9565ab723ba9844e18faea4e5d96f66` | locked all-target/all-feature test and Clippy; Kani 0.67.0 generated-proof test | compiler, Clippy and Kani pass; overall test command later refuses unrelated `ix-flow` matrix state |
| quire-contract-runtime `7caefac3b7181d7665caa8f83c6694134fac63ae` | locked all-target/all-feature test and Clippy; no-default-feature `thumbv7em-none-eabi` build | compiler, Clippy and embedded target pass; overall test command later refuses unrelated `ix-flow` matrix state |
| filament-core-data `65ea7fa6d4805752fff82d2cc2b7100eefe60bb8` | semantic-IR Rust tests and warning-denied Clippy | tests pass; four new Clippy diagnostics require cleanup and are not incompatibilities |
| quire-contract-ir revision above | `cargo +1.98.1 deny check`; `cargo +1.98.1 audit` | tools execute; existing transitive `idna 0.4.0` and `time 0.3.36` vulnerabilities fail the domain gate |

The implementation/review cycle must reproduce the applicable commands at its
exact candidate revision. This table cannot be promoted into release evidence.

## Consequences

The 1.75 compatibility lane and claims are removed rather than preserved as a
second unexplained release gate. Direct consumers must make their own reviewed
compatibility decision; Contract IR does not promise their old floor for them.
The broader ecosystem's copied 1.75 pins require a governed audit and must not
be bulk-changed without repository-specific target/tool checks.

The vulnerable transitive dependencies are a separate supply-chain defect and
remain release-relevant even after the compiler migration. Passing on 1.98.1
does not waive them.

## Rejected alternatives

- **Keep 1.75 because it is already declared.** No decision evidence exists.
- **Use 1.85 because Filament declares it.** That value is also stale and its
  existence is not a Contract IR requirement.
- **Use floating `stable` as qualification identity.** It is not reproducible.
- **Treat rustfmt or Clippy drift as incompatibility.** Both are expected inputs
  to migration and remediation.
