// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (C) 2026 Agent-IX

//! Compile real default-feature consumers independently of root dev features.

use std::{
    fs,
    path::{Path, PathBuf},
    process::{Command, Output},
    sync::atomic::{AtomicUsize, Ordering},
};

use ix_trace_rs::trace;
use serde_json::Value;

static NEXT_SCRATCH: AtomicUsize = AtomicUsize::new(0);
const CONSUMER: &str = "default-mapping-consumer";

struct Scratch(PathBuf);

impl Scratch {
    fn new() -> Self {
        let serial = NEXT_SCRATCH.fetch_add(1, Ordering::Relaxed);
        let root = std::env::temp_dir().join(format!(
            "quire-default-mapping-{}-{serial}",
            std::process::id()
        ));
        fs::create_dir(&root).expect("unique consumer directory");
        let scratch = Self(root);
        fs::create_dir(scratch.0.join("src")).expect("consumer source directory");
        let model = Path::new(env!("CARGO_MANIFEST_DIR")).join("crates/quire-contract-model");
        // A TOML literal string preserves the actual path without escape conversion.
        let model = model.to_str().expect("UTF-8 model path");
        assert!(
            !model.contains(['\'', '\n', '\r']),
            "TOML literal model path"
        );
        fs::write(
            scratch.0.join("Cargo.toml"),
            format!(
                "[workspace]\n[package]\nname = '{CONSUMER}'\nversion = '0.0.0'\nedition = '2021'\n\
                 [dependencies]\nquire-contract-model = {{ path = '{model}' }}\n"
            ),
        )
        .expect("independent default-feature consumer manifest");
        scratch
    }

    fn check(&self, source: &str) -> Output {
        fs::write(self.0.join("src/main.rs"), source).expect("consumer probe source");
        Command::new(env!("CARGO"))
            .args([
                "check",
                "--offline",
                "--workspace",
                "--message-format=json",
                "-j2",
            ])
            .arg("--manifest-path")
            .arg(self.0.join("Cargo.toml"))
            .current_dir(env!("CARGO_MANIFEST_DIR"))
            .env("CARGO_TARGET_DIR", self.0.join("target"))
            .output()
            .expect("run real consumer compiler")
    }

    fn assert_absent(&self, source: &str, symbol: &str, code: &str) {
        let output = self.check(source);
        let report = diagnostic(&output);
        assert_eq!(output.status.code(), Some(101), "{report}");
        let mut errors = 0;
        for line in output
            .stdout
            .split(|byte| *byte == b'\n')
            .filter(|line| !line.is_empty())
        {
            let message: Value = serde_json::from_slice(line).expect("Cargo JSON message");
            if message["reason"] != "compiler-message" || message["message"]["level"] != "error" {
                continue;
            }
            errors += 1;
            assert_eq!(message["target"]["name"], CONSUMER, "{report}");
            assert_eq!(
                message["target"]["src_path"].as_str().map(Path::new),
                Some(self.0.join("src/main.rs").as_path()),
                "{report}"
            );
            assert_eq!(message["message"]["code"]["code"], code, "{report}");
            let spans = message["message"]["spans"]
                .as_array()
                .expect("diagnostic spans");
            assert!(
                spans.iter().any(|span| {
                    let start = span["byte_start"]
                        .as_u64()
                        .and_then(|n| usize::try_from(n).ok());
                    let end = span["byte_end"]
                        .as_u64()
                        .and_then(|n| usize::try_from(n).ok());
                    span["is_primary"] == true
                        && span["file_name"].as_str().map(|path| self.0.join(path))
                            == Some(self.0.join("src/main.rs"))
                        && start
                            .zip(end)
                            .and_then(|(start, end)| source.get(start..end))
                            == Some(symbol)
                }),
                "expected primary span on {symbol}: {report}"
            );
        }
        assert_eq!(errors, 1, "exactly the intended consumer error: {report}");
    }
}

impl Drop for Scratch {
    fn drop(&mut self) {
        fs::remove_dir_all(&self.0).expect("remove own compiler scratch directory");
    }
}

fn diagnostic(output: &Output) -> String {
    format!(
        "stdout={} stderr={}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    )
}

#[test]
#[trace("TC-018", "FR-019-AC-4")]
fn default_mapping_surface_excludes_allocation_fault_injection() {
    let consumer = Scratch::new();
    let healthy = consumer
        .check("fn main() { let _ = quire_contract_model::MappingExecutionControl::active(); }\n");
    assert!(
        healthy.status.success(),
        "healthy default consumer: {}",
        diagnostic(&healthy)
    );
    consumer.assert_absent(
        "use quire_contract_model::MappingAllocationPoint; fn main() {}\n",
        "quire_contract_model::MappingAllocationPoint",
        "E0432",
    );
    consumer.assert_absent(
        "fn main() { let _ = quire_contract_model::MappingExecutionControl::fail_allocation_at; }\n",
        "fail_allocation_at",
        "E0599",
    );
}
