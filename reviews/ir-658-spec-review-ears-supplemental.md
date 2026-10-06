---
id: SR-2300
title: "Supplemental EARS review of IR-658 projection-owner refusal"
type: SpecReview
analysis: ears-conformance
scope: "agent-ix/quire-contract-ir@6f99c1f878af05da1fd24951c96e0e466a154fed; spec/checked_package/functional/FR-038-consume-checked-package-v2.md; spec/checked_package/matrix/TC-228-checked-package-v2-wire-owner-and-lock-joins.md"
review_set: subset
---

## Summary

Supplemental independent Codex EARS review for IR-658 SPEC PR #312, at artifact-inclusive head `6f99c1f878af05da1fd24951c96e0e466a154fed` against IR main `3ed1f7ceeb682745629d2b99fd517d30689e87fb`. Session `01a11329-23ba-7220-89b2-5777d529e2d3`; model `gpt-6`; run `592e1b8a-ee14-44eb-ac21-1b65a740a7a9`. Examined the one edited SHALL statement and FR-038-AC-154; TC-228 was procedure context only. The targeted Quire grammar check reported 1/1 documents grammar-clean and zero deterministic EARS warnings; semantic review found one low-severity EARS defect.

## Verdict

**FINDING** — the new normative sentence needs an explicit reader subject and unwanted-condition trigger. Its refusal outcome is concrete and the amended acceptance criterion is testable.

## Scope examined

- FR-038 owner-mismatch SHALL statement, spec/checked_package/functional/FR-038-consume-checked-package-v2.md: examined. Exact excerpt: "A well-shaped projection owner\ndiffering from its node's owner SHALL refuse `invalid_package`/`invalid-value`\nat `/identity_preimage/identity_projection/{i}/owner` after the schema step and\nbefore the owner join, even when the package id is recomputed over the changed\nprojection."
- FR-038-AC-154, spec/checked_package/functional/FR-038-consume-checked-package-v2.md: examined. Exact changed excerpt: "A well-shaped projection owner differing from its node's owner refuses `invalid_package`/`invalid-value` at `/identity_preimage/identity_projection/{i}/owner`, before the owner join and even with the package id recomputed over that projection."
- TC-228, spec/checked_package/matrix/TC-228-checked-package-v2-wire-owner-and-lock-joins.md: context only. Exact changed procedure excerpt: "Apply QSpec `adverse.json`'s\n   `projection_owner_mutations` entries\n   `projection-source-owner-differs-from-node` and\n   `projection-model-owner-differs-from-node` to their selected nodes in fresh\n   `positive-all-families` packages. Keep each node's owner unchanged and\n   recompute the package id over the changed projection. Check exact code,\n   cause and RFC 6901 pointer before the owner join or a key check can run."
- Surrounding owner presence, schema, projection, and join text: context for the actor and ordering; no other SHALL statement was edited in this PR diff.

## Findings

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-001 | low | Owner-mismatch SHALL sentence assigns refusal to an inanimate owner and embeds the unwanted condition in its subject; state "If ... then the reader SHALL refuse ..." | spec/checked_package/functional/FR-038-consume-checked-package-v2.md:2005-2009 |

## Dispositions

| FND | outcome | sha/reason |
| --- | --- | --- |
| FND-001 | fixed | d10e6c5de59d50a9beb256da8d0e1e883615f683 |

FND-001 `after_excerpt` (verbatim public IR text at `d10e6c5de59d50a9beb256da8d0e1e883615f683`):

```text
If a well-shaped projection
owner differs from its node's owner, the reader SHALL refuse
`invalid_package`/`invalid-value`
at `/identity_preimage/identity_projection/{i}/owner` after the schema step and
before the owner join, even when the package id is recomputed over the changed
projection.
```

Round 1 rechecked only the `d10e6c5de59d50a9beb256da8d0e1e883615f683` fix diff against FND-001. The sentence now uses an explicit unwanted `If` condition and names the reader as the refusing actor. The diagnostic code, cause, pointer, schema/owner-join order, recomputed package-id condition, and following SourceOwner/ModelOwner applicability remain; no new defect appeared in the changed lines. Model `gpt-6`; session `01a11329-23ba-7220-89b2-5777d529e2d3`; disposition run `20a7b4b2-3b18-441a-a235-5e467b6e81ea`.
