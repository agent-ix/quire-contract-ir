---
id: SR-800
title: "code review of PR 256 (IR-486 recursive equality and the recursion leaf)"
type: SpecReview
analysis: code-review
scope: "agent-ix/quire-contract-ir@c832ea54d6528369ac8de8f554d8b0d0bcd5d51b; Cargo.toml, Cargo.lock, crates/quire-contract-model/src/checked_package/v2/operations.rs, tests/it/checked_package_v2_recursive_leaves.rs, tests/it/main.rs"
review_set: subset
---
# SR-800: code review of PR 256

## Summary

Ticket: IR-486. Reviewed head c832ea54d6528369ac8de8f554d8b0d0bcd5d51b. The base is
current: the merge base equals origin/main ccf34fe57c0837bff56503568c7f03c732779a59.
The rust-review lane is folded into this file. Every check was run in a separate detached
worktree, not taken from the PR body.

Walk. `LeafWalk` keeps `OpenComposites` (the record and tuple nodes on the current path,
each with the segment count it was entered at). An edge to an open composite is a
reentry. It is not followed. It counts 1 when `component_text` says text is reachable
from the reentered composite's component, and 0 otherwise. The `Cycle` refusal and
`on_stack` are gone. Options and collections are noted in `noted` with the open-composite
count at entry. The note is popped in `leave`. Reaching the node again with the same count
refuses `Unresolved`, which maps to `ill_typed`/`operator-ineligible`.

Memo. `closed = !open.touches(component)`: a memo applies, and is stored, only when no
composite of the node's strongly connected component is open. This is exactly the spec's
condition. An open composite is an ancestor on the path, so it reaches the node, and if the
node also reaches it, both are in one component.

Components. `analyse` is iterative Tarjan over `calls`/`members` heap stacks. Edges to
nodes still on the member stack lower `low` by the target's discovery number. The child's
`low` is propagated to the parent on pop. A component closes at `low == number`.
`component_text` is computed at close, from members that are `text` and from edges into
already closed components (reverse topological order). It charges one unit per distinct
reachable node in `discover`. No function in the new code calls itself directly or
indirectly.

Pass. `first_leaf_fault` resets the open set. It opens each composite at `path.len()`,
so `d` is the number of segments when the composite was entered (Node 0, `Option<Node>` 1,
`Two` 1, `Wrap` 1). It places text leaves and recursion leaves in walk order. A recursion
leaf where a text leaf is due, a wrong prefix, a wrong `d`, or a leftover recursion leaf
faults at its `path`. A leftover text leaf faults at `operation.leaves/<i>`. A recursion
leaf with laws faults at `laws`. After the unselected-law check, a recursion leaf with a
mode refuses `operation-mode-mismatch` at `mode`. Text-leaf mode checks skip recursion
leaves. The count runs first (`len < derived` gives `operation-law-missing`).

Kept refusals: missing node, malformed node, text with no pin, unselected law,
wrong/missing/extra leaves, budget exhaustion. All are still present and still tested.
No compatibility layer, no pin, SHA or vendored copy, and no QSL dependency, in tests or
elsewhere. The `quire-verification-contracts` root dev-dependency reuses the model crate's
existing `branch = "main"` git dependency. It is one lock entry, and `make deny`, including
the one-copy awk gate, passes. No `unwrap`, `expect` or `panic` was added to production
code. `git diff --check` is clean.

Tests. Model lib, operations filter: 60 passed. `tests/it` recursive_leaves: 4 passed in
30.6 s debug; the 20000-record case is almost all of that. Mutation probes, each reverted:

- reintroduce the Cycle refusal at a reentry: 13 unit and 3 it tests fail;
- drop the recursion-leaf requirement (`reaches_text` always false): 10 unit and 2 it fail;
- memo regardless of the open set: Wrap, the work-unit test and the ten-record it test fail;
- never drop the wrapper note: the shared-option test, the work-unit test and ten-records fail;
- `d` off by one: the Option<Node>, Two and Wrap tests fail;
- remove the component-search charge: all 100 lib and 183 it tests pass (see SR-801).

## Findings

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-001 | low | Process: the coder reports one `sed -i` edit to a test file, which breaks the Edit-only rule, and says it was restored. The committed result is clean: `git diff --check` passes, the gate's `fmt-check` and clippy `-D warnings` pass, and the whole test file was read at the reviewed head. Recorded so the lapse stays on record; no code change is needed | tests/it/checked_package_v2_recursive_leaves.rs |

## Verdict

Approve. The walk matches FR-038 "Recursive compared types" clause for clause. It is
iterative, with no call-stack recursion. The SCC-based memo condition is correct and pinned
by the Wrap test and the ten-record test. Every requested mutation is killed. The one
finding is process only. The 20000-record test takes about 30 s in debug. The repo has no
slow-test lane: `make test` runs `--include-ignored` and no test is `#[ignore]`. So keeping
it in the default lane matches repo convention, and AC-72 requires that size.

## Dispositions

Round 1 at e44305c9dd30c38062d7bac75af5c2a7e2c223c0 (base still origin/main ccf34fe57c0837bff56503568c7f03c732779a59). The delta adds one unit test, one FR-038 clause and one tests.md edit, with no production code change. The gate log ends head=e44305c… exit=2, and only spec fails: 23 unbacked, 166/212, FR-038 45/70, re-measured with quire coverage. Lib 101 and it 184 pass. No new findings.

| FND | Outcome | sha/reason |
| --- | --- | --- |
| FND-001 | accepted-no-change | Process-only finding; there was never a code defect to fix. Re-checked at e44305c: git diff --check origin/main...HEAD is clean, and the gate's fmt-check and clippy pass (ir-486-r2-ci.log). The delta commit touches only operations.rs tests, FR-038 and tests.md. |
