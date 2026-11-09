#!/usr/bin/env python3
# SPDX-License-Identifier: AGPL-3.0-or-later
# Copyright (C) 2026 Agent-IX
"""Build/run a frozen original-driver corpus and export real compiler counts.

All Cargo/build and original-driver runs are serialized by the shared lock.
This tool neither installs toolchains nor changes manifests or global config.
"""
import argparse
import gzip
import json
import math
import os
from pathlib import Path
import re
import shutil
import subprocess
import sys
import time

import coverage_census as census

LOCKED = ["flock", "--no-fork", "/tmp/titan-cargo.lock", "nice", "-n", "10", "ionice", "-c", "3"]
CONTROL_CASES = {
    "wrapped_function_item_has_no_unaccounted_entry": True,
    "qspec_read_proof::provider_scope_tests::provider_designated_read_preserves_the_original_scalar_refusal": True,
    "qspec_read_proof::caller_budget_tests::caller_budget_observation_preserves_prior_charges_and_outer_capture": True,
    "unwrapped_type_alias_before_trusted_read_must_fail_census": False,
    "unwrapped_function_item_after_trusted_read_must_fail_census": False,
    "unwrapped_shared_helper_before_trusted_read_must_fail_census": False,
    "unwrapped_shared_helper_after_trusted_read_must_fail_census": False,
    "unwrapped_budgeted_function_item_before_trusted_read_must_fail_census": False,
    "unwrapped_budgeted_function_item_after_trusted_read_must_fail_census": False,
}


class RunError(ValueError):
    pass


SAFE_ENVIRONMENT = ["CARGO_BUILD_JOBS", "CARGO_INCREMENTAL", "CARGO_PROFILE_DEV_DEBUG",
                    "CARGO_PROFILE_TEST_DEBUG", "CARGO_PROFILE_DEV_OPT_LEVEL",
                    "CARGO_PROFILE_TEST_OPT_LEVEL", "CARGO_TARGET_DIR", "RUSTFLAGS",
                    "RUSTC", "RUSTDOC", "CARGO", "RUSTC_WRAPPER", "RUSTC_WORKSPACE_WRAPPER"]


def safe_environment(env):
    return {key: env[key] for key in SAFE_ENVIRONMENT if key in env}


def owned_target(repo, target):
    repo = repo.resolve()
    target = target.absolute()
    tree = repo / "target"
    if not target.is_relative_to(tree):
        raise RunError("target directory must stay under this worktree's own target tree")
    current = target
    while current != repo:
        if current.is_symlink():
            raise RunError("target directory must not traverse a symlink")
        current = current.parent
    resolved = target.resolve()
    if not resolved.is_relative_to(tree):
        raise RunError("resolved target directory escaped the owned target tree")
    return resolved



MIN_FREE_BYTES = 20 * 1024**3
MAX_EXPORT_BYTES = 2 * 1024**3


def require_space(path, phase):
    free = shutil.disk_usage(path).free
    if free < MIN_FREE_BYTES:
        raise RunError(f"{phase}: less than 20 GiB free; preserve artifacts and stop")


def archive_export(output):
    source = output / "coverage.json"
    compressed = output / "coverage.json.gz"
    with source.open("rb") as original, gzip.open(compressed, "wb") as archive:
        shutil.copyfileobj(original, archive)
    # Verify lossless retention before removing this run's uncompressed copy.
    with source.open("rb") as original, gzip.open(compressed, "rb") as archive:
        while True:
            expected = original.read(1024 * 1024)
            if expected != archive.read(1024 * 1024):
                raise RunError("compressed compiler evidence failed lossless readback")
            if not expected:
                break
    source.unlink()


def git(root, *args):
    return subprocess.check_output(["git", "-C", str(root), *args], text=True).strip()


def write_json(path, value):
    path.write_text(json.dumps(value, indent=2) + "\n")


