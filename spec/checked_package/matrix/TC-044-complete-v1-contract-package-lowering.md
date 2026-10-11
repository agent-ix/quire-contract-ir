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

Verify FR-035-AC-1 through FR-035-AC-4 against the FR-330 package contract and
FR-322's selected-dependency and tagged-reference rules. The FR-330 emitter
cases are planned and have not run.

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
   incompatible local node, and replace a protocol reference with a key naming
   an existing non-`protocol` node. Test both a valid explicit FR-322
   same-group recursion and a cycle with no common explicit group. Resolve
   each surviving local key and each external dependency reference at its
   specified boundary.

## Expected Results

Exact items preserve meaning and source correspondence. Each refused item
has one located record from the closed seven-member vocabulary and no
substitute node or backend artifact. A single item mutation leaves independent
siblings unchanged; exceeding the package byte ceiling fails every record.
Wrong-family local and protocol targets refuse even when their keys exist.
Stale, absent or wrong selected dependencies refuse without resolving through
local nodes. Explicit same-group recursion admits as keyed references, while
an undeclared cycle refuses. Every admitted local reference targets a
compatible local node and every admitted dependency reference targets only its
selected admitted package.

## Status

Planned / unrun for the FR-330 package emitter. Existing tests exercise the
current IR package and do not establish conformance to this revised wire.
