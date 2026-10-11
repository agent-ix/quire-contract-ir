---
id: SR-7087
title: spec-review/dependency review of IR-756 evidence repetition policy
type: SpecReview
analysis: dependency
scope: agent-ix/quire-contract-ir@eec76bc3712f5b49705a2f1b125c46d80b4368fa; IR-756;
  spec/checked_package/functional/FR-047-preserve-first-evidence-entry.md; spec/checked_package/matrix/TC-059-evidence-entry-repetition.md;
  spec/spec.md
review_set: subset
---

## Summary

Ticket: IR-756. Independent frozen diff review. Prerequisite graph: existing model/public interface FR-019 and admitted reader/equality FR-038 enable FR-047 implementation; FR-047 then enables TC-059 runtime controls. New references/traces_to/verifies edges resolve and do not introduce a prerequisite cycle. Requirement is consumer-visible behavior, not stand-alone infrastructure. IR-630 peer-owned clauses/source remain context only and unmodified.

## Verdict

**PASS** — no substantive defects in this method’s scoped spec review. Implementation and runtime acceptance remain planned/unrun.

## Findings

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-001 | low | No findings (placeholder) | - |

## Examined scope

```yaml
scope:
- id: FR-047-AC-1
  path: spec/checked_package/functional/FR-047-preserve-first-evidence-entry.md
  role: examined
  excerpt: A vacant domain key accepts its exact bytes; an identical raw-byte repeat
    returns `Ok(())` and preserves the complete evidence state. Equal values under
    distinct exact keys remain separate, and reading a valid selected document after
    one or two equal declarations gives the same admitted content.
- id: FR-047-AC-2
  path: spec/checked_package/functional/FR-047-preserve-first-evidence-entry.md
  role: examined
  excerpt: Different raw bytes under an occupied domain key return `DomainDocumentConflict`
    with that exact key and leave all evidence unchanged, in both insertion orders.
    Byte-distinct encodings of equal JSON also conflict; malformed document bytes
    under a vacant claimed key are stored for later reader admission.
- id: FR-047-AC-3
  path: spec/checked_package/functional/FR-047-preserve-first-evidence-entry.md
  role: examined
  excerpt: Equal admitted-content dependencies in distinct `Arc` allocations return
    `Ok(())` on repetition and retain the first package object. Separately admitted
    equal content under different byte ceilings remains an equal repeat and preserves
    the first object's ceiling and derived state in both orders. Independent exact
    identities remain separate; the public reader binds a selected valid dependency
    after one or two equal declarations.
- id: FR-047-AC-4
  path: spec/checked_package/functional/FR-047-preserve-first-evidence-entry.md
  role: examined
  excerpt: Unequal admitted dependencies under an occupied identity return `DependencyPackageConflict`
    with that exact identity and leave all evidence unchanged in both orders. Controls
    distinguish different package identities and equal package identities with independently
    changed, valid source map, diagnostics, capability disposition, graph occurrences
    or raw-source references. No same-identity or same-package-id shortcut accepts
    these conflicts.
- id: FR-047-AC-5
  path: spec/checked_package/functional/FR-047-preserve-first-evidence-entry.md
  role: examined
  excerpt: One and two exact feature declarations yield equal evidence and the same
    successful public read of an otherwise valid package requiring that feature. Declaring
    only a distinct case, whitespace, Unicode spelling or version spelling does not
    satisfy the required exact string; declarations perform no normalization or synonym
    substitution.
- id: FR-047-AC-6
  path: spec/checked_package/functional/FR-047-preserve-first-evidence-entry.md
  role: examined
  excerpt: 'Public model-crate callers handle both insert results and match the exact
    typed conflict variants/keys without message parsing. After a handled domain or
    dependency conflict, real public reading equals first-only reading in both orders:
    a valid matching first value admits, and an invalid or mismatching first value
    retains its ordinary reader outcome even when the rejected second value would
    admit. Insertion invokes no admission or encoding; inspection confirms the stated
    resource behavior and no changed reader refusal/limit contract.'
- id: FR-047
  path: spec/checked_package/functional/FR-047-preserve-first-evidence-entry.md
  role: examined
  excerpt: 'When a caller repeats an evidence declaration under an occupied key,

    `CheckedPackageEvidence` SHALL retain the first entry, succeeding for an equal

    declaration and returning a typed conflict for an unequal declaration.'
- id: FR-047/Outputs
  path: spec/checked_package/functional/FR-047-preserve-first-evidence-entry.md
  role: examined
  excerpt: 'Each insert method returns `Result<(), CheckedPackageEvidenceInsertionError>`.

    Both a vacant insertion and an equal repeat return `Ok(())`. An unequal repeat

    returns one of these closed variants:


    | Variant | Typed context |

    | --- | --- |

    | `DomainDocumentConflict { digest: Box<str> }` | The exact supplied occupied
    domain key. |

    | `DependencyPackageConflict { identity: Box<str> }` | The exact supplied occupied
    dependency key. |


    `support_feature` returns unit; it has no conflicting-value case.


    `CheckedPackageEvidenceInsertionError` is a PLANNED model-crate public export.

    When this requirement'
- id: FR-047/Exact keys and domain bytes
  path: spec/checked_package/functional/FR-047-preserve-first-evidence-entry.md
  role: examined
  excerpt: 'Keys are the exact supplied strings. Equality performs no trimming, case

    folding, Unicode normalization, parsing or synonym recognition. Domain keys

    and dependency identities occupy separate maps even if their strings match.

    Different keys are independent, including when their values are equal.


    When a domain key is vacant, the evidence builder SHALL store exactly the

    supplied byte sequence. When that key is occupied by equal bytes, the evidence

    builder SHALL return `Ok(())` without replacing the first sequence. When that

    key is occupied by different bytes, the evidence builder SHALL return

    `D'
