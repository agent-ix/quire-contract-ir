---
id: SR-14380
title: code-review review of IR-615 producer
type: SpecReview
analysis: code-review
scope: agent-ix/quire-contract-ir; IR-615 producer, profile admission and arithmetic/collection/graph
  preparation, changed status prose
review_set: subset
---

## Summary

Frozen candidate. Ticket: IR-615. PR: agent-ix/quire-contract-ir#488. No applicable AssuranceProfile exists. Reviewer performed no Cargo, rustc, fmt, metadata, native or Kani execution; original focused logs and exact healthy/red/restored source inputs were independently inspected. Full aggregate remains the lead’s gate. Exact deployment SKU is unavailable; model marker uses gpt-6. Excerpts longer than 600 characters are verbatim prefixes; complete changed files and cited criteria were read.

## Verdict

PASS — no new code/Rust defect. The closed enum removes the only admitted profile Inconclusive carrier, so both direct enum Deserialize and validated profile Deserialize reject the legacy variant independently of its cause spelling. No coercion to Refused/Unsupported or vacuity is introduced. Existing Supported/Refused arms and all semantic-family computations are byte-unchanged. Request census reads all entries in request order, preserving exact selected values beyond an early refusal. Tests assert both wire boundaries, paired admitted profiles, exact result vectors and three families’ refusal kind/cause/source/context/no-Boolean behavior through public APIs. Genuine run Inconclusive and zero-SUCCESS rule are untouched and the existing focused classifier test passes. No new panic/unsafe/integer cast/async/lock/process/resource boundary or public carrier; exhaustive matching remains compiler-checked. No copied authority, compatibility shim, new digest/catalog or unasked-for machinery.

## Findings

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-001 | low | No findings (placeholder) | - |

## Coverage

21 actual focused integration executions pass (12 shared plus three per family), with fmt check, locked workspace/all-target Clippy and separate default-feature model consumer Clippy. The compiled four-original-production-file restoration admits the obsolete disposition and fails the intended rejection test with exit 101; restoring the exact candidate makes the affected test pass. Scope scripts enforce actual memory.max 17179869184 and swap.max 0 before Cargo. Peaks: focused 3463081984, red 940281856, restored 939433984 bytes; all recorded memory events are zero. No lock/dependency/CI/gate change. No proof was run or qualified.

## Examined Scope

