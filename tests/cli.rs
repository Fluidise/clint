#![cfg(unix)]
use serde_json::Value;
use std::{
    ffi::{OsStr, OsString},
    fs,
    io::Write,
    os::unix::{ffi::OsStringExt, fs::PermissionsExt, fs::symlink},
    path::{Path, PathBuf},
    process::{Command, Output, Stdio},
    sync::atomic::{AtomicU64, Ordering},
};
static NEXT: AtomicU64 = AtomicU64::new(0);
struct Temp(PathBuf);
impl Temp {
    fn new() -> Self {
        let p = std::env::temp_dir().join(format!(
            "fluidise-clint-{}-{}-{}",
            std::process::id(),
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_nanos(),
            NEXT.fetch_add(1, Ordering::Relaxed)
        ));
        fs::create_dir(&p).unwrap();
        Self(p)
    }
    fn git(&self) {
        let out = Command::new("git")
            .args([
                "-c",
                "init.defaultBranch=main",
                "init",
                "--quiet",
                "--template=",
            ])
            .arg(&self.0)
            .env("GIT_CONFIG_NOSYSTEM", "1")
            .env("GIT_CONFIG_GLOBAL", "/dev/null")
            .env("GIT_CONFIG_COUNT", "0")
            .output()
            .unwrap();
        assert!(out.status.success());
    }
}
impl Drop for Temp {
    fn drop(&mut self) {
        fs::remove_dir_all(&self.0).unwrap();
    }
}
fn command(cwd: &Path) -> Command {
    let mut cmd = Command::new(env!("CARGO_BIN_EXE_fluidise-clint"));
    cmd.current_dir(cwd)
        .stdin(Stdio::null())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped());
    cmd
}
fn invoke(cwd: &Path, args: &[&str], input: Option<&[u8]>) -> Output {
    let mut cmd = command(cwd);
    cmd.args(args);
    if let Some(bytes) = input {
        let mut child = cmd.stdin(Stdio::piped()).spawn().unwrap();
        if let Some(mut stdin) = child.stdin.take() {
            match stdin.write_all(bytes) {
                Ok(()) => {}
                Err(e) if e.kind() == std::io::ErrorKind::BrokenPipe => {}
                Err(e) => panic!("failed to write to child stdin: {e}"),
            }
        }
        child.wait_with_output().unwrap()
    } else {
        cmd.output().unwrap()
    }
}
fn json(out: &Output, code: i32) -> Value {
    assert_eq!(out.status.code(), Some(code), "{:?}", out);
    assert!(out.stderr.is_empty(), "{:?}", out.stderr);
    assert!(out.stdout.ends_with(b"\n"));
    serde_json::from_slice(&out.stdout).unwrap()
}
#[test]
fn default_json_and_codes() {
    let t = Temp::new();
    for (message, status, code) in [
        ("feat: a", "valid", 0),
        ("wrong", "invalid", 1),
        ("Merge branch 'topic'", "ignored", 0),
    ] {
        let out = invoke(&t.0, &["--stdin", "--defaults"], Some(message.as_bytes()));
        let v = json(&out, code);
        assert_eq!(v["status"], status);
        assert_eq!(v["schema_version"], 1);
    }
    let out = invoke(&t.0, &["--stdin", "--defaults"], Some(&[0xff]));
    assert_eq!(
        json(&out, 2)["diagnostics"][0]["code"],
        "input.encoding.invalid"
    );
}
#[test]
fn arguments_never_read_or_wait_and_errors_always_json() {
    let t = Temp::new();
    let cases: Vec<Vec<&str>> = vec![
        vec![],
        vec!["--human"],
        vec!["--stdin"],
        vec!["--stdin", "--human"],
        vec!["--file", "missing"],
        vec!["--stdin", "--defaults", "--defaults"],
        vec!["--stdin", "--defaults=true"],
        vec!["--stdin", "--defaults="],
        vec!["--stdin", "--defaults", "value"],
        vec!["--stdin", "--config", "missing", "--defaults", "--human"],
        vec!["--stdin", "--defaults", "--config", "missing"],
        vec!["--human", "--unknown"],
        vec!["--unknown", "--human"],
        vec!["--stdin", "--stdin"],
        vec!["--stdin", "--file", "missing"],
        vec!["--file", "--human"],
        vec!["--config", "--stdin"],
        vec!["--file="],
        vec!["--config="],
        vec!["--stdin=true"],
        vec!["--stdin", "--human=true"],
        vec!["--stdin", "--json"],
        vec!["--help", "--version"],
        vec!["--help", "--human"],
        vec!["--help", "--file", "missing"],
        vec!["--stdin", "--"],
        vec!["-h"],
        vec!["--stdin", "positional"],
        vec!["--file", "a", "--file", "b"],
    ];
    for args in cases {
        let mut child = command(&t.0)
            .args(&args)
            .stdin(Stdio::piped())
            .spawn()
            .unwrap();
        // Keep stdin OPEN: a rejected invocation must not attempt to read it.
        let start = std::time::Instant::now();
        loop {
            if child.try_wait().unwrap().is_some() {
                break;
            }
            if start.elapsed() > std::time::Duration::from_secs(5) {
                child.kill().unwrap();
                panic!("argument error blocked on stdin: {args:?}");
            }
            std::thread::sleep(std::time::Duration::from_millis(5));
        }
        let out = child.wait_with_output().unwrap();
        let v = json(&out, 2);
        assert_eq!(v["status"], "error", "{args:?}");
        assert_eq!(v["diagnostics"][0]["source"], "arguments");
    }
}
#[test]
fn help_version_and_human_mode() {
    let t = Temp::new();
    for flag in ["--help", "--version"] {
        let out = invoke(&t.0, &[flag], None);
        assert!(out.status.success());
        assert!(out.stderr.is_empty());
        assert!(out.stdout.starts_with(b"fluidise-clint"));
    }
    let out = invoke(&t.0, &["--stdin", "--defaults", "--human"], Some(b"bad"));
    assert_eq!(out.status.code(), Some(1));
    assert_eq!(out.stdout, b"invalid\n");
    assert!(out.stderr.starts_with(b"commit.header.invalid"));
    assert!(!out.stderr.contains(&0x1b));
    let out = invoke(&t.0, &["--human", "--file", "missing", "--defaults"], None);
    assert_eq!(out.status.code(), Some(2));
    assert_eq!(out.stdout, b"error\n");
    assert!(out.stderr.starts_with(b"input.read_failed"));
    assert!(!String::from_utf8_lossy(&out.stderr).contains("missing"));
}
#[test]
fn paths_equal_dash_symlink_and_non_utf8() {
    let t = Temp::new();
    for name in ["-", "--human", "message"] {
        fs::write(t.0.join(name), "fix: a").unwrap();
    }
    symlink("message", t.0.join("link")).unwrap();
    for args in [
        vec!["--file", "-", "--defaults"],
        vec!["--file=--human", "--defaults"],
        vec!["--file", "./--human", "--defaults"],
        vec!["--file", "link", "--defaults"],
    ] {
        json(&invoke(&t.0, &args, None), 0);
    }
    let name = OsString::from_vec(b"msg-\xff".to_vec());
    if let Err(error) = fs::write(t.0.join(&name), "fix: a") {
        if cfg!(target_os = "macos") && error.raw_os_error() == Some(92) {
            // This filesystem refuses non-UTF-8 filenames; argument parsing must
            // still preserve the bytes and report a filesystem, not CLI, error.
            let out = command(&t.0)
                .arg("--defaults")
                .arg("--file")
                .arg(&name)
                .output()
                .unwrap();
            assert_eq!(json(&out, 2)["diagnostics"][0]["code"], "input.read_failed");
            eprintln!(
                "Filesystem rejected non-UTF-8 names; end-to-end filesystem proof requires Linux."
            );
            return;
        }
        panic!("cannot prepare native filename: {error}");
    }
    let out = command(&t.0)
        .arg("--defaults")
        .arg("--file")
        .arg(&name)
        .output()
        .unwrap();
    json(&out, 0);
    let mut joined = OsString::from("--file=");
    joined.push(&name);
    json(
        &command(&t.0)
            .arg("--defaults")
            .arg(joined)
            .output()
            .unwrap(),
        0,
    );
    let config = OsString::from_vec(b"config-\xfe".to_vec());
    fs::write(t.0.join(&config), "[commit]\nversion=1").unwrap();
    json(
        &command(&t.0)
            .args([
                OsStr::new("--file"),
                OsStr::new("message"),
                OsStr::new("--config"),
                &config,
            ])
            .output()
            .unwrap(),
        0,
    );
}
#[test]
fn regular_files_only_and_no_fifo_open_hang() {
    let t = Temp::new();
    fs::create_dir(t.0.join("dir")).unwrap();
    let fifo = t.0.join("pipe");
    assert!(
        Command::new("mkfifo")
            .arg(&fifo)
            .status()
            .unwrap()
            .success()
    );
    let _socket = std::os::unix::net::UnixListener::bind(t.0.join("socket")).unwrap();
    for path in ["dir", "pipe", "socket", "/dev/null"] {
        let mut child = command(&t.0)
            .args(["--file", path, "--defaults"])
            .spawn()
            .unwrap();
        let start = std::time::Instant::now();
        loop {
            if child.try_wait().unwrap().is_some() {
                break;
            }
            if start.elapsed() > std::time::Duration::from_secs(5) {
                child.kill().unwrap();
                panic!("special file blocks: {path}");
            }
            std::thread::sleep(std::time::Duration::from_millis(5));
        }
        assert_eq!(
            json(&child.wait_with_output().unwrap(), 2)["diagnostics"][0]["code"],
            "input.read_failed"
        );
    }
    fs::write(t.0.join("message"), "fix: a").unwrap();
    json(
        &invoke(&t.0, &["--file", "message", "--config", "pipe"], None),
        2,
    );
}
#[test]
fn explicit_configuration_relative_paths_no_mutation_and_foreign_sections() {
    let t = Temp::new();
    t.git();
    fs::create_dir(t.0.join("sub")).unwrap();
    fs::create_dir(t.0.join("sub/deep")).unwrap();
    let config = "[commit]\nversion=1\nallowed-types=['deps']\n[hooks]\nversion='future'\n";
    fs::write(t.0.join("fluidise.toml"), config).unwrap();
    let message = "deps: a";
    fs::write(t.0.join("sub/deep/msg"), message).unwrap();
    let out = invoke(
        &t.0.join("sub/deep"),
        &["--file", "msg", "--config", "../../fluidise.toml"],
        None,
    );
    let v = json(&out, 0);
    assert_eq!(
        v["configuration"]["used_sections"],
        serde_json::json!(["commit"])
    );
    assert_eq!(
        v["configuration"]["unrecognized_sections"],
        serde_json::json!(["hooks"])
    );
    fs::write(t.0.join("sub/deep/local.toml"), "[commit]\nversion=1").unwrap();
    json(
        &invoke(
            &t.0.join("sub/deep"),
            &["--file", "msg", "--config=local.toml"],
            None,
        ),
        1,
    );
    json(
        &invoke(
            &t.0.join("sub/deep"),
            &["--file", "msg", "--config", "absent"],
            None,
        ),
        2,
    );
    assert_eq!(
        fs::read_to_string(t.0.join("fluidise.toml")).unwrap(),
        config
    );
    assert_eq!(
        fs::read_to_string(t.0.join("sub/deep/msg")).unwrap(),
        message
    );
    assert!(!t.0.join(".git/index").exists());
}
#[test]
fn defaults_do_not_search_parent_configuration() {
    let t = Temp::new();
    fs::write(t.0.join("fluidise.toml"), "not toml").unwrap();
    fs::create_dir(t.0.join("sub")).unwrap();
    let v = json(
        &invoke(
            &t.0.join("sub"),
            &["--stdin", "--defaults"],
            Some(b"fix: a"),
        ),
        0,
    );
    assert_eq!(v["configuration"]["source"], "defaults");
}
#[test]
fn invalid_config_prevents_ignore_and_message_reads() {
    let t = Temp::new();
    fs::write(
        t.0.join("bad.toml"),
        "[commit]\nversion=1\nallowed-types=[]",
    )
    .unwrap();
    let out = invoke(
        &t.0,
        &["--stdin", "--config", "bad.toml"],
        Some(b"Merge branch 'topic'"),
    );
    let v = json(&out, 2);
    assert!(v.get("reason").is_none());
    assert_eq!(v["diagnostics"][0]["source"], "configuration");
}
#[test]
fn input_and_config_read_caps() {
    let t = Temp::new();
    let mut b = b"fix: ".to_vec();
    b.resize(fluidise_clint::INPUT_LIMIT, b'a');
    json(&invoke(&t.0, &["--stdin", "--defaults"], Some(&b)), 0);
    b.push(b'a');
    assert_eq!(
        json(&invoke(&t.0, &["--stdin", "--defaults"], Some(&b)), 2)["diagnostics"][0]["code"],
        "input.limit_exceeded"
    );
    fs::write(
        t.0.join("big.toml"),
        " ".repeat(fluidise_clint::INPUT_LIMIT + 1),
    )
    .unwrap();
    let out = invoke(&t.0, &["--stdin", "--config", "big.toml"], None);
    assert_eq!(
        json(&out, 2)["diagnostics"][0]["code"],
        "config.limit_exceeded"
    );
}
#[test]
fn metadata_expansion_uses_small_json_fallback() {
    let t = Temp::new();
    let key = "x".repeat(300000);
    fs::write(t.0.join("large.toml"), format!("['{key}']\nx=1")).unwrap();
    let v = json(
        &invoke(
            &t.0,
            &["--stdin", "--config", "large.toml"],
            Some(b"fix: a"),
        ),
        2,
    );
    assert_eq!(v["diagnostics"][0]["code"], "tool.output.limit_exceeded");
    assert!(v.get("configuration").is_none());
}
#[test]
fn broken_stdout_is_failure_without_fallback_rewrite() {
    let t = Temp::new();
    // /dev/full provides a deterministic write failure on Linux, if available.
    if Path::new("/dev/full").exists() {
        let out = command(&t.0)
            .arg("--version")
            .stdout(
                fs::OpenOptions::new()
                    .write(true)
                    .open("/dev/full")
                    .unwrap(),
            )
            .output()
            .unwrap();
        assert!(!out.status.success());
        assert!(out.stderr.is_empty());
    } else {
        let mut child = command(&t.0).arg("--version").spawn().unwrap();
        drop(child.stdout.take());
        let out = child.wait_with_output().unwrap();
        assert!(!out.status.success());
        assert!(out.stderr.is_empty());
    }
}
#[test]
fn repository_paths_preserve_native_bytes_and_newlines() {
    let t = Temp::new();
    let name = OsString::from("root-é\nend\n");
    let path = t.0.join(name);
    fs::create_dir(&path).unwrap();
    let repo = Temp(path.clone());
    repo.git();
    std::mem::forget(repo);
    fs::write(
        path.join("fluidise.toml"),
        "[commit]\nversion=1\nallowed-types=['deps']",
    )
    .unwrap();
    let v = json(
        &invoke(
            &path,
            &["--stdin", "--config", "fluidise.toml"],
            Some(b"deps: a"),
        ),
        0,
    );
    assert_eq!(v["configuration"]["source"], "file");
}