- id: FR-047/Admitted dependency content
  path: spec/checked_package/functional/FR-047-preserve-first-evidence-entry.md
  role: examined
  excerpt: 'When a dependency identity is vacant, the evidence builder SHALL hold
    the

    supplied admitted package. When that identity is occupied by equal admitted

    content, the evidence builder SHALL return `Ok(())` without replacing the first

    package. When that identity is occupied by unequal admitted content, the

    evidence builder SHALL return `DependencyPackageConflict` carrying the exact

    identity.


    Equal admitted content is exactly the existing `CheckedPackageV2::eq`

    relation: equality of all fields of `CheckedPackageWireV2` and the decoded

    node kinds. The wire includes contract version, identity preimag'
- id: FR-047/Feature declarations
  path: spec/checked_package/functional/FR-047-preserve-first-evidence-entry.md
  role: examined
  excerpt: 'When a feature is declared, the evidence builder SHALL retain its exact
    string

    once. Repeating that exact string is idempotent. Distinct case, whitespace,

    Unicode spelling or version spelling denotes a distinct declaration and does

    not imply support for another string. Declaration adds no feature-name grammar

    validation; the reader retains its existing wire-name rules and exact

    required-feature membership check.'
- id: FR-047/Atomicity, caller handling and reading
  path: spec/checked_package/functional/FR-047-preserve-first-evidence-entry.md
  role: examined
  excerpt: 'When either insert method returns a conflict, the evidence builder SHALL
    leave

    its complete state unchanged. This includes the first entry, every unrelated

    domain and dependency entry, and the feature set. An equal repeat also leaves

    the complete state unchanged. There is no latched error or deferred conflict

    in a later read, and no partial replacement before returning an error.


    The caller must observe the `Result` and choose whether to stop assembly or

    continue using the unchanged evidence. It can match the variant and key

    without parsing a message. A handled conflict does not install the se'
