## Project purpose

Quoin and Quire already cover the specification, assurance-plan, traceability,
verification-method selection, and evidence-collection sides of engineering
assurance. They can recommend and record property tests, mutation tests,
examples, reviews, analyses, and proofs.

This program fills a different gap: the missing Rust-native compilation layer
between a machine-readable requirement contract and the verification artifacts
that exercise it.

The intended flow is:

```text
versioned requirement contract
  ├── executable Rust oracle
  ├── tri-state test harness
  ├── constrained proptest strategy
  ├── Kani proof obligation
  ├── SMT consistency/implication query
  └── derivation, coverage, vacuity, and proof evidence
```

The source contract is authoritative. Tests, proof obligations, and evidence
records are derived from it rather than independently rewriting the same oracle.
That reduces drift and makes the relationship between a requirement and its
evidence mechanically inspectable.

A second, independent axis covers requirements whose subject is an event stream
rather than a single pre/post-state transition:

```text
bounded temporal requirement
  └── MLTL AST → parse/normalize/evaluate → R2U2/C2PO monitor input + evidence
```

## Why this is needed

Existing Rust tools solve important pieces, but not the complete assurance
problem:

- Property-testing libraries generate inputs but do not define requirement
  identity, execution-point anchoring, contract lowering, or evidence
  provenance.
- Proof tools can verify authored harnesses, but an independently authored proof
  contract can drift away from the executable test oracle.
- Ordinary coverage cannot show that an implication passed only because its
  antecedent was always false.
- Filtering invalid property-test inputs can make a campaign appear green while
  most generated cases were rejected.
- SMT solvers do not provide the requirement identity, shared-variable model,
  encoding versioning, or evidence envelope needed for specification analysis.
- Rust has production-capable runtime monitors such as R2U2, but lacks a small,
  reusable MLTL syntax/parser/rewrite/reference-semantics crate family.

The program therefore builds the assurance-specific connective tissue and
reuses established engines wherever they already exist.

## How the capabilities are applied

These methods are conditional, not a mandatory replacement for every existing
test:

| Capability | Apply when | Complements |
|---|---|---|
| Contract oracle lowering | A requirement has an executable clause anchored to a named initialization, handler, or pre/post execution point | Example, unit, integration, and property tests |
| Generator derivation | Preconditions contain ranges, membership, or supported relations that can shape an input domain | Property testing and boundary testing |
| Vacuity reporting | A contract contains implications, cases, guards, or rejected preconditions | Structural coverage and mutation testing |
| Kani lowering | The requirement is bounded and its state/data model is suitable for model checking | Executable tests and static analysis |
| SMT analysis | Multiple requirements share logically expressible variables or assumptions | Specification review and proof |
| MLTL evaluation/monitoring | The requirement concerns a bounded sequence of events or time-indexed values | Runtime monitoring, scenario tests, and simulation |

A project may use only one capability or combine several. Unsupported,
inconclusive, rejected, timed-out, and pending results remain explicit; none is
silently converted into successful evidence.

## Design principles

1. **One contract, multiple independent lowerings.** Proof and test artifacts
   share semantics without sharing hand-authored oracle code.
2. **Identity is first-class.** Requirement ID, revision, execution point,
   source span, dependencies, tool version, and artifact digest survive every
   transformation.
3. **Vacuity is observable.** Rejected preconditions and unexecuted implication
   consequents are reported separately from passes.
4. **Evidence is data.** Results are versioned, reproducible artifacts suitable
   for import into an assurance case, not just console messages.
5. **Small trusted boundaries.** Generated code and runtime support remain small
   and auditable; external solvers and monitors are identified and versioned.
6. **Permissive substrate.** Anything linked into customer software or intended
   as an ecosystem building block is available under `MIT OR Apache-2.0`.
7. **Qualification support, not certification theater.** The crates produce
   validation and provenance evidence that can support NASA-style tool
   validation/accreditation decisions. Accreditation remains specific to the
   consuming project, purpose, version, configuration, and decision authority.

## What we reuse

The program does not build new solvers, coverage engines, property-testing
frameworks, or production monitors. It targets established components:

