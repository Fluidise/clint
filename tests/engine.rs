use fluidise_clint::*;
fn policy() -> Policy {
    prepare_policy(None).unwrap()
}
#[test]
fn approved_catalogue() {
    let cases: serde_json::Value = serde_json::from_str(include_str!("catalogue.json")).unwrap();
    for c in cases.as_array().unwrap() {
        let r = check_message(c["message"].as_str().unwrap(), &policy());
        assert_eq!(
            r.status == Status::Valid,
            c["valid"].as_bool().unwrap(),
            "{}: {:?}",
            c["id"],
            r
        );
    }
}
#[test]
fn configuration_replacement_scopes_and_unicode_length() {
    let p = prepare_policy(Some(
        "[commit]\nversion=1\nallowed-types=['deps']\nallowed-scopes=['cli']\nmax-title-length=12",
    ))
    .unwrap();
    assert_eq!(check_message("deps: éééééé", &p).status, Status::Valid);
    for s in ["feat: a", "deps(CLI): a", "deps: abcdefg"] {
        assert_eq!(check_message(s, &p).status, Status::Invalid);
    }
    let p = prepare_policy(Some("[commit]\nversion=1\nmax-title-length=6")).unwrap();
    assert_eq!(check_message("fix: é\r\n\r\nx", &p).status, Status::Valid);
    let r = check_message("fix: e\u{301}", &p);
    assert_eq!(r.diagnostics[0].code, "commit.title.too_long");
    assert_eq!(r.diagnostics[0].span, Some(Span { start: 0, end: 8 }));
}
#[test]
fn config_validation_and_prerequisites() {
    for s in [
        "[commit]\nversion='1'",
        "[commit]\nversion=1.0",
        "[commit]\nversion=2",
        "[commit]",
        "commit=1",
        "[commit]\nversion=1\nallowed-types=[]",
        "[commit]\nversion=1\nallowed-types=['feat','feat']",
        "[commit]\nversion=1\nallowed-types=['FEAT']",
        "[commit]\nversion=1\nallowed-scopes=[' cli']",
        "[commit]\nversion=1\nallowed-scopes=['a(b)']",
        "[commit]\nversion=1\nmax-title-length=0",
        "[commit]\nversion=1\nmax-title-length=1.0",
        "[commit]\nversion=1\nignore-merges='true'",
        "[commit]\nversion=1\nother=1",
    ] {
        let r = prepare_policy(Some(s)).unwrap_err();
        assert_eq!(r.status, Status::Error, "{s}");
        assert!(!r.diagnostics.is_empty());
    }
    let r = prepare_policy(Some("[commit]\nversion=999\nunknown=1\nallowed-types=[]")).unwrap_err();
    assert_eq!(r.diagnostics.len(), 1);
    for v in ["1", "+1", "0x1", "0b1", "0o1"] {
        assert!(
            prepare_policy(Some(&format!("[commit]\nversion={v}"))).is_ok(),
            "{v}"
        );
    }
    assert!(prepare_policy(Some("[foreign]\nversion='future'\nanything={deep=['ok']}")).is_ok());
    assert_eq!(
        prepare_policy(Some("[foreign]\nbad="))
            .unwrap_err()
            .diagnostics[0]
            .code,
        "config.syntax.invalid"
    );
}
#[test]
fn config_diagnostics_are_bounded_and_located() {
    let s = format!(
        "[commit]\nversion=1\nallowed-types=[{}]\n",
        vec!["''"; 1000].join(",")
    );
    let r = prepare_policy(Some(&s)).unwrap_err();
    assert_eq!(r.diagnostics.len(), 20);
    assert!(!r.diagnostics_complete);
    for (i, d) in r.diagnostics.iter().enumerate() {
        assert_eq!(
            d.field,
            Some(vec![
                Field::Key("commit".into()),
                Field::Key("allowed-types".into()),
                Field::Index(i)
            ])
        );
        let span = d.span.as_ref().unwrap();
        assert_eq!(&s[span.start..span.end], "''");
    }
    let r = prepare_policy(Some("[commit]\nunknown=1")).unwrap_err();
    assert!(r.diagnostics[0].span.is_none());
}
#[test]
fn input_content_and_boundaries() {
    let p = policy();
    for s in ["fix: a\0", "fix: a\rb"] {
        assert_eq!(
            check_message(s, &p).diagnostics[0].code,
            "input.content.unsupported"
        );
    }
    assert_eq!(check_message("\u{feff}fix: a", &p).status, Status::Invalid);
    for s in ["fix: a", "fix: a\n", "fix: a\n\n", "fix: a\r\n\r\nbody\n"] {
        assert_eq!(check_message(s, &p).status, Status::Valid);
    }
    let mut s = "fix: ".to_string();
    s.push_str(&"a".repeat(INPUT_LIMIT - s.len()));
    assert_eq!(check_message(&s, &p).status, Status::Valid);
    s.push('a');
    assert_eq!(
        check_message(&s, &p).diagnostics[0].code,
        "input.limit_exceeded"
    );
    assert_eq!(
        prepare_policy(Some(&" ".repeat(INPUT_LIMIT + 1)))
            .unwrap_err()
            .diagnostics[0]
            .code,
        "config.limit_exceeded"
    );
    let deep = format!("x={}0{}", "[".repeat(100), "]".repeat(100));
    assert_eq!(
        prepare_policy(Some(&deep)).unwrap_err().diagnostics[0].code,
        "config.syntax.invalid"
    );
}
#[test]
fn merges_reverts_and_no_provenance_assumption() {
    let p = policy();
    for title in [
        "Merge branch 'topic'",
        "Merge branch 'topic' into main",
        "Merge pull request #42 from patch-1",
        "Merge pull request #123456789012345678901234567890 from owner/branch",
    ] {
        let r = check_message(&format!("{title}\nopaque body"), &p);
        assert_eq!(r.status, Status::Ignored, "{title}");
        assert_eq!(r.status.exit_code(), 0);
    }
    for title in [
        "Merge branch ''",
        "Merge branch 'two words'",
        "Merge branch 'topic' garbage",
        "Merge pull request #01 from topic",
        "Merge pull request #0 from topic",
        "Merge pull request #42 from topic x",
    ] {
        assert_eq!(check_message(title, &p).status, Status::Invalid, "{title}");
    }
    for size in [40, 64] {
        for action in ["Revert", "Reapply"] {
            let first = "a".repeat(size);
            let second = "B".repeat(size);
            for reference in [
                format!("This reverts commit {first}."),
                format!("This reverts commit {first}, reversing\nchanges made to {second}."),
            ] {
                let text = format!("{action} \"Revert \"feat: a\"\"\n\n{reference}");
                assert_eq!(check_message(&text, &p).reason, Some("revert.git"));
                assert_eq!(
                    check_message(&(text.clone() + "\n\nReason."), &p).status,
                    Status::Ignored
                );
                assert_eq!(
                    check_message(&(text + "\nReason."), &p).status,
                    Status::Invalid
                );
            }
        }
    }
    for s in [
        "Revert \"feat: a\"",
        "Revert \"feat: a\"\n\nThis reverts commit abc123.",
    ] {
        assert_eq!(check_message(s, &p).status, Status::Invalid);
    }
    let p = prepare_policy(Some(
        "[commit]\nversion=1\nignore-merges=false\nignore-reverts=false",
    ))
    .unwrap();
    assert_eq!(
        check_message("Merge branch 'topic'", &p).status,
        Status::Invalid
    );
    assert_eq!(
        check_message(
            &format!(
                "Revert \"feat: a\"\n\nThis reverts commit {}.",
                "a".repeat(40)
            ),
            &p
        )
        .status,
        Status::Invalid
    );
}
#[test]
fn footer_continuation_boundaries_and_independent_errors() {
    let r = check_message("fix: a\r\n\r\nRefs #\r\nReviewed-by: ", &policy());
    assert_eq!(r.diagnostics.len(), 2);
    assert_eq!(r.diagnostics[0].span, Some(Span { start: 10, end: 16 }));
    let r = check_message("fix: a\n\nRefs #\ncontinued\n\nReviewed-by: x", &policy());
    assert_eq!(r.status, Status::Valid);
    let s = format!("fix: a\n\n{}", vec!["Refs #"; 30].join("\n"));
    let r = check_message(&s, &policy());
    assert_eq!(r.diagnostics.len(), 20);
    assert!(!r.diagnostics_complete);
    assert_eq!(check_message("feat!: a", &policy()).status, Status::Valid);
}
#[test]
fn diagnostic_text_bound() {
    let d = Diagnostic::new("test", &"é".repeat(1000), Source::Tool);
    assert_eq!(d.message.chars().count(), 512);
    assert!(d.message.ends_with("..."));
}