- id: FR-047/Resource behavior
  path: spec/checked_package/functional/FR-047-preserve-first-evidence-entry.md
  role: examined
  excerpt: 'For E stored entries, lookup has the existing ordered-map/set O(log(E+1))
    bound

    in key comparisons, including key-length comparison costs. Domain equality

    compares at most the supplied finite byte lengths. Dependency equality uses

    the admitted-content relation directly, with work bounded by the finite

    compared content and the admitted grammar''s existing nesting bound. The

    builder SHALL compare without encoding documents/packages, cloning complete

    packages, parsing document bytes or building a second lookup/model index.

    Retaining an `Arc` expresses sharing, not a package clone.


    An equal repeat'
- id: TC-059/Domain documents
  path: spec/checked_package/matrix/TC-059-evidence-entry-repetition.md
  role: examined
  excerpt: "Build distinct valid domain documents A and B. Build otherwise reader-valid\n\
    parent package bytes selecting A under its canonical selection digest K.\nChoose\
    \ sufficient finite read limits and declare any required feature. Include\nunrelated\
    \ document, dependency and feature sentinels in every evidence fixture\nso atomicity\
    \ checks cover the entire builder state.\n\n1. Insert A under K and observe `Ok(())`.\
    \ Clone the evidence as the first-only\n   control; read the parent through `CheckedPackageV2::read`\
    \ and\n   `read_checked_package`. Require admission and compare admitted content.\n\
    \   Repeat exact A byt"
- id: TC-059/Admitted dependencies
  path: spec/checked_package/matrix/TC-059-evidence-entry-repetition.md
  role: examined
  excerpt: "Independently admit D1 and D2 with different package identities through\
    \ the\nreal reader. Build an otherwise valid parent selecting D1 under identity\
    \ I;\nensure a dependency reference resolves through that package. Keep unrelated\n\
    evidence sentinels in every fixture.\n\n1. Insert an `Arc` of D1 under I; clone\
    \ first-only evidence and require parent\n   admission. Repeat D1 in a separate\
    \ content-equal `Arc`, assert `Ok(())`,\n   evidence equality and the same admitted\
    \ parent content through both public\n   reader entrypoints. Store equal content\
    \ under separate exact identities\n   and independently selec"
- id: TC-059/Exact feature declarations
  path: spec/checked_package/matrix/TC-059-evidence-entry-repetition.md
  role: examined
  excerpt: 'Build an otherwise valid package requiring a valid exact feature F, reported

    available. After one and two exact declarations of F, compare evidence and

    admitted content through both public reader entrypoints (AC-5).


    For distinct case, leading/trailing whitespace, Unicode normalization and

    version spellings, create F2 unequal to F. Choose an otherwise wire-valid

    required F for each pair, including a Unicode pair where necessary. Declaring

    only F2 leaves F unsupported under the ordinary reader outcome. Declare F and

    F2 in both orders and compare to corresponding first-only controls: separate

    st'
- id: TC-059/Caller surface, atomicity and mutation controls
  path: spec/checked_package/matrix/TC-059-evidence-entry-repetition.md
  role: examined
  excerpt: 'External public model-crate controls receive both insertion results, match

    both variants and their exact typed keys, and compare whole evidence snapshots

    including sentinels after repeats/conflicts. After handling a conflict, insert

    an independent key and read with the preserved first value to prove the builder

    has no latched failure (AC-1 through AC-4, AC-6).


    Inspect for direct equality, single-entry storage, no insertion-time admission

    or encode, no complete package clone or second derived index, no normalization

    and no reader refusal/limit change. Verify the actual error export and update

    '
- id: Spec-index
  path: spec/spec.md
  role: examined
  excerpt: '[FR-047](checked_package/functional/FR-047-preserve-first-evidence-entry.md)

    defines the planned evidence-builder repetition policy and typed insertion

    conflicts, with planned runtime controls in

    [TC-059](checked_package/matrix/TC-059-evidence-entry-repetition.md).'
- id: CheckedPackageV2::eq
  path: crates/quire-contract-model/src/checked_package/v2/mod.rs
  role: context_only
  excerpt: "impl PartialEq for CheckedPackageV2 {\n    fn eq(&self, other: &Self)\
    \ -> bool {\n        self.wire == other.wire && self.kinds == other.kinds\n  \
    \  }\n}"