- `proptest` for generated property campaigns
- Kani for bounded Rust proof harnesses
- Z3 and cvc5 through SMT-LIB2 for logical analysis
- LLVM/cargo coverage tooling for consequent-execution evidence
- R2U2/C2PO for deployed stream monitoring
- `serde`, `quote`, `syn`, and related Rust infrastructure
- permissively licensed Sireum/HAMR lowering implementations and INSPECTA
  outputs as attributed design references and differential fixtures

Ambiguously licensed grammars and unlicensed repositories are not copied.

## Outcome

Deliver eight independently usable, public Rust crates for contract lowering,
SMT-backed specification analysis, and bounded temporal-logic assurance:

| Repository | Responsibility |
|---|---|
| `quire-contract-ir` | Versioned contract model, anchors, typed expressions, canonical identities |
| `quire-contract-runtime` | Small `no_std` oracle runtime and tri-state harness verdicts |
| `quire-contract-codegen` | Rust oracle, proptest, Kani, vacuity-map, and evidence generation |
| `quire-analyze` | SMT-backed consistency, contradiction, implication, and vacuity analysis |
| `tl-syntax` | `no_std` MLTL AST and semantic-profile model |
| `tl-parse` | MLTL parsing, formatting, and diagnostics |
| `tl-rewrite` | Semantics-preserving normalization and rewrite evidence |
| `tl-mltl` | Finite-trace reference evaluation, horizon analysis, and R2U2 interoperability |

## Workstreams

- Contract lowering and executable evidence
- SMT consistency analysis
- MLTL syntax, rewriting, evaluation, and R2U2 interoperability

The contract workstream is the first delivery priority. Its initial vertical
slice is contract IR → executable oracle → tri-state proptest harness → vacuity
report. SMT analysis then reuses the stable IR. Temporal logic can proceed as an
independent parallel workstream.

## Relationship to shared assurance

The crates remain independently usable. Contract and temporal domain producers
execute verification and own their native structured results and diagnostics.
Quire exports static definitions without invoking producers; Quoin consumes
explicit structured results for retention, integrity, audit, and report views
without invoking producers; ix-flow records attributed human decisions.

Shared integration is common work governed by
[Engineering Assurance #7](https://github.com/agent-ix/engineering-assurance/issues/7),
not deferred architecture. The accepted sequence is this repository's
[#38](https://github.com/agent-ix/quire-contract-ir/issues/38), Quire CLI #74,
Quoin CLI #322, Engineering Assurance #9, Quoin #323, Engineering Assurance #8,
and Engineering Assurance #10 before any repository migration. Conditional
post-release catalog and adapter opportunities remain in
[#7](https://github.com/agent-ix/quire-contract-ir/issues/7).

Published runtime and domain crates do not acquire runtime dependencies on Quire
or Quoin. Development-time adoption uses exact reviewed release pins.

## Program rules

- Every crate is licensed `MIT OR Apache-2.0`.
- Cargo registry publication remains disabled through v0.1 review.
- Every repository follows specify → matrix → review → plan → implementation →
  gap analysis.
- Each repository carries a Quire-validated Assurance Profile, Architecture
  Description, Component Assurance Contract, Measurement Plan, and Assurance
  Argument.
- Established engines and libraries are reused; this program does not implement
  a new SMT solver or production runtime monitor.
- Agent-assisted contributions retain truthful provenance and require human
  review and release ownership.

## Explicitly out of scope

- Runtime dependencies on, or producer execution by, Quoin or Quire
- Repository migrations before the common-work releases, exact pins, and migration contract
- A new SMT solver
- A replacement production runtime monitor
- Automatic claims of NASA, DO-178C, or other certification/accreditation
- Treating every requirement as suitable for contracts, SMT, proofs, or temporal
  logic

## Completion

All three workstream epics are complete, cross-repository conformance passes,
source tags and checksums are published, and the human release owner records the
v0.1 assurance decision. The completed program must demonstrate that:

- one contract produces semantically aligned test, proof, and evidence artifacts;
- vacuous satisfaction and rejected preconditions are visible per requirement;
- SMT results retain every non-conclusive state and solver dependency;
- MLTL transformations preserve the selected finite-trace semantics; and
- downstream users can adopt each crate independently without depending on
  Quoin or Quire.

