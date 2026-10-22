---
id: TC-459
title: "Read checked-package extension form nodes and refusals"
type: TC
relationships:
  - target: ix://agent-ix/quire-contract-ir/FR-348
    type: verifies
---
# TC-459: Read checked-package extension form nodes and refusals

## Description

Exercise the public V2 reader with QSpec FR-469's published extension packages
and the X01..X17 controls described in QSpec TC-951. Read the fixture files
from the selected QSpec checkout and construct the TC-951 mutations in the test;
do not copy a schema, package, binary or expected-output file into Contract IR.

## Test Procedure

Supply the validated FR-467 `retention` definition through the reader's
held-definition evidence and read QSpec's positive extension fixtures. Check
the admitted node, clause references, types, declaration, owner, occurrence,
source region, structural key and package identity. Supply a second node kind
under the same tag/form. Change each keyed member and the source region in
turn, recomputing fixture identities where required. For TC-951 X02, compare
the changed node and package identities through the published preimage rule;
only the owner-change branch needs a reader admission check. Run each other
X01..X17 control through the public reader where TC-951 specifies an
admission or refusal outcome; check identity for admitted branches and the
exact first refusal for adverse branches.
Repeat with no held definition, with no selected definition, with
conflicting evidence, and with simultaneous faults in different reader stages.

## Expected Results

The positive packages admit with their published exact identities and their
source/selection facts retained. Source-only movement preserves identity;
keyed-member changes do not. Each adverse case yields the specified code,
cause, pointer and locus, with no admitted prefix. The missing-held-definition
case is `unknown_required_capability` with no cause and the node id as locus;
it is never `unknown_required_feature`. The reader never obtains extension
meaning from a feature entry or a copied fixture.
