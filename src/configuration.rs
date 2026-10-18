use crate::*;
use std::collections::HashSet;
use toml::de::{DeTable, DeValue};

/// Prepare rules from TOML in memory, or defaults. Never reads files.
pub fn prepare_policy(input: Option<&str>) -> Result<Policy, CheckResult> {
    let mut p = Policy {
        allowed_types: [
            "feat", "fix", "docs", "style", "refactor", "perf", "test", "build", "ci", "chore",
            "revert",
        ]
        .map(String::from)
        .to_vec(),
        allowed_scopes: None,
        max_title_length: None,
        ignore_merges: true,
        ignore_reverts: true,
        summary: ConfigurationSummary {
            source: if input.is_some() { "file" } else { "defaults" },
            used_sections: vec![],
            unrecognized_sections: vec![],
        },
    };
    let Some(input) = input else {
        return Ok(p);
    };
    if input.len() > INPUT_LIMIT {
        return Err(CheckResult::error(Diagnostic::new(
            "config.limit_exceeded",
            "Configuration exceeds the byte limit.",
            Source::Configuration,
        )));
    }
    let table = DeTable::parse(input).map_err(|e| {
        let mut d = Diagnostic::new(
            "config.syntax.invalid",
            "Invalid TOML syntax.",
            Source::Configuration,
        );
        if let Some(s) = e.span() {
            d = d.at(s.start, s.end);
        }
        CheckResult::error(d)
    })?;
    let table = table.get_ref();
    for key in table.keys() {
        if key.get_ref().as_ref() == "commit" {
            p.summary.used_sections.push("commit".into());
        } else {
            p.summary
                .unrecognized_sections
                .push(key.get_ref().to_string());
        }
    }
    let Some(commit) = table.get("commit") else {
        return Ok(p);
    };
    let Some(section) = commit.get_ref().as_table() else {
        let range = commit.span();
        let mut result = CheckResult::error(
            Diagnostic::new(
                "config.value.invalid",
                "commit must be a table.",
                Source::Configuration,
            )
            .field("", None)
            .at(range.start, range.end),
        );
        result.configuration = Some(p.summary);
        return Err(result);
    };
    let mut ds = Diagnostics::new();
    // Version is a prerequisite: never interpret a different schema.
    let version_error = match section.get("version") {
        None => Some(
            Diagnostic::new(
                "config.version.missing",
                "commit.version is required.",
                Source::Configuration,
            )
            .field("version", None),
        ),
        Some(v) => {
            let code = match v.get_ref() {
                DeValue::Integer(i) => {
                    if i64::from_str_radix(i.as_str(), i.radix()).ok() == Some(1) {
                        None
                    } else {
                        Some("config.version.unsupported")
                    }
                }
                _ => Some("config.value.invalid"),
            };
            code.map(|code| {
                let s = v.span();
                Diagnostic::new(
                    code,
                    "commit.version must be the supported integer 1.",
                    Source::Configuration,
                )
                .field("version", None)
                .at(s.start, s.end)
            })
        }
    };
    if let Some(d) = version_error {
        ds.push(d);
        return Err(ds.result(Status::Valid, Status::Error, Some(p.summary)));
    }
    for (key, value) in section.iter() {
        if ds.full() {
            break;
        }
        let name: &str = key.get_ref().as_ref();
        let v = value.get_ref();
        let span = value.span();
        let bad = || {
            Diagnostic::new(
                "config.value.invalid",
                "Invalid value for this configuration field.",
                Source::Configuration,
            )
            .field(name, None)
            .at(span.start, span.end)
        };
        match name {
            "version" => {}
            "allowed-types" | "allowed-scopes" => {
                let Some(array) = v.as_array() else {
                    ds.push(bad());
                    continue;
                };
                if array.is_empty() {
                    ds.push(bad());
                    continue;
                }
                let mut seen = HashSet::new();
                let mut values = Vec::new();
                for (index, value) in array.iter().enumerate() {
                    if ds.full() {
                        break;
                    }
                    let text = value.get_ref().as_str();
                    let valid = text.is_some_and(|s| {
                        if name == "allowed-types" {
                            valid_type(s)
                        } else {
                            valid_scope(s)
                        }
                    });
                    if !valid || !seen.insert(text.unwrap_or_default()) {
                        let s = value.span();
                        ds.push(
                            Diagnostic::new(
                                "config.value.invalid",
                                "Expected a valid, unique nonempty string.",
                                Source::Configuration,
                            )
                            .field(name, Some(index))
                            .at(s.start, s.end),
                        );
                    } else {
                        values.push(text.unwrap().to_owned());
                    }
                }
                if name == "allowed-types" {
                    p.allowed_types = values;
                } else {
                    p.allowed_scopes = Some(values);
                }
            }
            "max-title-length" => {
                let n = v
                    .as_integer()
                    .and_then(|i| {
                        u64::from_str_radix(i.as_str().trim_start_matches('+'), i.radix()).ok()
                    })
                    .filter(|n| *n > 0 && *n <= i64::MAX as u64);
                if let Some(n) = n {
                    p.max_title_length = Some(n);
                } else {
                    ds.push(bad());
                }
            }
            "ignore-merges" | "ignore-reverts" => {
                if let Some(b) = v.as_bool() {
                    if name == "ignore-merges" {
                        p.ignore_merges = b;
                    } else {
                        p.ignore_reverts = b;
                    }
                } else {
                    ds.push(bad());
                }
            }
            _ => {
                let s = key.span();
                ds.push(
                    Diagnostic::new(
                        "config.key.unknown",
                        "Unknown key in commit configuration.",
                        Source::Configuration,
                    )
                    .field(name, None)
                    .at(s.start, s.end),
                );
            }
        }
    }
    if ds.items.is_empty() {
        Ok(p)
    } else {
        Err(ds.result(Status::Valid, Status::Error, Some(p.summary)))
    }
}
