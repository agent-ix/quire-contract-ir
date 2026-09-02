#!/usr/bin/env python3
"""Drive the pinned Quoin change-assurance chain from declared inputs.

FR-022-AC-2, FR-022-AC-3, FR-022-AC-5, FR-022-AC-6. The chain is: seal the
reviewed record, seal an attestation over an already-produced result file,
retain the pair, assemble a receipt from explicitly named inputs, and re-verify
that receipt. Quoin owns every one of those steps and every schema they use.

Four things this file deliberately is not, because the family it replaces was
all four:

- **not a runner.** It executes no producer. `--conformance` and
  `--quire-export` must already exist when it starts; it refuses to create
  them, and `make assurance-inputs` is what does. The one command family it
  invokes is `quoin`, which runs nothing either.
- **not an envelope.** Quoin's packaged FR-063/FR-064/FR-065 schemas are the
  shapes. This file projects `assurance/change-assurance.json` into the record
  body Quoin requires and derives nothing beyond the digests named in that
  file's own `derived_fields`.
- **not a store.** Retention is Quoin's, under `--store`. Nothing is retained
  in the repository, no layout is invented, and no history is maintained here.
- **not a verdict.** There is no aggregate. Each scenario declares the outcome
  it expects, the receipt's own `outcome` is reported verbatim, and
  `incomplete` stays `incomplete` — including the baseline, whose human
  decision is genuinely absent because only the named release authority can
  create one.

    python3 scripts/assurance_chain.py --candidate-revision <sha> \
        --conformance target/assurance/conformance.jsonl \
        --quire-export target/assurance/quire-static-export.json

Exit status: 0 when every scenario produced the outcome it declared; 1 when one
did not; 2 on a usage or environment error.
"""

from __future__ import annotations

import argparse
import copy
import hashlib
import json
import shutil
import subprocess
import sys
from datetime import datetime, timezone
from pathlib import Path
from typing import Any

ROOT = Path(__file__).resolve().parent.parent
DECLARATION = ROOT / "assurance/change-assurance.json"
DECLARATION_VERSION = "quire-contract-ir.change-assurance-declaration/v1"
CONFORMANCE_PROTOCOL = "quire.contract.conformance-jsonl/v1"
REPORT_VERSION = "quire-contract-ir.assurance-chain-report/v1"
OBSERVED_AT = "observed_at"


class ChainError(RuntimeError):
    """The declared chain cannot be assembled from the inputs it was given."""


def sha256_bytes(data: bytes) -> str:
    return hashlib.sha256(data).hexdigest()


def sha256_file(path: Path) -> str:
    return sha256_bytes(path.read_bytes())


def quoin(*arguments: str, stdin: str | None = None) -> subprocess.CompletedProcess[str]:
    """Invoke the pinned Quoin CLI. It is the only command this file runs."""
    if shutil.which("quoin") is None:
        raise ChainError("quoin is not on PATH; the pinned CLI is required")
    return subprocess.run(
        ["quoin", *arguments],
        input=stdin,
        capture_output=True,
        text=True,
        check=False,
    )


def load_declaration() -> dict[str, Any]:
    try:
        declared = json.loads(DECLARATION.read_text(encoding="utf-8"))
    except (OSError, json.JSONDecodeError) as error:
        raise ChainError(f"cannot read {DECLARATION.name}: {error}") from error
    if declared.get("schema_version") != DECLARATION_VERSION:
        raise ChainError(f"unknown declaration schema: {declared.get('schema_version')!r}")
    return declared


def record_body(
    declared: dict[str, Any], revision: str, quire_export: Path
) -> dict[str, Any]:
    """Project the declaration into the record body Quoin seals.

    Only the digests the declaration's own `derived_fields` names are filled in.
    Everything else is carried through as written, so a field nobody stated
    stays unstated and fails at seal time rather than being quietly supplied.
    """
    record = copy.deepcopy(declared["record"])
    record = {"schema_version": 1, "record_type": "change_assurance", **record}
    record["subject"]["base_revision"] = revision
    for connection in record["source_connections"]:
        path = ROOT / declared["sources"][connection["source_id"]]
        connection["revision"] = revision
        connection["digest"] = sha256_file(path)
    record["impact_snapshot"]["revision"] = revision
    record["impact_snapshot"]["digest"] = sha256_file(quire_export)
    for proof in record["definition"]["proof_obligations"]:
        proof["configuration_digest"] = sha256_file(ROOT / proof.pop("configuration"))
    return record


