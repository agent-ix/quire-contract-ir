---
id: FR-343
title: "Map bounded OCL 2.4 output without approximation"
type: FR
relationships:
  - target: ix://agent-ix/quire-contract-ir/FR-033
    type: depends_on
  - target: ix://agent-ix/quire-contract-ir/FR-342
    type: depends_on
  - target: ix://agent-ix/quire-specification/FR-121
    type: implements
  - target: ix://agent-ix/quire-specification/FR-122
    type: implements
  - target: ix://agent-ix/quire-specification/FR-269
    type: implements
  - target: ix://agent-ix/quire-specification/NFR-060
    type: constrained_by
---
# FR-343: Map bounded OCL 2.4 output without approximation

## Description

When an exactly bound OCL 2.4 mapper receives a ready admitted obligation, the
mapper shall emit a deterministic named invariant, precondition, or
postcondition only when the entire checked expression belongs to the selected
OCL fragment; otherwise it shall return one explicit non-preserved candidate
for the whole obligation without substitute text.

## Inputs

- One ready or non-ready admitted mapping obligation from the common seam.
- Its checked typed expression, clause kind, execution anchor, source identity,
  discharged definedness obligations, and exact correspondence catalog.
- The remaining aggregate mapping-work budget.

## Outputs

- One deterministic UTF-8 Complete OCL fragment and one represented local byte
  region with a `preserved` or `conditional` candidate; or
- One `unrepresented` or `refused` candidate with exact qualified causes and no
  target bytes or regions; or
- One typed operational failure when work/resource accounting cannot complete.

## Behavior

- The admitted renderer shall cover total Boolean literals and `not`, `and`,
  `or`, and `implies`; equality and ordered comparison of equal admitted types;
  bounded reject-on-overflow integer literals and checked `+`, `-`, and `*`;
  exact value/field reads; immutable quantifier locals; bounded ordered
  duplicate-preserving `Sequence` literals, `size`, `includes`, `count`,
  `forAll`, and `exists`; and exactly bound zero-argument pre/post anchors.
- The renderer shall emit `context <qualified-type>` plus exactly one named
  `inv`, `pre`, or `post` constraint.
- The renderer shall use `Sequence{...}` and OCL arrow operations for collection
  expressions.
- The renderer shall use only the closed target names admitted by
  [FR-342](FR-342-admit-ocl24-correspondence.md).
- The renderer shall use explicit parentheses and canonical whitespace, names,
  line endings, expression order, and local order without consulting path,
  time, locale, observer, parser, tool, or prior output.
- The mapper shall append OCL `@pre` only to an exact pre-state read inside the
  matched postcondition operation.
- The mapper shall not substitute a current or post-state read for pre-state.
- The mapper shall retain a machine-readable native-domain condition whenever
  OCL's wider integer domain or arithmetic semantics require the authoritative
  native bounds to establish correspondence.
- If an authoritative integer bound is missing or incompatible, then the
  mapper shall return a non-preserved candidate instead of assuming the bound.
- Optional/absent/invalid/unavailable values without a total selected
  definedness correspondence, non-integral rationals, divide/remainder,
  saturating arithmetic, index access, unsupported collection operations,
  unsupported graph or relationship navigation, temporal behavior, protocol
  behavior, unsupported clause kinds, and unresolved correspondence shall
  produce an exact non-preserved candidate for the whole obligation.
- If an exact total-definedness and relationship/endpoint correspondence is
  selected, then the mapper shall represent option presence/unwrap navigation
  only as a `conditional` candidate carrying both qualified conditions.
- The mapper shall not classify option or relationship navigation as
  unconditional preservation.
- A non-ready source fact, conflicting correspondence, unsupported required
  capability, or exhausted work budget shall never become represented output or
  Boolean success.
- Generated OCL and downstream parser/typechecker observations shall not be
  accepted as native input, semantic truth, preservation evidence, or a mapping
  input for another target.

## Classification

| Source condition | Required outcome |
| --- | --- |
| Wholly admitted total Boolean expression | `preserved`, represented output, no conditions or causes |
| Wholly admitted bounded integer expression | `conditional`, represented output, condition `native-bounded-integer-domain` |
| Exactly mapped option/relationship navigation | `conditional`, represented output, conditions `total-definedness-correspondence` and `relationship-endpoint-correspondence` |
| Valid but nonselected expression, clause, graph, temporal, or protocol meaning | `unrepresented`, no output, one or more exact `ocl24-unrepresented-*` causes |
| Non-ready source fact or conflicting/ambiguous correspondence | `refused`, no output, one or more exact `ocl24-refused-*` causes |
| Exhausted/overflowing work or allocation/cancellation failure | typed operational error; no candidate population or partial package |

## Acceptance Criteria

| ID | Criteria | Verification |
| --- | --- | --- |
| FR-343-AC-1 | Total Boolean/scalar invariants and exact pre/post field comparisons emit canonical OCL with complete source, model, anchor, and profile dependencies. | Test (TC-220) |
| FR-343-AC-2 | Every admitted bounded integer case agrees over its exact native domain and retains the domain condition; zero, 1000, and just-outside ConfigVersion cases cannot enlarge the native claim. | Test (TC-220) |
| FR-343-AC-3 | Ordered duplicate-preserving Sequence size/includes/count/forAll/exists cases preserve order, multiplicity, local scope, and bounded work exactly. | Test (TC-220) |
| FR-343-AC-4 | Optional/invalid/rational/index/unsupported collection/graph/temporal/protocol, saturating/overflow, unresolved-anchor, non-ready, and resource cases return the specified typed non-success with no approximate fragment. | Test (TC-220) |
| FR-343-AC-5 | The bounded ConfigVersion invariant and exact operation pre/post example retain bounds and anchors; ParentPrecedes is wholly unrepresented or refused unless its complete relationship/endpoint mapping is selected. | Test (TC-220) |
| FR-343-AC-6 | Equal admitted inputs produce byte-identical fragments, records, and packages, while parser/tool/path/time/locale/observer variation cannot promote or mutate semantic mapping state. | Test (TC-220) |

## Dependencies

- [FR-033](FR-033-account-for-output-obligations.md) validates the candidate and
  assembles its complete record without a preservation default.
- [FR-342](FR-342-admit-ocl24-correspondence.md) supplies exact OCL names and
  owner-qualified correspondence dependencies.
- `ix://agent-ix/quire-specification/FR-121`, `FR-122`, `FR-269`, and `NFR-060`
  own the portable loss vocabulary, bounded OCL profile, and resource behavior.