def checked(command, output, *, cwd, env):
    with output.open("w") as log:
        process = subprocess.Popen(command, cwd=cwd, env=env, stdout=log, stderr=subprocess.STDOUT)
        code = process.wait()
    if code:
        raise RunError(f"exit {code}: {command}; log {output}")
    return process.pid


def listed_tests(output):
    return [match.group(1) for line in output.splitlines()
            if (match := re.fullmatch(r"(.+): test", line))]


def test_results(text, intended):
    outcomes = {"passed_tests": [], "failed_tests": [], "ignored_tests": []}
    pending = None
    for line in text.splitlines():
        for name in intended:
            prefix = "test " + name
            if line.startswith(prefix + " ... ") or line.startswith(prefix + " - should panic ... "):
                if pending is not None:
                    raise RunError(f"missing outcome before next original test: {pending}")
                pending = name
                line = line.split(" ... ", 1)[1]
                break
        if pending is None:
            continue
        status = line.strip()
        key = ("passed_tests" if status == "ok" else
               "failed_tests" if status == "FAILED" else
               "ignored_tests" if status.startswith("ignored") else None)
        if key:
            outcomes[key].append(pending)
            pending = None
    if pending is not None:
        raise RunError(f"missing final original test outcome: {pending}")
    return outcomes


def persisted_test_summary(report):
    """Persist aggregate outcomes without a test/callsite inventory."""
    return {
        "test_count": len(report.get("intended_tests", [])),
        "passed_count": len(report.get("passed_tests", [])),
        "failed_count": len(report.get("failed_tests", [])),
        "ignored_count": len(report.get("ignored_tests", [])),
        "exit_code": report.get("exit_code"),
        "timed_out": report.get("timed_out", False),
        "native_started": report.get("native_started", False),
        "evidence_error_count": len(report.get("evidence_errors", [])),
        "scope": report.get("scope", "selected original tests"),
        "full_corpus": report.get("full_corpus", False),
    }


def validate_snapshot(root, config):
    if git(root, "status", "--porcelain"):
        raise RunError("source tree changed after the clean build")
    qspec = Path(config["qspec"]["path"])
    if git(qspec, "status", "--porcelain"):
        raise RunError("selected QSpec tree changed during proof")


def build(args, target, env, config):
    command = LOCKED + [str(args.toolchain_bin / "cargo"), "test", "--workspace", "--locked",
                        "--test", target, "--no-run", "--message-format=json", "-j", "2"]
    if args.features:
        command.extend(["--features", ",".join(args.features)])
    config["build_command"] = command
    write_json(args.out / "config.json", config)
    build_profiles = args.out / "build-profiles"
    build_profiles.mkdir()
    build_env = dict(env, LLVM_PROFILE_FILE=str(build_profiles / "build-%p-%m.profraw"))
    checked(command, args.out / "build.log", cwd=args.repo, env=build_env)
    validate_snapshot(args.repo, config)
    binaries = []
    artifacts = []
    for line in (args.out / "build.log").read_text().splitlines():
        try:
            message = json.loads(line)
        except ValueError:
            continue
        if message.get("reason") == "compiler-artifact":
            artifacts.append({key: message.get(key) for key in ["package_id", "target", "features", "profile"]})
            if message.get("executable") and message.get("target", {}).get("name") == target:
                binaries.append(Path(message["executable"]))
    write_json(args.out / "artifacts.json", artifacts)
    if len(set(binaries)) != 1:
        raise RunError(f"expected one actual {target} test binary, got {binaries}")
    return binaries[0]


def discover_tests(args, binary, env):
    listing_env = dict(env, LLVM_PROFILE_FILE=str(args.out / "listing-%p-%m.profraw"))
    checked(LOCKED + [str(binary), "--list", "--format", "terse"], args.out / "list.log",
            cwd=args.repo, env=listing_env)
    intended = listed_tests((args.out / "list.log").read_text())
    if not intended or len(set(intended)) != len(intended):
        raise RunError("empty or duplicate compiled original-driver test inventory")
    return intended



