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
package.

## Test Procedure

Dispatch an unknown, empty, absent and malformed `contract_version`, and
independently a malformed document, a duplicate top-level member and
noncanonical bytes, before any version is selected. Admit each in-repo
positive V2 fixture. Apply each authored adverse
structural mutation and compare the outcome. Independently inject malformed
JSON, a duplicate member, an unknown member, noncanonical bytes, an absent or
mismatched context digest, an unreported or unsupported required feature, a
dangling reference and an incomplete source map. Admit at exact byte, depth, node,
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
those members alone, and read it: it admits. Change only the selected domain
package's version and re-read: the node key is unchanged. Add a `version`
member to the owner and re-read: it refuses `unknown_member` at that member.
Empty the owner's `identity` and, separately, its `node`, and re-read each: it
refuses `invalid_semantic_graph`. Select a domain package, key a model
declaration node of it under the version-free `ModelOwner`, add a `field`
member on that node and read it: it admits; change only the selected version
and re-read: the declaration node's key is unchanged and it admits; key the
node under another domain package's identity and re-read: it refuses
`missing_declaration`/`missing-selection` at the member's `declaration`.

## Expected Results

An unknown, empty or absent `contract_version` refuses as
`unknown_contract_version`; a malformed `contract_version`, document, or a
duplicate or noncanonical top-level member refuses as `malformed_wire`,
`duplicate_member` or `noncanonical_wire` before any version-specific
decoding. Positive fixtures admit with their recorded package ids. Every adverse and
injected case returns its exact refusal code or incomplete accounting with no
package. Excluded edits keep the id and included edits change it. Every nominal
preimage digest matches its node key; every nominal mutation and contradictory cross-field join
refuses as `invalid_semantic_graph`. The model-owned package admits; each model join mismatch refuses as
`invalid_semantic_graph`, a compiled-model owner or lock shape carrying
`authority`, `revision` or `export` as `unknown_member`, a foreign digest
domain as `digest_domain_mismatch`, a domain package digest with no supplied
document as `missing_import`, and `model_export` as `invalid_semantic_graph`.
A model owner of `{kind, identity, node}` with no `version` admits and its node
keys, nominal and declaration, are unchanged by a version-only change of the
selection; an owner carrying `version` refuses `unknown_member`, an owner with an
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
first in document order, and no package is returned. Remove `identity`,
`digest_domain` and `digest` in turn from a model row and give each a non-string,
and read. Select the same domain package as two documents at different
versions, each in its own package under its own `sha256-jcs` digest, and read
both: each admits and they yield the same model-owned node keys. Give a model
owner an identity no row names and read it. Supply a dependency package under
its entry's `identity` and read it, supply it under another identity, and supply
a package of another `package_id`. Put two rows of one identity and different
digests in the lock, each document supplied, in both orders, and read: each
refuses `stale_dependency`; give one of the two a digest domain other than
`sha256-jcs` and read: it refuses `digest_domain_mismatch`.

Expected: the package admits; a `version` member at any of the four places
refuses `unknown_member` at that member; a missing or wrong-kind member refuses
`malformed_wire`; both documents admit with equal node keys; the unnamed owner
refuses `invalid_semantic_graph`; the dependency supplied under its identity
with its `package_id` admits, under another identity `missing_import`/
`missing-selection`, and with another `package_id`
`stale_dependency`/`byte-digest-mismatch`; the same-identity pair refuses as
stated. Compare the whole refusal code, cause and pointer.

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
`collection.set` over `text`. Give a record a field that reaches the record
again through an option, and another naming a node that is not in the graph,
and check each refuses `ill_typed` with cause `operator-ineligible` at
`operation.leaves`. Nest `text` under 40 options and check it refuses
`operation-law-missing`, and nest integers 2000 deep and check the work budget
refuses. Chain 12 record levels of 4 fields each naming the next level and
check it admits within the budget. Put a leaf whose mode disagrees with its
field's pinned rounding beside too few leaves and beside a cyclic type, and
check the refusals are `operation-law-missing` and `operator-ineligible`, the
leaf count settling before any leaf mode.

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