#[test]
fn native_termination_does_not_claim_a_final_json() {
    use std::os::unix::process::ExitStatusExt;
    let t = Temp::new();
    let child = command(&t.0)
        .args(["--stdin", "--defaults"])
        .stdin(Stdio::piped())
        .spawn()
        .unwrap();
    assert!(
        Command::new("kill")
            .args(["-TERM", &child.id().to_string()])
            .status()
            .unwrap()
            .success()
    );
    let output = child.wait_with_output().unwrap();
    assert_eq!(output.status.signal(), Some(15));
    assert!(output.stdout.is_empty());
}

#[test]
fn policy_choice_is_required_exclusive_and_never_invokes_git() {
    let t = Temp::new();
    fs::write(t.0.join("fluidise.toml"), "not valid toml").unwrap();
    fs::write(t.0.join(".git"), "broken git metadata").unwrap();
    fs::write(
        t.0.join("selected.toml"),
        "[commit]\nversion=1\nallowed-types=['deps']",
    )
    .unwrap();
    let defaults = command(&t.0)
        .args(["--stdin", "--defaults"])
        .env("PATH", "")
        .env("GIT_DIR", "hostile")
        .output()
        .unwrap();
    assert_eq!(json(&defaults, 1)["configuration"]["source"], "defaults");
    let mut explicit = command(&t.0);
    explicit
        .args(["--file", "message", "--config", "selected.toml"])
        .env("PATH", "")
        .env("GIT_DIR", "hostile");
    fs::write(t.0.join("message"), "deps: a").unwrap();
    assert_eq!(
        json(&explicit.output().unwrap(), 0)["configuration"]["source"],
        "file"
    );
    for (args, code) in [
        (vec!["--stdin"], "cli.configuration.required"),
        (
            vec!["--stdin", "--defaults", "--config", "missing"],
            "cli.configuration.conflict",
        ),
        (
            vec!["--stdin", "--config", "missing", "--defaults"],
            "cli.configuration.conflict",
        ),
    ] {
        let out = command(&t.0).args(args).output().unwrap();
        assert_eq!(json(&out, 2)["diagnostics"][0]["code"], code);
    }
}

