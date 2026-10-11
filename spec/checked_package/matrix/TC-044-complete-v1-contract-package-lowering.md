---
id: TC-044
title: "Complete-V1 target-neutral ContractPackage lowering conforms"
type: TC
relationships:
  - target: ix://agent-ix/quire-contract-ir/FR-035
    type: verifies
  - target: ix://agent-ix/quire-specification/TC-217
    type: references
---
# TC-044: Complete-V1 target-neutral ContractPackage lowering conforms

## Description

Verify FR-035-AC-1 through FR-035-AC-4 and FR-035-AC-9 against the FR-330 package contract and
FR-322's selected-dependency and tagged-reference rules. Focused FR-330
emitter and reader controls run in TC-047; the broader TC-044 cases remain
to be verified against the completed change.

## Test Procedure

1. Lower independently constructed complete-V1 requests for every
   V1-BACK-001 through V1-BACK-004 and V1-BACK-014 family. Compare each
   represented node with its admitted native meaning and its source locus.
2. Submit a mixed request containing exact, unsupported, unbounded, malformed
   and unavailable items. Reorder the requests and mutate only one item at a
   time. Inspect all seven possible per-item dispositions and each independent
   sibling's node, record and source correspondence. Repeat at the exact
   package byte ceiling and one byte over it.
3. Mutate source, semantic type, anchor, node identity, bound, selected
   dependency identity and contract version independently. For a selected
   dependency mutation, keep an identically keyed local node present and
   check that it cannot satisfy the external reference. Also omit the selected
   dependency package and test a wrong node in the selected admitted package.
4. Replace a local semantic reference with a key naming an existing but
   incompatible local node. Test both a valid explicit FR-322
   same-group recursion and a cycle with no common explicit group. Resolve
   each surviving local key and each external dependency reference at its
   specified boundary.

## Expected Results

Exact items preserve meaning and source correspondence. Each refused item
has one located record from the closed seven-member vocabulary and no
substitute node or backend artifact. A single item mutation leaves independent
siblings unchanged; exceeding the package byte ceiling fails every record.
Wrong-family local targets refuse even when their keys exist.
Stale, absent or wrong selected dependencies refuse without resolving through
local nodes. Explicit same-group recursion admits as keyed references, while
an undeclared cycle refuses. Every admitted local reference targets a
compatible local node and every admitted dependency reference targets only its
selected admitted package.


## Typed source-selection vectors

Use these exact FR-322 `Selection` rows (the short labels below denote typed
values, never identifier-parsing rules):

```json
{
  "T": {"role": "temporal_profile", "definition": {"authority": "agent-ix", "identity": "quire.temporal.infinite-trace/v1"}},
  "D": {"role": "temporal_profile", "definition": {"authority": "agent-ix", "identity": "quire.temporal.event-position.false-extension/v1"}},
  "U": {"role": "protocol_profile", "definition": {"authority": "agent-ix", "identity": "quire.protocol.finite-global/v1"}}
}
```

Construct independently admitted FR-322 checked packages with the selected rows
below and graph meaning valid under their own selections; do not mutate an
admitted value's lock. Retain each input's exact package id and full lock as the
independent expected output provenance. Use two distinct existing requested keys
`a`, `b` with available source occurrences, in request order `[b, a]`; both can
otherwise lower under the resolved profile's supported tags/bounds. Use resource
limits sufficient for comparison and encoding. Its explicit `reference` is
unchanged across the rows.

| Case | `required_source_selections` | Input `lock.profile_selections` | Expected result |
| --- | --- | --- | --- |
| matching | `[T]` | `[T]` | Both requested items lower; source lock and package id equal that input exactly. |
| missing | `[T]` | `[]` | Exactly `invalid_input(b)`, `invalid_input(a)`; no semantic nodes; both exact request occurrence sets retained. |
| differing identity | `[T]` | `[D]` | The same two `invalid_input` records; no substitute `T`, no node or backend artifact; output still retains `[D]` and its originating package id. |
| unrelated selection | `[T]` | `[U, T]` | Both items lower; output retains `[U, T]` in that order, including the unrelated row. |

Repeat the unrelated case with lock order `[T, U]` in a separately admitted
package: compatibility remains true and the output preserves that input's own
order/id, rather than equating the two package identities. Repeat with requirements
`[T, T]` and `[U, T]`/`[T, U]`: duplicate and order changes do not change
compatibility. With requirements `[T, D]`, lock `[T]` refuses and an independently
admitted lock containing both rows matches. Repeat a mismatch with only an
unrelated row `[U]` to prove it cannot satisfy `T`. With requirements `[]`, the
`[D]` input lowers without inferred requirements or lock repair.
For a same-identity/different-authority control, require the `T` row with only
its authority changed to `other-authority`; the original `[T]` input refuses
rather than matching its display identity alone. In every refusal, the serialized
record has only `node_id` and `disposition: "invalid_input"`; no fabricated
cause/pointer is added. Compare the entire emitted source lock and originating
package id against the retained expected values on every successful and refused
run, not merely the row used by the predicate. These are producer controls; the
ContractPackage reader is not given or assumed to reconstruct requirements.

## Status

Planned / unrun for the FR-330 package emitter. Existing tests exercise the
current IR package and do not establish conformance to this revised wire.