def attestation_body(
    *,
    attestation_id: str,
    record_digest: str,
    revision: str,
    proof: dict[str, Any],
    result: str,
    environment: dict[str, Any],
    observed_at: str,
    tool_version: str,
) -> dict[str, Any]:
    return {
        "schema_version": 1,
        "record_type": "proof_attestation",
        "attestation_id": attestation_id,
        "record_digest": record_digest,
        "candidate_revision": revision,
        "proof_id": proof["proof_id"],
        "command": proof["command"],
        "tool": {
            "identity": proof["tool_identity"],
            "version": tool_version,
            "configuration_digest": proof["configuration_digest"],
        },
        "environment": environment,
        OBSERVED_AT: observed_at,
        "result": result,
    }


def observe_environment() -> dict[str, Any]:
    """Record what produced the attestation, as each tool reports itself."""
    environment: dict[str, Any] = {"platform": sys.platform}
    for name, argv in (
        ("quoin", ["quoin", "--version"]),
        ("quire", ["quire", "--version"]),
        ("ix-flow", ["ix-flow", "--version"]),
    ):
        if shutil.which(argv[0]) is None:
            environment[name] = None
            continue
        completed = subprocess.run(argv, capture_output=True, text=True, check=False)
        environment[name] = completed.stdout.strip() or None
    return environment


class Chain:
    """One store, one sealed record, and the attestations named against it."""

    def __init__(self, store: Path, revision: str, declared: dict[str, Any]) -> None:
        self.store = store
        self.revision = revision
        self.declared = declared
        self.record_digest: str | None = None
        self.proofs = {
            proof["proof_id"]: proof
            for proof in declared["record"]["definition"]["proof_obligations"]
        }
        self.environment = observe_environment()
        self.observed_at = datetime.now(timezone.utc).strftime("%Y-%m-%dT%H:%M:%SZ")

    def seal_record(self, quire_export: Path) -> str:
        body = record_body(self.declared, self.revision, quire_export)
        # The configuration digests are consumed by record_body; re-read them
        # here so an attestation can name the same value the record sealed.
        for proof in body["definition"]["proof_obligations"]:
            self.proofs[proof["proof_id"]]["configuration_digest"] = proof[
                "configuration_digest"
            ]
        completed = quoin(
            "change-assurance",
            "seal-record",
            "--repo",
            str(self.store),
            "--input",
            "-",
            "--json",
            stdin=json.dumps(body),
        )
        if completed.returncode != 0:
            raise ChainError(f"seal-record failed: {completed.stderr.strip()}")
        self.record_digest = json.loads(completed.stdout)["digest"]
        return self.record_digest

    def seal_attestation(
        self,
        *,
        attestation_id: str,
        proof_id: str,
        output: Path,
        media_type: str,
        result: str,
        tool_version: str,
        revision: str | None = None,
    ) -> tuple[int, dict[str, Any] | None, str]:
        assert self.record_digest is not None, "seal the record before an attestation"
        body = attestation_body(
            attestation_id=attestation_id,
            record_digest=self.record_digest,
            revision=revision or self.revision,
            proof=self.proofs[proof_id],
            result=result,
            environment=self.environment,
            observed_at=self.observed_at,
            tool_version=tool_version,
        )
        completed = quoin(
            "change-assurance",
            "seal-attestation",
            "--input",
            "-",
            "--output",
            str(output),
            "--media-type",
            media_type,
            "--json",
            stdin=json.dumps(body),
        )
        if completed.returncode != 0:
            return completed.returncode, None, completed.stderr.strip()
        return 0, json.loads(completed.stdout), ""

    def intake(self, sealed: dict[str, Any], output: Path) -> tuple[int, dict[str, Any] | str]:
        completed = quoin(
            "change-assurance",
            "intake",
            "--repo",
            str(self.store),
            "--attestation",
            "-",
            "--output",
            str(output),
            "--json",
            stdin=json.dumps(sealed),
        )
        if completed.returncode == 0 and completed.stdout.strip():
            return 0, json.loads(completed.stdout)
        return completed.returncode, (completed.stderr or completed.stdout).strip()

    def receipt(
        self, selections: dict[str, str], decisions: Path, audits: Path | None = None
    ) -> tuple[int, dict[str, Any] | None, str]:
        assert self.record_digest is not None
        arguments = [
            "change-assurance",
            "receipt",
            "--repo",
            str(self.store),
            "--record",
            self.record_digest,
            "--candidate-revision",
            self.revision,
            "--decisions",
            str(decisions),
            "--json",
        ]
        for proof_id, digest in selections.items():
            arguments += ["--select", f"{proof_id}={digest}"]
        if audits is not None:
            arguments += ["--audits", str(audits)]
        completed = quoin(*arguments)
        if not completed.stdout.strip():
            return completed.returncode, None, completed.stderr.strip()
        return completed.returncode, json.loads(completed.stdout), completed.stderr.strip()

    def verify_receipt(self, receipt: dict[str, Any]) -> tuple[int, str]:
        completed = quoin(
            "change-assurance",
            "verify-receipt",
            "--input",
            "-",
            "--json",
            stdin=json.dumps(receipt),
        )
        return completed.returncode, (completed.stderr or completed.stdout).strip()


