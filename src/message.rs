use crate::*;

/// Validate the exact UTF-8 text, preserving original byte positions.
pub fn check_message(text: &str, policy: &Policy) -> CheckResult {
    let summary = || Some(policy.summary.clone());
    if text.len() > INPUT_LIMIT {
        let mut r = CheckResult::error(Diagnostic::new(
            "input.limit_exceeded",
            "Message exceeds the byte limit.",
            Source::Message,
        ));
        r.configuration = summary();
        return r;
    }
    if let Some(i) = text.bytes().enumerate().find_map(|(i, b)| {
        (b == 0 || (b == b'\r' && text.as_bytes().get(i + 1) != Some(&b'\n'))).then_some(i)
    }) {
        let mut r = CheckResult::error(
            Diagnostic::new(
                "input.content.unsupported",
                "NUL and bare CR are unsupported.",
                Source::Message,
            )
            .at(i, i + 1),
        );
        r.configuration = summary();
        return r;
    }
    let mut lines: Vec<(&str, usize)> = Vec::new();
    let mut offset = 0;
    for line in text.split('\n') {
        let s = line.strip_suffix('\r').unwrap_or(line);
        lines.push((s, offset));
        offset += line.len() + 1;
    }
    while lines.len() > 1 && lines.last().is_some_and(|(s, _)| s.is_empty()) {
        lines.pop();
    }
    let title = lines[0].0;
    if let Some(reason) = ignored(&lines, policy) {
        return CheckResult {
            status: Status::Ignored,
            reason: Some(reason),
            diagnostics_complete: true,
            diagnostics: vec![],
            configuration: summary(),
        };
    }
    let mut ds = Diagnostics::new();
    if title.starts_with("fixup!") || title.starts_with("squash!") {
        ds.push(
            Diagnostic::new(
                "commit.temporary.not_allowed",
                "Temporary commit messages are not allowed.",
                Source::Message,
            )
            .at(0, title.len()),
        );
    } else {
        let parsed = header(title);
        if let Some((kind, scope)) = parsed {
            if !valid_type(kind) {
                ds.push(
                    Diagnostic::new(
                        "commit.type.case",
                        "Commit types must be lowercase ASCII.",
                        Source::Message,
                    )
                    .at(0, kind.len()),
                );
            } else if !policy.allowed_types.iter().any(|t| t == kind) {
                ds.push(
                    Diagnostic::new(
                        "commit.type.not_allowed",
                        "Commit type is not allowed.",
                        Source::Message,
                    )
                    .at(0, kind.len()),
                );
            }
            if let (Some(scope), Some(allowed)) = (scope, &policy.allowed_scopes) {
                if !allowed.iter().any(|s| s == scope) {
                    ds.push(
                        Diagnostic::new(
                            "commit.scope.not_allowed",
                            "Commit scope is not allowed.",
                            Source::Message,
                        )
                        .at(kind.len() + 1, kind.len() + 1 + scope.len()),
                    );
                }
            }
        } else {
            ds.push(
                Diagnostic::new(
                    "commit.header.invalid",
                    "Expected type, optional scope/!, colon, space and a nonblank description.",
                    Source::Message,
                )
                .at(0, title.len()),
            );
        }
    }
    if policy
        .max_title_length
        .is_some_and(|max| title.chars().count() as u64 > max)
    {
        ds.push(
            Diagnostic::new(
                "commit.title.too_long",
                "Commit title exceeds the configured length.",
                Source::Message,
            )
            .at(0, title.len()),
        );
    }
    if lines.len() > 1 && !lines[1].0.is_empty() {
        let (s, i) = lines[1];
        ds.push(
            Diagnostic::new(
                "commit.body.invalid",
                "Body must be separated from the title by an empty line.",
                Source::Message,
            )
            .at(i, i + s.len()),
        );
    } else {
        let mut in_footers = false;
        let mut current: Option<(usize, usize, bool)> = None;
        for (index, (line, start)) in lines.iter().enumerate().skip(2) {
            if ds.full() {
                break;
            }
            let eligible = in_footers || lines[index - 1].0.is_empty();
            if let Some(value) = eligible.then(|| footer(line)).flatten() {
                if let Some((a, b, nonblank)) = current.take() {
                    if !nonblank {
                        ds.push(
                            Diagnostic::new(
                                "commit.footer.invalid",
                                "Footer value must not be blank.",
                                Source::Message,
                            )
                            .at(a, b),
                        );
                        if ds.full() {
                            break;
                        }
                    }
                }
                in_footers = true;
                current = Some((
                    *start,
                    *start + line.len(),
                    value.chars().any(|c| !c.is_whitespace()),
                ));
            } else if let Some((_, end, nonblank)) = current.as_mut() {
                *end = *start + line.len();
                *nonblank |= line.chars().any(|c| !c.is_whitespace());
            }
        }
        if !ds.full() {
            if let Some((a, b, false)) = current {
                ds.push(
                    Diagnostic::new(
                        "commit.footer.invalid",
                        "Footer value must not be blank.",
                        Source::Message,
                    )
                    .at(a, b),
                );
            }
        }
    }
    ds.result(Status::Valid, Status::Invalid, summary())
}
fn header(title: &str) -> Option<(&str, Option<&str>)> {
    let end = title.find(['(', '!', ':'])?;
    let (kind, mut rest) = title.split_at(end);
    if !kind.bytes().next()?.is_ascii_alphabetic()
        || !kind.bytes().all(|b| b.is_ascii_alphanumeric() || b == b'-')
    {
        return None;
    }
    let scope = if rest.starts_with('(') {
        let close = rest.find(')')?;
        let scope = &rest[1..close];
        if !valid_scope(scope) {
            return None;
        }
        rest = &rest[close + 1..];
        Some(scope)
    } else {
        None
    };
    let rest = rest.strip_prefix('!').unwrap_or(rest);
    let description = rest.strip_prefix(": ")?;
    description
        .chars()
        .any(|c| !c.is_whitespace())
        .then_some((kind, scope))
}
fn footer(line: &str) -> Option<&str> {
    for marker in ["BREAKING CHANGE: ", "BREAKING-CHANGE: "] {
        if let Some(value) = line.strip_prefix(marker) {
            return Some(value);
        }
    }
    let end = line
        .bytes()
        .position(|b| !b.is_ascii_alphanumeric() && b != b'-')?;
    let (token, rest) = line.split_at(end);
    let value = rest
        .strip_prefix(": ")
        .or_else(|| rest.strip_prefix(" #"))?;
    // Reserved rupture tokens only accept the colon separator.
    if token == "BREAKING-CHANGE" || token == "BREAKING CHANGE" {
        return None;
    }
    token
        .split('-')
        .all(|s| !s.is_empty() && s.bytes().all(|b| b.is_ascii_alphanumeric()))
        .then_some(value)
}
fn ignored(lines: &[(&str, usize)], p: &Policy) -> Option<&'static str> {
    let title = lines[0].0;
    let word = |s: &str| !s.is_empty() && !s.chars().any(|c| c.is_whitespace() || c.is_control());
    if p.ignore_merges {
        if let Some(rest) = title.strip_prefix("Merge pull request #") {
            if let Some((number, source)) = rest.split_once(" from ") {
                if !number.starts_with('0')
                    && !number.is_empty()
                    && number.bytes().all(|b| b.is_ascii_digit())
                    && word(source)
                {
                    return Some("merge.github");
                }
            }
        }
        if let Some(rest) = title.strip_prefix("Merge branch '") {
            if let Some((source, tail)) = rest.split_once('\'') {
                if word(source)
                    && (tail.is_empty() || tail.strip_prefix(" into ").is_some_and(word))
                {
                    return Some("merge.git");
                }
            }
        }
    }
    if p.ignore_reverts {
        let subject = title
            .strip_prefix("Revert \"")
            .or_else(|| title.strip_prefix("Reapply \""));
        if subject.is_none_or(|s| s.strip_suffix('"').is_none_or(|s| s.is_empty()))
            || lines.len() < 3
            || !lines[1].0.is_empty()
        {
            return None;
        }
        let hash =
            |s: &str| (s.len() == 40 || s.len() == 64) && s.bytes().all(|c| c.is_ascii_hexdigit());
        let reference = lines[2].0.strip_prefix("This reverts commit ")?;
        let end = if reference.strip_suffix('.').is_some_and(hash) {
            3
        } else {
            let first = reference.strip_suffix(", reversing")?;
            let second = lines
                .get(3)?
                .0
                .strip_prefix("changes made to ")?
                .strip_suffix('.')?;
            if !hash(first) || !hash(second) || first.len() != second.len() {
                return None;
            }
            4
        };
        if lines.len() == end || lines[end].0.is_empty() {
            return Some("revert.git");
        }
    }
    None
}
