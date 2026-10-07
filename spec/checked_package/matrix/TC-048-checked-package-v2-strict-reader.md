---
id: TC-048
title: "CheckedPackage V2 strict reader re-derives package and nominal identities"
type: TC
relationships:
  - target: ix://agent-ix/quire-contract-ir/FR-038
    type: verifies
  - target: ix://agent-ix/quire-specification/TC-217
    type: references
---
# TC-048: CheckedPackage V2 strict reader re-derives package and nominal identities

## Description

Verify FR-038-AC-1 through FR-038-AC-5 (QSpec FR-322-AC-4, FR-322-AC-8,
FR-322-AC-10) against the V2 package fixtures this repository builds from its
own public vocabulary (`tests/it/support/checked_package.rs`), and
FR-038-AC-27 through FR-038-AC-30 (QSpec FR-322-AC-29 through FR-322-AC-34
"Model-owned members") against a self-built domain package document, and
FR-038-AC-31 through FR-038-AC-33 (QSpec FR-322-AC-35) against the
`DependencySelection` entries of a self-built package, and
FR-038-AC-35 through FR-038-AC-38 (QSpec FR-322-AC-36 and FR-322-AC-37)
against a self-built dependency package and a package that calls it, and
FR-038-AC-45 (QSpec FR-322-AC-28) against a model-owned nominal declaration and
a model declaration node keyed under the content-only `ModelOwner`, and
FR-038-AC-46 through FR-038-AC-61 (QSpec FR-322 `lock`, `DefinitionRef` and
`RawSourceRef`) against the artifact references of a self-built package and the
operation catalog's `law_roles` entries, and FR-038-AC-62 through FR-038-AC-64
(QSpec FR-322 `ModelRef` and `DependencySelection`) against the
`model_selections` rows and `dependency_selections` entries of a self-built
package, and FR-038-AC-65 through FR-038-AC-69 against the operation catalog's
words and the applications of a self-built package that name them, and
FR-038-AC-70 through FR-038-AC-72 against self-built compared types that reach
themselves, and FR-038-AC-74 through FR-038-AC-80 against the V2 wire types'
encoding through `quire-canonical` and the reader's canonical-bytes check, and
FR-038-AC-81 through FR-038-AC-88 against the operation identity, law, mode,
member and operand checks of a single application and the key of an application
node, as unit tests of `crates/quire-contract-model/src/checked_package/v2/operations.rs`,
and FR-038-AC-89 through FR-038-AC-95 against every identity digest's move to
`quire-canonical` (node keys, lowered identities, the nominal digest's byte
limit, the absence of an encoder of this repository's own, a selected model
document's integer check and byte limit, and the lowering byte ceiling), and
FR-038-AC-96 through FR-038-AC-108 (QSpec FR-370, FR-440 and FR-250) against the
temporal, fairness, union and `case` nodes of a self-built package, the
temporal step's placement, clause, profile-fit and interval-bound checks, and
QSpec's own positive fixtures read from the QSpec checkout. FR-038-AC-66 is
retired, its ID not reused (ADR-0056), and has no case. FR-038-AC-109 through
FR-038-AC-111 are against a number whose exact value its RFC 8785 encoding loses
(the `inexact-number` and `inexact-integer` causes of a selected model document,
and the package's own byte-stream refusal). FR-038-AC-112 and FR-038-AC-113 are
against QSpec's own `adverse.json` mutations and `dependency-selection-vectors.json`
identity, read from the QSpec checkout. FR-038-AC-114 through FR-038-AC-118 (IR-495) are against the flat wire of merged QSpec FR-322 "Body grammar": nested and
misplaced applications, the pre-order pointer, the order ahead of identity recomputation,
the absence of a depth limit and of call-stack recursion, and the five body-grammar
mutations. FR-038-AC-119 through FR-038-AC-122 (IR-551) are against the timed interval form of
merged QSpec FR-370 and FR-370-AC-8: the four end variants, rational bounds in lowest
terms, the refusal of a negative, malformed or non-reduced bound, a null upper bound,
an open end with equal bounds and a missing or extra member, and the profile fit.
FR-038-AC-159 through FR-038-AC-164 verify the typed scalar operand accessor
over admitted application nodes and explicit post-admission mutation rows.

## Test Procedure

Dispatch an unknown, empty, absent and malformed `contract_version`, and
independently a malformed document, a duplicate top-level member and
noncanonical bytes, before any version is selected. Admit each in-repo
positive V2 fixture. Apply each authored adverse
structural mutation and compare the outcome. Independently inject malformed
JSON, a duplicate member, an unknown member, noncanonical bytes, an absent or
mismatched context digest, an unreported or unsupported required feature, a
dangling reference and an incomplete source map. Admit at exact byte, node,
edge, occurrence, diagnostic and work limits, then lower each limit by one.
Recompute every fixture's package id, edit excluded and included preimage
members, and re-read. Re-derive each in-repo nominal preimage's digest; apply
each authored invalid nominal mutation with
the retained digest, remove or swap a preimage, and change a retained
preimage's enum case, `semantic_type`, dependency and unit target while
mirroring the identity projection and package id. Re-own the nominal enum
declaration by a model owner (`identity`, `node`) over a locked `sha256-jcs`
domain package and re-read it; then name an unselected package, select none,
empty the node, restore a compiled-model owner or lock shape carrying
`authority`, `revision`, `export` or `version`, change the selection's digest
domain or evidence domain, and set a model node to `model_export`.

For FR-038-AC-45, build the model-owned nominal declaration with the owner
`{kind: model, identity, node}` and no `version`, derive its node key from
those members alone, and read it: it admits. Select another document of the
same identity under its own digest, in another package, and re-read: the node
key is unchanged. Add a `version`
member to the owner and re-read: it refuses `unknown_member` at that member.
Empty the owner's `identity` and, separately, its `node`, and re-read each: it
refuses `invalid_semantic_graph`. Select a domain package, key a model
declaration node of it under the version-free `ModelOwner`, add a `field`
member on that node and read it: it admits; select another document of the
same identity under its own digest, in another package, and re-read: the
declaration node's key is unchanged and it admits; key the
node under another domain package's identity and re-read: it refuses
`missing_declaration`/`missing-selection` at the member's `declaration`.

## Expected Results

An unknown, empty or absent `contract_version` refuses as
`unknown_contract_version`; a malformed `contract_version`, document, or a
duplicate or noncanonical top-level member refuses as `malformed_wire`,
`duplicate_member` or `noncanonical_wire` before any version-specific
decoding. Positive fixtures admit. The owner-free nominal fixture retains its
recorded pre-owner package id; an owner-bearing fixture derives its package id
from its owner-bearing identity projection and is compared under FR-038-AC-176
with QSpec's published positive packages and
`model-member-type-vectors.json`. Every adverse and
injected case returns its exact refusal code or incomplete accounting with no
package. Excluded edits keep the id and included edits change it. Every nominal
preimage digest matches its node key; every nominal mutation and contradictory cross-field join
refuses as `invalid_semantic_graph`. The model-owned package admits; each model join mismatch refuses as
`invalid_semantic_graph`, a compiled-model owner or lock shape carrying
`authority`, `revision`, `export` or `version` as `unknown_member`, a foreign digest
domain as `digest_domain_mismatch`, a domain package digest with no supplied
document as `missing_import`, and `model_export` as `invalid_semantic_graph`.
A model owner of `{kind, identity, node}` with no `version` admits and its node
keys, nominal and declaration, are unchanged when another document of the same
identity is selected; an owner carrying `version` refuses `unknown_member`, an owner with an
empty `identity` or `node` refuses `invalid_semantic_graph`, and a declaration
node keyed under an unselected domain package refuses
`missing_declaration`/`missing-selection` (FR-038-AC-45).

## Selections bind by identity (FR-038-AC-62 through FR-038-AC-64)

Build a package whose lock and identity preimage carry the same
`model_selections` rows `{identity, digest_domain, digest}` and the same
`dependency_selections` entries `{identity, package_id}`, re-derive its
`package_id` and read it: it admits. Add a `version` member to a model row in
the lock, then only in the identity preimage, then to a dependency entry in the
lock and then only in the preimage, and to a row beside an otherwise well-formed
row, and read each: each refuses `unknown_member` at that `version` member, the
first in document order, and no package is returned. Give a dependency entry
`{identity, version}` with no `package_id` and read: it refuses `malformed_wire`
at the entry. Remove `identity`,
`digest_domain` and `digest` in turn from a model row and give each a non-string,
and read. Select the same domain package as two documents at different
versions, each in its own package under its own `sha256-jcs` digest, and read
both: each admits and they yield the same model-owned node keys. Read a
matching document that has no `package.version`, and one whose `package.version`
is not a string: each admits. Give a model
owner an identity no row names and read it. Supply a dependency package under
its entry's `identity` and read it, supply it under another identity, and supply
a package of another `package_id`. Put two rows of one identity and different
digests in the lock, each document supplied, in both orders, and read: each
refuses `stale_dependency` at the later row's `digest`; repeat it with no
document supplied for one of the pair, with a forged one, with one naming
another identity, and with an earlier row whose document is missing: each
refuses the same way, before any document is read; give one of the two a digest domain other than
`sha256-jcs` and read: it refuses `digest_domain_mismatch`.

Expected: the package admits; a `version` member at any of the four places
refuses `unknown_member` at that member; a dependency entry lacking `package_id`
and a model row or entry with a missing or wrong-kind member refuse
`malformed_wire`; both documents admit with equal node keys, and so do the
documents with no or a non-string `package.version`; the unnamed owner
refuses `invalid_semantic_graph`; the dependency supplied under its identity
with its `package_id` admits, under another identity `missing_import`/
`missing-selection`, and with another `package_id`
`stale_dependency`/`byte-digest-mismatch`; the same-identity pair refuses as
stated. Compare the whole refusal code, cause and pointer.

## Catalog words (FR-038-AC-65, FR-038-AC-67 through FR-038-AC-69)

Read the production catalog and convert each of `case`, `temporal_formula`,
`temporal_fairness`, `temporal_interval`, `fairness` and `union_arms` from its
wire string to its enum member and back. Compare every operator class, member
kind and constraint kind the catalog declares with the enum's member set. Read
catalog bytes that name an operator class, a member kind and a constraint kind
outside the vocabulary, one at a time. Put an operator outside the closed
operator vocabulary on a package application and read the package.

Name `quire.op.control.case` under operator `unary`, and an identity the catalog
lacks, and read each. Build two defective body-root nodes, one a
`temporal_formula` application in a `function` node and one other operation
defect, in both digest orders (AC-67). Put a `temporal_interval` member on
`quire.op.boolean.not` and on `quire.op.temporal.clause`, and a `fairness`
member on `quire.op.boolean.not` and on `quire.op.temporal.holds` (AC-69).

Run the operation check of a `quire.op.temporal.clause` application node as a
unit and, since IR-549, through a package read whose formula node is a
`temporal_formula` application (AC-68): with its law selected, no member and the
six arguments, the first a `reference` to a `value`/`parameter` node of a record
type, with five and seven arguments, with a `text` literal first and a
`reference` to a Boolean node sixth, with a `reference` to a `temporal`/`formula`
node sixth and in a `boolean` operand position and an `any_term` position, and
with a `profile_operator` member and a `fairness` member.

Expected: every word converts both ways and the catalog's sets equal the
enums'; each unreadable catalog returns the typed error of FR-038-AC-58
naming the word; the foreign operator refuses `invalid_semantic_graph` at the
term; the `unary` case refuses `operation-class-mismatch`, the unknown identity
`unknown-operation`; the lower digest is reported in both orders; the stray
members on `quire.op.boolean.not`, `quire.op.temporal.holds` and
`quire.op.temporal.clause` refuse `operation-member-mismatch` at the member; the
clause passes in the unit and the package, the wrong counts refuse
`operator-ineligible` at `arguments`, the two wrong families at their arguments,
the formula reference fits the sixth operand and the `any_term` position and not
the `boolean` one, and the two members refuse `operation-member-mismatch`.
Compare the whole refusal code, cause and pointer.

## Temporal, fairness and case admission (FR-038-AC-96 through FR-038-AC-108)

Implemented (IR-549 code change) in `tests/it/checked_package_v2_temporal.rs`
(AC-96 through AC-98, AC-100 through AC-105, AC-108), `tests/it/checked_package_v2_union.rs`
(AC-99, AC-101, AC-106), `tests/it/checked_package_v2_catalog_words.rs` (AC-100's
nested and diagnostics cases), the unit tests of `operations.rs` and `temporal.rs`
(AC-96, AC-97, AC-98, AC-103: an `over` outside its clause's `dependencies` is
reachable only at the step, since the dependency join refuses it first in a
package read) and `tests/conformance_qspec/main.rs` (AC-107, run by
`make conformance-qspec` alone). The two-clause cases of AC-102 and AC-108 find
the digest order they need by varying the clauses' names, and the adjacent pairs
of one clause are swapped where the two defects' positions can be (the
operator order of a formula tree); `over` and the fairness argument have one
position each.
Build, from this crate's own vocabulary and never from a copy of QSpec's
fixtures, a package whose `temporal`/`formula` nodes apply each of the fifteen
`temporal_formula` identities with their catalogued member and operands under a
clause selecting a bounded profile, a `temporal`/`fairness` node applying
`quire.op.temporal.fair` under a clause selecting `quire.temporal.infinite-trace/v1`,
and a clause whose formula applies `quire.op.temporal.eventually` with the
interval `{0, 3}` over `quire.op.temporal.holds`; read it, then lower it under a
profile that supports the `temporal` tag and under one that does not (AC-96). On
each of the eight interval operators read the member as `{0, 3}`, `{9, 10}`,
`{2, null}`, `null` interval, `{-1, 3}`, `{0, -2}`, `{-5, -2}`, `{3, 0}`, `{10, 9}`, `{2^64+1, 2^64}`,
a `fairness` member, a `null` member, an interval with a third member, an integer
`lower`, and bounds `"1.5"`, `"01"`, `"+1"`, `""` and `"3x"`, and put a member on
`holds`, `not` and the clause (AC-97). Read
the `fairness` member well formed with `weak`/`whole` and `strong`/`each`, then
with `medium`, with `part`, without `name`, with an extra member, `null`, of kind
`temporal_interval`, and on `quire.op.boolean.not` (AC-98). Build the `Shape`
union, the `Shape::Rect(2, 3)` value and the three-arm `case` and read them; then
read the `case` with its arms out of order, an arm omitted, an arm repeated, a binder aggregate of
the wrong count, a binder of the wrong type and an arm body of another type; a
union with a duplicated member and one with a non-type payload; and a union value
naming an absent member, with a wrong payload count and with a payload of the
wrong type (AC-99). Put an application of each of the four classes in a node of
another form, as an element of another application's `arguments`, as a `binding`
value, inside an `aggregate`, and as a root and as a nested term in a diagnostic
entry's `details`, with a `details` reference to a formula, a fairness, a union and a
union value node (the first two refuse, the last two admit; merged QSpec FR-370, #182), a `case` nested in another term, and an `expression` node whose form
contradicts its root operator class (`invalid_semantic_graph`); give a formula,
fairness and clause node an empty `aggregate`
body, a `literal` body and another class's application; reference a formula node
from a `function` body, a fairness argument and a `case` argument, and a fairness
node from a formula argument and a formula operand; read each, then read each
class, node and reference at its own place (AC-100). Check that temporal/formula
and expression/case applications are not
blanket-refused `unsupported_construct`/`expression-form`; the planned IR-661
relationship-navigation additions require the lead-owned existing TC-048 Rust
refusal-enum assertion to be amended, not claimed as executed here. Read a
diagnostics entry whose `code` is `unsupported_construct`, and lower the admitted
`temporal`/`formula` and `expression`/`case` nodes under supporting and
non-supporting profiles, comparing the lowered body and `ir_id` with the admitted
node's (AC-101). Build a placement defect beside a lower-digest node with an
unknown identity or a class mismatch, in both digest orders, two placement
defects, and, within one clause, each adjacent pair of the order `over`, fairness
resolution, profile fit, bounds, with the two defects' positions swapped; then two
clauses, the lower-digest one holding a profile-fit defect and the higher-digest
one an `over` defect, in both digest orders; and a member `{lower: "1.5", upper:
"0"}` and a wrong-kind member under a bounded and the infinite-trace profile;
`{0, -2}` and `{-1, -3}`, and `{-1, null}` under a bounded profile, under
infinite-trace and beside a placement defect at a lower-digest node, each
refused at the negative bound at schema validation (strict wire validation, the flat wire pass),
as is `{"1.5", "0"}` at its malformed bound, none of them `operation-member-mismatch`
(AC-102). Give a clause's
`over` a `value`/`parameter` dependency, one that is no dependency and a
`scalar_type` dependency, and a fairness member an existing operation, a missing
name, an absent declaration, a non-`model` declaration, a name matching two
operations, an operation the node only inherits and a declaring node whose owner
is not recovered (AC-103). Select
`quire.temporal.event-position.false-extension/v1` and
`quire.temporal.infinite-trace/v1` in turn over closed, `{lower, null}` and `null`
intervals and an empty and a non-empty fairness argument, with a second clause
whose tree shares no node with the first (AC-104). Give `holds` a formula
reference and a Boolean reference, `until` one argument, `not` two, `true` one,
`and` a Boolean reference and `not` a formula reference (AC-105). Compare two
`Shape` values, a `Shape` and another union, a union reaching no `text` and the
`Label` union with its leaf list correct, missing a leaf, repeating one and with a
leaf for `Empty`; the recursive unions `IntList` (no `text`, `leaves` empty) and
`TextList` with the leaves `["member:Cons", "position:0"]` and
`["member:Cons", "position:1", "recursion:0"]`, the recursion leaf missing, written
`recursion:1`, and placed at an `IntList` reentry (AC-106; `member:<Ident>` and union
cycles are merged QSpec FR-322-AC-45 and AC-46; the recursion-leaf entry for a
cycle that reaches `text` is merged FR-322 text, #182). Read an empty union and a union value with two
bindings (AC-99, `invalid_semantic_graph` at the body); a fairness `declaration`
naming a `scalar_type` node (AC-103, `invalid_model_binding`/`malformed-declaration`
with path `.../member/declaration` on the fairness node and the target's key as
locus, comparing path and locus for every temporal-step refusal as the "Path and
locus" table of FR-038 lists them); a law with a member's identity under another `authority` (AC-108,
`operation-law-unselected`); a case arm body of unresolvable type; an unreached
formula node with `{3, 0}` (AC-97); and the placement defect in the shape of a
formula application in a `function` node beside an unknown-identity node and beside a
class-mismatch node (AC-102). Run
`make conformance-qspec` with `QUIRE_SPECIFICATION_DIR` set to a QSpec checkout, to
the empty string, to an unset value and to a path without the fixtures directory,
reading `positive-all-families.json`, `positive-clause-operations.json` and
`positive-union-nodes.json` from it, and compare the self-built packages' features
with the fixtures' (AC-107); the same target reads `adverse.json` and applies each of its mutations to
`positive-all-families.json` with no identity refreshed, and each body-grammar mutation's `flattened`
form as a positive control (AC-112), and reads `dependency-selection-vectors.json` and derives the
`package_id` through the reader's own derivation over its version-free `dependency_selections` in
the lock and the identity preimage (AC-113). Run each of AC-112 and AC-113 with the variable
unset, empty and naming a path without the fixtures, with the file missing and not JSON, with a
mutation list absent or empty, with a body-grammar entry lacking `flattened`, and, for AC-112,
with an expected-failure entry naming an id absent from `adverse.json`, one the reader refuses as
recorded, and one the reader refuses with a different code than listed.

Expected: the package of AC-96 admits with its recorded identities and lowers or
returns `unsupported` naming `temporal` as stated; AC-97's admitted members admit,
`{-1, 3}`, `{0, -2}` and `{-5, -2}` and the malformed bounds `"1.5"`, `"01"`, `"+1"`,
`""`, `"3x"` and the JSON integer `0` refuse `invalid_package`/`invalid-value` at the
bound, and `{3, 0}`, `{10, 9}` and `{2^64+1, 2^64}` refuse `invalid_package`/`invalid-value`
at the application; a `null` member on an interval operator refuses
`operation-member-mismatch` at the application, and each other member shape
`operation-member-mismatch` at `operation.member`; AC-98's two well-formed members admit and each other refuses
`operation-member-mismatch` at `operation.member`; the `Shape` package admits and
each defective `case` refuses `ill_typed`/`operator-ineligible` at the `case` node,
the duplicate member `invalid_package`/`duplicate-member` and each defective
union value `ill_typed`/`type-mismatch` at the node; each misplaced application,
node or reference refuses `ill_typed`/`operator-ineligible` as AC-100 locates it
and each is admitted at its own place; the reader's refusal types hold neither
word, the diagnostics entry reads, and the lowered body equals the admitted body;
the placement defect is reported ahead of every other defect in both digest
orders, and each later stage ahead of the next; the `over` and fairness cases
refuse `missing_declaration`/`missing-name` or `invalid_model_binding`/
`malformed-declaration` as AC-103 states; the bounded profile refuses a `null`
interval, a `null` `upper` and a non-empty fairness argument
`operation-member-mismatch` and the infinite-trace profile admits them; the
operand cases refuse `ill_typed`/`operator-ineligible` at the operation step; the
`Shape` and `Label` comparisons admit and refuse as AC-106 states; and
`make conformance-qspec` passes with each QSpec fixture admitted with its recorded
`package_id` and fails when the variable is unset, empty or names a path without
the fixtures; `make test` does not run it, and wiring it into CI is a separate
decision. Selecting `quire.fixture.temporal-profile/v1` (selected nowhere else),
an empty identity and a differently spelled infinite-trace identity as the clause
profile refuses `unknown_profile`/`unsupported-selection`; naming
`quire.package.composed/v1` with the lock selecting it as a `binding_contract` row,
and `quire.protocol.complete/v1` with the lock selecting it as a `protocol_profile`
row, refuses `unknown_profile`/`wrong-selection-role`, while the same two
identities with no such lock row refuse `unsupported-selection` (merged QSpec
FR-370 "Profile check" and FR-370-AC-10, decided from the package alone); a clause with
no `temporal_profile` law or two skips the check; each at the law's
`definition`, first among the clause's checks, and each of the five FR-250
members admits (AC-108). Compare the whole
refusal code, cause and pointer. Every structural mutation of `adverse.json` refuses with its
recorded code, every body-grammar mutation refuses `malformed_wire` as recorded or is listed with
its current refusal and owning ticket, each flattened form is not refused `malformed_wire`, and
the version-free dependency selections derive the recorded `package_id` through the reader's own
derivation while the unchanged base derives a different one (AC-112 and AC-113); each fail-closed
case above fails the run and never skips.

## Parameter and compound-unit nodes (FR-038-AC-22, FR-038-AC-23)

Build a package in QSL FR-092's shapes for `function both(a: Boolean, b:
Boolean): Boolean { a and b }` plus the compound unit `Metre^2`, read it and
lower every node.
Then mutate one parameter, the compound unit or the application node at a
time and re-read. The package admits and every node lowers; each parameter
body mutation (level, name or binding) refuses as `invalid_semantic_graph` at
the parameter node's `body`, every other mutation as `invalid_semantic_graph`
at the member it breaks, each located at the mutated node, and the
dimensionless compound unit admits.

## Model-owned members (FR-038-AC-27 through FR-038-AC-30)

Build a Semantic IR 2.0.0 document with an object type `Order` declaring the
operation `total(Integer): Integer`, select it in a package's lock and add a
`dispatch_call` on `Order.total` over a `Reference<Order>` receiver. Read it:
it admits. Rename the called operation to one the document does not declare
and read it: it refuses `ill_typed`/`operator-ineligible` at the member's
`name`. Give `Order` a supertype the document does not declare and read it:
the refusal is the document's, `missing_declaration`/`missing-name`, at
`/lock/model_selections/0`. Read the document directly: supply none, one
under another digest, forged bytes, another identity, and the matching one;
give a node an invalid object id and refer to it as a supertype and a
`typeRef` from nodes on both sides of it; give one field a dangling `typeRef`
and a multiplicity of `lower > upper` in both member orders; name a
relationship as a `typeRef` from a node read before and one read after its
owner; give two nodes no identity. Compare the whole refusal and its pointer
with the expected one. Read with the `work` limit at zero, and with each work
limit one below what the direct read and a member resolution used, and check
the pointer of the `incomplete` outcome resolves in the lock.

## Artifact references (FR-038-AC-46 through FR-038-AC-61)

Build a package whose `lock.edition.definition`, a `lock.profile_selections`
definition, `lock.definition_selections` entries, their `identity_preimage`
mirrors, `diagnostics.catalog` and an operation law `definition` are each
`{authority, identity}`, and whose `lock.sources` rows, `source_map` region
`source` and a `diagnostics.entries[].loci[].source` are `{authority, identity,
digest_domain, digest}`. Read it: it admits. Then, one at a time and at each
lock, preimage and catalog definition place, add `revision`, `digest_domain`,
`digest` and `export` to the reference and read: each refuses `unknown_member`
at that member, and a reference of the earlier five-member shape at its first
extra member. Add `revision` and `export` to each of the three source places and
read. Add an extra member, remove `authority`, and give `identity` a non-string
in an operation law `definition` and read. Remove `authority`, then `identity`,
from a definition reference at a lock, preimage and catalog place, give each a
non-string, and at the lock and catalog places the empty string. Remove each
member of a source reference at the three source places and give each a
non-string. In a `lock.sources` row set `digest_domain` to
`quire.definition.bytes/v1` beside an empty `authority`, then beside a non-hex
`digest`; then with the right domain empty `authority` and `identity` and give
`digest` uppercase and short forms. Over a region `source` and a locus `source`
that equal no lock row, differ each of the four members in turn, once with an
empty member, a non-hex digest and a wrong domain.

Over an `application` node carrying an `integer_division` law, apply a law
`definition` the catalog does not list (and an empty one), one it lists that the
lock does not select, and the selected one; and, over a `temporal_profile` law,
the lock's profile pair and another pair. Differ one `definition_selections`
row between the lock and the preimage by `authority`, then by `identity`; edit
one definition row's `identity` in both and re-derive the `package_id`, then
read the same edit under the old `package_id`. Point a `source` and a
`definition` nominal owner at pairs no lock row carries.

Test the operation catalog read as a unit test of the parser in
`crates/quire-contract-model/src/checked_package/v2/operation_catalog.rs` over
supplied bytes, not through a package: bytes whose `law_roles` entries are
`{authority, identity}` return the catalog, and bytes in which one entry
carries `revision`, `digest_domain` or `digest` return an error naming the
entry and do not panic.

Expected: the package admits; each extra member at a lock, preimage, catalog or
source place refuses `unknown_member` at that member; a missing or wrong-kind
member there refuses `malformed_wire` before any other check, and an empty
member at a lock or catalog definition place `malformed_wire`; the same
defects in an operation law `definition` refuse `invalid_semantic_graph`; a
`lock.sources` row of another domain refuses `digest_domain_mismatch` ahead of
its empty-member and digest-form checks, and with the right domain
`malformed_wire`; a region or locus `source` equal to no lock row refuses
`invalid_source_map` whatever differs; a law the role does not catalogue refuses
`operation-law-mismatch`, an unselected one `operation-law-unselected`, each at
the law's `definition`; the lock/preimage difference refuses `stale_dependency`
at the first differing value; the re-derived edit admits and the old
`package_id` refuses `stale_dependency`; the owner pairs refuse
`invalid_semantic_graph`. Compare each whole refusal code, cause and pointer.

## Dependency selections (FR-038-AC-31 through FR-038-AC-33)

Set the lock's and identity preimage's `dependency_selections` to two
`DependencySelection` entries in ascending identity order, re-derive the
package id and read: it admits, and both members read back as supplied.
Change one entry's `package_id` and check the package id changes. Mutate one
entry at a time to a wrong-domain, empty-identity, short-digest, bare-digest or
`Selection`-shaped entry and read. Repeat an identity,
adjacent and not, and reverse two entries, and read. Order two identities
whose UTF-8 and UTF-16 orders differ and read. Compare the whole refusal code,
cause and pointer with the expected one.

## Operation identity, laws, mode, member and operands (FR-038-AC-81 through FR-038-AC-88)

Run the operation check of one `application` node as a unit, over a graph of the
node and the type nodes it names, and the application key check of a graph of
application nodes. Name an identity the catalog lacks, and `quire.op.integer.add`
under `unary`; admit `quire.op.integer.add` under `binary` with two plain
operands, and give `operation` a bare string (AC-81). Supply no law to `quire.op.integer.div`, one law to
`quire.op.integer.add`, and to `quire.op.integer.div` a law whose `role` is
`not_integer_division` over a catalogued `integer_division` definition (AC-82).
Give `quire.op.decimal.add` a `null` mode; supply the mode `toward-zero` over a
first operand that is a `reference` to a `decimal_range` binding `rounding`
`nearest-even`, and over a `literal` typed at it (AC-83). Give a `null` member to
`quire.op.quantity.convert` with its `rounding` mode supplied; name an undeclared field in
`quire.op.record.project`; name no node as the `declaration` of
`quire.op.model.reaches_field` (AC-84). Give `quire.op.integer.add` three
arguments, an `integer`-typed and a `boolean`-typed literal first, and
`quire.op.boolean.not` a `quire.op.state.clause` application (AC-85); give
`quire.op.structural.eq` two parameters of one record type and of two, two typed
literals of one type and of two, and two nested applications of one result type
and of two (AC-86). Compare the operand family and the type-shaped predicate of
every kind of the closed node taxonomy with the stated pairs, the
`temporal`/`formula` pair added when the IR-503 code lands and the
`composite_type`/`union` pair when the IR-549 code lands (AC-87). Key an
application node that is member 1 of a group of two and compare its canonical
preimage text with the object FR-038-AC-88 lists, member by member; rewrite group references in an aggregate member, a
binding value and nested applications' arguments; read a node with a stale
`node_id` and one keyed by its own preimage (AC-88).

Expected: the uncatalogued identity refuses `unknown-operation` at
`operation.identity`, the `unary` case `operation-class-mismatch` at `operator`,
and the correct use admits; `operation-law-missing` at `operation.laws`,
`operation-law-mismatch` at `operation.laws/0` and at the law's `role`; the
absent mode `operation-mode-mismatch` at `operation.mode` and the disagreeing
value `operation-mode-type-mismatch` at `operation.mode/value`, for the
`reference` and the `literal` (the `null` mode and `null` member cases; an omitted
member reads as `null`, a reader leniency this case does not pin); `operation-member-mismatch` at `operation.member`
and `operator-ineligible` at `operation.member.name` and
`operation.member.declaration`; `operator-ineligible` at `arguments` for the
third argument and at the argument for the `boolean` literal and the clause, and
at `arguments/1` for each differing type; every same-type admission admits; the
table equals the stated pairs; the bare string refuses `invalid_semantic_graph`
at `operation`; the preimage equals the object AC-88 lists, each reference is rewritten
as stated, the stale key refuses `stale-node-key` at `node_id` and the genuine
one admits. Compare the whole refusal code, cause and pointer. The `definition`
of a law of the right role is FR-038-AC-56 and FR-038-AC-57's, and the modes of a
leaf are FR-038-AC-44's.

## Typed scalar application operands (FR-038-AC-159 through FR-038-AC-164)

Read an admitted package containing integer add, subtract, multiply and negate
applications. For each, call `scalar_application_operands` with its node id and
an actual occurrence key. Supply, across the applications, references to two
distinct `value`/`parameter` nodes of the same type, a reference to a nested
application result, a reference to a graph literal, and inline integer
literals. Include two occurrences of one application and two inline literal
positions of the same value. Swap an add application's arguments in a
separately admitted package, then call again. An admitted example argument is
`{"term":"literal","type":<integer-type-node-ref>,"value_kind":"integer","value":"7"}`;
unlike `{"term":"reference","target":<literal-node-ref>}`, this inline term
has no child graph node id.

Expected: each result contains exactly the argument count and order, with
zero-based ordinals; parameter and subterm references return their distinct
`GraphChild` node ids and admitted ranges, and both literal forms return
singleton `(7, 7)` for value `7`. The graph literal returns its node id;
each inline literal returns `InlineLiteral` with the application id, supplied
occurrence and position, distinct across either change. Swapping arguments
swaps entries. Supply an unknown id, a non-application node, a different
occurrence key, and a referenced child or range removed by a crate-internal
post-admission mutation; assert the respective typed errors and no partial
result. In separately admitted packages, put a parameter typed at unbounded
`Integer`, then a parameter typed at an `integer_range` with upper endpoint
`170141183460469231731687303715884105728` (one above `i128::MAX`), in an
otherwise eligible add application. Assert `UnboundedRange` for the first and
`RangeOutOfI128` for the second. Also use a graph literal reference and an
inline integer literal with that same out-of-`i128` value as separate operand
cases; each returns `RangeOutOfI128`. Repeat the bound case with lower endpoint
`-170141183460469231731687303715884105729` (one below `i128::MIN`). Assert
each call returns only the typed refusal, without a partial list, narrowed or
saturated endpoint, or panic. The `i128` endpoints themselves return their
exact values. Mutate an admitted application's catalog identity to an unknown
one in the same way, and call an admitted application of a catalogued ineligible
identity; assert `UnknownOperator` and `IneligibleOperator` respectively.
An external API fixture exhaustively matches the public identity and error
enums and consumes every result member without JSON access. Repeated calls
and a package clone return equal values, with package equality unchanged.

## Typed package-authored composite operand projection (FR-038-AC-177 through FR-038-AC-182)

Implemented by `composite_application_operands` and exercised by the `tc_048_`
composite operand integration tests and narrowly scoped model-unit tests. Admit structural.eq applications through the
production reader and published catalog using independently authored inputs.
Use authentic parameter and closed graph-composite value references, with two
actual occurrences of one application. Assert the tabled application, occurrence,
ordinals, child identities, type identities and domain fields. Swap arguments
and repeat one parameter without deduplication (AC177).

For AC178, compare a supported closed record/tuple/collection/Option graph value
with its original node identity and assert Literal disposition/no positions.
Include a graph value whose member reads a parameter and an application subterm;
assert UnsupportedOperand with exact reason, enclosing ordinal and available
type identity. A closed union-value root and one nested in a closed record value
shall instead refuse UnsupportedDomain with the enclosing ordinal and original
union semantic-type identity, never NonliteralGraphValue or Literal success.
Use a reader-admitted noninteger inline term with structural type
where possible, such as an inline none term typed at Option, to exercise
InlineNonInteger. Defensive inline integer handling may use a crate-internal
post-admission mutation only: the catalog excludes Integer from structural_kind,
so no successful public structural.eq integer-inline fixture is claimed. Assert
InlineInteger refusal/no fabricated identity or value; retain reader admission's
original refusal on a separately authored integer-inline structural.eq input.
Future eligible integer-inline success/value transport requires an owning catalog
and accessor specification, not a test that bypasses admission.

For AC179, use a record with a bounded Sequence of integer-range elements.
Independently enumerate field and element paths from the original admitted type.
Assert exact lower/upper strings, including integer endpoints beyond i128 and
collection maxima beyond u64; preserve collection minimum too. Replace a path's
type in a separately admitted input and assert only its affected descriptor changes.
Repeat with unbounded Integer and collection and assert no invented maximum.

For AC180, build root List with head and an optional-presence tail. Its field
binding value is the prescribed aggregate/optional binding referencing Option<List>.
Assert the field edge retains optional_presence=true, the aggregate/binding add
no index, the actual reentry is [1,0] and the Depth key path is []. Repeat with a
direct field reference to Option<List>, asserting optional_presence=false and the
same path. Root Tree with left/right optional self-fields retains both [0,0] and
[1,0] reentry paths in lexicographic order at a single Depth key []. Contrast two
nonrecursive sibling fields sharing a type: both paths remain, with no Depth.
Cover tuple and Option ordinal edges. Assert each key's actual parameter root.

Enumerate all eleven scalar and seven bounded-domain table rows through eligible
containing structural types and actual admitted preimages. Assert Boolean's zero
positions, each Whole source, each unbounded descriptor and each bounded source.
In particular text_bounds yields one Whole at its wrapper, and model_population
as a leaf yields a Node-keyed Whole with no Population key or member walk.
For an unordered Color enum, the admitted preimage member order is Blue, Green,
Red with ordered=false, independent of its source spelling. For an ordered enum,
assert ordered=true and its semantic preimage member order. No accessor output
supplies the expected identifier list. Assert the exact introduced key set.

For AC181, exercise all twelve public error variants and their exact tabled fields.
Use public admitted inputs for unknown node, nonapplication, missing occurrence,
known ineligible identity (including structural.ne), unsupported operands and
unsupported union projection; retain authentic ordinal/type fields. Crate-internal
mutations may exercise unknown catalog identity, dangling child, malformed domain
or optional wrapper and checked-index conversion ordinarily blocked by admission.
For a reference with a valid but absent target, assert MissingChild and the actual
child id; for an absent or malformed target id, assert MalformedChild with only
the enclosing ordinal, without fabricating or salvaging a child identity.
Do not allocate a multi-billion-element graph to test the conversion helper.
An unanchored forwarding/Option/collection cycle refuses MalformedDomain, not a
fabricated depth. Pair an early operand defect with a later one to assert order.
Measure successful consumed_work, then use its exact value, one less and zero.
Assert the same logical work for equivalent node order and lookup strategies;
expected charges come from the specified semantic visits, not implementation scans.

For AC182, compile an external consumer in the normal workspace test lane. It
exhaustively matches each named public output/key/descriptor/edge/error type and
reads the specified fields without body JSON or a QSL model import. Assert repeated
and clone projections and pre/post package equality. API inspection checks the
absence of drawn bound inputs, incoming-key validator, coverage verdict or obligation
encoder. structural.ne remains the declared consumer gap, not an enabled alternative.

## Operation leaf count (FR-038-AC-43)

Apply `structural.eq` and `structural.ne` to two parameters of an all-integer
record, and `collection.contains` to a set of integers and an integer, each
with `leaves` empty, and check each admits. Repeat over a record with a nested
`text` field and over a set of `text` with `leaves` empty, and over two `text`
fields with one leaf supplied, and check each refuses `invalid_package` with
cause `operation-law-missing` at `operation.leaves`. Apply `collection.flatten`
from a sequence of sequences of `text` to a sequence of `text` with `leaves`
empty and check it admits, then to a `set`, `bag` and `ordered_set` of `text`
and check each refuses `operation-law-missing`; do the same for
`collection.set` over `text`. Give a record a field naming a node that is not
in the graph, and a type that reaches an option again with no record or tuple
between the two visits, and check each refuses `ill_typed` with cause
`operator-ineligible` at `operation.leaves`. Give a record a field that reaches
the record again through an option and check it is no longer refused
(FR-038-AC-70), and a record whose two optional text fields name one option node
and check it admits. Nest `text` under 40 options and check it refuses
`operation-law-missing`, and nest integers 2000 deep and check the work budget
refuses. Chain 12 record levels of 4 fields each naming the next level and
check it admits within the budget. Put a leaf whose mode disagrees with its
field's pinned rounding beside too few leaves and beside a type naming a node
that is not in the graph, and check the refusals are `operation-law-missing`
and `operator-ineligible`, the leaf count settling before any leaf mode.

## Exact operation leaves (FR-038-AC-44)

Over a record of two text fields and an option of text, the text type a
`text_bounds` domain binding `nfc`, supply the three derived leaves, each with
one catalogued `text_profile` law the lock selects and the `nfc` mode, and
check it admits; do the same over a tuple, and over a set of text with the
one empty path. Supply two leaves over an all-integer record, one leaf more than
the text leaves, and a leaf to an operation that names no leaf source, and
check each refuses `operation-law-mismatch` at the first extra leaf. Supply
three unrelated paths, a path without its `inner` segment, two leaves swapped
and a `position` segment for a `field` one, and check each refuses
`operation-law-mismatch` at that leaf's `path`. Give a leaf no law, two laws, a
law of another role and an uncatalogued definition, and check each refuses at
that leaf's `laws`. Run the lock without the selection and check
`operation-law-unselected` at the `definition`. Leave a leaf's mode out, give
it another kind, an uncatalogued value, and a catalogued value other than the
pin, and check `operation-mode-mismatch` at `mode`, `mode/kind` and
`mode/value`, then `operation-mode-type-mismatch` at `mode/value`. Make one
text type bind no profile and check `operator-ineligible`, with and without
leaves supplied. Apply `collection.flatten` to a `sequence` of text with one
leaf and with none, and `integer.div` with an unselected law and a leaf, and
check `operation-law-mismatch`, admission, and `operation-law-mismatch` ahead
of the unselected law. Chain 16 levels of 10 fields all
naming the next over text and supply one leaf, and check
`operation-law-missing` comes back at once; make nine of the ten fields
integers and check the one 16-segment path admits and one wrong segment refuses.
Compare the whole refusal code, cause and pointer with the expected one.

## Recursive compared types (FR-038-AC-70 through FR-038-AC-72)

Declare `Node { label: Text[0, 8; nfc]; next?: Node; }` and apply
`structural.eq` to two parameters of it with the `field:label` leaf and the
recursion leaf `field:next`, `inner`, `recursion:0` after it, and check it
admits, and with the text leaf alone and the recursion leaf alone and check each
refuses `operation-law-missing` at `operation.leaves`. Compare `Option<Node>`,
the mutually recursive `A` and `B` at `A` and at `B` in one package, `Two`,
`Tree2`, a declared tuple `Pair` and `Wrap` over `X` and `Y` with the leaves
FR-038-AC-70 lists, recursion leaves included, and check each admits; compare
a `List` of integers by `structural.eq` and `collection.contains` with `leaves`
empty and check both admit. Give `Wrap` its four text leaves alone, and then all
six and a further text leaf, and check `operation-law-missing` and
`operation-law-mismatch` at the seventh entry. Add a second text leaf after
`Node`'s two leaves and check `operation-law-mismatch` at `operation.leaves/2`.
Supply the recursion leaves FR-038-AC-71 lists (before the text leaf, a wrong
segment, a wrong `d`, a second one, one with a law, one with a mode, one at a
reentry of the integer `List`) and check the code, cause and pointer of each; bind no profile on a text type inside the cycle,
unselect a leaf law, and use `T` as an `Option` of itself, a `Sequence` of itself
and a record holding a field of such a type under a work limit of 1000, and check
`operator-ineligible`, `operation-law-unselected` and `operator-ineligible`
(never `incomplete`); check `R { x: Option<R> }` admits. Build the 20000-record
cycle with the byte, node, edge and work limits raised and decide it on a 256 KiB
thread, build the ten all-referencing records and check `incomplete` for `work`
under the default limits, and bisect the work limit of a ring of 12 records, compared with its
12 text leaves and its recursion leaf, to the exact work (FR-038-AC-72). Compare the whole refusal code, cause and pointer
with the expected one.

## QSL-shaped optional record fields (FR-038-AC-151 and FR-038-AC-152)

Implemented by IR-644 in `tests/it/checked_package_v2_recursive_leaves.rs`; the assertions below run as TC-048.

Build a checked package with the QSL-emitted `List` shape: its `next` field's
value is an aggregate containing one `optional` binding whose value references
an `Option<List>` node; the option node references `List`. Compare two `List`
values by `structural.eq`. With an integer field, assert admission with no
leaves. Replace it with selected-profile text and assert admission with the
text leaf followed by `["field:next", "inner", "recursion:0"]`; omit the
recursion leaf and assert `operation-law-missing` at `operation.leaves`. Add a
text sibling after `next` and assert its leaf follows the recursion leaf in
declaration order. Run a direct-reference-to-`Option<List>` control and check that it admits
with the same field and inner path.

For the same otherwise well-formed comparison, replace the `next` value in
fresh packages with an empty aggregate, two `optional` bindings, another
binding name, a non-binding member, and an `optional` binding that does not
reference an option type. Recompute the package's identity members each time.
The flat body grammar admits each mutant. Assert that every one reaches the
operation check and refuses `ill_typed`/`operator-ineligible` at
`operation.leaves`, with no partial leaf derivation. An earlier grammar or
identity refusal fails the test.

## Canonical encoding (FR-038-AC-74 through FR-038-AC-80)

For each in-repo positive fixture, encode the identity preimage, graph, lock,
each source-map entry, each capability, each semantic id and the diagnostics
through `quire_canonical::to_vec` and compare the bytes with
`serde_json::to_vec(&serde_json::to_value(..))` of the same value, one assertion
per type, and check `quire_canonical::sha256` of the preimage equals the
fixture's `package_id.digest`; change one preimage member and check
`stale_dependency` at `/package_id/digest`. Add a graph with all four nominal
preimage versions, a `declaration`, a `recursion_group` and bodies of every
scalar kind, with non-ASCII and astral strings and the integers 9007199254740992,
-9007199254740992, 0 and -1, and compare again. Nest a body 100000 levels deep, put it in a
graph node, a preimage projection and a diagnostic's `details`, encode each on a
256 KiB thread and compare with the expected text written by repetition. Encode
the same values under a byte ceiling of the exact length and one byte below it,
encode a body nested 20000 levels, and encode a body holding the integer
9007199254740993. Read package documents (FR-038-AC-79) holding 9007199254740993
and -9007199254740993, the float `2.0` and a body object whose members are named
U+E000 and U+10000 in UTF-8 byte order in a node body, each also carrying a
grammar defect, and check `noncanonical_wire` with no pointer in every case;
read the same documents with 9007199254740992, `2` and the UTF-16 order and
check none refuses `noncanonical_wire`. For FR-038-AC-80, a test reads the
manifests, `deny.toml` and the crate source, lists every `derive` of
`FixedShape` and every `impl` of `Encode` and `FixedShape`, searches for
`const DEPTH`, and checks each of the seven wire types and each type they hold
is on the stated path; `make deny` passes.

## Identity digests through quire-canonical (FR-038-AC-89 through FR-038-AC-95)

Code change A of IR-274 implements the V2 reader and lowering cases below and
code change C the output-mapping identity steps and code change B the v1 part,
so FR-038-AC-91 and FR-038-AC-92 are scanned over `checked_package/`, over the
production source of `output_mapping.rs` (FR-038-AC-91's symbols and `u64::MAX`
ceilings) and over that of `canonical.rs` and `binding.rs` (the symbols and
calls, and the bound identity envelope deriving `FixedShape` with no `Value`).
For the owner-free nominal fixture, recompute every node key and `package_id`
and compare each with the digest the fixture recorded before the move; lower
every node and compare each `ir_id`, the lowered package's `package_id` and its
canonical bytes with the values recorded from the lowering before the move.
For owner-bearing fixtures, compare unchanged owner-free nodes with their
pre-owner golden, assert owner/projection equality and key recomputation in
TC-228, and use the authoritative owner-bearing golden from QSpec's published
positive packages and `model-member-type-vectors.json` under FR-038-AC-176.
Compare the canonical bytes of one
preimage of each kind with an
expected byte string written out in the test, not computed by the code under test
(FR-038-AC-89). Run
`make conformance-qspec` for FR-038-AC-176 with `QUIRE_SPECIFICATION_DIR`
pointing to the authoritative QSpec checkout. Require all nine
`proposals/checked-package-v2/fixtures/positive-*.json` packages named by the
criterion, not just the three AC-107 originally read. Supply the selected
`acme/orders` document from
`proposals/checked-package-v2/domain-package-acme-orders.json` and compare
every admitted package id with the published wire. Read node ids and owners
through each admitted package's graph accessor and compare them with its
published graph and identity projection; the corresponding declared `Point`
and `List` nodes in the two source-owner fixtures have different keys (AC-153).
This does not claim production source-owner key re-derivation (IR-630).
For every `model_declaration_nodes[*].preimage` and `.sha256` pair in
`proposals/checked-package-v2/model-member-type-vectors.json`, call the
production model-declaration key path in `model_members.rs`, or admit the
vector's published wire node through that path, and compare the production
digest with the recorded `.sha256`. Hashing only the vector's JSON preimage
cannot pass.
Fail the run when the selected domain document is missing or malformed, a
required positive fixture is absent, a model selection names an unknown
identity or lacks its digest, or a positive package is not admitted. For each
case, remove or corrupt only that input in a fresh run and assert a failure
with the affected file or selection named; never skip the case or silently
read an in-repo substitute. These negative runs are separate from the
AC-112/AC-113 fail-closed probes above.

FR-038-AC-176 is implemented by `tests/conformance_qspec/main.rs` for all nine positive packages,
their selected `acme/orders` document, graph/owner readback and fail-closed inputs. The same explicit
`make conformance-qspec` target runs the `model_members::declaration_key` unit test against every
published model-declaration vector through the private production function; ordinary
`make test` needs no external QSpec checkout. The focused target passed against the named checkout.

Call
`NominalIdentityPreimage::digest` for a preimage of each of the four versions with
a limit equal to its canonical length and one byte lower (FR-038-AC-90). A test
reads `canonical.rs`, `binding.rs`, `output_mapping.rs` and `checked_package/` and
counts `CanonicalWriter`, `canonical_envelope_bytes`, `digest_json`,
`serde_json_canonicalizer`, and `serde_json::to_vec` and `serde_json::to_value`
calls reaching a digest (FR-038-AC-91, including its no-`value_to_vec`, no-`Encode`-for-`Value`
and `serde_json` feature clauses); it lists
the derives and `Encode` implementations of the six preimage types of the
assignment table (FR-038-AC-92). Read a package that selects a model document
holding 9007199254740993, -9007199254740993, 9.007199254740993e15, 1e20 and
18446744073709551617 at `/package/count`, under the document's own digest and
under another, and check `noncanonical_wire` at
`/lock/model_selections/0/digest` with `document_pointer` `/package/count` and no
pointer on a package-stream refusal; repeat with 9007199254740992,
-9007199254740992, 9.007199254740992e15 and `0.5` and check none refuses for it;
put two such numbers in the document and check the first in document order is
named; give an integer value type the upper bound 9007199254740993 and check it
refuses (FR-038-AC-93). Supply a model document whose bytes are exactly
`limits.bytes` long, then lower `limits.bytes` by one and check `incomplete` for
`bytes` with no pointer, and a work limit one low with the row pointer
(FR-038-AC-94). Drive the lowering through a ceiling seam at a node preimage's
length and one byte lower through `identify_node`, and through a whole call of
`lower` at a ceiling the package fits under but one node's preimage does not
(the siblings `lowered`, and equal to the records of the same call with that
request removed), and at a lowered package's length and one byte lower through
`lower`, and through `identify_node` at a ceiling mid-way through a string value
of at least two bytes (chosen per fixture so the byte is not one the encoder
writes alone, nor inside an escape run) and with an in-memory preimage holding an
integer past 2^53 (the reader would refuse it, so it is built past the reader)
under a ceiling that reaches the number and is below `u64::MAX`; check that each `failed`
record names the `bytes` limit with the retained limit as `limit` and as
`consumed` the `required` of encoding the same value under that limit (above
`limit`, at most the canonical length; at the mid-string ceiling neither `limit + 1`
nor the full length; `limit + 1` for the integer past 2^53), that a
package over the ceiling returns no lowered node and no dependency node, and
that a work-budget failure still names `work` (FR-038-AC-95).

## Inexact numbers (FR-038-AC-109 through FR-038-AC-111)

FR-038-AC-109 through FR-038-AC-111 are implemented and verified in
`tests/it/checked_package_v2_model_members.rs` and
`tests/it/checked_package_v2_canonical_encoding.rs`, and the existing FR-038-AC-93
tests there carry the cause `inexact-integer`; the `1e400` and `-1e400` mapping
of FR-038-AC-110 (IR-555) is verified through `quire-canonical`'s
`NumberOutOfRange`, `1e-400` and `-1e-400`, which read as zero with their
text kept, through the rule on that text, and the first-fault rule (QSL
FR-056, QSL #625) in `tc_048_the_first_reader_fault_decides_when_faults_coexist`
(IR-573). The causes are those
of quire-specification:FR-271 and FR-272. Read a package that selects a model
document holding `0.1000000000000000000001`, `9007199254740993.5`,
`-0.1000000000000000000001`, `4.9e-324` and `1e-400` at `/package/ratio`, under
the document's own digest and under another, and check `noncanonical_wire` at
`/lock/model_selections/0/digest` with `document_pointer` `/package/ratio` and
cause `inexact-number`; repeat with `0.1`, `0.5`, `1.5`, `-0.25`, `5e-324`,
`2.5e-10`, `1.0`, `-0` and `1e2` and check none refuses for it; repeat with the tie texts `1125899906842624.2`, `1500000000000000.2` and `2.9802322387695312e-8` and check none refuses, and with `1125899906842624.3`, `1500000000000000.3` and `2.9802322387695313e-8` and check each refuses `inexact-number`; digest two documents differing only in
`0.1` and `0.1000000000000000000001` and check the second refuses
(FR-038-AC-109). Repeat the numbers of FR-038-AC-93 (including the integer value type
upper bound), `9007199254740993.0`, `1e400` and `-1e400` and check cause
`inexact-integer` and `noncanonical_wire`, not `byte-digest-mismatch`; check `9007199254740992.5` refuses
`inexact-number` and the admitted whole numbers still admit; put an inexact
number and a whole number past 2^53 in one document in both orders and check the
first in document order is named with its own cause; put a number past the
double range after an inexact number (`{"b":0.1000000000000000000001,"a":[1e400]}`,
`{"b":9007199254740993,"a":[1e400]}`) and check the out-of-range number is named
(`/a/0`, `inexact-integer`); read `{"a":1,"a":2,"n":1e400}` and `[1e400` and
check `/n` and `/0` are named, and `[{"a":1,"a":2},1e400]` and
`{"a":1,"a":2,"n":1e-400}` and `[1e400,"<0xFF>"]` under a digest that is not
their raw digest and check `stale_dependency`/`byte-digest-mismatch`, and
`[1e400,"\ud800"]` and check `/0`, the first fault `read` returns deciding
(FR-038-AC-110; the outcome under a raw-path document's own raw digest is not
pinned, IR-578). Read a
package document holding `0.1000000000000000000001` and `9007199254740993.5` in a
node body and check `noncanonical_wire` with no pointer, no `document_pointer`
and no cause, and `0.1`, `1.2793061557049685`, `1.2106592671318679` and `1.3567384036451073` not refused for it; read the manifest of the crate that holds the
reader and check it declares `serde_json` with the feature `float_roundtrip` (a
source-level check; the package document is the read that goes through
`serde_json`) (FR-038-AC-111).

## Dependency references (FR-038-AC-35 through FR-038-AC-38)

Declare the fixture's `function` node as a dependency function over a
bounded integer returning Boolean, read it, and lock its `package_id` as a
`dependency_selections` entry of a package whose function call has a
`dependency_reference` callee with that entry's `package` and the function as
`node`, its application key re-derived and no `dependencies` entry for the
target. Read it with the dependency supplied and check it admits and the term
is in the body verbatim. List the target in `dependencies`. Change only the
term's `package`, then only its `node`, keeping the key, and check each is a
stale key; re-derive each and check the three ids differ. Mutate the term to a
bare-digest, other-domain and short-digest `package`, another-domain `node`,
a missing `node` and an extra member. Place the well-formed term as a second
argument, as the argument of another operation, as an aggregate member and as a
node body root. Supply no package, another identity and a package of another
`package_id`. Name a `package` no entry carries, a `node`
that is absent, one with no `declaration` and a declared record. Give the
dependency function a declared-record parameter and result, a `Set` of a
declared record, a tuple holding one and a `Reference` to a `model` node, and a
`Set` of a bounded integer. Compare the whole refusal code, cause and pointer
with the expected one.

## Enum operand family (FR-038-AC-42)

Verify QSpec FR-322's rule that an enum is `ordered_enum` when its nominal
preimage is ordered and `enum` otherwise, and FR-141-AC-5, in
`tests/it/checked_package_v2_enum_order.rs`. Build a package holding an enum
declaration, its member nodes and a `binary` application of one enum
operation over two operands of that enum, as member literals and as
`value`/`parameter` nodes. Apply `quire.op.enum.lt`, `le`, `gt` and `ge` to an
ordered enum, one listing its members in declaration order and one whose
declaration order is also sorted, and to an unordered enum, one listing its
members sorted. Apply `quire.op.enum.eq` and `ne` to each. Apply `lt` to
operands of two different ordered enums.

Expected: the four ordering operations admit over an ordered enum however its
members are listed; they refuse `ill_typed`/`operator-ineligible` at the first
argument of the application over an unordered enum; `eq` and `ne` admit over
either; `lt` over two different enums refuses `ill_typed`/`operator-ineligible`
at the second argument.

## Refusal and limit locations (FR-038-AC-24 through FR-038-AC-26)

For every refusal above, compare the whole refusal, including its pointer,
with the expected one. Check that each pointer resolves in the mutated
document. Add an unknown member whose name holds `~` and `/` at the top
level, on a node and on a locked source, and repeat a top-level member whose
name holds both. Dispatch the unknown versions again and read the version
each refusal carries. For each one-over limit, compare the pointer the
`incomplete` outcome carries and check that it resolves.

Each refusal points at exactly the value its mutation changed, with `~`
escaped as `~0`, `/` as `~1`, and array elements by index. A missing member
is located at the object that lacks it. Malformed JSON and non-canonical
bytes carry no pointer. Every refusal code is unchanged. Each
`unknown_contract_version` refusal carries the exact string read, including
the empty one. Each one-over limit other than the byte limit names the value
whose charge failed, and that pointer resolves; the byte limit names none.

## Flat wire and no depth limit (FR-038-AC-114 through FR-038-AC-118)

Implemented (IR-495 code) in `tests/it/checked_package_v2_flat_wire.rs`, and AC-118
in `tests/conformance_qspec/main.rs`. In a fresh copy of a self-built package, place each of
`quire.op.function.call`, `quire.op.state.clause` and an application of the
`temporal`, `temporal_formula` and `temporal_fairness` classes inside an
application's `arguments`, as an `aggregate` member and as a `binding` value of
a node body, and read each; put an `aggregate` inside a Group's members, a
`binding` of an `aggregate` inside an `aggregate`'s members, a `binding` whose
value is a `binding`, a `binding` and a Tuple inside a Tuple's members, and a
`binding` as a body root; then write each of those bodies with its composite
subterm as its own node reached by `reference` (AC-114). Nest a `case`
application inside another term; put a non-`case` application at the body root
of a node its class does not place it in; put a `temporal_formula` application
with a nested `case` argument, an aggregate of a `temporal_formula` and then a
`case` application, and an application of another class in a diagnostic's
`details` term, and in a node body a nested `case` at `arguments/0` beside a
nested call at `arguments/1` and the two swapped (AC-115). Leave `node_id`,
`identity_preimage` and `package_id` stale in a package whose body holds a nested
non-`case` application, and in one whose body holds a nested `case`, and read the
first again with the body flattened (AC-116). Scan the source files under
`crates/quire-contract-model/src/checked_package/` (not the v1 modules or the
manifest) for `MAXIMUM_DEPTH`, any `MAX_*DEPTH`, `stacker`, `serde_stacker` and
`on_stack_for`, and check `CheckedPackageReadLimits` and `CheckedPackageLimit` for
a depth member and variant; build a chain of 100000 nodes each referencing the
previous, read it on a thread whose stack is 256 KiB under every limit sized so
that none decides the outcome, lower its last node, and compare its JSON nesting
depth with that of a one-level package; read an otherwise canonical document whose
node body nests a term 300 levels deep on a 256 KiB thread, and the same document
nested 20 levels deep and nested 41 to 61 aggregates deep (up to 126 JSON levels,
within the parse's limit; a `details` term nested 50 deep too) in a debug build
(AC-117). Run `make conformance-qspec` and read the harness's
expected-failure list (AC-118).

Expected: each nested non-`case` application refuses `malformed_wire` at the
nested application, each Group and binding misplacement at that value, and each
flattened form is not refused `malformed_wire`; the nested `case` and the
misplaced body root refuse `ill_typed`/`operator-ineligible` at the pointer
AC-115 gives, the `details` and node-body cases at the first construct in document
pre-order, and the `details` application of another class `malformed_wire`; the
stale non-`case` package refuses `malformed_wire`, the stale nested-`case` package
`ill_typed`/`operator-ineligible` at the `operator`, and the flattened form refuses
at an identity check; the scan finds none of the names and no depth member or
variant, and the 100000-node chain is admitted and lowered with no outcome naming a
depth and no stack overflow; the 300-level document refuses `malformed_wire` with no
pointer at the strict parse and the 20-level one, and each one within the parse's
limit, at the first value outside the body grammar, with no stack overflow; and each of the five
`body_grammar_mutations` refuses `malformed_wire` with the expected-failure list
empty of them.

## Timed interval form (FR-038-AC-119 through FR-038-AC-122)

Implemented (IR-551). Bounds are written `{numerator, denominator}` as `{n, d}`.

Admission (AC-119). Under a clause whose `temporal_profile` law names
`quire.temporal.timed/v1`, on each of the eight interval operators of a
self-built package, read the timed form `{lower, upper, lower_end, upper_end}`
with bounds `{0, 1}` and `{3, 1}` for each of the four end pairs, with `lower`
`{1, 2}`, as the punctual `[3, 3]` (both bounds `{3, 1}`, both ends `closed`), and
read the `null` interval.

Pattern refusals (AC-120). Under `timed/v1`, and again under
`quire.temporal.infinite-trace/v1` and each of the three bounded profiles, read the timed form with
`lower` numerator `"-1"`, `lower` numerator `"01"`, `upper` denominator `"0"`,
`upper` denominator `"-2"`, `lower` a JSON integer, `lower` the string `"3"`, and
`upper` `null`; with `lower` numerator `"-1"` and `upper` denominator `"0"` together;
a package with a non-reduced bound `{2, 4}` in the first node and a pattern
failure in a later node, and the two swapped; and a package whose node ids and
`package_id` are all stale, and one whose `package_id` alone is stale, holding a timed
or an integer bound outside its pattern.

Lowest terms and bounds (AC-121). Read `lower` `{2, 4}`; `lower` `{2, 4}` with `upper`
numerator `"-1"`; `lower` numerator `"-1"` with `upper` `{2, 4}`; `upper` `{2, 4}`
alone; `lower` `{2, 4}` beside a profile-fit defect in a lower-digest node; `(3, 3]`,
`[3, 3)` and `(3, 3)` with both bounds `{3, 1}`; `lower` `{5, 2}` over `upper` `{2, 1}`;
`lower` `{1, 2}` with `upper` `{2, 3}` and the two swapped; `lower`
`{18446744073709551617, 3}` with `upper` `{18446744073709551616, 3}`, both ends
`closed`, and the two swapped; and that beyond-2^64 package (`lower`
`{18446744073709551617, 3}`, `upper` `{18446744073709551616, 3}`) under a work limit
that its GCD or cross-multiplication takes past.

Profile fit and member-set defects (AC-122). Read the timed form with bounds `{0, 1}`
and `{3, 1}` and both ends `closed` under `infinite-trace` and under each of the three
bounded profiles and under `timed/v1`; the form with valid rational bounds and
`lower_end` `"half"`, and with `upper_end` `"half"`; `{lower: "0", upper: "3",
lower_end: "closed"}` and the same with the four ends and a fifth member `extra`, both
with integer-string bounds; the same two with the rational-object bounds of the timed
form; and a clause holding such a member-set defect beside a profile-fit defect in a
higher-digest clause.

Expected: AC-119's inputs all admit. AC-120's inputs refuse
`invalid_package`/`invalid-value` at `.../interval/lower` or `.../interval/upper`
(the failing bound, first in member order, so `lower` for the two-failure input), in
strict wire validation and under each profile read, the package with the later pattern
failure refuses at that later node, and the stale-identity packages refuse
`invalid-value` at the bound and not at an identity check. AC-121: `{2, 4}` refuses
`invalid_semantic_graph` at its bound, including beside the profile-fit defect; the
`{2, 4}` lower with a negative upper refuses `invalid-value` at `upper`, the negative
lower with a non-reduced upper `invalid-value` at `lower`, the `{2, 4}` upper alone
`invalid_semantic_graph` at `upper`; `(3, 3]`, `[3, 3)`, `(3, 3)` and `{5, 2}` over
`{2, 1}` refuse `invalid-value` at `/semantic_graph/nodes/{n}/body`; the `{1, 2}`
to `{2, 3}` pair admits and its swap refuses; the beyond-2^64 pair refuses and its swap
admits; and the work-limited read returns `incomplete` naming the work limit. AC-122:
the closed `closed` form refuses `operation-member-mismatch` at the application under
`infinite-trace` and the bounded profiles and admits under `timed/v1`; the two `"half"`
inputs, the missing-end and fifth-member inputs with integer-string bounds refuse
`operation-member-mismatch` at `operation.member`, reported after the higher-digest
clause's profile-fit defect; and the two inputs with rational-object bounds refuse
`invalid-value` at `.../interval/lower`.

## FCD relationships and named ends (FR-038-AC-165 through FR-038-AC-173)

Status: **PLANNED / UNRUN**. These procedures are new acceptance work for
IR-661. Existing TC-048 implementation and trace tags do not establish them.
They require independently authored minimal Semantic IR 2.0.0 documents and
packages built from this repository's public vocabulary. Do not copy an FCD
schema, fixture, binary or another repository's package into this repository.
The FCD primary-source links in FR-038 "Selected relationship declarations"
identify the shape to implement; the cases below are procedures, not execution
results.

1. Author one selected package of object types `Order` and `Customer`, with a
   relationship owned by `Order` whose identity is
   `ix://example/shop/relationship/Order-billedTo-Customer`, source role
   `billedTo`, source type `ix://example/shop/Order`, target role `bills`, and
   target type `ix://example/shop/Customer`. Supply each end's independently
   authored multiplicity and the FCD-shaped category, composite, direction and
   origin members. Keep fields and an operation on `Order` owner-nested, and
   the operation's parameter nested under the operation. Derive the selection
   evidence and package identities through the authoritative canonical encoder
   after every document edit. Admit the document and a correctly owned
   `relation`/`relationship` graph declaration. Independently change only the
   relationship identity to an owner-nested, foreign-package and wrong-slot
   identity; change a field and then an operation to the global relationship
   slot. Compare each typed `invalid_model_binding`/`malformed-declaration`
   outcome. Separately key a `model`/`object_type` node with the relationship
   owner: the owner join refuses `missing_declaration`/`missing-selection`,
   as FR-038-AC-155 requires.
2. Read with both roles, then remove only the target role. The selected
   declaration admits in both cases and the latter exposes only the forward
   named end. Change source role `billedTo` to `billedToChanged`, updating the
   relationship's minted identity and all canonical joins: the old name
   refuses `missing_declaration`/`missing-name`, while the new name admits the
   otherwise same forward navigation. Repeat this observation for the target
   role on an eligible finite inverse case. These calls, not a new accessor,
   observe role retention. Separately remove the source role; set each role to
   null, `""`
   and a non-string. Every malformed-role mutation refuses
   `invalid_model_binding`/`malformed-declaration` at the selection row, even
   with no application naming that relationship. Use an arbitrary non-empty
   target role to confirm the reader does not enforce the frontend registry.
3. Mutate each end's type to a missing identity and to the identity of a
   relationship. The missing targets refuse `missing_declaration`/`missing-name`
   and the relationship targets refuse
   `invalid_model_binding`/`malformed-declaration`. Run each with the target
   sorting before and after the referring relationship. Independently make
   an end malformed, remove its type or multiplicity, make a multiplicity
   malformed, and reverse otherwise valid bounds. Also point the source type
   at a valid object type other than the owning type: this refuses
   `invalid_model_binding`/`malformed-declaration`. Compare the declared typed
   refusals and selection-row pointer. Under the planned retention contract, also
   require the actual relationship declaration identity and its valid typed origin,
   distinct from the enclosing owner and any graph-node locus. Compare source
   strings/coordinates/optional ends exactly; compare generated strings and ordered
   input identities exactly without a fabricated span. Missing/malformed origin
   stays absent. These retention assertions remain PLANNED / UNRUN until CODE
   implements them; no row returns a package or changes refusal precedence.
4. Repeat one relationship identity under the same owner and under a second
   owner: each is `invalid_model_binding`/`conflicting-binding`. Remove the
   identity from two relationships: each is a malformed declaration, never
   a conflict. Combine a malformed role, a missing end type and reversed
   multiplicity on one relationship, then restore the role, and compare
   FR-154's earlier-row refusal. Point at a declaration independently refused
   for identity or kind and confirm its own refusal survives in either node
   order. Measure the cumulative work threshold through the successful selection
   stage, keeping other limits sufficient. One below that threshold returns
   `incomplete` at the selection row with no partial admitted package; the
   threshold pays the stage, not necessarily the later complete read. Separately
   measure the total-reader limit in step 8.

5. Build a `quire.op.model.navigate` application with a
   `relationship_end` member whose `declaration` names the relationship graph
   node from step 1 and whose `name` is `billedTo`. The relationship node's
   body is empty and its content-only `ModelOwner` names the selected global
   relationship identity. Give operand 0 type `Reference<Order>` and the
   result type `Reference<Customer>`; with FCD-shaped `source-to-target`
   direction and target multiplicity `[1,1]`, it admits. The source's
   multiplicity is `[0,unbounded]` and does not determine the forward result.
   Remove only the target role and confirm the same forward application still
   admits. On that document, request `bills`, a wholly unknown role, and the
   relationship identity's terminal segment as `name`, independently: each
   refuses `missing_declaration`/`missing-name` at `member.name`. Keep the
   target role absent and combine the unknown name with a wrong receiver,
   disallowed direction and wrong result: missing name still wins. Change
   `declaration` to the valid source object node, then the valid target object
   node: owner recovery succeeds for that object and the new member-kind
   binding check refuses `ill_typed`/`operator-ineligible` at
   `member.declaration`, without rejecting those object nodes' own owners.
   Combine that valid object declaration with an unknown role to confirm
   declaration-kind eligibility precedes role lookup. Independently make the
   relationship node's owner undeclared or wrong-kind and observe the earlier
   existing owner join (FR-038-AC-155). Finally, give both end roles the same
   name and request it: two role matches refuse
   `ambiguous_declaration`/`ambiguous-name`.
6. On an independently authored schema-valid relationship with `bidirectional`
   direction and unordered finite multiplicities, navigate forward over
   `Reference<Order>` and inverse over `Reference<Customer>`, using the same
   relationship graph node and its two role names. Change only the destination
   multiplicity through `[1,1]`, `[0,1]`, and `[2,3]` with unique true and
   false. Compare the derived result nodes against `Reference<U>`,
   `Option<Reference<U>>`, `Set<Reference<U>>[2,3]` and
   `Bag<Reference<U>>[2,3]`, where `U` is Customer forward and Order inverse.
   Vary source and target multiplicities independently to show that the
   opposite destination end, not the receiver end, determines each result.
   These finite bidirectional documents are self-authored contract cases,
   not a claim that today's extraction frontend emits that direction or
   source multiplicity. Add `PriorityOrder extends Order` and
   `PreferredCustomer extends Customer`; a receiver of each subtype admits
   the inherited forward or inverse end and keeps the declared destination
   result node. A third unrelated object type refuses. Change the destination
   type to another valid object type, updating its relationship identity and
   joins, and observe that only the corresponding new result node admits.
   Independently give a forward call the unrelated target receiver
   type and an inverse call the source receiver type, and give a well-typed
   receiver a different result node: each refuses
   `ill_typed`/`operator-ineligible`, respectively at `body.arguments[0]`
   and `body.result_type`.
7. Retain both role names and finite unordered destination multiplicities.
   For each direction, read the corresponding forward and inverse
   applications: `source-to-target` admits only forward, `target-to-source`
   only inverse, and `bidirectional` and `undirected` admit both. A disallowed
   traversal refuses `ill_typed`/`operator-ineligible` at `member.name` even
   when the inverse name exists. With an eligible direction, give the
   destination an unbounded upper and then `ordered: true`: each refuses
   `unsupported_construct`/`expression-form` at `member.name`. Restore the
   exact FCD source-to-target shape, including the unbounded source end, and
   name its present inverse: direction refuses first, never a fabricated
   inverse result type. No row changes reader limits to admit unsupported
   navigation.
8. For each new operation refusal above, compare the typed code, cause,
   RFC 6901 pointer and calling application key. Keep graph dependencies,
   derived identities and other operation members current after each mutation
   so the intended check is reached. Combine owner-recovery, role and receiver
   defects to verify recovery wins; combine wrong receiver, disallowed
   direction, unsupported destination multiplicity and result mismatch to
   verify receiver wins; then repair successive defects to observe direction,
   multiplicity and result comparison in order. Measure the total work of a
   successful complete relationship-end read; the exact measured total admits
   and one less returns `incomplete` at the first unpayable charge in the
   existing stage order without a partial package. Operation validation is
   charged before later graph-body reference edges, so a final argument-edge
   charge may report `/semantic_graph/nodes/<n>/body/arguments/0/target`;
   do not force this total-reader oracle to the selection row. Separately
   retain step 4's selection-stage-minus-one selection-row oracle. These are
   acceptance procedures to execute in the code
   lane, not recorded measurements. External QSpec #191 relationship fixtures
   remain planned and unrun until merged and published by their owner; this
   procedure neither copies them nor tags them as current evidence.

9. Before any application uses the relationship, independently remove
   `direction`, `category`, `composite` and `origin`; set each to null and a
   wrong JSON type; use `diagonal` for direction and `unknown` for category;
   and supply an origin with no admitted branch, both branches, a missing
   required source-locus member or a malformed generated-origin member.
   Each refuses `invalid_model_binding`/`malformed-declaration` at the
   selection row, retaining typed code, cause and pointer under step 3's
   existing refusal contract. Require its authentic available declaration identity;
   missing/malformed origin retains no origin rather than a partial branch or span.
   These added metadata checks remain PLANNED / UNRUN. Valid
   independently authored source and generated origin branches each admit.
   No missing direction becomes source-to-target by default. Combine a
   malformed non-end member with a dangling end type and reversed
   multiplicity to confirm the malformed-declaration row precedes both.
10. Author a self-relationship on `Order` with the same selected relationship
    graph-node mapper, two distinct roles, bidirectional direction and finite
    unordered singleton destination. Read `quire.op.model.reaches` over two
    `Reference<Order>` values with the source role, then the inverse role:
    both admit with Boolean result. Make destination `[0,1]` and admit the
    corresponding optional edge; use `PriorityOrder` subtype operands and
    admit by static-edge conformance. Independently substitute an unrelated
    operand 0, then operand 1, and a heterogeneous destination type: each
    refuses `ill_typed`/`operator-ineligible`, at arguments0, arguments1 and
    member.name respectively. Give the homogeneous edge a finite Set or Bag
    destination rather than the admissible Reference/Option edge: each
    refuses the reference-edge check at member.name. Remove the requested
    inverse role and name it: missing name wins before operand checks. Name
    a valid object node as member.declaration: the new declaration-kind check
    refuses `ill_typed`/`operator-ineligible` there. Give an otherwise valid
    reaches application a navigation result type instead of Boolean: it
    refuses at body.result_type. Keep both operands' static object owners
    distinct from the relationship graph node; no Reference<relationship>
    substitutes for an object reference. These are static reader procedures,
    not runtime graph execution evidence.

Expected: each success and refusal above satisfies FR-038-AC-165 through
FR-038-AC-173. Compare typed code, cause and pointer, and the calling graph-node
key only
where the existing refusal retains its locus. At the private `admit_selection` return, selected-document declaration refusals
additionally retain authentic declaration identity and valid origin under
FR-038-AC-174/175 and FR-038-AC-186 through FR-038-AC-196, with absence for missing/malformed origin. The existing
model-intake conversion to `ValidationFailure` retains code/path/cause only;
these procedures assert no public metadata extension. Do not parse diagnostic
prose or infer a graph locus from a semantic declaration URI. All outcomes and
new retention checks remain **PLANNED / UNRUN** until the implementation lane
records executed evidence.


### Private intake refusal-origin checks

These Test and Inspection procedures back FR-038-AC-174/175 and
FR-038-AC-186 through FR-038-AC-196. IR-663 implements the private retention
contract. The final-head runtime tests and both Clippy lanes are UNRUN; full
pre-PR and premerge gates remain pending. AC-193/194 are source Inspections.
Existing AC-167/173 trace tags verify refusal/admission semantics only and
supply no evidence for these retention criteria. Exercise the real private model-intake return,
`admit_selection` yielding `SelectionFailure::Refused(SelectionRefusal)`, in
crate-local tests. Author minimal domain-package documents in this repository
and admit their selected bytes through the existing content checks. Follow the
owning FCD origin contract by reference; do not copy foreign schemas or fixtures.
No public reader/dispatch/driver metadata observation is allocated.

1. Cause a declaration reference failure with a valid source origin, then with a
   valid generated origin. At the private intake return after selected-document
   release, require the authentic referring declaration identity and every supplied
   origin member, with unchanged refusal code/cause and selection-row member.
   A nested relationship shall have identity/origin distinct from its parent;
   require the actual relationship's context, never the owner's or missing target's.
   Independently cause an unresolved field type, operation return type and parameter
   type, each with its own identity/origin different from the containing type and
   operation. Require the field, operation and parameter context respectively;
   remove each nested identity/origin to require genuine None rather than owner
   substitution. These are existing intake declaration forms, not new admissions.
   Separately check the existing public model-intake conversion's code, cause and
   selection-row pointer; preserve unrelated public refusal fields and graph locus.
2. Exercise all four end-coordinate presence combinations, positive coordinates
   at the existing selected-document numeric boundary, and otherwise valid origins
   that would fail an invented paired-end or span-order rule. Assert exact values
   and optional presence. Above-bound input shall retain its earlier numeric refusal,
   not acquire declaration metadata or a new origin classification. Vary identity,
   path and version spelling and generated input order/repetition. Assert no span
   on Generated, with typed exact comparison rather than prose parsing.
3. Remove origin; supply both branches, unknown members, invalid members and partial
   source/generated branches. Require None for origin, while retaining any actual
   supplied declaration identity. Missing/non-string identity shall retain None;
   an object-id spelling refused by intake shall remain supplied, not repaired.
   No default string, partial branch or enclosing-owner identity shall pass.
4. Group same-identity type declarations with identical valid origins, distinct
   valid origins, no valid origins, one valid plus missing origin, and one valid
   plus malformed origin. Require the common identity. Retain origin only if
   every candidate is valid and every typed value/presence agrees; otherwise
   require None. Repeat these cases both for conflicting-binding and for a
   malformed-declaration group (bad common object-id spelling or bad candidate
   kind). Permute candidate order without changing origin or first refusal.
   For AC-196, separately repeat a valid relationship identity under two owners with different
   origins: in the existing sequential node/member-path order require the later
   actual offending relationship's metadata, not the earlier node or owner.
   Give that later relationship missing/malformed origin to require None with
   its existing first refusal. This control must not use type-group consensus.

5. Audit every private `SelectionRefusal` constructor and declaration-to-selection
   conversion. Exercise pre-declaration missing bytes, content mismatch and wrong
   selection identity; require metadata absence and original code/cause/member.
   Combine declaration defects with earlier table/member-path defects and bounded
   work exhaustion; assert the same first refusal or distinct Limit/InexactNumber
   result and no partial package. For the inexact-number control, author bytes
   with an origin integer above 2^53 and a declaration defect, then supply them
   through the actual selection-evidence path under their raw-byte digest and
   under another document's selected digest. The original typed inexact-number
   result must precede dependency/declaration checks; this is an adverse read,
   not evidence that inexact input admits. Removing available metadata, attaching a foreign
   declaration's metadata, normalizing members or losing owned values on document
   release must fail a typed intake-return assertion.
6. Inspect that the `admit_selection` refusal conversion to `ValidationFailure`
   still carries existing code/path/cause and adds no declaration metadata fields
   to `CheckedPackageRefusal` or the IR-owned dispatch result. Retained `locus`, expected
   node key, contract-version and document-pointer behavior elsewhere remain intact.
   Future concrete typed consumers remain an open owner allocation, not this test's
   promised interface. No second parse, raised ceiling or diagnostic collection
   supplies retention. Source inspection alone cannot close the runtime Tests.
7. For AC-194, inspect that full typed-origin equality actually determines the
   production group retention decision. Follow the owned metadata in
   `SelectionRefusal` through `SelectionFailure::Refused` after `admit_selection`
   releases the document, to the existing `SelectionFailure::Refused(refused)`
   mapping arm in `checked_package/v2/mod.rs`. Pass only if that arm releases the
   actual owner by its end, by explicit ownership teardown or ordinary scoped
   destruction, while preserving code/path/cause. Fail if a field is read solely
   to silence lint, or its read affects neither the group decision nor ownership
   transfer. A blanket dead-code allowance, observer or public metadata field
   also fails. For AC-195, the CODE slice shall run both repository Clippy lanes,
   including the feature-off model-consumer command, and record their actual exits.
   If this ownership lifecycle is unavailable, report the source/API gap rather
   than claim the criterion satisfied. Inspect the named ownership and projection
   in `model_members.rs` and `v2/mod.rs`. Final-head runtime/Clippy checks are
   UNRUN; full gates remain pending.

## Retained expected node key (FR-038-AC-183 through FR-038-AC-185)

Implemented and run by the tagged `checked_package_v2_structural_keys` and
`checked_package_v2_model_members` integration tests, and the application-key
unit test. Exercise the real production reader and typed
`CheckedPackageRefusal::expected_node_id()` accessor. No test-side FR-092 encoder, copied digest, diagnostic parser or
public derive/rekey API is an oracle. Existing expected refusal values remain
**test-authored** tuples of the public fields, independent of reader output;
compare those fields one by one and assert the new accessor separately. For a
derived expected key, the fresh package's full admission and the changed-key
control below are the oracle; comparing two reader outputs with each other
does not pass.

1. For the closed-shape `derived_key` branch, use an unreferenced
   `integer_range` such as the builder's `base.zero`. Mutate a preimage bound
   while retaining its old node id, mirror the identity projection and refresh
   the package identity only. Require `invalid_package`/`stale-node-key` at
   that original `node_id`, the original typed `locus`, and a distinct typed
   expected id from the reader. In a fresh package, give this unreferenced
   node the returned key at every occurrence, including its source-map entry
   (the existing `rename_node` and `rebuild_source_map` fixture helpers cover
   these updates), mirror its projection, refresh the package identity
   and require the **entire package to admit**. A second bound value changes
   the returned expected digest. Separately, for each of `boolean`, `integer`,
   `reference`, `option`, `set`, `bag`, `sequence` and `ordered_set`, changing a
   **unreferenced** self-typed node's retained id while leaving
   `semantic_type` stale refuses `invalid_package`/`stale-node-key` at that
   retained id with `None`; changing both id and `semantic_type` to the same
   new id lets the reader derive `Some` where the body otherwise fits. This
   is AC-183.
2. Repeat AC-183 with an unreferenced owner-free, ungrouped form outside the
   closed-shape table, such as a valid state snapshot or record, so the
   `UngroupedPreimage` branch must retain its already-computed key. Its fresh
   rekeyed package with every occurrence, including the source map, updated
   and its identity refreshed admits. Use `rename_node` and
   `rebuild_source_map` for the fixture. A test exercising only
   `integer_range` cannot discharge this branch.
3. For AC-184, use an application node with no referrer, while its own
   operands and type remain valid. Change one key-covered preimage member,
   retain the old id, mirror projection and refresh package identity. Assert
   the original `locus` and the typed digest `validate_application_keys`
   already computed. A fresh package rekeyed to that returned id at every
   occurrence, including its source-map entry, and identity-refreshed admits
   in full; use `rename_node` and `rebuild_source_map` for the fixture. A
   second preimage change returns a different expected digest. No referrer is
   allowed to create a stale-key
   cascade that masks the positive control.
4. For AC-185, assert `None` on a malformed structural body with no derived
   key, a non-self-typed shape whose semantic type is inconsistent, a
   `collection_bounds` naming a noncollection semantic type, a fixed-member
   model declaration shape failure, a canonical encoding failure and a
   pre-key-stage refusal. Require the existing code/cause/path/locus and the
   original first refusal in each case. Make two independently authored
   preimage mutations with the same retained id, path, code, cause and locus
   but different derived keys; require `PartialEq`/`Eq` to distinguish the
   resulting refusals, after each key has its own admitted positive control;
   clone a genuine refusal and require the accessor to retain the key. Derived
   `Debug` may show it; no Display or serialized refusal form is added.
5. Inspect the six production `CheckedPackageRefusal` construction sites in
   `checked_package/common.rs`, the two key sites, model-declaration
   conversion, V2 reader and version dispatch. Each genuine expected value
   and genuine absence crosses the public boundary unchanged, with no new
   byte/work charge. Keep the six IR integration-test literals' expected
   values test-authored (including both current struct-update sites), compare
   actual reader refusals field by field, and require `expected_node_id()`
   assertions; no public synthetic constructor is added. A dropped or
   corrupted expected key mutant fails independently. The expected key is
   meaningful with the original reader-returned `locus`: code, path, cause and
   locus remain publicly mutable, and changing them on a clone cannot make
   the retained private key an authenticated substitution instruction.