fn generated_hook(cwd: &Path, message: &OsStr) -> Command {
    // Exercise the actual generated body without executing the installer.
    let (_, body) = include_str!("../scripts/install-hooks.sh")
        .split_once("<< 'EOF'\n")
        .unwrap();
    let (body, _) = body.split_once("\nEOF\n").unwrap();
    let mut command = Command::new("bash");
    command
        .args([
            OsStr::new("-c"),
            OsStr::new(body),
            OsStr::new("commit-msg"),
            message,
        ])
        .current_dir(cwd)
        .stdin(Stdio::null());
    for (key, _) in std::env::vars_os() {
        if key.as_encoded_bytes().starts_with(b"GIT_") {
            command.env_remove(key);
        }
    }
    command
        .env("GIT_CONFIG_NOSYSTEM", "1")
        .env("GIT_CONFIG_GLOBAL", "/dev/null");
    command
}

fn hook_linter(root: &Path, profile: &str) {
    let directory = root.join("target").join(profile);
    fs::create_dir_all(&directory).unwrap();
    symlink(
        env!("CARGO_BIN_EXE_fluidise-clint"),
        directory.join("fluidise-clint"),
    )
    .unwrap();
}

fn hook_script(path: &Path, source: &str) {
    fs::write(path, source).unwrap();
    fs::set_permissions(path, fs::Permissions::from_mode(0o755)).unwrap();
}

