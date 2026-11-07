---
id: SR-202611070939003
title: "spec-review/integrity of IR-1404 strict V2 read exact charging"
type: SpecReview
analysis: integrity
scope: "agent-ix/quire-contract-ir; spec/checked_package/functional/FR-038-consume-checked-package-v2.md, spec/checked_package/functional/FR-358-compose-native-identity-join-budget.md, spec/checked_package/matrix/TC-472-strict-v2-read-exact-budget-charging.md, spec/assurance/AD-001-contract-ir-architecture.md, spec/checked_package/matrix/tests.md"
review_set: subset
---

## Summary

Independent spec-review/integrity of quire-contract-ir PR #948 at 81c8c041e23e63edaf33b378e2b7702bd5a99569 (base main c838196ce), scope `git diff origin/main...HEAD`. Ticket: IR-1404. Cross-checked against quire-canonical PR #17 and PR #18 heads and the current crate source. Focus: completeness and consistency of the public API changes, the locus vocabulary and stale prose.

## Examined scope

- FR-038-CheckedJson (examined): A JSON payload that the admitted public typed wire carries without a closed type, a `literal` term's `value` and a diagnostic's `details`, is a `CheckedJson` value: an owned, cloneable handle to one node of the admitted package's retained `quire_canonical::Document`
- FR-038-AC-514 (examined): A `literal` term's `value` and each diagnostic detail of an admitted package are `CheckedJson` handles into the package's one retained document. Cloning one shares the retained document without allocating a tree
- FR-038-outputs (examined): A closed read result: admitted, refused, incomplete (limit kind, limit, consumed, and the location of the value whose charge failed) or exhausted
- FR-038-stage-table (examined): | `WireDecode` | root-member item index (graph node index for `semantic_graph.nodes`) | Each typed wire container ... |
- FR-358-AC-11 (examined): A strict V2 read denied in each stage leaves `first_unpaid_locus` naming exactly that stage, the index FR-038 names and its static `form`
- FR-038-AC-499 (examined): WireDecode stage. ... refuses at it with locus `PackageRead(WireDecode, i)` and the root member as `form`.
- FR-038-AC-500 (examined): WireChecks stage. ... the last allocation of the header, owner-schema, flat-wire or interval-bound checks refuses at it with locus `PackageRead(WireChecks, i)`
- FR-038-AC-504 (examined): Graph stage. For genuine packages that together reach every node family validator ... refuses at it with locus `PackageRead(Graph, i)`
- FR-038-AC-380 (examined): NI-13's 100000-node left-nested chain ... reads on a 256 KiB thread stack under serde_json's default 128 recursion limit.
- FR-038-prose-697 (examined): The v2 literal grammar admits any `i64` or `u64` today and the document is canonical by `serde_json`'s spelling, so it is admitted today

## Verdict

FAIL. D-2 introduces a public type with no specified API; two more public enum changes are implied but not stated; locus index/form are not fully determined; several stale serde_json statements remain.

## Findings

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-001 | high | D-2: CheckedJson has no specified construction, accessor or serde contract; QSL/CG cannot adapt from the spec | spec/checked_package/functional/FR-038-consume-checked-package-v2.md:425-431 |
| FND-002 | medium | New public variants Exhausted and CheckedPackageLimit::Storage are implied but not stated as API changes | spec/checked_package/functional/FR-038-consume-checked-package-v2.md:205-208 |
| FND-003 | medium | Locus index and form are not determined for several stages, so FR-358-AC-11's expected locus is unknowable | spec/checked_package/functional/FR-038-consume-checked-package-v2.md:5613-5629 |
| FND-004 | low | Stale serde_json statements contradict the new reader | spec/checked_package/functional/FR-038-consume-checked-package-v2.md:5673 |

## Finding detail

### FND-001

CheckedTerm and CheckedDiagnosticV2 derive Serialize/Deserialize today and a public test deserializes CheckedTerm with serde_json (crates/quire-contract-model/tests/typed_terms_deserialize.rs, IR-1208). QSL matches on literal values (qsl-package/src/emit/timed_order.rs:286-291, 527) and external writers build CheckedDiagnosticsV2 for encode_checked_package_v2 (v2/mod.rs:826). The spec defines CheckedJson only as a handle into an admitted package's Document, so it leaves open: how a writer or test constructs one (from bytes? from a Document?), what read accessors replace Value's (kind, string, bool, number text), whether Serialize/Deserialize stay on CheckedTerm, CheckedDiagnosticV2 and the wire types (and how Deserialize builds a document without serde_json), and whether Debug is iterative. Fix: add a CheckedJson subsection naming its public constructor(s), its borrowed view API, equality, Debug, and the fate of the serde derives, with ACs.

