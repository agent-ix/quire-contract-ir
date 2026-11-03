---
id: SR-20261103150646
title: "gap-analysis of QSL-1538 channel delivery interval (IR)"
type: SpecReview
analysis: gap-analysis
scope: "agent-ix/quire-contract-ir; PR #855 at d4d4876a; FR-038-AC-286, FR-038-AC-325 (context); crates/quire-contract-model/src/checked_package/v2/protocol_body.rs, crates/quire-contract-model/src/checked_package/v2/refinement.rs, tests/it/checked_package_v2_protocol_body.rs"
review_set: subset
---

## Summary

Planless gap analysis of quire-contract-ir PR #855. Ticket: QSL-1538. Plan completion was not assessed.

Single source for the vocabulary: a grep of crates/, schemas/, corpus/, tests/, src/ and spec/ for `unknown`, `at-most-once`, `at-least-once` and `exactly-once-premise` finds the policy words only in `ChannelDelivery::parse`, its rustdoc, and test inputs. No schema or corpus file lists the delivery vocabulary. The other `delivery` mentions are the layout field shape `field("delivery", Shape::Text)` and a test fixture default `text("unknown")`.

`quire coverage --strict` on the PR head reports 93 unbacked rows and 0 contradicted statuses, the same as the origin/main baseline, so the PR adds no regression. No build was run, because of disk space (see the code-review SR).

## Examined scope

- FR-038-AC-286 (examined): "... a channel admits only same-clause role endpoints, an eligible message type, explicit ordering, explicit delivery (one of the four QSpec FR-465 policy words or a canonical delivery count interval `[lo,hi]` with `lo <= hi`, any other spelling, `lo > hi` included, refusing `malformed_wire` at the term), and a FIFO key with equality-stable type; ..."
- FR-038-AC-325 (context_only): "The protocol step compares every present refinement `class_catalog` against the complete concrete and abstract classes reconstructed from selected model declarations and the admitted local or directly selected dependency protocol graph ..."
- TC-048 step 4 (context_only): "Exercise static and replicated roles, channels with each delivery and ordering premise ... pair an admitted case with the local adverse forms ... of TC-947".

## Verdict

PASS with one low trace finding. The code implements AC-286's delivery clause, and both the unit and integration tests assert it, including the `malformed_wire` locus. The integration test's admitted and refused sets cover the QSpec FR-465-AC-10 vector exactly, plus `[0,3]x`, `[,3]` and `exactly-once`.

The refinement-class behaviour (an interval gives no `duplicate` or `lose` class, per QSpec FR-465 and FR-426) is implemented and unit-tested. Its test, however, traces FR-038-AC-286, which is about reader admission. Class reconstruction belongs to FR-038-AC-325. There is also no package-level test where a refinement catalog over an interval channel is compared, but the unit test pins the function that AC-325's reconstruction calls.

## Findings

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-001 | low | `channel_classes_follow_delivery_and_an_interval_has_none` traces FR-038-AC-286 (reader admission of channel shapes), but it asserts FR-432 class reconstruction for the refinement catalog, which is FR-038-AC-325's subject; AC-286 says nothing about classes, so the interval-gives-no-class behaviour is backed under the wrong AC | crates/quire-contract-model/src/checked_package/v2/refinement.rs:3946-3970 |
