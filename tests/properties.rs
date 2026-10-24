use fluidise_clint::{
    DIAGNOSTIC_LIMIT, DIAGNOSTIC_TEXT_LIMIT, Status, check_message, prepare_policy,
};
use proptest::prelude::*;

proptest! {
    #![proptest_config(ProptestConfig::with_cases(250))]

    #[test]
    fn arbitrary_input_never_panics_and_respects_limits(input in "\\PC{0,1024}") {
        let policy = prepare_policy(None).expect("default policy is valid");
        let result = check_message(&input, &policy);

        // Invariant 1: Diagnostics never exceed the maximum configured limit
        prop_assert!(result.diagnostics.len() <= DIAGNOSTIC_LIMIT);

        // Invariant 2: Diagnostic text is always bounded
        for diag in &result.diagnostics {
            prop_assert!(diag.message.chars().count() <= DIAGNOSTIC_TEXT_LIMIT);
        }

        // Invariant 3: Valid status implies no diagnostic errors
        if result.status == Status::Valid {
            prop_assert!(result.diagnostics.is_empty());
        }

        // Invariant 4: Non-valid non-ignored status implies at least one diagnostic
        if result.status != Status::Valid && result.status != Status::Ignored {
            prop_assert!(!result.diagnostics.is_empty());
        }
    }

    #[test]
    fn valid_conventional_commit_syntax_evaluates_valid(
        commit_type in prop_oneof![
            Just("feat"),
            Just("fix"),
            Just("docs"),
            Just("style"),
            Just("refactor"),
            Just("perf"),
            Just("test"),
            Just("build"),
            Just("ci"),
            Just("chore"),
        ],
        has_scope in any::<bool>(),
        scope in "[a-z0-9_-]{1,16}",
        desc in "[a-z][a-z0-9 ]{1,40}",
    ) {
        let policy = prepare_policy(None).expect("default policy is valid");
        let message = if has_scope {
            format!("{commit_type}({scope}): {desc}")
        } else {
            format!("{commit_type}: {desc}")
        };

        let result = check_message(&message, &policy);
        prop_assert_eq!(result.status, Status::Valid, "Expected valid for: {}", message);
        prop_assert!(result.diagnostics.is_empty());
    }

    #[test]
    fn revert_and_merge_patterns_never_panic(
        prefix in prop_oneof![
            Just("Merge branch "),
            Just("Merge pull request #"),
            Just("Revert \""),
            Just("Reapply \""),
        ],
        body in "\\PC{0,256}",
    ) {
        let policy = prepare_policy(None).expect("default policy is valid");
        let message = format!("{prefix}{body}");
        let result = check_message(&message, &policy);
        prop_assert!(result.diagnostics.len() <= DIAGNOSTIC_LIMIT);
    }
}
