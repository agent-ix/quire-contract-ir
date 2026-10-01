---
id: SR-680
title: "spec review of PR 242 (AD-005 restores R3-Q1)"
type: SpecReview
analysis: base
scope: "agent-ix/quire-contract-ir@5e34517a8fe9c80711ff4a65f2a6ae6aeeaa49e9; spec/assurance/AD-005-qsl-consumption-seam.md"
review_set: subset
relationships:
  - target: ix://agent-ix/quire-contract-ir/AD-005
    type: reviews
---
# SR-680: spec review of PR 242

## Summary

Ticket: IR-323. Reviewed head 5e34517 against origin/main 40ce5b5. The diff is one file, AD-005,
5 insertions and 3 deletions. It changes three places. The identity paragraph now says the
dependency alias should be the crate's real name. The "QSL to the IR root crate" Gap cell is
reworded. Row R3-Q1 is back in the "To QSL" routed table. The "Answered, no longer routed"
sentence no longer says R3-Q1 is not a requirement.

Measured at QSL origin/main 8337d52a. Two manifests name the dependency, the workspace
`Cargo.toml:73` and `qsl-package/Cargo.toml:36`. Both use the key `quire-contract-ir` with
`package = "quire-contract-model"`. The rename has not landed in QSL. 101 QSL `.rs` files use
the `quire_contract_ir` crate path.

Measured in IR. The root package of IR is named `quire-contract-ir` (`Cargo.toml:6`), and the
model package is `quire-contract-model`. So "the alias only reads as the root crate" is correct.
The AD still states the true direction: QSL depends on the model crate, and never on the root
crate.

Cross-references checked. No remaining text in AD-004, AD-005 or AD-006 says the rename is not a
requirement or is unwanted. The same holds for codegen origin/main (PRs 214 and 215 are merged)
and runtime origin/main. AD-004:81 still says "under the dependency name `quire-contract-ir`".
That is a true statement of the current state, so it is not a finding.

All three new mentions carry the relayed label: "R3-Q1, relayed" and "relayed by the IR planner;
QSL-owned change". No new requirement or routing id is minted, and no fact is added without a
source. `quire validate` passes. `make spec` reports the same 17 unbacked rows as before, none
from this diff.

## Findings

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-001 | low | The edited identity paragraph cites QSL `Cargo.toml:75` for the dependency. At QSL origin/main the line is `Cargo.toml:73`. The `quire-canonical` citation in the same section (`Cargo.toml:41`) has the same drift; the line is now 39. These citations predate this PR, but the sentence was edited without re-measuring them | spec/assurance/AD-005-qsl-consumption-seam.md:66-68,72 |

## Verdict

Mergeable. The change matches what the coder claimed, and every claim holds at origin/main of
IR, QSL, codegen and runtime. FND-001 is a line-number nit, which can be fixed here or left as
it is. For the record, outside this AD: QSL's rename means more than changing the manifest key.
101 QSL source files name `quire_contract_ir`. That cost belongs to QSL, the owner of the change.