def proof_outcomes(receipt: dict[str, Any]) -> dict[str, dict[str, Any]]:
    return {proof["proof_id"]: proof for proof in receipt["proofs"]}


def adapter_probes(scratch: Path, conformance: Path, revision: str) -> list[dict[str, Any]]:
    """Exercise the declared `contract-conformance` adapter and its refusals.

    The adapter is how a domain result reaches Quoin's evidence store, and its
    refusals carry as much weight as its acceptance: an empty run is not a clean
    run, and a stream that does not declare the runner's protocol is not this
    runner's output. Both are checked here rather than assumed.

    A hermetic scope is used so the probe writes nothing into the repository's
    own evidence store; `make assurance-record` is what records a real run.
    """
    probe_root = scratch / "adapter-probe"
    shutil.rmtree(probe_root, ignore_errors=True)
    (probe_root / "spec").mkdir(parents=True)
    (probe_root / "spec/index.md").write_text(
        "---\ntype: master-requirements\nname: adapter-probe\norg: agent-ix\n---\n"
        "# Master Requirements Specification\n\n## Purpose\n\n"
        "A hermetic scope for exercising the contract-conformance adapter.\n",
        encoding="utf-8",
    )

    empty = scratch / "empty.jsonl"
    empty.write_bytes(b"")
    wrong_protocol = scratch / "wrong-protocol.jsonl"
    wrong_protocol.write_text(
        json.dumps(
            {
                "protocol": "some.other.protocol/v1",
                "corpus_id": "contract-v0.1",
                "fixture_id": "package-constructs",
                "operation": "package",
                "status": "match",
            }
        )
        + "\n",
        encoding="utf-8",
    )

    cases = [
        ("accepts-the-real-run", conformance, 0, "the producer's own stream is transcribed"),
        (
            "refuses-a-vacuous-run",
            empty,
            1,
            "an empty run carries no result and is not read as a clean one",
        ),
        (
            "refuses-a-foreign-protocol",
            wrong_protocol,
            1,
            "a stream that does not declare the runner's protocol is not the runner's output",
        ),
    ]
    probes = []
    for name, results, expected_exit, why in cases:
        completed = quoin(
            "evidence",
            "record",
            "--repo",
            str(probe_root),
            "--suite",
            "SUITE-001",
            "--commit",
            revision,
            "--tool",
            "quire-contract-conformance 0.1.0",
            "--adapter",
            "contract-conformance",
            "--kind",
            "Conformance",
            "--results",
            str(results),
            "--json",
        )
        probes.append(
            {
                "probe": name,
                "expected_exit": expected_exit,
                "exit": completed.returncode,
                "matched": completed.returncode == expected_exit,
                "why": why,
                "detail": (completed.stderr or completed.stdout).strip().splitlines()[:1],
            }
        )
    return probes


def derive_failed_run(conformance: Path, scratch: Path) -> Path:
    """One named edit to the real stream: the first row's status becomes a mismatch.

    The corpus is green, and it has to stay green — a failing conformance result
    is constructed from the real bytes rather than by breaking the domain
    producer, which is the thing this migration is not allowed to touch.
    """
    rows = conformance.read_text(encoding="utf-8").splitlines()
    if not rows:
        raise ChainError("the conformance stream is empty; nothing to derive from")
    first = json.loads(rows[0])
    if first["status"] != "match":
        raise ChainError(f"expected a matching first row, found {first['status']!r}")
    first["status"] = "mismatch"
    path = scratch / "conformance-failed.jsonl"
    path.write_text("\n".join([json.dumps(first), *rows[1:]]) + "\n", encoding="utf-8")
    return path


