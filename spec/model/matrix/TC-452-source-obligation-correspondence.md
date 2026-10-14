---
id: TC-452
title: "Verify the neutral source-obligation correspondence input"
type: TC
relationships:
  - target: ix://agent-ix/quire-contract-ir/FR-052
    type: verifies
---
# TC-452: Verify the neutral source-obligation correspondence input

## Description

Verify that the typed model port retains complete authoritative inventories,
accepts only explicit source-author assertions, and deterministically records
valid, missing, foreign, dangling, duplicate, and conflicting relations.

## Test Procedure

Compile an external default-feature consumer importing the listed items from
`quire_contract_model::source_obligation` and attempting the same imports from
`quire_contract_ir`. Construct a fixture with two criteria, three semantic
occurrences, an empty module, two production functions and explicit
many-to-many assertions. Inspect every inventory row and accepted relation.
Repeat with an unmapped criterion, occurrence and symbol; a foreign inventory
item; a foreign assertion; each dangling endpoint; exact duplicate inputs;
and conflicting identity facts and provenance. Permute each equivalent input
set and compare the census. Remove one occurrence and then vary or omit proof
result, method and profile values. Inspect Cargo manifests and public exports
for ownership and dependency direction.

## Expected Results

The model path compiles and the IR root path does not. Every source inventory
entity remains inspectable, only explicit valid assertions are accepted, and
missing mappings remain typed gaps. Foreign and dangling input is preserved
with typed rejections; exact duplicates collapse; conflicts are retained with
stable typed dispositions. Equivalent input order produces equal ordered
censuses. Removing an authoritative occurrence changes the census, while
proof-result and method/profile changes cannot affect it. The model has no CG,
QSL, parser, or proof-runner dependency, and the IR root has no dependency on
CG.