- id: Evidence-builder-current
  path: crates/quire-contract-model/src/checked_package/evidence.rs
  role: context_only
  excerpt: "    pub fn insert_domain_package_document(\n        &mut self,\n     \
    \   digest: impl Into<Box<str>>,\n        document: impl Into<Box<[u8]>>,\n  \
    \  ) {\n        self.domain_packages.insert(digest.into(), document.into());\n\
    \    }"
- id: Reader-feature-check
  path: crates/quire-contract-model/src/checked_package/v2/mod.rs
  role: context_only
  excerpt: "    for (index, feature) in wire.lock.required_features.iter().enumerate()\
    \ {\n        let path = match reported.get(feature.as_ref()) {\n            //\
    \ Unreported: the report lacks the capability.\n            None => report(),\n\
    \            Some((disposition, reported_at))\n                if *disposition\
    \ != CheckedCapabilityDisposition::Available =>\n            {\n             \
    \   report().index(*reported_at).key(\"disposition\")\n            }\n       \
    \     // Reported available, but this reader does not support it.\n          \
    \  Some(_) if !evidence.supports(feature) => {\n                member_pointer(&"
- id: Reader-dependency-binding
  path: crates/quire-contract-model/src/checked_package/v2/dependency_references.rs
  role: context_only
  excerpt: "        let Some(package) = evidence.dependency_package(&entry.identity)\
    \ else {\n            return Err(ValidationFailure::refused_because(\n       \
    \         CheckedPackageRefusalCode::MissingImport,\n                at(index),\n\
    \                CheckedPackageRefusalCause::MissingSelection,\n            ));\n\
    \        };\n        // A `CheckedPackageV2` exists only as the reader's admitted\
    \ result,\n        // so its `package_id` is the digest the reader recomputed\
    \ from its\n        // identity preimage; the binding rests on that content digest.\n\
    \        if package.package_id() != &entry.package_id {\n       "
- id: FR-038/grammar-fixed-depth
  path: spec/checked_package/functional/FR-038-consume-checked-package-v2.md
  role: context_only
  excerpt: 'Every value an admitted package holds nests to a depth the grammar fixes,
    so its

    clone, comparison, `Debug` rendering, lowering and drop

    need no stack that follows the package''s size, and the V2 reader has no `stacker`,

    `serde_stacker` or `on_stack_for`. It shall

    admit only `quire.checked-package/v2`; any other version, or a missing or'
- id: FR-019/export-inventory
  path: spec/model/functional/FR-019-rust-library-interface.md
  role: context_only
  excerpt: '| `checked_package` | function `read_checked_package`; types `CheckedArtifactLocator`,
    `CheckedArtifactRef`, `CheckedCapability`, `CheckedNodeId`, `CheckedOccurrence`,
    `CheckedOccurrenceRole`, `CheckedPackageDispatchResult`, `CheckedPackageEvidence`,
    `CheckedPackageIncomplete`, `CheckedPackageLimit`, `CheckedPackageReadLimits`,
    `CheckedPackageRefusal`, `CheckedPackageRefusalCause`, `CheckedPackageRefusalCode`,
    `CheckedSelection`, `CheckedSemanticId`, `CheckedSourceMapEntry`, `CheckedSourceRef`,
    `CheckedSourceRegion`, `JsonPointer` | FR-038 |'
```

## Evidence and limits

Reviewed frozen head eec76bc3712f5b49705a2f1b125c46d80b4368fa. Selected three-document validation exit 0. quire 0.36.2 (engine 0.50.2); quoin 0.28.3. Duplicate archetype, duplicate part_of inverse and inline semantic schema advisories were observed, not repaired or bypassed. No Cargo, Kani, replay, runtime mutation campaign, full CI or full matrix was run. No applicable AssuranceProfile was found. Exact harness model ID is unavailable.

Criterion-strength schema value is installed, but the loaded skill declares its Jev client unavailable and forbids running the classifier. Manual falsifiability review belongs to base, not a claimed Jev result. Production gap-analysis does not apply to this spec-only diff.
