---
id: SR-2026103004
title: "IR-1141 PR #726 spec-review/base review"
type: SpecReview
analysis: base
scope: "agent-ix/quire-contract-ir; spec/checked_package/functional/FR-038-consume-checked-package-v2.md"
review_set: base
---

## Summary

Independent spec review of PR #726 at frozen head 6ec3467be4090b0ab83b0b9257b0a452fb137fa7. The sole edit deletes the obsolete nominal-preimage refusal from FR-038-AC-241. Current QSpec FR-322 describes structural node IDs and the protocol-reference carrier without any nominal-preimage carrier; QSpec FR-322-AC-66 and FR-464-AC-15 retain the closed-body, reference, and recursion behavior. The separate `nominal_structure` prohibition in QSpec FR-322 is a wire member rule, not a nominal preimage.

## Verdict

**READY** — no defect in the changed phrase. Declaration and owner refusals, closed reference body, cycle/group controls, and exact target/reference controls remain verbatim in AC-241; AC-239/240/294/295 still define admission, target join, and occurrence controls. The criterion retains TC-048 and TC-946 verification references and existing code trace tags. No new hash, digest, pin, compatibility layer or copied artifact appears.

## Scope

- `FR-038-AC-239` context_only: IMPLEMENTED (IR-782; reverified IR-1025). A package with a `protocol`/`protocol_reference` carrier whose closed reference body targets an exact local `protocol` node, with one `expression` occurrence, the same semantic type and exactly that target as its dependency, admits through the public V2 reader as `ProtocolForm::ProtocolReference`. Both a `protocol_clause` target and a different protocol form (`role` or `control`) admit; a reader that accepts only `protocol_clause` fails the second control. For each admitted target, the retained structural id and complete package value are compared with
- `FR-038-AC-240` context_only: An extra dependency or one unequal to a protocol-reference carrier body target refuses `invalid_semantic_graph` at its `dependencies`. With body target and sole dependency changed together, an absent local target refuses `missing_declaration`/`missing-name` at `/body/target`, even if a selected dependency has that id. A non-protocol target or unequal semantic type refuses `ill_typed`/`operator-ineligible` there. Carrier target defects precede frame and operation defects.
- `FR-038-AC-241` examined: IMPLEMENTED (IR-1025; QSpec FR-464-AC-15). A carrier with a non-reference or `dependency_reference` body or an extra body member fails the closed V2 schema. A carrier with a declaration or owner refuses at its offending member before target resolution. A carrier with a `recursion_group` outside a cycle refuses `malformed_wire` during graph validation; a cyclic component containing a carrier uses one shared non-empty label and exact structural IDs/references for every member, including non-protocol nodes. The predicate-join function's reference to its own protocol includes any carrier and non-p
- `FR-038-AC-294` context_only: A public-reader package with one local protocol target and one carrier admits after adding second and third `expression` occurrences with distinct ordinals and exact source-map rows. The one carrier keeps its structural node id and body `{term: "reference", target}`; its occurrences and source regions remain distinct. No identity discriminator or fixed two-occurrence maximum is added.
- `FR-038-AC-295` context_only: From that admitting repeated-reference package, a duplicate occurrence ordinal or source-map key refuses `invalid_source_map` at the later occurrence or row. Empty carrier occurrences refuse at `/occurrences`; an invalid role refuses `invalid_semantic_graph` at `/role`; a second graph node with the same structural carrier id refuses at the later `/node_id`. Every mutation is independent.

QSpec context (current main 686e49e812ecf88b89618e00cddddea32db84b90): FR-322 carrier body and prohibited members, FR-322-AC-66, and FR-464-AC-15 examined for cross-repo consistency. No applicable AssuranceProfile found in the changed module. The edited unit is an acceptance criterion, so EARS requirement-statement analysis does not apply; no structural, object, dependency, or other analysis lens is triggered by this single phrase deletion.

## Findings

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-001 | low | No findings (placeholder) | - |
