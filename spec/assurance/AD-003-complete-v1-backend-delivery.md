---
id: AD-003
title: "Complete V1 target-neutral IR and backend delivery architecture"
type: ArchitectureDescription
status: proposed
owner: kreneskyp
system: complete-V1 Contract IR, runtime, codegen, Kani, replay, and output mappings
relationships:
  - target: ix://agent-ix/quire-contract-ir/AD-001
    type: references
  - target: ix://agent-ix/quire-contract-ir/AD-002
    type: references
  - target: ix://agent-ix/quire-specification/AD-004
    type: references
  - target: ix://agent-ix/quire-specification/AD-010
    type: references
  - target: ix://agent-ix/quire-specification/AD-016
    type: references
---
# Complete V1 target-neutral IR and backend delivery architecture

## System Boundary

Contract IR owns the cycle-free, versioned `ContractPackage` representation and
the exact lowering boundary. Runtime owns typed native oracle execution; codegen
owns deterministic provider generation, bounded Kani harnesses, and replay
adapters. Output mapping remains a derived-output boundary. Native Quire remains
the sole source and semantic authority.

## Views

```text
checked native package -> exact per-item lowering -> ContractPackage
ContractPackage -> capability negotiation -> runtime oracle | codegen provider
codegen provider -> bounded Kani/artifacts/results -> canonical replay envelope
canonical replay envelope -> codegen replay adapter -> QSL complete-V1 executor -> parity or typed failure
ContractPackage -> output mapping package -> OCL | SysML/KerML | FRETish outputs
```

Every request item has one terminal lowering and provider record. A record is
`lowered`, `unsupported`, `requires_bound`, `invalid_input`, or `failed` at the
lowering boundary; provider negotiation uses `supported`, `requires_bound`,
`unsupported`, or `invalid_request` before any artifact exists. A rejected item
has no substitute node, harness, generated text, or partial package. Sibling
items remain independently accounted for.

## Decisions

| Choice | Rejected alternative | Consequence |
| --- | --- | --- |
| ContractPackage is core typed data | A test-only model or backend-local wire shapes | Runtime and providers share one cycle-free identity-bearing contract. |
| Stable identities, not positions, bind nodes | Array index or source-spelling joins | Reordering cannot alter semantic reference resolution. |
| Exact negotiation before emission | Best-effort generation | Unbounded or unsupported meaning produces no approximating artifact. |
| Canonical replay through the QSL complete-V1 executor | Opaque backend diagnostics | Refutation evidence is independently checkable against the same package and domain. |
| Target mappers consume the shared IR seam | Separate target source authorities | OCL, SysML/KerML, and FRETish stay derived outputs with explicit loss records. |

## Risks

| Risk | Control |
| --- | --- |
| A complete-V1 node is dropped or narrowed | TC-044 mutation and round-trip vectors cover every family and per-item accounting. |
| A Kani bound is invented by a backend | Model-domain derivation, exact tool/options lock, and `requires_bound` refusal. |
| Backend and native verdicts differ | TC-046 canonical replay retains a typed parity failure. |
| A generated target becomes source authority | I16 mapping packages remain output-only and retain source/profile correspondence. |
