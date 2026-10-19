---
id: SR-12991
title: integrity review of the TC-456 registered-code exemption matrix rows
type: SpecReview
analysis: integrity
scope: agent-ix/quire-contract-ir@1e5ddf3dfa66f92aa3fe48b082a3bac61ee5a1f5; spec/core/matrix/TC-456-public-name-portability.md, spec/core/matrix/tests.md
review_set: subset
---

## Summary

Ticket: IR-880. PR agent-ix/quire-contract-ir#474, head 1e5ddf3, diff against origin/main 3bc244a. This review checks that the amended TC-456 expected results and status, the TC-456 and NFR-002 matrix rows, and the rollups that cite TC-456 agree with each other and with the tree. It also checks that `quire validate` is clean and that strict coverage does not regress.

## Examined scope

```yaml
scope:
- id: TC-456
  path: spec/core/matrix/TC-456-public-name-portability.md
  role: examined
  excerpt: "Planned until the NFR-002-AC-1 inventory test covers the amended scope. Its existing trace tag alone does not establish that coverage. TC-019 remains planned for NFR-001's separate determinism measurements."
- id: TC-456
  path: spec/core/matrix/tests.md
  role: examined
  excerpt: "| TC-456 | ... | NFR-002-AC-1 | 🚧 planned: ... it also lacks the exact STD-001 registry-derived exemption for registered `Std001Code` constant names and values, plus mutations for an unregistered `solver` constant and a removed registry row; adverse controls must pass through the inventory extractors |"
- id: NFR-002
  path: spec/core/matrix/tests.md
  role: examined
  excerpt: "| NFR-002 | vocabulary, schema and API inspection | TC-015, TC-016, TC-456 | AC-3/4 implemented; AC-1 public-name inventory, including the exact STD-001 registered `Std001Code` exemption, planned under TC-456 |"
- id: StR-002
  path: spec/core/matrix/tests.md
  role: context_only
  excerpt: "| StR-002 | FR-016 through FR-018, FR-020 | TC-017, TC-018, TC-019, TC-456 | 🚧 existing canonicalization and coverage behavior implemented; ..."
- id: Core
  path: spec/tests.md
  role: context_only
  excerpt: "| Core | StR-001 through StR-004, FR-011, FR-012, FR-044, FR-045, NFR-001, NFR-002, NFR-003, STD-001 | [core](./core/matrix/tests.md) | 🚧 ..."
```

## Findings

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-001 | low | No findings (placeholder) | - |

## Verdict

PASS. The TC-456 status still reads planned, and the TC-456 and NFR-002 matrix rows still read planned (🚧). The new "lacks" clause of the TC-456 row names exactly what the amended procedure adds: the derived exemption, the unregistered-`solver` mutation and the removed-row mutation. The tree confirms the gap: `tests/it/public_interface.rs::tc_456_public_schema_and_model_names_are_portable` neither parses STD-001 nor applies any exemption. No row claims coverage the code lacks, and the StR-002 and Core rollups, which this PR does not change, stay consistent. `quire validate --scope . 'spec/**/*.md'` exits 0 on the head, with output byte-identical to origin/main (module warnings only). `quire coverage --scope . --strict` output is byte-identical on origin/main 3bc244a and on the head: 40 unbacked rows existed before this PR, 0 contradicted statuses, exit 1 on both. Strict coverage therefore does not regress. The procedure-level defects are recorded under SR-12990.
