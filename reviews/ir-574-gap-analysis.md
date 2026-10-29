---
id: SR-11751
title: "IR-574 Slice 1 FR-023-AC-6 gap analysis"
type: SpecReview
analysis: gap-analysis
scope: "agent-ix/quire-contract-ir; PR #412 diff origin/main...: tests/it/executable_binding.rs, spec/model/matrix/tests.md, spec/core/matrix/tests.md"
review_set: subset
---

# IR-574 Slice 1 FR-023-AC-6 gap analysis

## Summary

Ticket: IR-574. PR: quire-contract-ir#412. Plan completion was not assessed: the analysis is planless and scoped to the PR diff. The analysis compared FR-023-AC-6 with the computed Test Matrix (`quoin matrix --json`, quoin 0.28.3), the tagged test, the binder (`crates/quire-contract-model/src/binding.rs`) and the three published schemas.

## Verdict

**PASS. No gaps.**

- **Matrix.** FR-023-AC-6 has one binder, `tc_035_projection_revisions_and_offsets_obey_the_schema_boundary`, with static status `tagged`. It has no run evidence, the same as the other FR-023 criteria.
- **Behaviour.** Every clause of the AC is asserted on the public `BoundPackage::from_json_bytes` path:
  - all five members, each at 2^53+1 and `u64::MAX`;
  - refused with code `invalid_wire_format` at path `projection`;
  - `expect_err`, so no partial package is returned;
  - the three registered codes are absent;
  - the 2^53 baseline binds.
- **Code.** `decode_bounded` calls `validate_schema` before any typed decode. The package schema and the conformance schema bound `revision` and `byteOffset` at 9007199254740992. The test-oracle strength of the schema-stage claim is recorded in the code review (SR-11750 FND-001). The production behaviour is correct.
- **Matrix rows.** The prose in the core and model matrix rows matches the tested behaviour. FR-011-AC-3 and FR-012-AC-6 stay scoped to the constructors and decoders, and no other row changed.
- **Out of scope.** No corpus, probe or coverage fixture changed.

## Findings

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-001 | low | No findings (placeholder) | - |

## Examined scope

```yaml
scope:
- id: FR-023-AC-6
  path: spec/model/functional/FR-023-executable-projection-binding.md
  role: examined
  excerpt: "A projection whose package source revision, package requirement revision, binding clause reference revision, expression source-identity revision or expression byte offset is 9007199254740993 (and, for each, `u64::MAX`) is refused `invalid_wire_format` at path `projection`, with no partial BoundPackage, and never `invalid_requirement_revision`, `invalid_source_revision` or `invalid_source_span`; the same projection with that member at 9007199254740992 binds."
- id: spec/model/matrix/tests.md#FR-023
  path: spec/model/matrix/tests.md
  role: examined
  excerpt: "| FR-023 | FR-023-AC-1 through FR-023-AC-6 | TC-035 | ✅ implemented: AC-1 through AC-5 and AC-6... (TC-035). |"
- id: spec/core/matrix/tests.md#FR-011-FR-012
  path: spec/core/matrix/tests.md
  role: examined
  excerpt: "(FR-023-AC-6, implemented in TC-035)"
- id: FR-011-AC-3
  path: spec/core/functional/FR-011-package-identity.md
  role: context_only
  excerpt: "a whole executable projection is FR-023-AC-6's"
- id: FR-012-AC-6
  path: spec/core/functional/FR-012-anchors-clauses-dependencies.md
  role: context_only
  excerpt: "a whole executable projection refuses such an offset `invalid_wire_format` at its schema stage (FR-023-AC-6)"
```
