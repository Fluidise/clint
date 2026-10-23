use fluidise_clint::{CheckResult, Diagnostic, INPUT_LIMIT, Source};
#[cfg(unix)]
use std::os::unix::fs::OpenOptionsExt;
use std::{fs::OpenOptions, io::Read, path::Path};

fn error(code: &'static str, source: Source, message: &str) -> CheckResult {
    CheckResult::error(Diagnostic::new(code, message, source))
}
pub fn bounded(reader: impl Read, config: bool) -> Result<Vec<u8>, CheckResult> {
    bounded_with_capacity(reader, config, 0)
}
fn bounded_with_capacity(
    mut reader: impl Read,
    config: bool,
    capacity: usize,
) -> Result<Vec<u8>, CheckResult> {
    let (source, read, limit) = if config {
        (
            Source::Configuration,
            "config.read_failed",
            "config.limit_exceeded",
        )
    } else {
        (Source::Message, "input.read_failed", "input.limit_exceeded")
    };
    let mut bytes = Vec::with_capacity(capacity);
    // Read no more than the limit plus one probe byte, even for growing streams.
    (&mut reader)
        .take((INPUT_LIMIT + 1) as u64)
        .read_to_end(&mut bytes)
        .map_err(|_| error(read, source, "Unable to read input."))?;
    if bytes.len() > INPUT_LIMIT {
        return Err(error(limit, source, "Input exceeds the byte limit."));
    }
    Ok(bytes)
}
pub fn file(path: &Path, config: bool) -> Result<Vec<u8>, CheckResult> {
    let (code, source) = if config {
        ("config.read_failed", Source::Configuration)
    } else {
        ("input.read_failed", Source::Message)
    };
    let fail = || error(code, source, "Expected a readable regular file.");
    let mut options = OpenOptions::new();
    options.read(true);
    // Native flags prevent a raced replacement with a FIFO from blocking open.
    // Darwin sys/fcntl.h; Linux asm-generic/fcntl.h (x86_64/aarch64).
    #[cfg(target_os = "macos")]
    options.custom_flags(0x00000004);
    #[cfg(target_os = "linux")]
    options.custom_flags(0o4000);
    let file = options.open(path).map_err(|_| fail())?;
    let meta = file.metadata().map_err(|_| fail())?;
    if !meta.is_file() {
        return Err(fail());
    }
    let capacity = ((meta.len().saturating_add(1)) as usize).min(INPUT_LIMIT + 1);
    bounded_with_capacity(file, config, capacity)
}
pub fn utf8(bytes: Vec<u8>, config: bool) -> Result<String, CheckResult> {
    String::from_utf8(bytes).map_err(|_| {
        if config {
            error(
                "config.syntax.invalid",
                Source::Configuration,
                "Configuration must be UTF-8 TOML.",
            )
        } else {
            error(
                "input.encoding.invalid",
                Source::Message,
                "Message must be UTF-8.",
            )
        }
    })
}
pub fn configuration(path: &Path) -> Result<String, CheckResult> {
    file(path, true).and_then(|bytes| utf8(bytes, true))
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io;
    struct Counting {
        read: usize,
        total: usize,
    }
    impl Read for Counting {
        fn read(&mut self, b: &mut [u8]) -> io::Result<usize> {
            let n = b.len().min(self.total - self.read);
            b[..n].fill(b'x');
            self.read += n;
            Ok(n)
        }
    }
    #[test]
    fn actual_reads_stop_after_one_probe_byte() {
        let mut r = Counting {
            read: 0,
            total: INPUT_LIMIT * 4,
        };
        assert!(bounded(&mut r, false).is_err());
        assert_eq!(r.read, INPUT_LIMIT + 1);
        let mut r = Counting {
            read: 0,
            total: INPUT_LIMIT,
        };
        assert_eq!(bounded(&mut r, true).unwrap().len(), INPUT_LIMIT);
        assert_eq!(r.read, INPUT_LIMIT);
    }
}
