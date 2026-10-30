---
id: TC-059
title: "Check first-entry retention for repeated evidence declarations"
type: TC
relationships:
  - target: ix://agent-ix/quire-contract-ir/FR-047
    type: verifies
---
# TC-059: Check first-entry retention for repeated evidence declarations

## Description

Verify the six criteria of
[FR-047](../functional/FR-047-preserve-first-evidence-entry.md) through the
public model-crate evidence builder and reader. This artifact is a test
procedure, not a runtime binder or computed coverage result. The Result API and
named insertion-error export are implemented, and the error appears in FR-019's
implemented Public items table. Six traced tests in
`tests/it/checked_package_evidence_insertion.rs` exercise the public builder and
both reader entrypoints. The focused run passed all six tests; the scoped
Clippy check of the root integration-test target also passed. Neither result
is a full repository gate result.

Use repository-owned fixture builders and the existing canonical library.
Do not copy another repository's artifacts or create a test encoder. Runtime
tests must carry each criterion they actually assert as a `Trace:` doc line
or existing trace attribute. Selection-row duplicate tests do not exercise
these repeated builder calls.

## Test Procedure

Run the following independent fixtures through the public builder and reader;
each control checks the criterion IDs named beside it, without substituting a
selection-row test for an insertion call.

### Domain documents

Build distinct valid domain documents A and B. Build otherwise reader-valid
parent package bytes selecting A under its exact package name K.
Choose sufficient finite read limits and declare any required feature. Include
unrelated document, dependency and feature sentinels in every evidence fixture
so atomicity checks cover the entire builder state.

1. Insert A under K and observe `Ok(())`. Clone the evidence as the first-only
   control; read the parent through `CheckedPackageV2::read` and
   `read_checked_package`. Require admission and compare admitted content.
   Repeat exact A bytes under K, assert `Ok(())` and evidence equality with
   the control, then read again and compare full results (AC-1).
2. Store equal bytes under separate exact keys and observe later insertion
   behavior at each key independently: equal bytes succeed and distinct bytes
   conflict with that key's exact payload. A document supplied under a different key does not satisfy the parent name selection;
   independent storage is proved by these builder outcomes. A document key
   string and dependency identity string also occupy separate maps (AC-1).
3. Run A-then-B and B-then-A under K in separate fresh fixtures. Clone each
   first-only control; require `Err(DomainDocumentConflict { identity })` with
   `identity == K` and whole-evidence equality after conflict. Read control and
   post-conflict evidence with the same parent bytes and limits. A-first must
   admit; B-first retains the ordinary selected-document body or owner mismatch even though
   rejected A would admit. Match the existing reader code/location contract;
   do not assign new causes to insertion (AC-2, AC-6).
4. Produce byte-distinct encodings of equal JSON, such as canonical and
   whitespace-modified bytes. Assert raw inequality and parsed-value equality
   as fixture preconditions. Insert under one key in each order and require
   the typed conflict and unchanged evidence; do not normalize inside the
   insertion under test (AC-2).
5. Store malformed JSON under a vacant claimed domain key and require
   insertion success. Compare a later public read to first-only evidence;
   the reader owns the malformed-document outcome. Attempt conflicting bytes
   under that key and require insertion conflict without admission. Run
   malformed-first/valid-first directions to prove no replacement or delayed
   conflict (AC-2, AC-6).

### Admitted dependencies

Independently admit D1 and D2 with different package identities through the
real reader. Build an otherwise valid parent selecting D1 under identity I;
ensure a dependency reference resolves through that package. Keep unrelated
evidence sentinels in every fixture.

1. Insert an `Arc` of D1 under I; clone first-only evidence and require parent
   admission. Repeat D1 in a separate content-equal `Arc`, assert `Ok(())`,
   evidence equality and the same admitted parent content through both public
   reader entrypoints. Store equal content under separate exact identities
   and independently select each with valid parent packages (AC-3).
2. Independently admit the same content under different sufficient byte
   ceilings L1 and L2. Assert full content equality and distinct `Arc`
   allocations. Hold caller Arcs and measure strong counts with no other
   clones in scope, insert the first and repeat the second. The first gains
   and retains the evidence-held strong reference; the second gains no
   retained reference. Reverse ceilings and insertion order. This observation
   distinguishes equal-package replacement that evidence equality would
   miss because ceilings are excluded. The original immutable object retains
   its ceiling/index/model state; parent admission is not claimed to observe
   dependency ceilings (AC-3).
3. Run D1-then-D2 and D2-then-D1 under I. Clone each first-only control; match
   `Err(DependencyPackageConflict { identity })` with `identity == I` and
   unchanged complete evidence. Parent reading equals its first-only control:
   D1-first admits, D2-first retains the ordinary reader outcome for its selected name and content.
   Exercise both public reader entrypoints with identical bytes/limits
   (AC-4, AC-6).
4. Produce independently admitted pairs with equal package `name` but different
   complete content, one dimension per pair: valid source-map regions, diagnostics,
   capability disposition, occurrences with matching valid source-map rows,
   and raw-source references updated consistently. Assert independent
   admission, equal package names and unequal full content. Run each pair
   in both orders under I; require exact conflict/key and unchanged complete
   evidence (AC-4). Parent dependency reading may admit either metadata
   variant, which alone is not a retention oracle. Builder variant/state
   assertions must distinguish the metadata conflict.

### Exact feature declarations

Build an otherwise valid package requiring a valid exact feature F, reported
available. After one and two exact declarations of F, compare evidence and
admitted content through both public reader entrypoints (AC-5).

For distinct case, leading/trailing whitespace, Unicode normalization and
version spellings, create F2 unequal to F. Choose an otherwise wire-valid
required F for each pair, including a Unicode pair where necessary. Declaring
only F2 leaves F unsupported under the ordinary reader outcome. Declare F and
F2 in both orders and compare to corresponding first-only controls: separate
strings coexist, F is then supported, and repeating either changes nothing.
An invalid wire feature name must not make an exact-string control pass (AC-5).

### Caller surface, atomicity and mutation controls

External public model-crate controls receive both insertion results, match
both variants and their exact typed keys, and compare whole evidence snapshots
including sentinels after repeats/conflicts. After handling a conflict, insert
an independent key and read with the preserved first value to prove the builder
has no latched failure (AC-1 through AC-4, AC-6).

Inspect for direct equality, single-entry storage, no insertion-time admission
or encode, no complete package clone or second derived index, no normalization
and no reader refusal/limit change. Verify the actual error export and update
its implemented inventory only with implementation (AC-6).

Assess sensitivity to unconditional replacement, replacement on equal repeat,
package-id-only equality, Arc-address equality, parsed/canonical document
equality and feature normalization. Replacement fails result/state and
both-order reader controls; equal-package replacement fails held-Arc counts;
package-id-only equality fails metadata pairs; pointer equality fails equal
separate Arcs; document normalization fails byte-distinct JSON; feature
normalization fails the exact-string adverse controls. This defines required
discrimination without claiming a mutation campaign ran.

## Expected Results

Vacant and equal inserts succeed. Unequal inserts return the exact typed
family/key and preserve full evidence, in either order. Document equality is
raw bytes; dependency equality is full existing admitted content; equal
admissions under different ceilings retain the first object. Exact feature
repeats change nothing and distinct strings never imply each other's support.
Handled conflicts preserve the real reader's first-only positive or adverse
outcome without introducing a reader cause or altering precedence.

The traced public controls exercise these insertion-result and retention
outcomes. Selected Markdown validation is structural evidence only; it does
not establish runtime behavior or replace the focused test results.
