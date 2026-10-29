---
id: SR-12981
title: integrity review of the amended TC-456 status and matrix row
type: SpecReview
analysis: integrity
scope: agent-ix/quire-contract-ir; spec/core/matrix/TC-456-public-name-portability.md, spec/core/matrix/tests.md
review_set: subset
---

## Summary

Ticket: IR-880. PR agent-ix/quire-contract-ir#473,, original frozen candidate. This review checks that the amended TC-456 procedure, expected results, status, matrix row and every other reference to TC-456 or NFR-002-AC-1 agree with each other and with the tree. It also checks that strict coverage does not regress.

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
  excerpt: "| TC-456 | Public schema and model names exclude implementation-specific vocabulary | Unit | P0 | NFR-002-AC-1 | planned: the tagged source/schema inventory test exists, but does not yet cover schema `const` values, feature-gated exports, or public fields, enum variants and associated functions of exported types |"
- id: NFR-002
  path: spec/core/matrix/tests.md
  role: examined
  excerpt: "| NFR-002 | vocabulary, schema and API inspection | TC-015, TC-016, TC-456 | AC-3/4 implemented; AC-1 public-name inventory planned under TC-456 |"
- id: StR-002
  path: spec/core/matrix/tests.md
  role: examined
  excerpt: "| StR-002 | FR-016 through FR-018, FR-020 | TC-017, TC-018, TC-019, TC-456 | existing canonicalization and coverage behavior implemented;... NFR-002-AC-1 public-name inventory is planned |"
- id: Core
  path: spec/tests.md
  role: examined
  excerpt: "NFR-001..NFR-003 and STD-001 are partial: cross-platform determinism (TC-019), the public-name inventory (TC-456), and the rest of `kani_outcome_invalid`'s raise sites (TC-223) are planned"
- id: NFR-002-AC-1
  path: spec/core/non-functional/NFR-002-portability-independence.md
  role: context_only
  excerpt: "Public schema and model names contain no Rust-layout, architecture-language, solver, runtime, pointer-width, or operating-system-path vocabulary. | Test (TC-456)"
```

## Findings

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-001 | low | No findings (placeholder) | - |

## Verdict

PASS. The TC-456 status, the core matrix TC-456 row, the NFR-002 and StR-002 rollup rows, the top-level `spec/tests.md` Core row and `spec/spec.md` all describe the public-name inventory as planned, so no reference claims coverage the code lacks. The amended row's list of uncovered kinds (const values, feature-gated exports, fields, variants, associated functions) matches the procedure text. That list is accurate on the tree: `collect_schema_names` reads no `const`, and `exported_items` skips `output_mapping::MappingAllocationPoint`. `quire validate --scope.` is clean on both changed files. `quire coverage --scope. --strict` output is byte-identical on `origin/main`  and on the head (40 unbacked rows existed before this PR, 0 contradicted statuses), so strict coverage does not regress. Every scope gap in the procedure text is recorded under SR-12980.