#[test]
fn generated_hook_preserves_defaults_and_ignores_root_configuration() {
    let t = Temp::new();
    t.git();
    hook_linter(&t.0, "release");
    let message = OsStr::new("./message 'é\n.txt");
    let config = t.0.join("fluidise.toml");
    for configuration in [
        None,
        Some("[commit]\nversion=1\nallowed-types=['deps']\n"),
        Some("[commit\n"),
    ] {
        if let Some(text) = configuration {
            fs::write(&config, text).unwrap();
        }
        for text in ["feat: fixture\n", "deps: fixture\n", "not conventional\n"] {
            fs::write(t.0.join(message), text).unwrap();
            let direct = command(&t.0)
                .args([
                    OsStr::new("--file"),
                    message,
                    OsStr::new("--defaults"),
                    OsStr::new("--human"),
                ])
                .output()
                .unwrap();
            let out = generated_hook(&t.0, message).output().unwrap();
            assert_eq!(
                out.status.code(),
                Some(if direct.status.success() { 0 } else { 1 })
            );
            assert_eq!(out.stdout, direct.stdout);
            let mut stderr = direct.stderr;
            if !direct.status.success() {
                stderr.extend_from_slice("\n❌ Commit rejected by fluidise-clint.\nPlease update your commit message to adhere to Conventional Commits.\n".as_bytes());
            }
            assert_eq!(out.stderr, stderr);
            assert_eq!(fs::read_to_string(t.0.join(message)).unwrap(), text);
            if let Some(text) = configuration {
                assert_eq!(fs::read_to_string(&config).unwrap(), text);
            }
        }
    }
    fs::remove_file(&config).unwrap();
    let missing = generated_hook(&t.0, OsStr::new("absent")).output().unwrap();
    assert_eq!(missing.status.code(), Some(1));
    assert!(
        String::from_utf8(missing.stderr)
            .unwrap()
            .contains("input.read_failed")
    );
    assert!(!t.0.join(".git/index").exists());
    assert!(!t.0.join(".git/hooks/commit-msg").exists());
}