#[test]
fn regression_scope_separator_is_outside_parentheses() {
    let p = prepare_policy(Some(
        "[commit]\nversion=1\nallowed-scopes=['a: b', 'é: x!']",
    ))
    .unwrap();
    for message in ["fix(a: b): desc", "fix(a: b)!: desc", "fix(é: x!): desc"] {
        assert_eq!(
            check_message(message, &p).status,
            Status::Valid,
            "{message}"
        );
    }
    for message in [
        "fix(a: b: desc",
        "fix(a(b)): desc",
        "fix(a: b)!!: desc",
        "fix(a: b): ",
    ] {
        assert_eq!(
            check_message(message, &p).status,
            Status::Invalid,
            "{message}"
        );
    }
}

#[test]
fn regression_footer_separator_is_before_its_value() {
    for footer in ["Refs #123: text", "Refs: text #123"] {
        let message = format!("fix: a\n\n{footer}\nBREAKING CHANGE: ");
        let result = check_message(&message, &policy());
        assert_eq!(result.status, Status::Invalid, "{footer}");
        assert_eq!(result.diagnostics.len(), 1);
        assert_eq!(result.diagnostics[0].code, "commit.footer.invalid");
        let span = result.diagnostics[0].span.as_ref().unwrap();
        assert_eq!(&message[span.start..span.end], "BREAKING CHANGE: ");
        assert_eq!(
            check_message(&(message + "\nactual change"), &policy()).status,
            Status::Valid
        );
    }
}

