use std::{
    ffi::OsString,
    io::{self, Write as _},
    path::Path,
    process::ExitCode,
};

use quire_contract_model::{
    run_corpus, FixtureStatus, RunnerError, RunnerErrorCode, CONFORMANCE_PROTOCOL,
};
use serde::Serialize;

fn main() -> ExitCode {
    let status = match execute(std::env::args_os().skip(1).collect()) {
        Ok((bytes, mismatched)) => {
            if ix_cli_kit::streams::write_primary_stdout(&bytes).is_err() {
                write_error(RunnerError::new(
                    RunnerErrorCode::FixtureIo,
                    "stdout",
                    "result stream is unwritable",
                ));
                2
            } else {
                u8::from(mismatched)
            }
        }
        Err(error) => {
            write_error(error);
            2
        }
    };
    // This protocol selects only 0..=2; handle the owner's reserved-status error
    // explicitly rather than letting Result termination print another diagnostic.
    ix_cli_kit::exit::caller_exit_code(status).unwrap_or(ExitCode::from(2))
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
    if arguments.len() != 5
        || arguments[0] != "run"
        || arguments[1] != "--corpus"
        || arguments[3] != "--schemas"
    {
        return Err(invalid_invocation());
    }
    let corpus = arguments[2].to_str().ok_or_else(invalid_invocation)?;
    let schemas = arguments[4].to_str().ok_or_else(invalid_invocation)?;
    let results = run_corpus(Path::new(corpus), Path::new(schemas))?;
    let mismatched = results
        .iter()
        .any(|result| result.status() == FixtureStatus::Mismatch);
    Ok((result_bytes(&results)?, mismatched))
}

fn result_bytes<T: Serialize>(results: &[T]) -> Result<Vec<u8>, RunnerError> {
    let mut output = Vec::new();
    for result in results {
        ix_cli_kit::json::write_compact(&mut output, result).map_err(|_| {
            RunnerError::new(
                RunnerErrorCode::ResourceExhausted,
                "results",
                "result buffer unavailable",
            )
        })?;
        output.push(b'\n');
    }
    Ok(output)
}

fn invalid_invocation() -> RunnerError {
    RunnerError::new(
        RunnerErrorCode::InvalidInvocation,
        "arguments",
        "expected run --corpus DIR --schemas DIR or --version",
    )
}

fn write_error(error: RunnerError) {
    let _ = io::stderr().lock().write_all(&error_bytes(&error));
}

fn error_bytes(error: &impl Serialize) -> Vec<u8> {
    let mut bytes = Vec::new();
    if ix_cli_kit::json::write_compact(&mut bytes, error).is_err() {
        bytes.clear();
        bytes.extend_from_slice(b"{\"protocol\":\"quire.contract.conformance-jsonl/v1\",\"code\":\"resource_exhausted\",\"path\":\"error\",\"detail\":\"error buffer unavailable\"}");
    }
    bytes.push(b'\n');
    bytes
}

#[cfg(test)]
mod tests {
    use std::cell::Cell;

    use ix_trace_rs::trace;
    use serde::{ser::SerializeStruct, Serialize, Serializer};

    use super::{error_bytes, result_bytes};
    use quire_contract_model::{RunnerError, RunnerErrorCode};

    struct RecordAttempt<'a> {
        visits: &'a Cell<usize>,
        fail: bool,
    }

    impl Serialize for RecordAttempt<'_> {
        fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
            self.visits.set(self.visits.get() + 1);
            let mut record = serializer.serialize_struct("RecordAttempt", 2)?;
            record.serialize_field("prefix", "already written")?;
            if self.fail {
                return Err(serde::ser::Error::custom("record serialization failed"));
            }
            record.serialize_field("complete", &true)?;
            record.end()
        }
    }

    #[trace("TC-018", "FR-020-AC-17")]
    #[test]
    fn compact_result_records_keep_field_order_escaping_and_one_newline() {
        #[derive(Serialize)]
        struct OrderedRecord<'a> {
            zeta: &'a str,
            alpha: i64,
        }
        let records = [
            OrderedRecord {
                zeta: "one\ntwo",
                alpha: -7,
            },
            OrderedRecord {
                zeta: "\"quoted\"",
                alpha: 0,
            },
        ];
        assert_eq!(
            result_bytes(&records).unwrap(),
            b"{\"zeta\":\"one\\ntwo\",\"alpha\":-7}\n{\"zeta\":\"\\\"quoted\\\"\",\"alpha\":0}\n"
        );
        assert!(result_bytes::<OrderedRecord<'_>>(&[]).unwrap().is_empty());
    }

    #[trace("TC-018", "FR-020-AC-17")]
    #[test]
    fn result_serialization_failure_discards_prior_records_and_partial_prefix() {
        let visits = Cell::new(0);
        let failing = RecordAttempt {
            visits: &visits,
            fail: true,
        };
        let mut partial = Vec::new();
        assert!(ix_cli_kit::json::write_compact(&mut partial, &failing).is_err());
        assert_eq!(partial, b"{\"prefix\":\"already written\"");
        visits.set(0);
        let records = [
            RecordAttempt {
                visits: &visits,
                fail: false,
            },
            failing,
            RecordAttempt {
                visits: &visits,
                fail: false,
            },
        ];
        assert_eq!(
            result_bytes(&records).unwrap_err(),
            RunnerError::new(
                RunnerErrorCode::ResourceExhausted,
                "results",
                "result buffer unavailable"
            )
        );
        assert_eq!(visits.get(), 2, "serialization stops at the failed record");
    }

    #[trace("TC-018", "FR-020-AC-17")]
    #[test]
    fn diagnostic_serialization_failure_replaces_the_partial_record() {
        let visits = Cell::new(0);
        let failing = RecordAttempt {
            visits: &visits,
            fail: true,
        };
        assert_eq!(error_bytes(&failing), b"{\"protocol\":\"quire.contract.conformance-jsonl/v1\",\"code\":\"resource_exhausted\",\"path\":\"error\",\"detail\":\"error buffer unavailable\"}\n");
        assert_eq!(visits.get(), 1);
        let actual = RunnerError::new(
            RunnerErrorCode::InvalidInvocation,
            "arguments",
            "bad argument",
        );
        assert_eq!(error_bytes(&actual), b"{\"protocol\":\"quire.contract.conformance-jsonl/v1\",\"code\":\"invalid_invocation\",\"path\":\"arguments\",\"detail\":\"bad argument\"}\n");
    }
}
