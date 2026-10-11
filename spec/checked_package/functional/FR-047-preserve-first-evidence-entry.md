---
id: FR-047
title: "Preserve the first evidence entry across repeated declarations"
type: FR
relationships:
  - target: ix://agent-ix/quire-contract-ir/StR-001
    type: traces_to
  - target: ix://agent-ix/quire-contract-ir/FR-038
    type: references
  - target: ix://agent-ix/quire-contract-ir/FR-019
    type: references
---
# FR-047: Preserve the first evidence entry across repeated declarations

## Description

PLANNED/UNRUN (IR-756). The insertion results, error export and acceptance
controls below are required upon implementation. This specification does not
claim that the current unit-returning insert methods implement this policy or
that its runtime controls have run.

When a caller repeats an evidence declaration under an occupied key,
`CheckedPackageEvidence` SHALL retain the first entry, succeeding for an equal
declaration and returning a typed conflict for an unequal declaration.

This requirement owns calls to the evidence builder. Selection-row uniqueness
and reader refusal ordering remain owned by
[FR-038](./FR-038-consume-checked-package-v2.md); those wire rules do not define
repeated builder calls.

## Inputs

An existing mutable `CheckedPackageEvidence`; an exact supplied domain digest
string and document bytes for `insert_domain_package_document`; an exact
supplied library identity string and already admitted `CheckedPackageV2` for
`insert_dependency_package`; or an exact feature string for `support_feature`.

The insert methods retain the existing ownership conversions:
`impl Into<Box<str>>` for keys, `impl Into<Box<[u8]>>` for documents and
`impl Into<Arc<CheckedPackageV2>>` for dependencies. Only the reader constructs
an admitted dependency package. The caller's evidence key is a claim, not
proof that a document or dependency binds to that key.

## Outputs

Each insert method returns `Result<(), CheckedPackageEvidenceInsertionError>`.
Both a vacant insertion and an equal repeat return `Ok(())`. An unequal repeat
returns one of these closed variants:

| Variant | Typed context |
| --- | --- |
| `DomainDocumentConflict { digest: Box<str> }` | The exact supplied occupied domain key. |
| `DependencyPackageConflict { identity: Box<str> }` | The exact supplied occupied dependency key. |

`support_feature` returns unit; it has no conflicting-value case.

`CheckedPackageEvidenceInsertionError` is a PLANNED model-crate public export.
When this requirement is implemented, `quire_contract_model` SHALL export this
error type by name. Its variants and fields determine the condition and key;
formatted error text is not a classification source. This is an insertion
error, not a `CheckedPackageRefusal`, reader cause or wire diagnostic.
[FR-019](../../model/functional/FR-019-rust-library-interface.md)'s implemented
Public items table remains unchanged until the error is actually exported;
the implementation change must update that inventory and its export control
together. No future export is counted as present by this draft.

## Behavior

### Exact keys and domain bytes

Keys are the exact supplied strings. Equality performs no trimming, case
folding, Unicode normalization, parsing or synonym recognition. Domain keys
and dependency identities occupy separate maps even if their strings match.
Different keys are independent, including when their values are equal.

When a domain key is vacant, the evidence builder SHALL store exactly the
supplied byte sequence. When that key is occupied by equal bytes, the evidence
builder SHALL return `Ok(())` without replacing the first sequence. When that
key is occupied by different bytes, the evidence builder SHALL return
`DomainDocumentConflict` carrying the exact key.

Document equality means equal lengths and equal bytes at every position. It
does not mean equal parsed JSON, canonical JSON, claimed digest or domain
identity. Two encodings of the same JSON value with different whitespace
therefore conflict under one key. Insertion neither parses nor canonicalizes
either document.

### Admitted dependency content

When a dependency identity is vacant, the evidence builder SHALL hold the
supplied admitted package. When that identity is occupied by equal admitted
content, the evidence builder SHALL return `Ok(())` without replacing the first
package. When that identity is occupied by unequal admitted content, the
evidence builder SHALL return `DependencyPackageConflict` carrying the exact
identity.

Equal admitted content is exactly the existing `CheckedPackageV2::eq`
relation: equality of all fields of `CheckedPackageWireV2` and the decoded
node kinds. The wire includes contract version, identity preimage, package
identity, lock, complete semantic graph, source map, capability report and
diagnostics. Equality preserves array order and compares all retained wire
fields, including graph occurrences. It is not `package_id` equality or `Arc`
address equality. Equal `package_id` values do not make packages with different
source maps, diagnostics, capability dispositions, occurrences or raw-source
references equal evidence values.

The existing relation excludes the retained read byte ceiling,
admission-derived node lookup index and retained domain-model tables. Two
admissions of equal content under different read byte ceilings are therefore
equal repeats. In that case, the evidence builder SHALL retain the original
package object, including its original ceiling and derived state. It neither
replaces the first `Arc` with a content-equal second `Arc` nor merges their
state. A second admission's ceiling cannot upgrade or downgrade the first.
The same retention rule applies with the insertion order reversed. Different
first-value histories need not produce equivalent retained state.