```yaml
scope:
- id: FR-029-AC-1
  path: spec/kani/functional/FR-029-versioned-bounded-kani-profile.md
  role: examined
  excerpt: A selected profile binds its versioned lowering, harness, oracle/strategy,
    replay, bounds, and capability-matrix identities before lowering.
- id: FR-029-AC-2
  path: spec/kani/functional/FR-029-versioned-bounded-kani-profile.md
  role: examined
  excerpt: Every encountered construct has exactly one supported, refused, or unsupported
    matrix entry; an unsupported entry settles its item `unsupported` with a warning
    naming the item's capability kind from the closed `quire.capability-kind/v1` vocabulary
    (QSpec FR-290) and emits no artifact, `KaniOutcome` or `TerminalValue`; no matrix
    entry records resource exhaustion, cancellation or an absent executable; unknown,
    conflicting, and incomplete matrices refuse with the originating source identity.
- id: FR-029-AC-3
  path: spec/kani/functional/FR-029-versioned-bounded-kani-profile.md
  role: examined
  excerpt: A scalar capability refuses an object, reference, graph, collection, query,
    temporal, or unbounded-population request with its source identity.
- id: FR-030-AC-3
  path: spec/kani/functional/FR-030-bounded-kani-domain-and-outcomes.md
  role: examined
  excerpt: No invalid, refused, incomplete, unavailable, timed-out, exhausted, cancelled,
    or inconclusive outcome carries Boolean success; earliest failures emit no partial
    artifact.
- id: FR-030-AC-4
  path: spec/kani/functional/FR-030-bounded-kani-domain-and-outcomes.md
  role: examined
  excerpt: A `proved` outcome built from a run with three SUCCESS checks carries the
    count three; a run with zero SUCCESS checks yields `inconclusive` with cause `kani_vacuous_proof`,
    carries no count and no Boolean claim; and a request for a `proved` outcome with
    a count of zero returns `KaniOutcomeError` with code `kani_outcome_invalid` and
    no outcome.
- id: FR-030-AC-5
  path: spec/kani/functional/FR-030-bounded-kani-domain-and-outcomes.md
  role: examined
  excerpt: An `unavailable` outcome built for an absent checker carries cause `kani_checker_absent`
    and one built for an absent backend carries `kani_backend_absent`, each with no
    Boolean claim; a request for an `unavailable` outcome with any other cause code,
    an `inconclusive` outcome with a cause other than `kani_vacuous_proof`, or a `proved`
    or `counterexample` outcome through the non-success constructor returns `KaniOutcomeError`
    with code `kani_outcome_invalid` and no outcome; and a struct-literal `KaniOutcome`
    outside the `kani` module fails to compile.
- id: FR-030-AC-8
  path: spec/kani/functional/FR-030-bounded-kani-domain-and-outcomes.md
  role: examined
  excerpt: An external consumer calls each of the nine public infallible methods with
    an explicit `KaniOutcome` return type. The six coded methods retain a consumer-owned
    unregistered `Std001Code`; `checker_unavailable`, `backend_unavailable` and `vacuous_proof`
    carry their exact fixed causes. Every outcome retains distinct supplied source/context
    strings, has the specified kind and no Boolean claim, and survives a serialization/deserialization
    round trip with exactly the same kind, code, source and context. Both cancellation
    sources remain distinct in the wire form.
- id: FR-030-AC-9
  path: spec/kani/functional/FR-030-bounded-kani-domain-and-outcomes.md
  role: examined
  excerpt: External compile-fail probes cannot pass `KaniOutcomeKind::Proved` or `Counterexample`
    as a kind to the infallible route, cannot supply a cause argument to its three
    fixed-cause methods, cannot omit the cancellation source, and cannot pass a `&str`
    or `String` as a coded method's cause. Each probe is paired with a passing public
    call of the intended signature. Existing fallible `non_success` requests for `Proved`
    and `Counterexample` still return `KaniOutcomeError` naming the exact requested
    kind and cause and build no outcome.
- id: spec/kani/functional/FR-029-versioned-bounded-kani-profile.md
  path: spec/kani/functional/FR-029-versioned-bounded-kani-profile.md
  role: examined
  excerpt: "---\nid: FR-029\ntitle: \"Select a versioned bounded Kani capability profile\"\
    \ntype: FR\nrelationships:\n - target: ix://agent-ix/quire-contract-ir/StR-001\n\
    \ type: traces_to\n - target: ix://agent-ix/quire-contract-ir/FR-015\n \
    \ type: depends_on\n - target: ix://agent-ix/quire-contract-ir/ADR-0053\n \
    \ type: depends_on\n---\n# FR-029: Select a versioned bounded Kani capability\
    \ profile\n\n## Description\n\nThe Contract IR Kani boundary shall select an immutable,\
    \ named bounded-Kani profile before it lowers a checked native clause or creates\
    \ a harness.\n\n## Inputs\n\nAn exact checked native clause, its select"
- id: spec/kani/functional/FR-030-bounded-kani-domain-and-outcomes.md
  path: spec/kani/functional/FR-030-bounded-kani-domain-and-outcomes.md
  role: examined
  excerpt: "---\nid: FR-030\ntitle: \"Bind bounded Kani domains and non-Boolean outcomes\"\
    \ntype: FR\nrelationships:\n - target: ix://agent-ix/quire-contract-ir/StR-001\n\
    \ type: traces_to\n - target: ix://agent-ix/quire-contract-ir/FR-029\n \
    \ type: depends_on\n - target: ix://agent-ix/quire-contract-ir/FR-015\n type:\
    \ depends_on\n - target: ix://agent-ix/quire-contract-ir/FR-044\n type: depends_on\n\
    ---\n# FR-030: Bind bounded Kani domains and non-Boolean outcomes\n\n## Description\n\
    \nThe bounded-Kani boundary shall use a typed finite model-domain/snapshot/population\
    \ ABI and a common typed outcome envelope. The bo"
- id: spec/kani/matrix/tests.md
  path: spec/kani/matrix/tests.md
  role: examined
  excerpt: "---\nid: TM-007\ntitle: \"quire-contract-ir bounded Kani test matrix\"\
    \ntype: TestMatrix\n---\n# quire-contract-ir bounded Kani test matrix\n\n## Functional\
    \ Requirement Coverage\n\n| Functional Req | Acceptance Criteria | Test Cases\
    \ | Status |\n|---|---|---|---|\n| FR-029 | FR-029-AC-1 through FR-029-AC-3 |\
    \ TC-042 | \U0001F6A7 AC-1 and AC-3 implemented; AC-2 is planned: the matrix admits\
    \ only `Supported` and `Refused` and rejects legacy `inconclusive` dispositions;\
    \ the missing `unsupported` entry must settle the item at negotiation with a warning\
    \ naming the item's capability kind from the closed `quire.capability-k"
