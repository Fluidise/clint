use fluidise_clint::{CheckResult, Diagnostic, Source};
use lexopt::Arg;
use std::{collections::HashSet, ffi::OsString, path::PathBuf};

pub enum Configuration {
    Defaults,
    File(PathBuf),
}

pub enum Action {
    Help,
    Version,
    Check {
        file: Option<PathBuf>,
        config: Configuration,
        human: bool,
    },
}
fn invalid() -> CheckResult {
    CheckResult::error(Diagnostic::new(
        "cli.arguments.invalid",
        "Invalid, repeated or unknown arguments.",
        Source::Arguments,
    ))
}
pub fn parse(args: Vec<OsString>) -> Result<Action, CheckResult> {
    if args.len() == 1 && args[0] == "--help" {
        return Ok(Action::Help);
    }
    if args.len() == 1 && args[0] == "--version" {
        return Ok(Action::Version);
    }
    // Only exact long options are public. No positional tail after `--`.
    if args.iter().any(|arg| arg == "--") {
        return Err(invalid());
    }
    let mut parser = lexopt::Parser::from_args(args);
    let mut seen = HashSet::new();
    let mut file = None;
    let mut config = None;
    let mut defaults = false;
    let mut stdin = false;
    let mut human = false;
    while let Some(arg) = parser.next().map_err(|_| invalid())? {
        let Arg::Long(name) = arg else {
            return Err(invalid());
        };
        let name = name.to_owned();
        if !seen.insert(name.clone()) {
            return Err(invalid());
        }
        match name.as_str() {
            "file" | "config" => {
                let joined = parser.optional_value();
                let attached = joined.is_some();
                let value = match joined {
                    Some(v) => v,
                    None => parser.value().map_err(|_| invalid())?,
                };
                let bytes = value.as_encoded_bytes();
                if bytes.is_empty() || (!attached && bytes.starts_with(b"-") && bytes != b"-") {
                    return Err(invalid());
                }
                if name == "file" {
                    file = Some(PathBuf::from(value));
                } else {
                    config = Some(PathBuf::from(value));
                }
            }
            "stdin" | "human" | "defaults" => {
                if parser.optional_value().is_some() {
                    return Err(invalid());
                }
                if name == "stdin" {
                    stdin = true;
                } else if name == "human" {
                    human = true;
                } else {
                    defaults = true;
                }
            }
            _ => return Err(invalid()),
        }
    }
    if stdin && file.is_some() {
        return Err(CheckResult::error(Diagnostic::new(
            "cli.source.conflict",
            "Select exactly one message source.",
            Source::Arguments,
        )));
    }
    if !stdin && file.is_none() {
        return Err(CheckResult::error(Diagnostic::new(
            "cli.source.required",
            "Select --file or --stdin.",
            Source::Arguments,
        )));
    }
    let config = match (config, defaults) {
        (Some(path), false) => Configuration::File(path),
        (None, true) => Configuration::Defaults,
        (None, false) => {
            return Err(CheckResult::error(Diagnostic::new(
                "cli.configuration.required",
                "Select --config or --defaults.",
                Source::Arguments,
            )));
        }
        (Some(_), true) => {
            return Err(CheckResult::error(Diagnostic::new(
                "cli.configuration.conflict",
                "Select exactly one configuration source.",
                Source::Arguments,
            )));
        }
    };
    Ok(Action::Check {
        file,
        config,
        human,
    })
}

#[cfg(all(test, unix))]
mod tests {
    use super::*;
    #[test]
    fn native_path_bytes_are_not_normalized() {
        use std::os::unix::ffi::{OsStrExt, OsStringExt};
        let name = OsString::from_vec(b"name-\xff".to_vec());
        for attached in [false, true] {
            let mut args = if attached {
                let mut arg = OsString::from("--file=");
                arg.push(&name);
                vec![arg]
            } else {
                vec!["--file".into(), name.clone()]
            };
            args.push("--defaults".into());
            let Action::Check {
                file: Some(path), ..
            } = parse(args).unwrap_or_else(|r| panic!("{r:?}"))
            else {
                panic!("wrong action")
            };
            assert_eq!(path.as_os_str().as_bytes(), name.as_bytes());
        }
    }
}

#[cfg(test)]
mod policy_tests {
    use super::*;

    fn arguments(args: &[&str]) -> Vec<OsString> {
        args.iter().map(OsString::from).collect()
    }

    #[test]
    fn policy_selection_is_explicit_on_every_platform() {
        assert!(matches!(
            parse(arguments(&["--stdin", "--defaults"])),
            Ok(Action::Check {
                config: Configuration::Defaults,
                ..
            })
        ));
        assert!(matches!(
            parse(arguments(&["--stdin", "--config=rules.toml"])),
            Ok(Action::Check { config: Configuration::File(path), .. }) if path == std::path::Path::new("rules.toml")
        ));
        for (args, code) in [
            (vec!["--stdin"], "cli.configuration.required"),
            (
                vec!["--stdin", "--config=x", "--defaults"],
                "cli.configuration.conflict",
            ),
            (
                vec!["--stdin", "--defaults", "--defaults"],
                "cli.arguments.invalid",
            ),
            (vec!["--stdin", "--defaults=true"], "cli.arguments.invalid"),
            (vec!["--stdin", "--defaults="], "cli.arguments.invalid"),
        ] {
            let Err(result) = parse(arguments(&args)) else {
                panic!("accepted invalid arguments: {args:?}");
            };
            assert_eq!(result.diagnostics[0].code, code);
            assert_eq!(result.status.exit_code(), 2);
        }
    }
}
