mod cli;
mod input;
mod output;
use fluidise_clint::{CheckResult, check_message, prepare_policy};
use std::process::ExitCode;
const HELP: &str = "fluidise-clint\n\nUsage: fluidise-clint (--file <path> | --stdin) (--config <path> | --defaults) [--human]\n\nJSON is the default. Exactly one message source and one configuration source are required.\n--file <path>    Read a regular file; '-' is a filename, not stdin.\n--stdin          Read UTF-8 until EOF.\n--config <path>  Use this configuration file; no discovery.\n--defaults       Use built-in rules without a configuration file.\n--human          Human output after valid argument parsing.\n--help           Print this text; must be used alone.\n--version        Print the version; must be used alone.\n\nExit codes: 0 valid/ignored, 1 invalid message, 2 operational error.\nMessage/config: 1 MiB each. No Git message cleanup or file mutation.\n";
fn run(
    file: Option<std::path::PathBuf>,
    config: cli::Configuration,
) -> Result<CheckResult, CheckResult> {
    let config = match config {
        cli::Configuration::Defaults => None,
        cli::Configuration::File(path) => Some(input::configuration(&path)?),
    };
    let policy = prepare_policy(config.as_deref())?;
    let text = (match file {
        Some(path) => input::file(&path, false),
        None => input::bounded(std::io::stdin().lock(), false),
    })
    .and_then(|b| input::utf8(b, false));
    let text = text.map_err(|mut r| {
        r.configuration = Some(policy.configuration().clone());
        r
    })?;
    Ok(check_message(&text, &policy))
}
fn main() -> ExitCode {
    let code = match cli::parse(std::env::args_os().skip(1).collect()) {
        Err(r) => output::emit(&r, false),
        Ok(cli::Action::Help) => output::text(HELP),
        Ok(cli::Action::Version) => {
            output::text(concat!("fluidise-clint ", env!("CARGO_PKG_VERSION"), "\n"))
        }
        Ok(cli::Action::Check {
            file,
            config,
            human,
        }) => output::emit(&run(file, config).unwrap_or_else(|r| r), human),
    };
    ExitCode::from(code)
}