def select_original_tests(compiled, selectors, controls=False, class_controls=False):
    """Exact original symbols only; selection never creates full-corpus evidence."""
    if selectors and (controls or class_controls):
        raise RunError("--original-test cannot mix with entry or class controls")
    if not compiled or len(set(compiled)) != len(compiled):
        raise RunError("empty or duplicate compiled original-driver test inventory")
    if not selectors:
        return list(compiled)
    if len(set(selectors)) != len(selectors):
        raise RunError("duplicate --original-test selectors")
    unknown = set(selectors) - set(compiled)
    if unknown:
        raise RunError(f"unknown exact original-test selectors: {sorted(unknown)}")
    return [name for name in compiled if name in set(selectors)]


def select_control_tests(compiled, selectors):
    if len(set(selectors)) != len(selectors) or set(selectors) - set(CONTROL_CASES):
        raise RunError("unknown or duplicate exact entry-control selector")
    selected = selectors or list(CONTROL_CASES)
    if set(selected) - set(compiled):
        raise RunError("selected entry control is absent from the actual compiled inventory")
    return selected


def selection_metadata(args):
    selected = bool(getattr(args, "original_test", []))
    scope = ("selected-only original tests" if selected else
             "minimal canonical class-denial controls only" if getattr(args, "class_controls", False) else
             "selected-only reader entry controls" if getattr(args, "control_test", []) else
             "executed malformed-input reader entry controls only" if getattr(args, "controls", False) else
             "declared original corpus")
    return {"scope": scope,
            "full_corpus": False,
            "complete_declared_inventory": not selected and not getattr(args,"controls",False) and not getattr(args,"class_controls",False)}


def select_tc048(args, compiled):
    # Structural claims choose original test symbols, never JSON files/rows.
    # Intersect with actual compiled tests fail-closed: missing/ambiguous names
    # cannot silently shrink the declared corpus.
    command = ["quire", "trace", "--scope", str(args.repo), "--id", "TC-048", "--json"]
    result = subprocess.run(command, cwd=args.repo, text=True, capture_output=True, check=True)
    (args.out / "tc048-trace.json").write_text(result.stdout)
    (args.out / "tc048-trace.stderr").write_text(result.stderr)
    trace = json.loads(result.stdout)
    if not trace.get("resolved"):
        raise RunError("TC-048 structural trace query did not resolve")
    selected = {}
    for claim in trace.get("claims", {}).get("verifies", []):
        path = Path(claim["path"])
        if not path.is_relative_to("tests/it"):
            continue
        if claim.get("form") != "rust-trace-attribute" or claim.get("provenance") != "canonical":
            raise RunError("TC-048 original test selection lacks canonical source attributes")
        module = "::".join(path.relative_to("tests/it").with_suffix("").parts)
        name = module + "::" + claim["symbol"]
        if name not in compiled:
            raise RunError(f"claimed TC-048 original test not compiled: {name}")
        selected[name] = claim
    if not selected:
        raise RunError("no actual TC-048 original integration tests selected")
    return sorted(selected)


def merge_export(args, output, profiles, binary, env):
    require_space(output, "before profile merge/export")
    merged = output / "driver.profdata"
    merge = [str(args.llvm_bin / "llvm-profdata"), "merge", "-sparse", *map(str, profiles), "-o", str(merged)]
    export = [str(args.llvm_bin / "llvm-cov"), "export", str(binary), "-instr-profile=" + str(merged)]
    write_json(output / "compiler-evidence-commands.json", {"merge": merge, "export": export,
               "maximum_export_bytes": MAX_EXPORT_BYTES, "minimum_free_bytes": MIN_FREE_BYTES})
    checked(merge, output / "profile-merge.log", cwd=args.repo, env=env)
    exported = output / "coverage.json"
    with exported.open("wb") as data, (output / "coverage.stderr").open("wb") as error:
        process = subprocess.Popen(export, stdout=subprocess.PIPE, stderr=error, env=env)
        try:
            total = 0
            while chunk := process.stdout.read(1024 * 1024):
                total += len(chunk)
                if total > MAX_EXPORT_BYTES:
                    raise RunError("compiler export exceeded 2 GiB evidence budget; partial evidence retained")
                require_space(output, "during compiler export")
                data.write(chunk)
            code = process.wait()
        except BaseException:
            process.terminate()
            try:
                process.wait(timeout=10)
            except subprocess.TimeoutExpired:
                process.kill()
                process.wait()
            raise
        finally:
            process.stdout.close()
    if code or (output / "coverage.stderr").read_text().strip():
        raise RunError("compiler coverage export failed or reported an evidence warning")
    require_space(output, "after compiler export")
    return json.loads(exported.read_text())