### FND-002

The closed CheckedPackageV2ReadResult (v2/mod.rs:879-886) gains an exhausted outcome, read_checked_package's CheckedPackageDispatchResult must carry it too, and 'limit_kind Work or Storage' needs a Storage member of CheckedPackageLimit (shared.rs:48-61), a serde/FixedShape vocabulary shared with the V1 reader. None is stated as a public change, the variant names and the CheckedPackageReadExhausted field types are not given, and the dispatch result shape is absent. These are breaking changes for QSL/CG beyond D-2. Fix: name each new variant and field, state that V1 never yields Storage or Exhausted, and give the dispatch mapping. (Removing BTreeMap/serde_json::Map from the read path changes no public type: node_index and the other maps are private fields; lower.rs:239 supported_tags is lowering output, not the read.)

### FND-003

WireDecode's 'root-member item index' is undefined for object-valued root members (lock, diagnostics, capability_report wrapper) and for nested arrays (lock.dependency_selections items). Graph uses 'graph node index' but validate_graph also validates per-row sidecars (temporal_bindings, temporal_clause_names, refinement_step_rows, schedulability_claims). Only WireDecode names its form ('the root member'); every other stage's static form string is unstated, yet FR-358-AC-11 and TC-472 assert 'its static form'. Fix: give index rules for object members and sidecar rows, and list the form strings per stage (or say form is not asserted).

### FND-004

AC-380 still says the chain 'reads ... under serde_json's default 128 recursion limit'; FR-038:697 says the document 'is canonical by serde_json's spelling'; AC-381's evidence name 'before_the_parse_limit' and the Dependencies note on quire-canonical's 'Encode for serde_json::Value' (FR-038:6201-6202) describe the removed path. Fix: reword to the shared reader, and say whether the quire-canonical serde_json feature and the float_roundtrip/raw_value/unbounded_depth features stay.


## New findings (disposition pass 1)

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-005 | medium | Downstream V2 read callers are not listed, and the zero read-storage default makes the obvious migration refuse every read | spec/checked_package/functional/FR-038-consume-checked-package-v2.md:6272 |

### FND-005

The read signature change forces every downstream production caller of CheckedPackageV2::read / read_checked_package to pass a budget: quire-spec-language qsl-package/src/checked_v2.rs, qsl-package/src/emit/extent_agreement.rs, src/checked_handoff.rs, qsl-replay/src/hybrid/process_request.rs; quire-contract-codegen src/kani/generate/negotiate.rs, src/kani/generate/outcome.rs, src/replay/composite_builder.rs. The spec lists the CheckedJson consumer changes but not these, and the scope freeze says QSL is untouched, which a compile-breaking signature cannot honour. Because NativeJoinBudget::new(work, storage) now leaves the read-storage limit at zero (FR-358-AC-14), the natural migration NativeJoinBudget::new(u64::MAX, u64::MAX) compiles and then returns Exhausted on every read. In-repo nothing breaks silently: no production IR code builds a budget before a read today, and every call site must be edited anyway. Fix: list the downstream read call sites as public uses that change, with the required form NativeJoinBudget::new(..).with_read_storage_limit(..), or add an unlimited constructor (for example NativeJoinBudget::unlimited()).


## Dispositions

Verified against fix commit 03518d07b1afac4ecd962d57d44c2a7742ce7cb6 (81c8c041e..03518d0).

| FND | outcome | reason |
| --- | --- | --- |
| FND-001 | fixed | Fix commit 03518d0; after: ### The `CheckedJson` payload type -- Construction: `CheckedJson::from_canonical_bytes(bytes, max_input_bytes)` and `CheckedJson::from_value(&serde_json::Value) |
| FND-002 | fixed | Fix commit 03518d0; after: `CheckedPackageV2ReadResult` gains `Exhausted(CheckedPackageReadExhausted)`; `CheckedPackageDispatchResult` gains `Exhausted(CheckedPackageReadExhausted)`, to w |
| FND-003 | fixed | Fix commit 03518d0; after: / `WireDecode` / the position of the decoded value in the innermost wire array holding it, or 0 for a value in no array / the dotted wire path of that array, or |
| FND-004 | fixed | Fix commit 03518d0; after: AC-380: reads on a 256 KiB thread stack through the shared strict reader, which has no recursion limit. FR-038:748: canonical by the shared writer's spelling of |