### Feature declarations

When a feature is declared, the evidence builder SHALL retain its exact string
once. Repeating that exact string is idempotent. Distinct case, whitespace,
Unicode spelling or version spelling denotes a distinct declaration and does
not imply support for another string. Declaration adds no feature-name grammar
validation; the reader retains its existing wire-name rules and exact
required-feature membership check.

### Atomicity, caller handling and reading

When either insert method returns a conflict, the evidence builder SHALL leave
its complete state unchanged. This includes the first entry, every unrelated
domain and dependency entry, and the feature set. An equal repeat also leaves
the complete state unchanged. There is no latched error or deferred conflict
in a later read, and no partial replacement before returning an error.

The caller must observe the `Result` and choose whether to stop assembly or
continue using the unchanged evidence. It can match the variant and key
without parsing a message. A handled conflict does not install the second
value. After a handled conflict, a read against that evidence has the same
result as a read against first-only evidence under the same package bytes and
read limits.

Insertion stores claims, including malformed or mismatching keys/documents;
it is not admission. An occupied-key conflict is decided without invoking
reader validation, even if either document would fail later reading.
[FR-038](./FR-038-consume-checked-package-v2.md) continues to own document
recomputation, canonicality, dependency binding, feature admission and their
existing refusal precedence. This requirement adds no reader refusal, cause,
work charge or altered priority. An invalid first value still reaches the
reader's ordinary outcome even when the rejected second value would admit.

### Resource behavior

For E stored entries, lookup has the existing ordered-map/set O(log(E+1)) bound
in key comparisons, including key-length comparison costs. Domain equality
compares at most the supplied finite byte lengths. Dependency equality uses
the admitted-content relation directly, with work bounded by the finite
compared content and the admitted grammar's existing nesting bound. The
builder SHALL compare without encoding documents/packages, cloning complete
packages, parsing document bytes or building a second lookup/model index.
Retaining an `Arc` expresses sharing, not a package clone.

An equal repeat or conflict retains no second entry or second value. A vacant
insertion adds one entry. No new capacity limit, builder work meter, depth cap
or caller-selected builder budget is introduced. Existing read limits bound
reader work, not construction of caller-owned evidence. Input ownership
conversions, key storage/error context and vacant entries may allocate;
recoverable content conflict does not promise recoverable allocator
exhaustion. This change raises no limit and introduces no digest computation
or encoder.

## Acceptance Criteria

| ID | Criteria | Verification |
| --- | --- | --- |
| FR-047-AC-1 | A vacant domain key accepts its exact bytes; an identical raw-byte repeat returns `Ok(())` and preserves the complete evidence state. Equal values under distinct exact keys remain separate, and reading a valid selected document after one or two equal declarations gives the same admitted content. | Test |
| FR-047-AC-2 | Different raw bytes under an occupied domain key return `DomainDocumentConflict` with that exact key and leave all evidence unchanged, in both insertion orders. Byte-distinct encodings of equal JSON also conflict; malformed document bytes under a vacant claimed key are stored for later reader admission. | Test |
| FR-047-AC-3 | Equal admitted-content dependencies in distinct `Arc` allocations return `Ok(())` on repetition and retain the first package object. Separately admitted equal content under different byte ceilings remains an equal repeat and preserves the first object's ceiling and derived state in both orders. Independent exact identities remain separate; the public reader binds a selected valid dependency after one or two equal declarations. | Test |
| FR-047-AC-4 | Unequal admitted dependencies under an occupied identity return `DependencyPackageConflict` with that exact identity and leave all evidence unchanged in both orders. Controls distinguish different package identities and equal package identities with independently changed, valid source map, diagnostics, capability disposition, graph occurrences or raw-source references. No same-identity or same-package-id shortcut accepts these conflicts. | Test |
| FR-047-AC-5 | One and two exact feature declarations yield equal evidence and the same successful public read of an otherwise valid package requiring that feature. Declaring only a distinct case, whitespace, Unicode spelling or version spelling does not satisfy the required exact string; declarations perform no normalization or synonym substitution. | Test |
| FR-047-AC-6 | Public model-crate callers handle both insert results and match the exact typed conflict variants/keys without message parsing. After a handled domain or dependency conflict, real public reading equals first-only reading in both orders: a valid matching first value admits, and an invalid or mismatching first value retains its ordinary reader outcome even when the rejected second value would admit. Insertion invokes no admission or encoding; inspection confirms the stated resource behavior and no changed reader refusal/limit contract. The new error export and controls remain planned until implemented. | Test, Inspection |

## Dependencies

[StR-001](../../core/stakeholder/StR-001-authoritative-contract.md) owns the
authoritative contract need. [FR-038](./FR-038-consume-checked-package-v2.md)
owns public reading and selection binding.
[FR-019](../../model/functional/FR-019-rust-library-interface.md) owns the
implemented model export inventory.
[TC-059](../matrix/TC-059-evidence-entry-repetition.md) defines the planned
positive, adverse and mutation-sensitive repetition controls. No runtime
binder or passing result is implied by that procedure artifact.
