use fluidise_clint::{CheckResult, Diagnostic, INPUT_LIMIT, Source};
#[cfg(unix)]
use std::{
    ffi::OsString,
    os::unix::{ffi::OsStringExt, fs::OpenOptionsExt},
};
use std::{
    fs::{self, OpenOptions},
    io::{self, Read},
    path::{Path, PathBuf},
    process::{Command, Stdio},
};

fn error(code: &'static str, source: Source, message: &str) -> CheckResult {
    CheckResult::error(Diagnostic::new(code, message, source))
}
pub fn bounded(mut reader: impl Read, config: bool) -> Result<Vec<u8>, CheckResult> {
    let (source, read, limit) = if config {
        (
            Source::Configuration,
            "config.read_failed",
            "config.limit_exceeded",
        )
    } else {
        (Source::Message, "input.read_failed", "input.limit_exceeded")
    };
    let mut bytes = Vec::new();
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
    if !fs::metadata(path).map_err(|_| fail())?.is_file() {
        return Err(fail());
    }
    let mut options = OpenOptions::new();
    options.read(true);
    // Native flags prevent a raced replacement with a FIFO from blocking open.
    // Darwin sys/fcntl.h; Linux asm-generic/fcntl.h (x86_64/aarch64).
    #[cfg(target_os = "macos")]
    options.custom_flags(0x00000004);
    #[cfg(target_os = "linux")]
    options.custom_flags(0o4000);
    let file = options.open(path).map_err(|_| fail())?;
    if !file.metadata().map_err(|_| fail())?.is_file() {
        return Err(fail());
    }
    bounded(file, config)
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
fn git(cwd: &Path, arg: &str) -> io::Result<std::process::Output> {
    let mut cmd = Command::new("git");
    // Ignore caller repository routing and personal config; never run hooks.
    for (key, _) in std::env::vars_os() {
        if key.as_encoded_bytes().starts_with(b"GIT_") {
            cmd.env_remove(key);
        }
    }
    cmd.env("GIT_CONFIG_NOSYSTEM", "1")
        .env("GIT_CONFIG_GLOBAL", "/dev/null")
        .env("GIT_OPTIONAL_LOCKS", "0")
        .args([
            "-c",
            "core.fsmonitor=false",
            "-c",
            "core.hooksPath=/dev/null",
            "-c",
            "core.quotePath=false",
            "rev-parse",
            arg,
        ])
        .current_dir(cwd)
        .stdin(Stdio::null())
        .stderr(Stdio::null())
        .output()
}
pub fn configuration(explicit: Option<PathBuf>) -> Result<Option<String>, CheckResult> {
    if let Some(path) = explicit {
        return file(&path, true).and_then(|b| utf8(b, true)).map(Some);
    }
    let cwd = std::env::current_dir().map_err(|_| {
        error(
            "config.read_failed",
            Source::Configuration,
            "Cannot determine working directory.",
        )
    })?;
    // A repository marker distinguishes 'outside Git' from a broken repository.
    let marker = cwd.ancestors().find(|p| {
        p.join(".git").exists()
            || (p.join("HEAD").is_file()
                && p.join("objects").is_dir()
                && p.join("config").is_file())
    });
    if marker.is_none() {
        return Ok(None);
    }
    let fail = || {
        error(
            "config.read_failed",
            Source::Configuration,
            "Cannot discover Git configuration root.",
        )
    };
    let output = git(&cwd, "--show-toplevel").map_err(|_| fail())?;
    let output = if output.status.success() {
        output
    } else {
        let bare = git(&cwd, "--is-bare-repository").map_err(|_| fail())?;
        if !bare.status.success() || bare.stdout != b"true\n" {
            return Err(fail());
        }
        let path = git(&cwd, "--absolute-git-dir").map_err(|_| fail())?;
        if !path.status.success() {
            return Err(fail());
        }
        path
    };
    let mut bytes = output.stdout;
    if bytes.last() != Some(&b'\n') {
        return Err(fail());
    }
    bytes.pop();
    #[cfg(unix)]
    let root = PathBuf::from(OsString::from_vec(bytes));
    #[cfg(not(unix))]
    let root = PathBuf::from(String::from_utf8(bytes).map_err(|_| fail())?);
    let path = root.join("fluidise.toml");
    // Broken symlinks and inaccessible files are not an absent optional config.
    match fs::symlink_metadata(&path) {
        Ok(_) => file(&path, true).and_then(|b| utf8(b, true)).map(Some),
        Err(e) if e.kind() == io::ErrorKind::NotFound => Ok(None),
        Err(_) => Err(fail()),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
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
