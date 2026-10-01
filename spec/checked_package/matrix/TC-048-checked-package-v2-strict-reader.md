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
against a self-built dependency package and a package that calls it.

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
`authority`, `revision` or `export`, change the selection's digest domain,
version or evidence domain, and set a model node to `model_export`.

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
under another digest, forged bytes, another version, and the matching one;
give a node an invalid object id and refer to it as a supertype and a
`typeRef` from nodes on both sides of it; give one field a dangling `typeRef`
and a multiplicity of `lower > upper` in both member orders; name a
relationship as a `typeRef` from a node read before and one read after its
owner; give two nodes no identity. Compare the whole refusal and its pointer
with the expected one. Read with the `work` limit at zero, and with each work
limit one below what the direct read and a member resolution used, and check
the pointer of the `incomplete` outcome resolves in the lock.

## Dependency selections (FR-038-AC-31 through FR-038-AC-33)

Set the lock's and identity preimage's `dependency_selections` to two
`DependencySelection` entries in ascending identity order, re-derive the
package id and read: it admits, and both members read back as supplied.
Change one entry's `package_id` and check the package id changes. Mutate one
entry at a time to a wrong-domain, empty-identity, short-digest, bare-digest,
`version`-less or `Selection`-shaped entry and read. Repeat an identity,
adjacent and not, and reverse two entries, and read. Order two identities
whose UTF-8 and UTF-16 orders differ and read. Compare the whole refusal code,
cause and pointer with the expected one.

## Operation leaves (FR-038-AC-43)

Apply `structural.eq` and `structural.ne` to two parameters of an all-integer
record, and `collection.contains` to a set of integers and an integer, each
with `leaves` empty, and check each admits. Repeat over a record with a nested
`text` field and over a set of `text` with `leaves` empty, and over two `text`
fields with one leaf supplied, and check each refuses `invalid_package` with
cause `operation-law-missing` at `operation.leaves`.

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
node body root. Supply no package, another identity, another version and a
package of another `package_id`. Name a `package` no entry carries, a `node`
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
