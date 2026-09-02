---
id: PLAN-004
title: "Migrate to the shared assurance evidence contract"
type: Plan
status: done
relationships:
  - target: ix://agent-ix/quire-contract-ir/issues/39
    type: references
  - target: ix://agent-ix/quire-contract-ir/TASK-013
    type: contains
  - target: ix://agent-ix/quire-contract-ir/TASK-014
    type: contains
  - target: ix://agent-ix/quire-contract-ir/TASK-015
    type: contains
---

# PLAN-004: Migrate to the shared assurance evidence contract

## Dependency DAG

```text
TASK-013 inventory, pins, and specification
  -> TASK-014 shared intake, legacy compatibility view, and state fixtures
    -> TASK-015 dual-run at one candidate revision, then deletion
      -> review artifacts and authorized admin merge
```

## Plan Delta

- Reach Quire, Quoin, ix-flow, and Engineering Assurance only at the accepted
  released pins, classified by Engineering Assurance's own matrix rather than by
  a second copy of its rules kept here.
- Deliver the native conformance result to Quoin through the declared
  `contract-conformance` adapter, and retain the exact producer bytes through
  `quoin change-assurance`. Neither Quire nor Quoin runs a producer.
- Read the ten immutable PGM-01 records and the retained correction only
  through Engineering Assurance's read-only compatibility mapping.
- Demonstrate pass, fail, unavailable, not-computed, malformed, partial, stale,
  suspect, vacuous, tampered, unsupported, and inconclusive, each with a case,
  and prove none collapses into another.
- Delete the repository-local evidence verifier and its tests, last, in a
  separate commit.
- Freeze the two generic PGM-01 schemas in place rather than deleting them:
  every immutable record names one of them by path and digest, and the family
  the migration removes is the verifier, not the reference those bytes carry.
- Change no workflow trigger. Hosted CI stays manual-dispatch only.
- Change no domain behavior: the contract model, expressions, definedness,
  canonical bytes, digests, corpus, and diagnostics are untouched.

## Dual-run result

The playbook requires the shared path to pass at the same candidate revision
before the old path is deleted. At `8e0953e`:

| Path | Result |
| --- | --- |
| `python3 scripts/verify_evidence.py` | exit 1 — no retained record matches the candidate; each of the ten is rejected because its subject tree differs from `HEAD` |
| `scripts/pgm01_compatibility_view.py` | exit 0 — all ten read, every one `lossy`, every source digest preserved, 0 bytes moved |

The old path does not pass, and has not since `4634eda`, the revision its
newest record was minted for: two squash merges later, no retained record's
subject tree equals `main`'s. A whole-tree retention model cannot survive a
branch that moves, which is the concrete reason this repository is not the right
owner for retention. The shared path reads every record the old one could not
use at all, so deletion removes a gate that was already red rather than one that
was working.

## Exit Criteria

All three tasks are done; every FR-022 and revised FR-009 criterion resolves to
a passing requirement-tagged test; `make ci` and `make spec` pass at the exact
merge head; the twelve required states each have a demonstrated case; mutation
probes turn every load-bearing check red; no verdict is read from a console
stream; every byte under `evidence/` is unchanged; and no hosted workflow is
dispatched.