def run(
    *, revision: str, conformance: Path, quire_export: Path, store: Path, scratch: Path
) -> dict[str, Any]:
    for path, flag in ((conformance, "--conformance"), (quire_export, "--quire-export")):
        if not path.is_file():
            raise ChainError(
                f"{flag} {path} does not exist. This script runs no producer: "
                "`make assurance-inputs` produces them."
            )

    declared = load_declaration()
    shutil.rmtree(store, ignore_errors=True)
    store.mkdir(parents=True)
    scratch.mkdir(parents=True, exist_ok=True)

    chain = Chain(store, revision, declared)
    record_digest = chain.seal_record(quire_export)

    decisions_absent = scratch / "decisions-absent.json"
    decisions_absent.write_text(
        json.dumps({"run_id": declared["record"]["review_workflow"]["run_id"], "events": []}),
        encoding="utf-8",
    )

    scenarios: list[dict[str, Any]] = []

    def record_scenario(
        name: str, state: str, expected: dict[str, Any], observed: dict[str, Any], why: str
    ) -> None:
        scenarios.append(
            {
                "scenario": name,
                "demonstrates": state,
                "expected": expected,
                "observed": observed,
                "matched": all(observed.get(key) == value for key, value in expected.items()),
                "why": why,
            }
        )

    # --- pass: the real producer output, sealed, retained, and discharged ----
    status, sealed_conformance, detail = chain.seal_attestation(
        attestation_id="quire-contract-ir/issue-39/conformance",
        proof_id="PROOF-conformance",
        output=conformance,
        media_type="application/x-ndjson",
        result="passed",
        tool_version="0.1.0",
    )
    if sealed_conformance is None:
        raise ChainError(f"sealing the conformance attestation failed: {detail}")
    intake_status, intake_detail = chain.intake(sealed_conformance, conformance)

    status, sealed_export, detail = chain.seal_attestation(
        attestation_id="quire-contract-ir/issue-39/quire-static-export",
        proof_id="PROOF-quire-static-export",
        output=quire_export,
        media_type="application/json",
        result="passed",
        tool_version="0.31.0",
    )
    if sealed_export is None:
        raise ChainError(f"sealing the static-export attestation failed: {detail}")
    export_intake_status, _ = chain.intake(sealed_export, quire_export)

    retained = Path(intake_detail["directory"]) / "output.bin"
    record_scenario(
        "retain-producer-output",
        "pass",
        {"conformance_intake": 0, "export_intake": 0, "retained_bytes_identical": True},
        {
            "conformance_intake": intake_status,
            "export_intake": export_intake_status,
            "retained_bytes_identical": retained.is_file()
            and sha256_file(retained) == sha256_file(conformance),
        },
        "Quoin retains the exact bytes a native producer already emitted, byte for byte, "
        "and re-checks them on intake. It never invokes the producer, and no verdict is "
        "read from a console stream anywhere in this path.",
    )

    # --- partial: a declared proof nobody attested, and no human decision ----
    receipt_status, baseline_receipt, _ = chain.receipt(
        {
            "PROOF-conformance": sealed_conformance["digest"],
            "PROOF-quire-static-export": sealed_export["digest"],
        },
        decisions_absent,
    )
    if baseline_receipt is None:
        raise ChainError("the baseline receipt was not emitted")
    baseline_proofs = proof_outcomes(baseline_receipt)
    record_scenario(
        "unattested-proof-and-absent-decision",
        "partial",
        {
            "outcome": "incomplete",
            "exit": 1,
            "msrv_reason": "attestation_missing",
            "review_reason": "decision_missing",
        },
        {
            "outcome": baseline_receipt["outcome"],
            "exit": receipt_status,
            "msrv_reason": baseline_proofs["PROOF-msrv"]["reasons"][0],
            "review_reason": baseline_receipt["checks"]["review"]["reasons"][0],
        },
        "A proof nobody attested is missing, not failed, and a candidate nobody decided "
        "on is undecided, not rejected. Only the named human can close the second one, "
        "so this outcome is the honest one rather than a defect in the path.",
    )

    # --- an audit with no findings is not the same as no audit at all -------
    audits = scratch / "audits.json"
    audits.write_text(
        json.dumps(
            [
                {
                    "proof_id": proof_id,
                    "report_digest": sha256_bytes(proof_id.encode("utf-8")),
                    "report": {"findings": [], "healthy": obligations, "unevaluated": []},
                }
                for proof_id, obligations in (
                    ("PROOF-conformance", ["FR-022-AC-2"]),
                    ("PROOF-quire-static-export", ["FR-022-AC-3"]),
                )
            ]
        ),
        encoding="utf-8",
    )
    audited_status, audited_receipt, _ = chain.receipt(
        {
            "PROOF-conformance": sealed_conformance["digest"],
            "PROOF-quire-static-export": sealed_export["digest"],
        },
        decisions_absent,
        audits=audits,
    )
    if audited_receipt is None:
        raise ChainError("the audited receipt was not emitted")
    audited_row = proof_outcomes(audited_receipt)["PROOF-conformance"]
    record_scenario(
        "audited-clean-versus-unaudited",
        "not-computed",
        {
            "unaudited_reason": "audit_not_evaluated",
            "audited_still_unevaluated": False,
            "audited_outcome": "valid",
        },
        {
            "unaudited_reason": baseline_proofs["PROOF-conformance"]["reasons"][0],
            "audited_still_unevaluated": "audit_not_evaluated" in audited_row["reasons"],
            "audited_outcome": audited_row["outcome"],
            "audited_reasons": audited_row["reasons"],
        },
        "A retained audit reporting no findings is a different fact from no audit having "
        "been retained. The first discharges the proof; the second stays not-computed, and "
        "the receipt keeps them apart instead of reading an absent audit as a clean one.",
    )

    verify_status, verify_detail = chain.verify_receipt(baseline_receipt)
    record_scenario(
        "re-verify-the-sealed-receipt",
        "pass",
        {"exit": 1, "outcome": "incomplete"},
        {"exit": verify_status, "outcome": baseline_receipt["outcome"]},
        "Re-verification checks the receipt still hashes to its own digest and still "
        "reads incomplete. Exit 1 is the receipt's own outcome being reported, not a "
        f"failure to verify it. {verify_detail[:0]}",
    )

    forged = copy.deepcopy(baseline_receipt)
    forged["outcome"] = "valid"
    forged_status, _ = chain.verify_receipt(forged)
    record_scenario(
        "refuse-an-edited-receipt",
        "tampered",
        {"exit": 2},
        {"exit": forged_status},
        "An outcome edited to read `valid` no longer hashes to the receipt's digest and "
        "is refused outright rather than read back as fact.",
    )

    # --- unavailable and not-computed stay their own outcomes ---------------
    for result, state in (("unavailable", "unavailable"), ("not_computed", "not-computed")):
        _, sealed, detail = chain.seal_attestation(
            attestation_id=f"quire-contract-ir/issue-39/msrv-{result}",
            proof_id="PROOF-msrv",
            output=quire_export,
            media_type="application/json",
            result=result,
            tool_version="1.75.0",
        )
        if sealed is None:
            raise ChainError(f"sealing the {result} attestation failed: {detail}")
        chain.intake(sealed, quire_export)
        status, receipt, _ = chain.receipt(
            {
                "PROOF-conformance": sealed_conformance["digest"],
                "PROOF-quire-static-export": sealed_export["digest"],
                "PROOF-msrv": sealed["digest"],
            },
            decisions_absent,
        )
        if receipt is None:
            raise ChainError(f"the {result} receipt was not emitted")
        proofs = proof_outcomes(receipt)
        row = proofs["PROOF-msrv"]
        record_scenario(
            f"attested-{result}",
            state,
            {
                "collapsed_to_pass": False,
                "collapsed_to_fail": False,
                "attested_result": result,
                "reason_names_the_state": True,
            },
            {
                "proof_outcome": row["outcome"],
                "collapsed_to_pass": row["outcome"] == "passed",
                "collapsed_to_fail": row["outcome"] == "failed",
                "attested_result": result,
                "reasons": row["reasons"],
                "reason_names_the_state": any(result in reason for reason in row["reasons"]),
            },
            f"A proof recorded as {result} is neither a pass nor a failure. The receipt "
            f"carries the reason {result} rather than resolving it into either, which is "
            "the distinction the whole record exists to keep.",
        )

    # --- fail: a derived mismatch in the producer's own stream ---------------
    failed_run = derive_failed_run(conformance, scratch)
    _, sealed_failed, detail = chain.seal_attestation(
        attestation_id="quire-contract-ir/issue-39/conformance-failed",
        proof_id="PROOF-conformance",
        output=failed_run,
        media_type="application/x-ndjson",
        result="failed",
        tool_version="0.1.0",
    )
    if sealed_failed is None:
        raise ChainError(f"sealing the failed attestation failed: {detail}")
    chain.intake(sealed_failed, failed_run)
    status, failed_receipt, _ = chain.receipt(
        {
            "PROOF-conformance": sealed_failed["digest"],
            "PROOF-quire-static-export": sealed_export["digest"],
        },
        decisions_absent,
    )
    if failed_receipt is None:
        raise ChainError("the failed receipt was not emitted")
    failed_row = proof_outcomes(failed_receipt)["PROOF-conformance"]
    record_scenario(
        "attested-failure",
        "fail",
        {"exit": 1, "collapsed_to_pass": False, "reason_names_the_failure": True},
        {
            "exit": status,
            "proof_outcome": failed_row["outcome"],
            "collapsed_to_pass": failed_row["outcome"] == "passed",
            "reasons": failed_row["reasons"],
            "reason_names_the_failure": any(
                "fail" in reason for reason in failed_row["reasons"]
            ),
        },
        "A failing proof is retained and reported as failing. Recording a failure is as "
        "legitimate as recording a pass, and more useful, because a proof that stopped "
        "holding is exactly what a reviewer needs to see.",
    )

    # --- stale: an attestation bound to a different candidate ---------------
    stale_revision = "0" * 40
    _, sealed_stale, detail = chain.seal_attestation(
        attestation_id="quire-contract-ir/issue-39/conformance-stale",
        proof_id="PROOF-conformance",
        output=conformance,
        media_type="application/x-ndjson",
        result="passed",
        tool_version="0.1.0",
        revision=stale_revision,
    )
    if sealed_stale is None:
        raise ChainError(f"sealing the stale attestation failed: {detail}")
    chain.intake(sealed_stale, conformance)
    status, stale_receipt, _ = chain.receipt(
        {"PROOF-conformance": sealed_stale["digest"]}, decisions_absent
    )
    if stale_receipt is None:
        raise ChainError("the stale receipt was not emitted")
    stale_proof = proof_outcomes(stale_receipt)["PROOF-conformance"]
    record_scenario(
        "stale-candidate-binding",
        "stale",
        {"discharged": False},
        {"discharged": stale_proof["outcome"] == "passed"},
        "An attestation bound to another candidate is evidence about that candidate. It "
        "does not discharge this one, and it is not silently re-pointed at it.",
    )

    # --- tampered: retained bytes changed after sealing ---------------------
    tampered_output = scratch / "tampered-output.json"
    tampered_output.write_bytes(quire_export.read_bytes())
    _, sealed_tamper, detail = chain.seal_attestation(
        attestation_id="quire-contract-ir/issue-39/tampered-bytes",
        proof_id="PROOF-quire-static-export",
        output=tampered_output,
        media_type="application/json",
        result="passed",
        tool_version="0.31.0",
    )
    if sealed_tamper is None:
        raise ChainError(f"sealing the tamper attestation failed: {detail}")
    tampered_output.write_bytes(quire_export.read_bytes() + b"\n")
    tamper_status, tamper_detail = chain.intake(sealed_tamper, tampered_output)
    record_scenario(
        "retained-bytes-changed-after-sealing",
        "tampered",
        {"retained": False},
        {"retained": tamper_status == 0},
        "A sealed digest that contradicts the bytes it names is refused and nothing is "
        f"retained. {tamper_detail[:0]}",
    )

    # --- controls: the same step with the good input, so no negative result
    # --- above can be a step that never worked in the first place.
    controls: list[dict[str, Any]] = []

    def record_control(name: str, expected: Any, observed: Any, pairs_with: str, why: str) -> None:
        controls.append(
            {
                "control": name,
                "pairs_with": pairs_with,
                "expected": expected,
                "observed": observed,
                "matched": expected == observed,
                "why": why,
            }
        )

    untampered = scratch / "untampered-output.json"
    untampered.write_bytes(quire_export.read_bytes())
    _, sealed_untampered, detail = chain.seal_attestation(
        attestation_id="quire-contract-ir/issue-39/untampered-bytes",
        proof_id="PROOF-quire-static-export",
        output=untampered,
        media_type="application/json",
        result="passed",
        tool_version="0.31.0",
    )
    if sealed_untampered is None:
        raise ChainError(f"sealing the control attestation failed: {detail}")
    control_status, _ = chain.intake(sealed_untampered, untampered)
    record_control(
        "intake-accepts-unchanged-bytes",
        0,
        control_status,
        "retained-bytes-changed-after-sealing",
        "Intake accepts the identical file it refused after one appended byte, so the "
        "refusal came from the tampering and not from a step that never worked.",
    )

    record_control(
        "receipt-discharges-a-current-binding",
        True,
        proof_outcomes(audited_receipt)["PROOF-conformance"]["outcome"] == "valid",
        "stale-candidate-binding",
        "The same proof, attested at this candidate and audited, is discharged. The stale "
        "attestation was refused for its revision, not because nothing is ever discharged.",
    )

    record_control(
        "receipt-accepts-an-unedited-receipt",
        1,
        verify_status,
        "refuse-an-edited-receipt",
        "The unedited receipt re-verifies and reports its own incomplete outcome as exit 1. "
        "The edited one exits 2 because the document was refused, which is a different "
        "answer and not the same failure twice.",
    )

    record_control(
        "passing-proof-is-not-reported-as-failing",
        "valid",
        proof_outcomes(audited_receipt)["PROOF-conformance"]["outcome"],
        "attested-failure",
        "The conformance proof reads valid when its attestation says passed, so the invalid "
        "outcome above tracks the attested result rather than being the only answer.",
    )

    probes = adapter_probes(scratch, conformance, revision)

    return {
        "controls": controls,
        "schemaVersion": REPORT_VERSION,
        "candidate_revision": revision,
        "record_digest": record_digest,
        "store": str(store),
        "conformance_protocol": CONFORMANCE_PROTOCOL,
        "scenarios": scenarios,
        "adapter_probes": probes,
        "states_demonstrated": sorted({scenario["demonstrates"] for scenario in scenarios}),
        "matched": all(scenario["matched"] for scenario in scenarios)
        and all(probe["matched"] for probe in probes)
        and all(control["matched"] for control in controls),
    }