def execute_one(args, binary, config, env, name, index, *, preserve_failure=False):
    require_space(args.out, "before exact original test process")
    output = args.out / f"case-{index:05d}"
    output.mkdir()
    profiles = output / "profiles"
    profiles.mkdir()
    log = output / "driver-sites.jsonl"
    run_env = dict(env, IR1404_READ_PROOF_MODE=args.mode, IR1404_READ_PROOF_LOG=str(log),
                   IR1404_ORIGINAL_TEST_NAME=name,
                   LLVM_PROFILE_FILE=str(profiles / "driver-%p-%m.profraw"))
    # Exact one-test processes give custom/scoped reader threads an unambiguous
    # original test identity and retain first-read startup as observable work.
    marker=output/"native-exec.json"
    launch = ("import os,sys,time,json; "
              "open(sys.argv[1],'w').write(json.dumps({'pid':os.getpid(),'started_monotonic':time.monotonic()})); "
              "os.execv(sys.argv[2],sys.argv[2:])")
    command = LOCKED + [sys.executable,"-c",launch,str(marker),str(binary),"--exact",name,"--test-threads=1","--nocapture"]
    start = time.monotonic()
    native_start=None
    with (output / "run.log").open("w") as log_output:
        process = subprocess.Popen(command, cwd=args.repo, env=run_env,
                                   stdout=log_output, stderr=subprocess.STDOUT)
        timed_out = False
        timeout = getattr(args, "test_timeout_seconds", None)
        while process.poll() is None:
            if native_start is None and marker.exists():
                try:
                    acquired=json.loads(marker.read_text())
                except json.JSONDecodeError:
                    acquired=None
                if acquired is not None:
                    if acquired["pid"]!=process.pid:
                        raise RunError("lock exec chain changed the tested process identity")
                    native_start=acquired["started_monotonic"]
            if timeout is not None and native_start is not None and time.monotonic()-native_start>=timeout:
                timed_out=True
                process.terminate()
                try: process.wait(timeout=10)
                except subprocess.TimeoutExpired:
                    process.kill();process.wait()
                break
            try:process.wait(timeout=0.1)
            except subprocess.TimeoutExpired:pass
        code=process.returncode
        pid = process.pid
    if timed_out:
        write_json(output / "timeout.json", {"pid":pid,"timeout_seconds":timeout,
                   "classification":"timeout/incomplete; no pass or full-proof credit"})
    validate_snapshot(args.repo, config)
    require_space(output, "after exact original test process")
    parse_errors = []
    try:
        outcomes = test_results((output / "run.log").read_text(), [name])
    except RunError as error:
        outcomes = {"passed_tests": [], "failed_tests": [name], "ignored_tests": []}
        parse_errors.append(str(error))
    report = {"intended_tests": [name], **outcomes, "exit_code": code, "test_threads": 1,
              "command": command, "pid": pid, "elapsed_seconds": time.monotonic() - start,
              "timeout_seconds": timeout, "timed_out": timed_out,
              "native_started":native_start is not None,
              "lock_wait_seconds":(native_start-start) if native_start is not None else None,
              **selection_metadata(args)}
    errors = list(parse_errors)
    if timed_out:
        errors.append(f"original test exceeded recorded {timeout}s timeout; incomplete")
    if code or outcomes["passed_tests"] != [name] or outcomes["failed_tests"] or outcomes["ignored_tests"]:
        errors.append("the intended original-driver test did not execute successfully once")
    raw = sorted(profiles.glob("*.profraw"))
    if not raw:
        errors.append("no actual compiler profiles were emitted")
    if any(f"driver-{pid}-" not in path.name for path in raw):
        errors.append("child-process compiler profile requires its own binary/source census")
    try:
        records = [json.loads(line) for line in log.read_text().splitlines() if line.strip()] if log.exists() else []
    except json.JSONDecodeError as error:
        errors.append("invalid original-site data")
        records = []
    if any(record.get("test") != name for record in records):
        errors.append("captured read provenance does not belong to its exact original test")
    if errors and (output / "run.log").exists():
        tail = (output / "run.log").read_text(errors="replace").splitlines()[-8:]
        errors.append("test output ended with: " + " | ".join(tail))
    report["evidence_errors"] = errors
    write_json(output / "test-report.json", persisted_test_summary(report))
    if not getattr(args, "class_controls", False):
        log.unlink(missing_ok=True)
        (output / "run.log").unlink(missing_ok=True)
    if errors and not preserve_failure:
        raise RunError("; ".join(errors))
    return output, report, raw, records


