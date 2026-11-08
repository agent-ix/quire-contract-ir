#!/usr/bin/env python3
# SPDX-License-Identifier: AGPL-3.0-or-later
# Copyright (C) 2026 Agent-IX
"""Build/run a frozen original-driver corpus and export real compiler counts.

All Cargo/build and original-driver runs are serialized by the shared lock.
This tool neither installs toolchains nor changes manifests or global config.
"""
import argparse
import json
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
    "unwrapped_type_alias_before_trusted_read_must_fail_census": False,
    "unwrapped_function_item_after_trusted_read_must_fail_census": False,
    "unwrapped_shared_helper_before_trusted_read_must_fail_census": False,
    "unwrapped_shared_helper_after_trusted_read_must_fail_census": False,
}


class RunError(ValueError):
    pass


SAFE_ENVIRONMENT = ["CARGO_BUILD_JOBS", "CARGO_INCREMENTAL", "CARGO_PROFILE_DEV_DEBUG",
                    "CARGO_PROFILE_TEST_DEBUG", "CARGO_PROFILE_DEV_OPT_LEVEL",
                    "CARGO_PROFILE_TEST_OPT_LEVEL", "CARGO_TARGET_DIR", "RUSTFLAGS",
                    "RUSTC", "RUSTDOC", "CARGO"]


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


def merge_export(args, output, profiles, binary, env):
    merged = output / "driver.profdata"
    checked([str(args.llvm_bin / "llvm-profdata"), "merge", "-sparse", *map(str, profiles), "-o", str(merged)],
            output / "profile-merge.log", cwd=args.repo, env=env)
    exported = output / "coverage.json"
    with exported.open("w") as data, (output / "coverage.stderr").open("w") as error:
        result = subprocess.run([str(args.llvm_bin / "llvm-cov"), "export", str(binary),
                                 "-instr-profile=" + str(merged)], stdout=data, stderr=error, env=env)
    if result.returncode or (output / "coverage.stderr").read_text().strip():
        raise RunError("compiler coverage export failed or reported an evidence warning")
    return json.loads(exported.read_text())


def execute_one(args, binary, config, env, name, index):
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
    pid = checked(command, output / "run.log", cwd=args.repo, env=run_env)
    validate_snapshot(args.repo, config)
    outcomes = test_results((output / "run.log").read_text(), [name])
    report = {"intended_tests": [name], **outcomes, "exit_code": 0, "test_threads": 1,
              "command": command, "pid": pid, "elapsed_seconds": time.monotonic() - start,
              "source_head": config["source_head"]}
    write_json(output / "test-report.json", report)
    if outcomes["passed_tests"] != [name] or outcomes["failed_tests"] or outcomes["ignored_tests"]:
        raise RunError("the intended original-driver test did not execute successfully once")
    raw = sorted(profiles.glob("*.profraw"))
    if not raw:
        raise RunError("no actual compiler profiles were emitted")
    if any(f"driver-{pid}-" not in path.name for path in raw):
        raise RunError("child-process compiler profile requires its own binary/source census")
    records = [json.loads(line) for line in log.read_text().splitlines() if line.strip()] if log.exists() else []
    if any(record.get("test") != name for record in records):
        raise RunError("captured read provenance does not belong to its exact original test")
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
        write_json(output / "census.json", result)
        results.append({"control": name, "oracle_passed": True, "census": result})
    return results


def run_original_corpus(args, binary, config, env, intended):
    reports, profiles, records = [], [], []
    for index, name in enumerate(intended):
        _, report, raw, captured = execute_one(args, binary, config, env, name, index)
        reports.append(report)
        profiles.extend(raw)
        records.extend(captured)
    report = {"intended_tests": intended,
              "passed_tests": [name for item in reports for name in item["passed_tests"]],
              "failed_tests": [], "ignored_tests": [], "exit_code": 0, "test_threads": 1,
              "per_test_processes": reports, "source_head": config["source_head"]}
    write_json(args.out / "test-report.json", report)
    with (args.out / "driver-sites.jsonl").open("w") as log:
        for record in records:
            log.write(json.dumps(record) + "\n")
    evidence = merge_export(args, args.out, profiles, binary, env)
    result = census.check_run(evidence, args.repo, report, records, args.mode)
    write_json(args.out / "census.json", result)
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
    parser.add_argument("--features", action="append", default=[])
    parser.add_argument("--qspec-dir", type=Path, required=True)
    args = parser.parse_args()
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
        if git(args.qspec_dir, "status", "--porcelain"):
            raise RunError("select a clean QSpec checkout for the one recorded input snapshot")
        if shutil.disk_usage(args.repo).free < 20 * 1024**3:
            raise RunError("less than 20 GiB free; do not start a new compiler job")
        rustc = subprocess.check_output([str(args.toolchain_bin / "rustc"), "-vV"], text=True)
        llvm = subprocess.check_output([str(args.llvm_bin / "llvm-cov"), "--version"], text=True)
        version = re.search(r"LLVM version:\s*(\S+)", rustc)
        if not version or version[1] not in llvm:
            raise RunError("installed rustc and LLVM coverage tools do not match")
        flags = "-Cinstrument-coverage -Csymbol-mangling-version=v0"
        if os.environ.get("CARGO_ENCODED_RUSTFLAGS") or os.environ.get("RUSTFLAGS", flags) != flags:
            raise RunError("conflicting inherited rustflags; select the explicit recorded proof configuration")
        env = dict(os.environ, CARGO_BUILD_JOBS="2", CARGO_INCREMENTAL="0",
                   CARGO_PROFILE_DEV_DEBUG="0", CARGO_PROFILE_TEST_DEBUG="0",
                   CARGO_PROFILE_DEV_OPT_LEVEL="0", CARGO_PROFILE_TEST_OPT_LEVEL="0",
                   CARGO_TARGET_DIR=str(args.target_dir), RUSTFLAGS=flags,
                   RUSTC=str(args.toolchain_bin / "rustc"), RUSTDOC=str(args.toolchain_bin / "rustdoc"),
                   CARGO=str(args.toolchain_bin / "cargo"), QUIRE_SPECIFICATION_DIR=str(args.qspec_dir))
        env["PATH"] = str(args.toolchain_bin) + os.pathsep + env["PATH"]
        config = {"source_head": git(args.repo, "rev-parse", "HEAD"), "rustc": rustc, "llvm": llvm,
                  "mode": args.mode, "controls": args.controls, "features": args.features,
                  "qspec": {"path": str(args.qspec_dir), "head": git(args.qspec_dir, "rev-parse", "HEAD")},
                  "environment": safe_environment(env)}
        target = "read_entry_census_controls" if args.controls else "conformance_qspec"
        binary = build(args, target, env, config)
        intended = discover_tests(args, binary, env)
        results = (run_controls(args, binary, config, env, intended) if args.controls
                   else run_original_corpus(args, binary, config, env, intended))
        write_json(args.out / "result.json", {"passed": True, "results": results,
                   "normative_boundary_proof": args.mode == "full" and not args.controls,
                   "static_ac513_proven": False})
    except (RunError, census.CensusError, OSError, ValueError, subprocess.CalledProcessError) as error:
        write_json(args.out / "result.json", {"passed": False, "error": str(error)})
        raise SystemExit(str(error)) from error


if __name__ == "__main__":
    main()
