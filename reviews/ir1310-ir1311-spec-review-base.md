---
id: SR-2026110414560015
title: spec-review/base of combined Port roles and lexical protocol slots
type: SpecReview
analysis: base
scope: agent-ix/quire-contract-ir; IR-1310, IR-1311; spec/checked_package/functional/FR-038-consume-checked-package-v2.md,
  spec/model/functional/FR-019-rust-library-interface.md; crates/quire-contract-model/src/checked_package/{shared.rs,v2/model_members.rs,v2/protocol_body.rs};
  tests/it/checked_package_v2_protocol_body.rs; tests/it/support/{checked_package.rs,protocol_ports.rs};
  tests/conformance_qspec/protocol_vectors.rs
review_set: subset
---

## Summary

Ticket: IR-1310 and IR-1311. Reviewed four new FR-038 criteria and FR-019 API additions against QSpec FR-465 by reference. The authoritative third binding is exactly package/node; outdated IR-1310 declaration/kind prose is superseded by the normative contract and ticket clarification. Inputs, typed output/readback, malformed and semantic refusal classes, precedence, backward compatibility and lexical boundaries are concrete and observable. No hash, pin, duplicate upstream artifact or new dependency introduced. No applicable active AssuranceProfile exists in this repository; matches in historical reviews are not profiles.

## Verdict

PASS. No change-attributable defects found.

## Findings

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-001 | low | No findings (placeholder) | - |

## Coverage

Plan completion: not assessed

scope:
- id: FR-038-AC-463
  path: spec/checked_package/functional/FR-038-consume-checked-package-v2.md
  role: examined
  excerpt: Implemented. The public reader admits FR-465 roles with the optional third
    `model_member` binding, a Group of nonempty `package` and `node` text bindings
    naming an admitted Port in the selected package of the referenced model. Two Port
    identities remain distinct retained role bodies; the borrowed role read exposes
    both original texts and writer/readback preserves them. Existing two-argument
    type and whole-model roles still admit without a member.
- id: FR-038-AC-464
  path: spec/checked_package/functional/FR-038-consume-checked-package-v2.md
  role: examined
  excerpt: Implemented. Empty/malformed member identities, a missing/extra/reordered
    Group binding or unsupported term shape refuse `malformed_wire`. A different package,
    nonexistent declaration, non-Port declaration or member on a whole-model role
    refuses `invalid_model_binding`/`malformed-declaration` at the role node, retaining
    the supplied package/node texts; a nonexistent whole-model selection refuses `missing_declaration`/`missing-selection`
    first. Published FR-465 role-vector cases are read by reference; no selected kind
    is inferred from a declaration name.
- id: FR-038-AC-465
  path: spec/checked_package/functional/FR-038-consume-checked-package-v2.md
  role: examined
  excerpt: Implemented. A protocol slot's lexical let/query/count/sum binders are
    in scope only inside their bound expression bodies, including nested binders and
    shared graph nodes reached under different lexical scopes. Genuine local reads
    do not produce `BinderScope`; the reader walks admitted terms with finite metered
    work and does not scan unrelated graph nodes to infer local ownership.
- id: FR-038-AC-466
  path: spec/checked_package/functional/FR-038-consume-checked-package-v2.md
  role: examined
  excerpt: Implemented. A local binder does not scope its own let initializer or an
    unrelated sibling expression, and a same-name local binding does not forgive a
    known protocol binder unavailable at that stage. Such external reads retain the
    typed BinderScope owner/name/stage/read refusal. Barred model-state reads retain
    their existing precedence and `ModelStateRead` detail.
- id: FR-019
  path: spec/model/functional/FR-019-rust-library-interface.md
  role: examined
  excerpt: 'The borrowed `CheckedPackageV2::protocol_role_model_member` method returns

    the admitted selected-package and Port declaration text pair of FR-038-AC-463,

    with no replacement identity carrier. The implemented typed refusal context is

    `CheckedProtocolRefusalDetail::ModelMember` (FR-038-AC-464).'
- id: FR-465-AC-11
  path: agent-ix/quire-specification/spec/objects/interfaces/FR-465-checked-package-protocol-declaration-bodies.md
  role: context_only
  excerpt: 'Over the architecture fixture whose component `Sys` owns the Port `pump_out`
    and the Port `tank_in`, the role `Outlet` on `M::pump_out` carries the `model_member`
    Group `{package: P, node: N}` with `P` the package identity of `M` and `N` the
    declaration identity of `pump_out`, and its `model` reference targets `Sys`; the
    role `Inlet` on `M::tank_in` carries the same `package` and the `node` of `tank_in`,
    and the two roles have different node ids. The role `Operator` on the type `M::Sys`
    and a role on the whole model `M` carry no third argument, and a package with
    only such roles has the bytes '
- id: FR-465-AC-12
  path: agent-ix/quire-specification/spec/objects/interfaces/FR-465-checked-package-protocol-declaration-bodies.md
  role: context_only
  excerpt: 'Of a role on a Port: a `model_member` whose `package` names a package
    other than the one the `model` target belongs to, one whose `node` names a declaration
    that is a part, a component or a connection, and one whose `node` names no declaration
    each refuse `invalid_model_binding`/`malformed-declaration` at the role node,
    retaining `package` and `node`; an empty `package` or `node` text, a `model_member`
    that holds a third binding, and a `model_member` Group that is empty each refuse
    `malformed_wire`; a `model_member` on a role whose `model` is the text of an existing
    `model_root` node refuses `'

## Supplemental review round 1

The final aggregate gate found three API inventory assertion failures missed in the original review. The doc-only correction removes the associated method and enum variant from the crate-root item table and explicitly assigns them to the already inventoried CheckedPackageV2 and CheckedProtocolRefusalDetail types. Both remain documented with FR-038 ownership above that table. The exact exported_items()/expected_items() assertions, inventory parser, source API, trace bindings, public-reader tests and scanner gates are unchanged. Inspected the correction diff and the unchanged inventory guard implementation independently. Coder evidence reports all 25 public_interface tests passing. Five scanner failures in the first gate were missing-executable setup errors; the root lead corrected PATH without scanner code changes. No Cargo or aggregate gate was repeated in this supplemental review. No open or new finding: the doc correction is review-mergeable, subject to the lead's final aggregate gate. Existing repository baseline and unavailable evidence-advisor qualifications remain unchanged.

```yaml
clean: true
scope:
- id: FR-019
  path: spec/model/functional/FR-019-rust-library-interface.md
  role: examined
  excerpt: 'The borrowed

    protocol role member read and refusal variant documented above belong to the

    already listed CheckedPackageV2 and CheckedProtocolRefusalDetail types; they

    add no crate-root export.'
- id: public_interface_inventory
  path: tests/it/public_interface.rs
  role: examined
  excerpt: assert_eq!(exported_items(), expected_items());
findings: []
dispositions: []
```
