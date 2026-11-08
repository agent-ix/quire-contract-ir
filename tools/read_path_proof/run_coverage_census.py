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
import time

import coverage_census as census

LOCKED = ["flock", "--no-fork", "/tmp/titan-cargo.lock", "nice", "-n", "10", "ionice", "-c", "3"]
CONTROL_CASES = {
    "wrapped_function_item_has_no_unaccounted_entry": True,
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


def validate_snapshot(root, config):
    if git(root, "rev-parse", "HEAD") != config["source_head"] or git(root, "status", "--porcelain"):
        raise RunError("source snapshot changed after the clean frozen build")
    qspec = Path(config["qspec"]["path"])
    if git(qspec, "rev-parse", "HEAD") != config["qspec"]["head"] or git(qspec, "status", "--porcelain"):
        raise RunError("selected QSpec snapshot changed during proof")


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
    write_json(args.out / "tc048-selected-tests.json", selected)
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
    command = LOCKED + [str(binary), "--exact", name, "--test-threads=1", "--nocapture"]
    start = time.monotonic()
    with (output / "run.log").open("w") as log_output:
        process = subprocess.Popen(command, cwd=args.repo, env=run_env,
                                   stdout=log_output, stderr=subprocess.STDOUT)
        timed_out = False
        timeout = getattr(args, "test_timeout_seconds", None)
        try:
            code = process.wait(timeout=timeout)
        except subprocess.TimeoutExpired:
            timed_out = True
            process.terminate()
            try:
                code = process.wait(timeout=10)
            except subprocess.TimeoutExpired:
                process.kill()
                code = process.wait()
        pid = process.pid
    if timed_out:
        write_json(output / "timeout.json", {"pid":pid,"test":name,"timeout_seconds":timeout,
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
              "source_head": config["source_head"]}
    write_json(output / "test-report.json", report)
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
        errors.append(f"invalid original-site records retained at {log}: {error}")
        records = []
    if any(record.get("test") != name for record in records):
        errors.append("captured read provenance does not belong to its exact original test")
    report["evidence_errors"] = errors
    write_json(output / "test-report.json", report)
    if errors and not preserve_failure:
        raise RunError("; ".join(errors))
    return output, report, raw, records


def run_controls(args, binary, config, env, compiled_tests):
    missing = set(CONTROL_CASES) - set(compiled_tests)
    if missing:
        raise RunError(f"missing actual compiled counterexamples: {sorted(missing)}")
    results = []
    for index, (name, should_pass) in enumerate(CONTROL_CASES.items()):
        output, report, profiles, records = execute_one(args, binary, config, env, name, index)
        evidence = merge_export(args, output, profiles, binary, env)
        try:
            result = census.check(evidence, args.repo)
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
              "test_threads": 1, "per_test_processes": reports, "source_head": config["source_head"]}
    write_json(args.out / "test-report.json", report)
    with (args.out / "driver-sites.jsonl").open("w") as log:
        for record in records:
            log.write(json.dumps(record) + "\n")
    if any(item["evidence_errors"] for item in reports):
        raise RunError("all intended original tests attempted; failures/evidence errors retained in per-test reports")
    evidence = merge_export(args, args.out, profiles, binary, env)
    result = census.check_run(evidence, args.repo, report, records, args.mode)
    result["scope"]=("executed original TC-048 integration tests only" if args.corpus == "tc048"
                     else "executed original QSpec driver tests only")
    write_json(args.out / "census.json", result)
    archive_export(args.out)
    return [result]



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
        row={"test":full,"directory":str(output),"rust_oracle_passed":not report["evidence_errors"],"report":report}
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
        row={"test":test,"claimed_class":claim,"rejected":False}
        if actual:
            try:census.bind_canonical_denial(claim,actual["counts"],actual["facts"],source)
            except census.CensusError as error:row.update(rejected=True,reason=str(error))
        negatives.append(row)
    qualified=all(case["qualified"] for case in classes) and all(x["rejected"] for x in negatives)
    result={"schema":"canonical-class-denial/v1","qualified":qualified,
            "scope":"minimal canonical class-denial controls only; not positive/adverse corpus completeness or AC513",
            "configuration":"config.json","canonical_source":str(source),"classes":classes,
            "negative_controls":negatives,"runs":cases,"static_ac513_proven":False}
    write_json(args.out/"class-proof.json",result)
    if not qualified: raise RunError("class denial proof incomplete/RED; preserve individual run and binding results")
    return [result]


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--repo", type=Path, required=True)
    parser.add_argument("--out", type=Path, required=True)
    parser.add_argument("--target-dir", type=Path, required=True)
    parser.add_argument("--toolchain-bin", type=Path, required=True)
    parser.add_argument("--llvm-bin", type=Path, required=True)
    parser.add_argument("--mode", choices=["sample", "full"], default="sample")
    parser.add_argument("--controls", action="store_true")
    parser.add_argument("--class-controls",action="store_true")
    parser.add_argument("--corpus", choices=["qspec", "tc048"], default="qspec")
    parser.add_argument("--features", action="append", default=[])
    parser.add_argument("--qspec-dir", type=Path, required=True)
    parser.add_argument("--test-timeout-seconds", type=float,
                        help="per original process; sample defaults to120s, full requires an adequate explicit value")
    args = parser.parse_args()
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
        config = {"source_head": git(args.repo, "rev-parse", "HEAD"), "rustc": rustc, "llvm": llvm,
                  "mode": args.mode, "controls": args.controls, "class_controls":args.class_controls, "corpus": args.corpus, "features": args.features,
                  "test_timeout_seconds":args.test_timeout_seconds,
                  "qspec": {"path": str(args.qspec_dir), "head": git(args.qspec_dir, "rev-parse", "HEAD")},
                  "environment": safe_environment(env)}
        target = ("read_storage_alloc" if args.class_controls else "read_entry_census_controls" if args.controls else
                  "it" if args.corpus == "tc048" else "conformance_qspec")
        binary = build(args, target, env, config)
        intended = discover_tests(args, binary, env)
        if not args.controls and args.corpus == "tc048":
            intended = select_tc048(args, intended)
        results = (run_class_controls(args,binary,config,env,intended) if args.class_controls else
                   run_controls(args, binary, config, env, intended) if args.controls
                   else run_original_corpus(args, binary, config, env, intended))
        write_json(args.out / "result.json", {"passed": True, "results": results,
                   "normative_boundary_proof": args.mode == "full" and not args.controls and not args.class_controls,
                   "static_ac513_proven": False})
    except (RunError, census.CensusError, OSError, ValueError, subprocess.CalledProcessError) as error:
        write_json(args.out / "result.json", {"passed": False, "error": str(error)})
        raise SystemExit(str(error)) from error


if __name__ == "__main__":
    main()
