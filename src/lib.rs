//! Agent-first commit validation. No filesystem, Git, process exits or output.
//!
//! ```
//! use fluidise_clint::{prepare_policy, check_message, Status};
//! let rules = prepare_policy(None).expect("default rules are valid");
//! let result = check_message("feat: add search", &rules);
//! assert_eq!(result.status, Status::Valid);
//! ```
//!
//! A caller cannot bypass policy validation by mutating internal rules:
//! ```compile_fail
//! let mut rules = fluidise_clint::prepare_policy(None).unwrap();
//! rules.ignore_merges = false;
//! ```
mod configuration;
mod message;
pub use configuration::prepare_policy;
pub use message::check_message;

pub const INPUT_LIMIT: usize = 1_048_576;
pub const OUTPUT_LIMIT: usize = 262_144;
pub const DIAGNOSTIC_LIMIT: usize = 20;
pub const DIAGNOSTIC_TEXT_LIMIT: usize = 512;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum Status {
    Valid,
    Ignored,
    Invalid,
    Error,
}
impl Status {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Valid => "valid",
            Self::Ignored => "ignored",
            Self::Invalid => "invalid",
            Self::Error => "error",
        }
    }
    pub fn exit_code(self) -> u8 {
        match self {
            Self::Valid | Self::Ignored => 0,
            Self::Invalid => 1,
            Self::Error => 2,
        }
    }
}
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum Source {
    Message,
    Configuration,
    Arguments,
    Tool,
}
impl Source {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Message => "message",
            Self::Configuration => "configuration",
            Self::Arguments => "arguments",
            Self::Tool => "tool",
        }
    }
}
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Span {
    pub start: usize,
    pub end: usize,
}
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum Field {
    Key(String),
    Index(usize),
}
#[derive(Clone, Debug)]
pub struct Diagnostic {
    pub code: &'static str,
    pub message: String,
    pub source: Source,
    pub field: Option<Vec<Field>>,
    pub span: Option<Span>,
}
impl Diagnostic {
    pub fn new(code: &'static str, message: &str, source: Source) -> Self {
        let mut text: String = message.chars().take(DIAGNOSTIC_TEXT_LIMIT).collect();
        if message.chars().nth(DIAGNOSTIC_TEXT_LIMIT).is_some() {
            text = message.chars().take(DIAGNOSTIC_TEXT_LIMIT - 3).collect();
            text.push_str("...");
        }
        Self {
            code,
            message: text,
            source,
            field: None,
            span: None,
        }
    }
    pub(crate) fn at(mut self, start: usize, end: usize) -> Self {
        self.span = Some(Span { start, end });
        self
    }
    pub(crate) fn field(mut self, name: &str, index: Option<usize>) -> Self {
        let mut field = vec![Field::Key("commit".into())];
        if !name.is_empty() {
            field.push(Field::Key(name.into()));
        }
        if let Some(i) = index {
            field.push(Field::Index(i));
        }
        self.field = Some(field);
        self
    }
}
#[derive(Clone, Debug)]
pub struct ConfigurationSummary {
    pub source: &'static str,
    pub used_sections: Vec<String>,
    pub unrecognized_sections: Vec<String>,
}
#[derive(Clone, Debug)]
pub struct CheckResult {
    pub status: Status,
    pub reason: Option<&'static str>,
    pub diagnostics_complete: bool,
    pub diagnostics: Vec<Diagnostic>,
    pub configuration: Option<ConfigurationSummary>,
}
impl CheckResult {
    pub fn error(diagnostic: Diagnostic) -> Self {
        Self {
            status: Status::Error,
            reason: None,
            diagnostics_complete: true,
            diagnostics: vec![diagnostic],
            configuration: None,
        }
    }
}
/// Immutable validated rules. Only `prepare_policy` can construct this type.
#[derive(Clone, Debug)]
pub struct Policy {
    allowed_types: Vec<String>,
    allowed_scopes: Option<Vec<String>>,
    max_title_length: Option<u64>,
    ignore_merges: bool,
    ignore_reverts: bool,
    summary: ConfigurationSummary,
}
impl Policy {
    pub fn configuration(&self) -> &ConfigurationSummary {
        &self.summary
    }
}

pub(crate) struct Diagnostics {
    items: Vec<Diagnostic>,
}
impl Diagnostics {
    pub fn new() -> Self {
        Self { items: Vec::new() }
    }
    pub fn full(&self) -> bool {
        self.items.len() == DIAGNOSTIC_LIMIT
    }
    pub fn push(&mut self, d: Diagnostic) {
        if !self.full() {
            self.items.push(d);
        }
    }
    pub fn result(
        self,
        success: Status,
        failure: Status,
        summary: Option<ConfigurationSummary>,
    ) -> CheckResult {
        CheckResult {
            status: if self.items.is_empty() {
                success
            } else {
                failure
            },
            reason: None,
            diagnostics_complete: !self.full(),
            diagnostics: self.items,
            configuration: summary,
        }
    }
}
pub(crate) fn valid_type(s: &str) -> bool {
    let mut b = s.bytes();
    b.next().is_some_and(|c| c.is_ascii_lowercase())
        && b.all(|c| c.is_ascii_lowercase() || c.is_ascii_digit() || c == b'-')
}
pub(crate) fn valid_scope(s: &str) -> bool {
    !s.is_empty() && s.trim() == s && !s.chars().any(|c| c.is_control() || c == '(' || c == ')')
}
