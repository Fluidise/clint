use fluidise_clint::{CheckResult, Diagnostic, Field, OUTPUT_LIMIT, Source, Status};
use serde_json::{Value, json};
use std::io::{self, Write};
fn wire(r: &CheckResult) -> Value {
    let ds: Vec<_> = r
        .diagnostics
        .iter()
        .map(|d| {
            let mut v = json!({"code":d.code,"message":d.message,"source":d.source.as_str()});
            if let Some(s) = &d.span {
                v["span"] = json!({"start":s.start,"end":s.end});
            }
            if let Some(f) = &d.field {
                v["field"] = Value::Array(
                    f.iter()
                        .map(|c| match c {
                            Field::Key(s) => json!(s),
                            Field::Index(i) => json!(i),
                        })
                        .collect(),
                );
            }
            v
        })
        .collect();
    let mut v = json!({"schema_version":1,"status":r.status.as_str(),"diagnostics_complete":r.diagnostics_complete,"diagnostics":ds});
    if let Some(reason) = r.reason {
        v["reason"] = json!(reason);
    }
    if let Some(c) = &r.configuration {
        v["configuration"] = json!({"source":c.source,"used_sections":c.used_sections,"unrecognized_sections":c.unrecognized_sections});
    }
    v
}
struct Buffer {
    bytes: Vec<u8>,
}
impl Write for Buffer {
    fn write(&mut self, b: &[u8]) -> io::Result<usize> {
        if b.len() > OUTPUT_LIMIT - self.bytes.len() {
            return Err(io::Error::other("output byte limit"));
        }
        self.bytes.extend_from_slice(b);
        Ok(b.len())
    }
    fn flush(&mut self) -> io::Result<()> {
        Ok(())
    }
}
fn encode(r: &CheckResult) -> io::Result<Vec<u8>> {
    let mut buffer = Buffer { bytes: Vec::new() };
    serde_json::to_writer(&mut buffer, &wire(r)).map_err(io::Error::other)?;
    buffer.write_all(b"\n")?;
    Ok(buffer.bytes)
}
pub fn json_bytes(r: &CheckResult) -> (Vec<u8>, u8) {
    if let Ok(bytes) = encode(r) {
        return (bytes, r.status.exit_code());
    }
    let fallback = CheckResult::error(Diagnostic::new(
        "tool.output.limit_exceeded",
        "Encoded output exceeds the byte limit.",
        Source::Tool,
    ));
    // This constant-size fallback is independent of all abandoned metadata.
    (
        encode(&fallback).expect("fixed fallback is valid and within the output limit"),
        2,
    )
}
pub fn emit(r: &CheckResult, human: bool) -> u8 {
    if human {
        if writeln!(io::stdout().lock(), "{}", r.status.as_str()).is_err() {
            return 2;
        }
        let mut err = io::stderr().lock();
        for d in &r.diagnostics {
            let result = if let Some(s) = &d.span {
                writeln!(err, "{} [{}..{}): {}", d.code, s.start, s.end, d.message)
            } else {
                writeln!(err, "{}: {}", d.code, d.message)
            };
            if result.is_err() {
                return 2;
            }
        }
        if io::stdout().flush().is_err() || err.flush().is_err() {
            return 2;
        }
        r.status.exit_code()
    } else {
        emit_json_to(r, &mut io::stdout().lock())
    }
}
fn emit_json_to(r: &CheckResult, out: &mut impl Write) -> u8 {
    let (bytes, code) = json_bytes(r);
    if out.write_all(&bytes).and_then(|_| out.flush()).is_err() {
        2
    } else {
        code
    }
}
pub fn text(text: &str) -> u8 {
    let mut out = io::stdout().lock();
    if out
        .write_all(text.as_bytes())
        .and_then(|_| out.flush())
        .is_err()
    {
        Status::Error.exit_code()
    } else {
        0
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn output_limit_replaces_report_before_emission() {
        let mut r =
            CheckResult::error(Diagnostic::new("tool.internal_error", "Test", Source::Tool));
        r.configuration = Some(fluidise_clint::ConfigurationSummary {
            source: "file",
            used_sections: vec![],
            unrecognized_sections: vec!["\u{0001}".repeat(OUTPUT_LIMIT)],
        });
        let (b, c) = json_bytes(&r);
        assert_eq!(c, 2);
        assert!(b.len() < 1024);
        let v: Value = serde_json::from_slice(&b).unwrap();
        assert_eq!(v["diagnostics"][0]["code"], "tool.output.limit_exceeded");
        assert!(v.get("configuration").is_none());
        assert_eq!(v["diagnostics_complete"], true);
    }
    #[test]
    fn buffer_includes_final_newline_in_limit() {
        let mut b = Buffer {
            bytes: vec![b'x'; OUTPUT_LIMIT],
        };
        assert!(b.write_all(b"\n").is_err());
        assert_eq!(b.bytes.len(), OUTPUT_LIMIT);
    }
}

#[cfg(test)]
mod transport_tests {
    use super::*;
    struct Failed {
        calls: usize,
    }
    impl Write for Failed {
        fn write(&mut self, _: &[u8]) -> io::Result<usize> {
            self.calls += 1;
            Err(io::Error::new(io::ErrorKind::BrokenPipe, "closed"))
        }
        fn flush(&mut self) -> io::Result<()> {
            panic!("failed write must not flush")
        }
    }
    #[test]
    fn json_write_failure_never_retries_another_result() {
        let r = CheckResult::error(Diagnostic::new("tool.internal_error", "test", Source::Tool));
        let mut out = Failed { calls: 0 };
        assert_eq!(emit_json_to(&r, &mut out), 2);
        assert_eq!(out.calls, 1);
    }
}