- id: src/kani/arithmetic.rs
  path: src/kani/arithmetic.rs
  role: examined
  excerpt: "//! Definedness-preserving checked-arithmetic lowering preparation.\n\n\
    use quire_contract_model::std001_code;\n\nuse quire_contract_model::NumericOperator;\n\
    \nuse super::{\n CapabilityDisposition, DispatchIndex, KaniOutcome, KaniProfile,\
    \ NonSuccessKind, SemanticFamily,\n ValidatedFiniteInput,\n};\n\n/// A statically\
    \ admitted concrete arithmetic operation and its exact named range.\n#[derive(Clone,\
    \ Copy, Debug, Eq, PartialEq)]\npub struct CheckedArithmeticRequest {\n ///\
    \ Source clause or checked-expression identity.\n pub source_id: &'static str,\n\
    \ /// Native checked arithmetic operation.\n pu"
- id: src/kani/collections.rs
  path: src/kani/collections.rs
  role: examined
  excerpt: "//! Ordered duplicate-preserving bounded collection/query lowering.\n\n\
    use quire_contract_model::std001_code;\n\nuse super::{\n CapabilityDisposition,\
    \ DispatchIndex, KaniOutcome, KaniProfile, NonSuccessKind, SemanticFamily,\n \
    \ ValidatedFiniteInput,\n};\n\n/// Supported query form over one ordered finite\
    \ sequence.\n#[derive(Clone, Copy, Debug, Eq, PartialEq)]\npub enum QueryKind\
    \ {\n ForAllNonNegative,\n ExistsEqual(i128),\n}\n\n/// Exact selected collection\
    \ query.\n#[derive(Clone, Debug, Eq, PartialEq)]\npub struct CollectionQuery {\n\
    \ pub source_id: String,\n pub values: Vec<i128>,\n pub max_ite"
- id: src/kani/objects.rs
  path: src/kani/objects.rs
  role: examined
  excerpt: "//! Identity-preserving finite-reference and bounded-reachability lowering.\n\
    \nuse std::collections::{BTreeMap, BTreeSet};\n\nuse quire_contract_model::std001_code;\n\
    \nuse super::{\n CapabilityDisposition, DispatchIndex, KaniOutcome, KaniProfile,\
    \ NonSuccessKind, SemanticFamily,\n ValidatedFiniteInput,\n};\n\n/// One positive-length\
    \ reachability request over the validated finite universe.\n#[derive(Clone, Debug,\
    \ Eq, PartialEq)]\npub struct GraphRequest {\n /// Source clause identity.\n\
    \ pub source_id: String,\n /// Start object identity.\n pub start_id:\
    \ String,\n /// Target object identity.\n "
- id: src/kani/profile.rs
  path: src/kani/profile.rs
  role: examined
  excerpt: "//! Versioned profile and complete supported/refused capability matrix.\n\
    \nuse std::collections::BTreeSet;\n\nuse quire_contract_model::Std001Code;\nuse\
    \ serde::{Deserialize, Serialize};\nuse thiserror::Error;\n\nuse super::{KaniOutcome,\
    \ NonSuccessKind, PROFILE};\n\n/// One construct's selected bounded-Kani disposition.\n\
    #[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]\n#[serde(rename_all\
    \ = \"snake_case\")]\npub enum CapabilityDisposition {\n /// A named shared\
    \ or semantic-family module owns exact lowering.\n Supported { module: String\
    \ },\n /// The selected profile refuses the construct at"