def run_controls(args, binary, config, env, compiled_tests):
    selected = select_control_tests(compiled_tests, getattr(args, "control_test", []))
    results = []
    for index, name in enumerate(selected):
        should_pass = CONTROL_CASES[name]
        output, report, profiles, records = execute_one(args, binary, config, env, name, index)
        evidence = merge_export(args, output, profiles, binary, env)
        try:
            result = census.check(evidence, args.repo)
            if name == "qspec_read_proof::provider_scope_tests::provider_designated_read_preserves_the_original_scalar_refusal":
                census.validate_provenance(evidence, args.repo, {name}, records)
        except census.CensusError as error:
            if should_pass or not str(error).startswith("unaccounted "):
                raise RunError(f"control failed for the wrong reason: {error}") from error
            result = {"passed": False, "expected_counterexample": True, "error": str(error)}
        else:
            if not should_pass:
                raise RunError("a deliberate real-reader bypass incorrectly passed compiler census")
        result["scope"]="executed malformed-input reader entry controls only"
        write_json(output / "census.json", result)
        archive_export(output)
        results.append({"control": name, "oracle_passed": True, "census": result})
    return results


def run_original_corpus(args, binary, config, env, intended):
    reports, profiles, records = [], [], []
    for index, name in enumerate(intended):
        _, report, raw, captured = execute_one(args, binary, config, env, name, index, preserve_failure=True)
        reports.append(report)
        profiles.extend(raw)
        records.extend(captured)
    report = {"intended_tests": intended,
              "passed_tests": [name for item in reports for name in item["passed_tests"]],
              "failed_tests": [name for item in reports for name in item["failed_tests"]],
              "ignored_tests": [name for item in reports for name in item["ignored_tests"]],
              "exit_code": int(any(item["exit_code"] or item["evidence_errors"] for item in reports)),
              "test_threads": 1, "per_test_processes": reports, **selection_metadata(args)}
    failures = [item for item in reports if item["evidence_errors"]]
    if failures:
        write_json(args.out / "test-report.json", persisted_test_summary(report))
        raise RunError(
            f"{len(failures)} of {len(reports)} intended original tests failed: "
            + "; ".join(f"{item['intended_tests'][0]}: {'; '.join(item['evidence_errors'])}"
                        for item in failures))
    evidence = merge_export(args, args.out, profiles, binary, env)
    result = census.check_run(evidence, args.repo, report, records, args.mode)
    result.update(selection_metadata(args))
    result["scope"]=("selected-only original tests" if getattr(args, "original_test", []) else
                     "executed original TC-048 integration tests only" if args.corpus == "tc048"
                     else "executed original QSpec driver tests only")
    write_json(args.out / "census.json", result)
    write_json(args.out / "test-report.json", persisted_test_summary(report))
    archive_export(args.out)
    return [result]