#[test]
fn generated_hook_preserves_trailing_newline_root() {
    let t = Temp::new();
    let root = Temp(t.0.join("root 'é\nend\n"));
    fs::create_dir(&root.0).unwrap();
    root.git();
    hook_linter(&root.0, "release");
    fs::write(root.0.join("message"), "feat: fixture\n").unwrap();
    fs::write(
        root.0.join("fluidise.toml"),
        "[commit]\nversion=1\nallowed-types=['deps']\n",
    )
    .unwrap();
    let out = generated_hook(&root.0, OsStr::new("message"))
        .output()
        .unwrap();
    assert_eq!(out.status.code(), Some(0), "{out:?}");
    assert_eq!(out.stdout, b"valid\n");
    assert!(out.stderr.is_empty());
    assert!(!root.0.join(".git/hooks/commit-msg").exists());
}

#[test]
fn generated_hook_selects_binary_without_git_and_preserves_build_fallback() {
    let t = Temp::new();
    t.git();
    let release = t.0.join("target/release");
    let debug = t.0.join("target/debug");
    let bin = t.0.join("bin");
    for path in [&release, &debug, &bin] {
        fs::create_dir_all(path).unwrap();
    }
    hook_script(
        &bin.join("git"),
        "#!/bin/sh\necho unexpected-git >&2\nexit 99\n",
    );
    hook_script(
        &release.join("fluidise-clint"),
        "#!/bin/sh\nprintf 'release\\n'\n",
    );
    hook_script(
        &debug.join("fluidise-clint"),
        "#!/bin/sh\nprintf 'debug\\n'\n",
    );
    hook_script(
        &bin.join("cargo"),
        "#!/bin/sh\nprintf '%s\\n' \"$PWD\" \"$@\" > \"$HOOK_TEST_LOG\"\ncp \"$HOOK_TEST_LINTER\" target/debug/fluidise-clint\n",
    );
    let built = t.0.join("built-linter");
    hook_script(&built, "#!/bin/sh\nprintf 'built\\n'\n");
    let log = t.0.join("cargo.log");
    let path = std::env::join_paths(
        std::iter::once(bin).chain(std::env::split_paths(&std::env::var_os("PATH").unwrap())),
    )
    .unwrap();
    let invoke = || {
        generated_hook(&t.0, OsStr::new("message"))
            .env("PATH", &path)
            .env("HOOK_TEST_LOG", &log)
            .env("HOOK_TEST_LINTER", &built)
            .output()
            .unwrap()
    };
    for (expected, remove) in [("release\n", &release), ("debug\n", &debug)] {
        let out = invoke();
        assert_eq!(out.status.code(), Some(0), "{out:?}");
        assert_eq!(out.stdout, expected.as_bytes());
        assert!(out.stderr.is_empty());
        assert!(!log.exists());
        fs::remove_file(remove.join("fluidise-clint")).unwrap();
    }
    let out = invoke();
    assert_eq!(out.status.code(), Some(0), "{out:?}");
    assert_eq!(out.stdout, b"built\n");
    assert!(out.stderr.is_empty());
    assert_eq!(
        fs::read_to_string(&log).unwrap(),
        format!(
            "{}\nbuild\n--quiet\n--bin\nfluidise-clint\n",
            fs::canonicalize(&t.0).unwrap().display()
        )
    );
    fs::remove_file(debug.join("fluidise-clint")).unwrap();
    hook_script(&t.0.join("bin/cargo"), "#!/bin/sh\nexit 91\n");
    let out = invoke();
    assert_eq!(out.status.code(), Some(91));
    assert!(out.stdout.is_empty());
    assert!(out.stderr.is_empty());
    assert!(!t.0.join(".git/hooks/commit-msg").exists());
}
