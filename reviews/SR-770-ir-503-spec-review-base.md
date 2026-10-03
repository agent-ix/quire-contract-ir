---
id: SR-770
title: "base and correctness review of PR 254 (IR-503 new catalog words parse and are refused)"
type: SpecReview
analysis: base
scope: "agent-ix/quire-contract-ir@731e307dc132975d3f3f6fac172c8c81bfe83b18; spec/checked_package/functional/FR-038-consume-checked-package-v2.md (catalog-words paragraph, new section, AC-65..68); quire-verification-contracts contracts/checked-operation-catalog-v1.json diff ead78f3f1bea336e2cb230a2ed1b98fa10a913e1..ec4563ff33e03135ac4d0d9500cc53b0a956ecb6; QSpec origin/main FR-322, FR-370, FR-440, proposals/quire-v1/definitions/native-diagnostics.md; crates/quire-contract-model/src/checked_package/{common.rs,shared.rs}, v2/{vocabulary.rs,operation_catalog.rs,operations.rs,mod.rs}"
review_set: subset
---
# SR-770: base and correctness review of PR 254 (IR-503 new catalog words parse and are refused)

## Summary

Ticket: IR-503. I checked the new text against three sources: the catalog diff in
quire-verification-contracts (read-only, `git diff` ead78f3..ec4563f; #11 is 41bb306 and #12 is
ec4563f, and #12 does not touch the catalog file), QSpec origin/main, and the reader code at the
reviewed sha.

What I confirmed independently:

- The word list is complete for the enums. Operators `case` (only on `quire.op.control.case`:
  operands `["union"]`, rest `binder`, result `arm_body`, constraint `union_arms` on operand 0),
  `temporal_formula` (15 identities: 8 with member `temporal_interval`, 7 without) and
  `temporal_fairness` (only on `quire.op.temporal.fair`, member `fairness`). Member kinds
  `temporal_interval` and `fairness`. Constraint kind `union_arms`. Families `union` and
  `temporal`, and result form `arm_body`. `quire.op.temporal.clause` went from operands `[]`,
  rest `any_term`, member `profile_operator` to the six fixed operands with rest null and member
  null. No operator class, member kind or constraint kind is missed.
- QSpec FR-322 lists `unsupported_construct` in the v2 refusal code vocabulary, and FR-370 and
  FR-440 give the clause shape and member shapes the PR cites. CheckedPackageRefusalCode
  (shared.rs:80) has no `UnsupportedConstruct` variant yet; only the diagnostic enum has one
  (v2/mod.rs:248).
- Precedence is implementable. In `operation_defect` (operations.rs:405-444) the operation wire
  shape is read first, then the catalog lookup (`unknown-operation`) and then the class comparison
  (`operation-class-mismatch`), all before laws, mode, member, leaves and arguments. The node loop
  runs in ascending node-id order (operations.rs:351). Placing the refusal between lines 444 and
  445 gives exactly the stated order and the `body/operator` pointer.
- An unknown `operator` already refuses `invalid_semantic_graph` at the term (common.rs:680-684).
- The reader checks no `semantic_form` against the operator (only the state clause does), so a
  `case` or temporal application reaches the operation step in any decoded node form.

## Findings

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-001 | high | Nested applications get admitted. The term grammar admits any application term whose `operator` decodes (common.rs:680-684). `validate_operations` only checks an application at a node's body root, never one nested in `arguments` (operations.rs:42-49, 351-360). Today `case`, `temporal_formula` and `temporal_fairness` do not decode, so a nested application using one refuses `invalid_semantic_graph`. Once the enum gains them, the same nested application decodes, is never operation-checked, and is admitted silently. That contradicts the section's "Each is refused, never admitted" and the IR-503 rule "never silently admitted". AC-66 and TC-048 only cover root-bodied applications. QSpec's operation step checks nested applications in pre-order (native-diagnostics, reader order step 7). Fix: state that an application term with one of the three operator classes refuses `unsupported_construct` at its `operator` at any depth (decided in the term walk), and add a nested case and a binding-value case to AC-66 and TC-048. | spec/checked_package/functional/FR-038-consume-checked-package-v2.md:1004 |
| FND-002 | medium | The no-cause refusal conflicts with the diagnostics catalog. It says `unsupported_construct` carries no cause, "so FR-322's cause pairing does not constrain it". QSpec's native diagnostics catalog, which FR-322 defers to for every (code, cause) pair, lists `unsupported_construct` under the mandatory refinements with cause `declaration-form` or `expression-form`, and says "A producer cannot choose a broader listed alternative to discard a distinction that it knows". Every QSpec use of the code carries one of the two (FR-001, FR-376, FR-390, AD-016). The code itself is the right choice. `invalid_package` + cause would misreport a well-formed package, and its causes are the operation-defect causes with fixed meanings, so none fits. `unimplemented_capability` is not in FR-322's reader vocabulary. Fix: refuse `unsupported_construct`/`expression-form` (an application term is an expression form). Say that the code PR adds both the refusal-code variant and the cause, since neither exists in CheckedPackageRefusalCode or CheckedPackageRefusalCause. | spec/checked_package/functional/FR-038-consume-checked-package-v2.md:631-633 |
| FND-003 | medium | A conformant clause fails operand 0. AC-68 says a clause "with ... exactly six arguments passes the reader's operation checks" without saying which arguments. QSpec FR-370 makes argument 0 a `reference` to the clause's `over` `value`/`parameter` node. The reader resolves a reference to a node that is not type-shaped through the target's `semantic_type` (operand_type_node, operations.rs:1598-1605), so the `over` argument resolves to the parameter type's family (record, object, an integer and so on), not `reference`. It then refuses `ill_typed`/`operator-ineligible` at `arguments/0`. The sixth operand needs the new "formula reference has family `temporal`" rule (a formula node's `semantic_type` is the Boolean node, so today it resolves `boolean`). The first operand has no matching rule. Fix: say how a reference to a `value`/`parameter` node is matched against the `reference` operand family, or that operand 0 is not family-checked, and pin the six arguments of the passing case. | spec/checked_package/functional/FR-038-consume-checked-package-v2.md:1006 |
| FND-004 | medium | "The production catalog reads" depends on more than the new words. Since #11 the catalog's `law_roles` entries are `{authority, identity}`. `OperationCatalogWire.law_roles` decodes them as `CheckedArtifactRef` (operation_catalog.rs:96), which requires `revision`, `digest_domain` and `digest` (shared.rs:402-416). So the catalog at ec4563f does not read until AC-46..61 (IR-530) land, whatever IR-503 adds. The FR-038 matrix cell says the missing words alone are why "it cannot read the catalog". IR-530's gate, in turn, waits on these words. Fix: in AC-65's matrix cell (or the prose), state that the catalog reads once both AC-46..61 and AC-65 are implemented, and that the two code changes move the lock together. | spec/checked_package/functional/FR-038-consume-checked-package-v2.md:1003 |
| FND-005 | low | The group changes are not mentioned. The catalog diff also adds `union` to the `any_value`, `any_term` and `structural_kind` groups and `temporal` to `any_term`. The prose names only the families. Because of the new rule that a reference to a `temporal`/`formula` node has family `temporal`, such a reference also fits every `any_term` position: the rest of `quire.op.function.call`, `quire.op.protocol.control`, `quire.op.state.transition` and `quire.op.claim.clause`. "Fits the sixth operand and no `boolean` operand position" is true but leaves that out. Name the group memberships, and say that a formula reference fits `any_term` positions. | spec/checked_package/functional/FR-038-consume-checked-package-v2.md:607-609 |
| FND-006 | low | `union_arms` will go into a silent no-op. `check_operands` lists the constraint kinds it does not enforce as a silent no-op (operations.rs:1109-1123), and the exhaustive match will force `UnionArms` into some arm. The table's "no admitted application reaches them" holds only while the operator-level refusal stands. Put in the obvious no-op arm, `union_arms` becomes silently unenforced the day IR-506 lifts the `case` refusal. Fix: state that `union_arms`, and the `temporal_interval`/`fairness` member arms, are not added to the no-op lists, so that a reached `union_arms` refuses rather than passes. | spec/checked_package/functional/FR-038-consume-checked-package-v2.md:629 |

