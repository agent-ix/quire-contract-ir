// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (C) 2026 Agent-IX

//! Isolated FR-038-AC-214 reader peak lane. Run this explicit test target in
//! fresh processes; it is excluded from the ordinary `make test` suite.

#[path = "it/support/mod.rs"]
mod support;

use ix_trace_rs::trace;
use quire_contract_model::{
    CheckedPackageReadLimits, CheckedPackageV2, CheckedPackageV2ReadResult,
};
use std::sync::atomic::{AtomicBool, AtomicU64, Ordering};
use std::sync::Arc;
use std::time::{Duration, Instant};
use support::checked_package::{evidence_for, v2_all_families, v2_reference_chain};

fn generous_limits(nodes: usize) -> CheckedPackageReadLimits {
    let count = u64::try_from(nodes).expect("node count");
    CheckedPackageReadLimits {
        bytes: 1 << 36,
        nodes: count * 2,
        edges: count * 16,
        occurrences: count * 16,
        diagnostics: 1 << 20,
        work: count * 1024,
    }
}

/// Generate the AC-214 fixture in a process separate from every measured read.
/// Set `IR575_FIXTURE_PATH` to the owned temporary output path. Ignored so a
/// routine suite never generates a 177 MB document.
///
/// Tracing: TC-048, FR-038-AC-214
#[trace("TC-048", "FR-038-AC-214")]
#[test]
#[ignore]
fn tc_048_ir575_generate_reader_peak_fixture() {
    let path = std::env::var("IR575_FIXTURE_PATH").expect("IR575_FIXTURE_PATH");
    let (bytes, _) = v2_reference_chain(100_000);
    assert_eq!(bytes.len(), 176_923_194, "fixed AC-214 fixture identity");
    std::fs::write(&path, &bytes).expect("write fixture");
    eprintln!("fixture_bytes={} path={path}", bytes.len());
}

fn proc_memory_kib() -> (u64, u64) {
    let status = std::fs::read_to_string("/proc/self/status").expect("Linux proc status");
    let field = |name: &str| {
        status
            .lines()
            .find_map(|line| line.strip_prefix(name))
            .and_then(|value| value.split_whitespace().next())
            .and_then(|value| value.parse::<u64>().ok())
            .expect("memory field in KiB")
    };
    (field("VmRSS:"), field("VmHWM:"))
}

/// The AC-214 oracle: a fresh process, with the fixture and evidence prepared
/// before its baseline, measures only the reader. Ignored so ordinary CI does
/// not spend memory and minutes on the 100000-node performance case.
///
/// Tracing: TC-048, FR-038-AC-214
#[trace("TC-048", "FR-038-AC-214")]
#[test]
#[ignore]
fn tc_048_ir575_reader_only_peak() {
    assert_eq!(std::env::consts::OS, "linux");
    assert_eq!(std::env::consts::ARCH, "x86_64");
    let path = std::env::var("IR575_FIXTURE_PATH").expect("IR575_FIXTURE_PATH");
    let bytes = std::fs::read(&path).expect("read prebuilt fixture");
    assert_eq!(bytes.len(), 176_923_194, "fixed AC-214 fixture identity");
    let base = v2_all_families();
    let evidence = evidence_for(&base);
    let total = 100_000
        + base["semantic_graph"]["nodes"]
            .as_array()
            .expect("nodes")
            .len();
    assert_eq!(total, 100_022);
    drop(base);

    let (baseline_rss, baseline_hwm) = proc_memory_kib();
    assert_eq!(
        baseline_hwm, baseline_rss,
        "invalid_setup: pre-read HWM differs from RSS"
    );
    let running = Arc::new(AtomicBool::new(true));
    let peak = Arc::new(AtomicU64::new(baseline_rss));
    let (sample_running, sample_peak) = (Arc::clone(&running), Arc::clone(&peak));
    let sampler = std::thread::spawn(move || {
        while sample_running.load(Ordering::Relaxed) {
            let (rss, _) = proc_memory_kib();
            sample_peak.fetch_max(rss, Ordering::Relaxed);
            std::thread::sleep(Duration::from_millis(2));
        }
    });
    let started = Instant::now();
    let read = CheckedPackageV2::read(&bytes, generous_limits(total), &evidence);
    let elapsed = started.elapsed();
    running.store(false, Ordering::Relaxed);
    sampler.join().expect("sampler");
    let CheckedPackageV2ReadResult::Admitted(package) = read else {
        panic!("fixture must admit, got {read:?}");
    };
    let (post_rss, post_hwm) = proc_memory_kib();
    let sampled_peak = peak.load(Ordering::Relaxed);
    let incremental_peak = sampled_peak.max(post_hwm) - baseline_rss;
    eprintln!(
        "input_bytes={} admitted_nodes={} baseline_rss_kib={} baseline_hwm_kib={} sampled_peak_rss_kib={} post_read_rss_kib={} post_read_hwm_kib={} incremental_peak_kib={} elapsed_ms={} platform={}-{} allocator=System",
        bytes.len(), package.graph().nodes.len(), baseline_rss, baseline_hwm, sampled_peak,
        post_rss, post_hwm, incremental_peak, elapsed.as_millis(),
        std::env::consts::OS, std::env::consts::ARCH,
    );
    assert_eq!(package.graph().nodes.len(), total);
    assert!(
        incremental_peak <= 1_572_864,
        "reader incremental peak {incremental_peak} KiB exceeds AC-214 ceiling",
    );
    std::hint::black_box(&package);
}