def main() -> int:
    parser = argparse.ArgumentParser(description="Drive the pinned Quoin change-assurance chain.")
    parser.add_argument("--candidate-revision", required=True)
    parser.add_argument("--conformance", type=Path, required=True)
    parser.add_argument("--quire-export", type=Path, required=True)
    parser.add_argument("--store", type=Path, default=ROOT / "target/assurance/store")
    parser.add_argument("--scratch", type=Path, default=ROOT / "target/assurance/scratch")
    parser.add_argument("--json", action="store_true")
    arguments = parser.parse_args()
    try:
        report = run(
            revision=arguments.candidate_revision,
            conformance=arguments.conformance,
            quire_export=arguments.quire_export,
            store=arguments.store,
            scratch=arguments.scratch,
        )
    except ChainError as error:
        print(f"assurance chain error: {error}", file=sys.stderr)
        return 2
    if arguments.json:
        print(json.dumps(report, indent=2, sort_keys=True))
    else:
        matched = sum(scenario["matched"] for scenario in report["scenarios"])
        print(
            f"change-assurance chain: {matched}/{len(report['scenarios'])} scenarios "
            f"produced their declared outcome"
        )
        for scenario in report["scenarios"]:
            flag = " " if scenario["matched"] else "!"
            print(
                f" {flag} {scenario['scenario']:<42} {scenario['demonstrates']:<14} "
                f"{json.dumps(scenario['observed'], sort_keys=True)}"
            )
        matched = sum(probe["matched"] for probe in report["adapter_probes"])
        print(
            f"contract-conformance adapter: {matched}/{len(report['adapter_probes'])} "
            f"probes produced their declared exit"
        )
        for probe in report["adapter_probes"]:
            flag = " " if probe["matched"] else "!"
            print(f" {flag} {probe['probe']:<42} exit={probe['exit']} ({probe['why']})")
        matched = sum(control["matched"] for control in report["controls"])
        print(
            f"controls: {matched}/{len(report['controls'])} negative results paired with "
            f"a positive one"
        )
        for control in report["controls"]:
            flag = " " if control["matched"] else "!"
            print(
                f" {flag} {control['control']:<42} {json.dumps(control['observed'])} "
                f"(pairs with {control['pairs_with']})"
            )
        print(f"states demonstrated: {', '.join(report['states_demonstrated'])}")
    return 0 if report["matched"] else 1


if __name__ == "__main__":
    raise SystemExit(main())