def persisted_class_proof(result):
    """The class verdicts and compiler counts, without per-test facts or reports."""
    kept = {key: result[key] for key in ("schema","qualified","scope","configuration","static_ac513_proven")}
    kept["canonical_source"] = result["canonical_source"]
    kept["classes"] = [
        {key: ({"counts": value["counts"]} if key in ("kernel_allow", "kernel_deny", "whole_allow", "whole_deny", "whole_exact") and value else value)
         for key, value in item.items()}
        for item in result["classes"]]
    kept["negative_controls"] = result["negative_controls"]
    kept["runs"] = [{"test": run["test"], "rust_oracle_passed": run["rust_oracle_passed"]}
                    for run in result["runs"]]
    return kept


def run_class_controls(args,binary,config,env,compiled_tests):
    """Only per-class execution binding; not AC513 or general provider coverage."""
    manifests=[]
    for line in (args.out/"build.log").read_text().splitlines():
        try: item=json.loads(line)
        except ValueError: continue
        if item.get("reason")=="compiler-artifact" and item.get("target",{}).get("name")=="quire_canonical":
            manifests.append(Path(item["manifest_path"]))
    if len(set(manifests))!=1: raise RunError("actual canonical dependency source identity missing/ambiguous")
    source=manifests[0].parent/"src/read.rs"
    completed={}; cases=[]
    def execute(name,reference=None):
        full="canonical_class_denial::"+name
        if full not in compiled_tests: raise RunError("missing compiled class control "+full)
        child=dict(env)
        if reference: child["IR1404_CANONICAL_REFERENCE_LOG"]=str(reference)
        output,report,profiles,records=execute_one(args,binary,config,child,full,len(cases),preserve_failure=True)
        row={"test":full,"directory":str(output),"rust_oracle_passed":not report["evidence_errors"]}
        cases.append(row)
        if report["evidence_errors"]: return None
        if len(records)!=1: raise RunError("class control must emit exactly one actual fact record")
        export=merge_export(args,output,profiles,binary,env)
        counts=census.canonical_path_counts(export,source)
        archive_export(output)
        result={"facts":records[0],"counts":counts,"directory":str(output)}
        completed[name]=result
        return result
    kernel_empty=execute("kernel_empty")
    whole_empty=execute("whole_empty")
    classes=[]
    for name in census.CANONICAL_CLASS_SITES:
        allowed=execute("kernel_"+name+"_allow")
        denied=execute("kernel_"+name+"_deny")
        whole=execute("whole_"+name+"_allow")
        reference=Path(whole["directory"])/"driver-sites.jsonl" if whole else None
        short=execute("whole_"+name+"_deny",reference)
        exact=execute("whole_"+name+"_exact",reference)
        result={"class":name,"qualified":False,"kernel_allow":allowed,"kernel_deny":denied,
                "whole_allow":whole,"whole_deny":short,"whole_exact":exact}
        if denied:
            try:
                result["source_binding"]=census.bind_canonical_denial(name,denied["counts"],denied["facts"],source)
                result["kernel_bound"]=True
            except census.CensusError as error:result["binding_error"]=str(error)
        if result.get("kernel_bound") and all(x is not None for x in (kernel_empty,whole_empty,allowed,denied,whole,short,exact)):
            try:
                for kernel,actual in ((allowed,whole),(denied,short)):
                    for function in census.CANONICAL_PATH_FUNCTIONS:
                        # Three whole reads: OFF/count/record. The identical
                        # owned-catalog Parser instance cancels against empty.
                        left=actual["counts"][function]-whole_empty["counts"][function]
                        right=3*(kernel["counts"][function]-kernel_empty["counts"][function])
                        if left!=right: raise census.CensusError("whole/package Parser instance contrast disagrees with kernel at "+function)
                result["qualified"]=True
            except census.CensusError as error:result["binding_error"]=str(error)
        classes.append(result)
    negatives=[]
    for test,claim in [("negative_empty_string_is_not_text_growth","text"),
                       ("negative_empty_array_is_not_retained_children","children"),
                       ("negative_scratch_child_is_not_retained_children","children")]:
        actual=execute(test)
        row={"test":test,"claimed_class":claim,"rejected":False,"rust_oracle_passed":actual is not None}
        if actual:
            try:census.bind_canonical_denial(claim,actual["counts"],actual["facts"],source)
            except census.CensusError as error:row.update(rejected=True,reason=str(error))
        negatives.append(row)
    qualified=census.canonical_classes_complete(classes,negatives)
    result={"schema":"canonical-class-denial/v1","qualified":qualified,
            "scope":"minimal canonical class-denial controls only; not positive/adverse corpus completeness or AC513",
            "configuration":"config.json","canonical_source":str(source),"classes":classes,
            "negative_controls":negatives,"runs":cases,"static_ac513_proven":False}
    # A reference log is only a channel between one allow run and its deny
    # runs; nothing of it outlives the proof.
    for case in cases:
        for name in ("driver-sites.jsonl","run.log"):
            (Path(case["directory"])/name).unlink(missing_ok=True)
    write_json(args.out/"class-proof.json",persisted_class_proof(result))
    if not qualified: raise RunError("class denial proof incomplete/RED; preserve individual run and binding results")
    return [persisted_class_proof(result)]


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--repo", type=Path, required=True)
    parser.add_argument("--out", type=Path, required=True)
    parser.add_argument("--target-dir", type=Path, required=True)
    parser.add_argument("--toolchain-bin", type=Path, required=True)
    parser.add_argument("--llvm-bin", type=Path, required=True)
    parser.add_argument("--mode", choices=["sample", "full"], default="sample")
    parser.add_argument("--original-test", action="append", default=[], help="repeatable exact compiled original test symbol; selected-only evidence")
    parser.add_argument("--controls", action="store_true")
    parser.add_argument("--control-test", action="append", default=[], help="exact entry-control symbol; requires --controls, selected-only evidence")
    parser.add_argument("--class-controls",action="store_true")
    parser.add_argument("--corpus", choices=["qspec", "tc048"], default="qspec")
    parser.add_argument("--features", action="append", default=[])
    parser.add_argument("--qspec-dir", type=Path, required=True)
    parser.add_argument("--test-timeout-seconds", type=float,
                        help="per original process; sample defaults to120s, full requires an adequate explicit value")
    args = parser.parse_args()
    if args.control_test and (not args.controls or args.class_controls or args.original_test):
        parser.error("--control-test requires only the entry-control mode")
    if args.original_test and (args.controls or args.class_controls):
        parser.error("--original-test cannot mix with entry or class controls")
    if args.test_timeout_seconds is None:
        if args.mode == "full":
            parser.error("full mode requires an explicit adequate --test-timeout-seconds")
        args.test_timeout_seconds = 120.0
    if not math.isfinite(args.test_timeout_seconds) or args.test_timeout_seconds <= 0:
        parser.error("test timeout must be positive")
    args.target_dir = owned_target(args.repo, args.target_dir)
    for field in ["repo", "out", "target_dir", "toolchain_bin", "llvm_bin", "qspec_dir"]:
        setattr(args, field, getattr(args, field).resolve())
    if args.out.exists():
        raise SystemExit("output directory already exists; preserve each run separately")
    if args.out.is_relative_to(args.repo) and not args.out.is_relative_to(args.repo / "target"):
        raise SystemExit("proof artifacts belong in /tmp or the owned target directory")
    args.out.mkdir(parents=True)
    try:
        if git(args.repo, "status", "--porcelain"):
            raise RunError("freeze and commit the source before compiler evidence")
        if "test-support" not in set(",".join(args.features).split(",")):
            raise RunError("compiler proof requires explicit test-support for checked passive/seeded boundaries")
        if git(args.qspec_dir, "status", "--porcelain"):
            raise RunError("select a clean QSpec checkout for the one recorded input snapshot")
        require_space(args.repo, "before compiler build")
        rustc = subprocess.check_output([str(args.toolchain_bin / "rustc"), "-vV"], text=True)
        llvm = subprocess.check_output([str(args.llvm_bin / "llvm-cov"), "--version"], text=True)
        version = re.search(r"LLVM version:\s*(\S+)", rustc)
        if not version or version[1] not in llvm:
            raise RunError("installed rustc and LLVM coverage tools do not match")
        flags = "-Cinstrument-coverage -Csymbol-mangling-version=v0"
        if os.environ.get("CARGO_ENCODED_RUSTFLAGS") or os.environ.get("RUSTFLAGS", flags) != flags:
            raise RunError("conflicting inherited rustflags; select the explicit recorded proof configuration")
        if os.environ.get("RUSTC_WRAPPER") or os.environ.get("RUSTC_WORKSPACE_WRAPPER"):
            raise RunError("select the actual recorded compiler without inherited compiler wrappers")
        env = dict(os.environ, RUSTC_WRAPPER="", RUSTC_WORKSPACE_WRAPPER="", CARGO_BUILD_JOBS="2", CARGO_INCREMENTAL="0",
                   CARGO_PROFILE_DEV_DEBUG="0", CARGO_PROFILE_TEST_DEBUG="0",
                   CARGO_PROFILE_DEV_OPT_LEVEL="0", CARGO_PROFILE_TEST_OPT_LEVEL="0",
                   CARGO_TARGET_DIR=str(args.target_dir), RUSTFLAGS=flags,
                   RUSTC=str(args.toolchain_bin / "rustc"), RUSTDOC=str(args.toolchain_bin / "rustdoc"),
                   CARGO=str(args.toolchain_bin / "cargo"), QUIRE_SPECIFICATION_DIR=str(args.qspec_dir))
        env["PATH"] = str(args.toolchain_bin) + os.pathsep + env["PATH"]
        config = {"rustc": rustc, "llvm": llvm,
                  "mode": args.mode, "controls": args.controls, "class_controls":args.class_controls, "corpus": args.corpus, "features": args.features,
                  "test_timeout_seconds":args.test_timeout_seconds, "original_tests":args.original_test, "control_tests":args.control_test,
                  **selection_metadata(args),
                  "qspec": {"path": str(args.qspec_dir)},
                  "environment": safe_environment(env)}
        target = ("read_storage_alloc" if args.class_controls else "read_entry_census_controls" if args.controls else
                  "it" if args.corpus == "tc048" else "conformance_qspec")
        binary = build(args, target, env, config)
        intended = discover_tests(args, binary, env)
        if not args.controls and args.corpus == "tc048":
            intended = select_tc048(args, intended)
        intended = select_original_tests(intended, args.original_test, args.controls, args.class_controls)
        if args.controls:
            intended = select_control_tests(intended, args.control_test)
        write_json(args.out / "config.json", config)
        results = (run_class_controls(args,binary,config,env,intended) if args.class_controls else
                   run_controls(args, binary, config, env, intended) if args.controls
                   else run_original_corpus(args, binary, config, env, intended))
        write_json(args.out / "result.json", {"passed": True, "results": results, **selection_metadata(args),
                   "normative_boundaries_verified": args.mode == "full" and not args.controls and not args.class_controls,
                   "normative_boundary_proof": False,
                   "static_ac513_proven": False})
    except (RunError, census.CensusError, OSError, ValueError, subprocess.CalledProcessError) as error:
        write_json(args.out / "result.json", {"passed": False, "error": str(error), **selection_metadata(args)})
        raise SystemExit(str(error)) from error


if __name__ == "__main__":
    main()
