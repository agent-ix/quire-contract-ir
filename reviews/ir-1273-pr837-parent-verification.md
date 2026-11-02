---
id: SR-2026110283706
title: IR-1273 PR 837 parent verification coverage finding
type: SpecReview
analysis: integrity
scope: agent-ix/quire-contract-ir; spec/checked_package/functional/FR-038-consume-checked-package-v2.md
review_set: subset
---

## Summary

Additional parent verification finding against frozen 0328065cf043fef3211694bced70a429c03b8a27. This supplements the original five immutable PASS artifacts; their semantic conclusions remain unchanged. Native extraction receipt: /tmp/ir1273-properties.json.

## Examined scope

```yaml
scope:
- id: FR-038-AC-435
  path: spec/checked_package/functional/FR-038-consume-checked-package-v2.md
  role: examined
  excerpt: The closed source reference requires `authority`, `identity`, `source_integrity`;
    the last is lowercase hexadecimal SHA-256 over exact raw bytes. Matching ASCII,
    non-UTF-8, empty and newline-distinct sources admit through the public reader
    with caller-supplied evidence; writer output preserves the member. Definition
    refs and locators remain two-member shapes.
```

## Findings

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-001 | medium | AC-435 through AC-440 sit after intervening prose outside the acceptance-criteria table; native quire properties extraction emits none of these six logical IDs despite validation success. Move the rows into the actual table so their planned obligations and Test methods are minted. | spec/checked_package/functional/FR-038-consume-checked-package-v2.md:5629 |

## Verdict

CHANGES REQUIRED: restore native acceptance-criterion extraction before handoff. The earlier structural validation pass did not prove these obligations were minted.