- id: tests/it/kani_arithmetic.rs
  path: tests/it/kani_arithmetic.rs
  role: examined
  excerpt: "use ix_trace_rs::trace;\nuse quire_contract_ir::kani::{\n lower_checked_arithmetic,\
    \ CapabilityDisposition, CapabilityEntry, DispatchIndex, FiniteInput,\n KaniOutcomeKind,\
    \ KaniProfile, ModuleDescriptor, PopulationCompleteness, ProfileSelection,\n \
    \ ResourceBounds, SemanticFamily, PROFILE,\n};\nuse quire_contract_model::std001_code;\n\
    use quire_contract_model::NumericOperator;\n\nfn selection -> ProfileSelection\
    \ {\n ProfileSelection {\n profile: PROFILE.into,\n revision:\
    \ \"kani-bounded/1.0.0\".into,\n abi_revision: \"kani-abi/1\".into,\n\
    \ }\n}\n\nfn input -> quire_contract_i"
- id: tests/it/kani_collections.rs
  path: tests/it/kani_collections.rs
  role: examined
  excerpt: "use ix_trace_rs::trace;\nuse quire_contract_ir::kani::{\n lower_query,\
    \ CapabilityDisposition, CapabilityEntry, CollectionQuery, DispatchIndex,\n \
    \ FiniteInput, FiniteObject, KaniOutcomeKind, KaniProfile, ModuleDescriptor,\n\
    \ PopulationCompleteness, ProfileSelection, QueryKind, ResourceBounds, SemanticFamily,\
    \ PROFILE,\n};\nfn selection -> ProfileSelection {\n ProfileSelection {\n\
    \ profile: PROFILE.into,\n revision: \"kani-bounded/1.0.0\".into,\n\
    \ abi_revision: \"kani-abi/1\".into,\n }\n}\nfn input -> quire_contract_ir::kani::ValidatedFiniteInput\
    \ {\n FiniteInput {\n "
- id: tests/it/kani_objects.rs
  path: tests/it/kani_objects.rs
  role: examined
  excerpt: "use ix_trace_rs::trace;\nuse quire_contract_ir::kani::{\n lower_reaches,\
    \ CapabilityDisposition, CapabilityEntry, DispatchIndex, FiniteInput,\n FiniteObject,\
    \ FiniteReference, GraphRequest, KaniOutcomeKind, KaniProfile, ModuleDescriptor,\n\
    \ PopulationCompleteness, ProfileSelection, ResourceBounds, SemanticFamily,\
    \ PROFILE,\n};\n\nfn selection -> ProfileSelection {\n ProfileSelection {\n\
    \ profile: PROFILE.into,\n revision: \"kani-bounded/1.0.0\".into,\n\
    \ abi_revision: \"kani-abi/1\".into,\n }\n}\nfn input -> quire_contract_ir::kani::ValidatedFiniteInput\
    \ {\n FiniteInput {"
- id: tests/it/kani_shared.rs
  path: tests/it/kani_shared.rs
  role: examined
  excerpt: "use ix_trace_rs::trace;\nuse quire_contract_ir::kani::{\n CapabilityDisposition,\
    \ CapabilityEntry, DispatchError, DispatchIndex, FiniteInput,\n FiniteObject,\
    \ FiniteReference, KaniCancellationSource, KaniOutcome, KaniOutcomeKind,\n \
    \ KaniProfile, ModuleDescriptor, PopulationCompleteness, ProfileSelection, ResourceBounds,\n\
    \ SemanticFamily, PROFILE,\n};\nuse quire_contract_model::{std001_code, Std001Code};\n\
    \nfn selection -> ProfileSelection {\n ProfileSelection {\n profile:\
    \ PROFILE.into,\n revision: \"kani-bounded/1.0.0\".into,\n abi_revision:\
    \ \"kani-abi/1\".into,\n }\n}"
- id: CG-FR-015-AC-82
  path: quire-contract-codegen/spec/kani/functional/FR-015-bounded-kani-obligations.md
  role: context_only
  excerpt: Until FR-015-AC-111 deletes the bounded-profile classifier, a request for
    distinct named constructs returns one `CapabilityEntry` per requested construct
    in request order, preserving each admitted `Supported` module or `Refused` cause
    code exactly. The Contract IR profile reader rejects the removed `inconclusive`
    disposition rather than admitting it; no profile disposition or profile-supplied
    code becomes an `Inconclusive` run outcome or asserts proof vacuity.
```