## Verdict

Request changes. The word inventory and the root-node precedence are right. One high gap: adding
the three operator classes widens what nested applications admit. Three medium correctness gaps
remain: the no-cause `unsupported_construct`, the clause's operand 0, and the `law_roles`
dependency of "the production catalog reads".

## Dispositions

Round 1, reviewed at agent-ix/quire-contract-ir@7b11d6ee032c69e0ed36fcb5a40e5d25f7232eee (delta from 731e307dc132975d3f3f6fac172c8c81bfe83b18; base origin/main af733f23f42788ea2c8dfa1eb31adaff39e85960 unchanged; GitHub MERGEABLE). `make spec` at round 1: validate passes, 1 grammar finding (baseline), 23 strict unbacked rows (baseline), 163/209 rows backed, FR-038 42/67. AC ids now run to AC-69, so the next free id is AC-70.

Notes on the round-1 checks:

- FND-001: the body root is refused at the operation step. A nested term (an argument element, a binding value, or inside an aggregate) is refused by the body term walk at its own `operator`, whatever identity it names. That is implementable: `validate_term` already carries `is_body_root` (common.rs). AC-66 and TC-048 add the three nested positions.
- FND-002, the cause: QSpec's `proposals/checked-package-v2/schema.json` `Diagnostic.cause_tag` enum lacks `expression-form` and `declaration-form`. That enum types the diagnostics a package carries, not the reader's `ReadResult` refusal. The reader's refusal causes are a separate type (`CheckedPackageRefusalCause` in shared.rs, not `CheckedDiagnosticCause`), and the refusal is never serialized as a package `Diagnostic`. So this is a QSpec follow-up (the schema enum trails the native diagnostics catalog), not a defect of this PR. I recommend a QSpec ticket.
- FND-003: QSpec FR-370's first argument is a `reference` to the `over` `value`/`parameter` node, and the catalog's operand 0 is `reference`. The new text matches operand 0 when it is a `reference` term, without resolving it through the parameter's `semantic_type` (which would give the parameter's own family, operations.rs:1598-1605). The AC's passing case can therefore pass. A non-reference first argument (TC: a `text` literal) refuses. The rule sits inside the clause paragraph, so other `reference` operands are unchanged.
- FND-004: AC-65 now defers the unreadable-catalog error to AC-58's typed catalog read. AC-58 owns the fallible read and AC-65 only cites it, so the read is not specified twice. The joint lock move with IR-530 is stated in the prose and in the matrix.

| FND | outcome | sha/reason |
| --- | --- | --- |
| FND-001 | fixed | 7b11d6ee032c69e0ed36fcb5a40e5d25f7232eee |
| FND-002 | fixed | 7b11d6ee032c69e0ed36fcb5a40e5d25f7232eee |
| FND-003 | fixed | 7b11d6ee032c69e0ed36fcb5a40e5d25f7232eee |
| FND-004 | fixed | 7b11d6ee032c69e0ed36fcb5a40e5d25f7232eee |
| FND-005 | fixed | 7b11d6ee032c69e0ed36fcb5a40e5d25f7232eee |
| FND-006 | fixed | 7b11d6ee032c69e0ed36fcb5a40e5d25f7232eee |
