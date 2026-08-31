use std::{
    ffi::OsString,
    io::{self, Write as _},
    path::Path,
    process::ExitCode,
};

use quire_contract_ir::{
    run_manifest, FixtureStatus, RunnerError, RunnerErrorCode, CONFORMANCE_PROTOCOL,
};

fn main() -> ExitCode {
    match execute(std::env::args_os().skip(1).collect()) {
        Ok((bytes, mismatched)) => {
            if io::stdout().lock().write_all(&bytes).is_err() {
                write_error(RunnerError::new(
                    RunnerErrorCode::FixtureIo,
                    "stdout",
                    "result stream is unwritable",
                ));
                ExitCode::from(2)
            } else if mismatched {
                ExitCode::from(1)
            } else {
                ExitCode::SUCCESS
            }
        }
        Err(error) => {
            write_error(error);
            ExitCode::from(2)
        }
    }
}

fn execute(arguments: Vec<OsString>) -> Result<(Vec<u8>, bool), RunnerError> {
    if arguments.len() == 1 && arguments[0] == "--version" {
        return Ok((
            format!(
                "quire-contract-ir {} {CONFORMANCE_PROTOCOL}\n",
                env!("CARGO_PKG_VERSION")
            )
            .into_bytes(),
            false,
        ));
    }
    if arguments.len() != 3 || arguments[0] != "run" || arguments[1] != "--manifest" {
        return Err(invalid_invocation());
    }
    let manifest = arguments[2].to_str().ok_or_else(invalid_invocation)?;
    let results = run_manifest(Path::new(manifest))?;
    let mismatched = results
        .iter()
        .any(|result| result.status() == FixtureStatus::Mismatch);
    let mut output = Vec::new();
    for result in results {
        serde_json::to_writer(&mut output, &result).map_err(|_| {
            RunnerError::new(
                RunnerErrorCode::ResourceExhausted,
                "results",
                "result buffer unavailable",
            )
        })?;
        output.push(b'\n');
    }
    Ok((output, mismatched))
}

fn invalid_invocation() -> RunnerError {
    RunnerError::new(
        RunnerErrorCode::InvalidInvocation,
        "arguments",
        "expected run --manifest PATH or --version",
    )
}

fn write_error(error: RunnerError) {
    let mut bytes = serde_json::to_vec(&error).unwrap_or_else(|_| {
        b"{\"protocol\":\"quire.contract.conformance-jsonl/v1\",\"code\":\"resource_exhausted\",\"path\":\"error\",\"detail\":\"error buffer unavailable\"}".to_vec()
    });
    bytes.push(b'\n');
    let _ = io::stderr().lock().write_all(&bytes);
}