#[test]
fn trailing_empty_lines_preserve_complete_results() {
    let p = policy();
    let revert = format!(
        "Revert \"feat: é\"\n\nThis reverts commit {}.",
        "a".repeat(40)
    );
    let reversing = format!(
        "Reapply \"feat: é\"\n\nThis reverts commit {}, reversing\nchanges made to {}.",
        "a".repeat(40),
        "b".repeat(40)
    );
    for base in [
        "",
        "feat: é",
        "feat: é\nbody",
        "feat: é\n\nbody",
        "feat: é\n\nRefs #",
        "feat: é\n\nRefs #\ncontinued",
        "feat: é\n\nRefs #\nReviewed-by: ",
        "Merge branch 'topic'",
        &revert,
        &reversing,
    ] {
        for newline in ["\n", "\r\n"] {
            let text = base.replace('\n', newline);
            let expected = format!("{:?}", check_message(&text, &p));
            for count in [1, 2, 20] {
                let actual = check_message(&(text.clone() + &newline.repeat(count)), &p);
                assert_eq!(format!("{actual:?}"), expected, "{text:?}, {count}");
            }
        }
    }
}

#[test]
fn dense_lines_keep_limits_and_original_footer_spans() {
    let p = policy();
    for newline in ["\n", "\r\n"] {
        let prefix = format!(
            "feat: é{newline}{newline}{}",
            format!("x{newline}").repeat(10_000)
        );
        let message = format!("{prefix}{newline}Refs #{newline}Reviewed-by: {newline}{newline}");
        let result = check_message(&message, &p);
        assert_eq!(result.diagnostics.len(), 2);
        assert!(result.diagnostics_complete);
        for diagnostic in &result.diagnostics {
            assert_eq!(diagnostic.code, "commit.footer.invalid");
            let span = diagnostic.span.as_ref().unwrap();
            assert!(matches!(
                &message[span.start..span.end],
                "Refs #" | "Reviewed-by: "
            ));
        }
    }
    let mut text = "feat: a\n\n".to_string();
    text.push_str(&"\n".repeat(INPUT_LIMIT - text.len()));
    assert_eq!(check_message(&text, &p).status, Status::Valid);
    text.push('\r');
    // Size validation precedes content validation, including for newline floods.
    let result = check_message(&text, &p);
    assert_eq!(result.status, Status::Error);
    assert_eq!(result.diagnostics[0].code, "input.limit_exceeded");
    text.truncate(INPUT_LIMIT);
    text.replace_range(INPUT_LIMIT - 1.., "\r");
    assert_eq!(
        check_message(&text, &p).diagnostics[0].code,
        "input.content.unsupported"
    );
}
