use std::sync::LazyLock;

use regex::Regex;

pub(crate) struct ConditionalEscapeOptions {
    pub(crate) is_main_command: bool,
}

static PROGRAM_NAME_REGEX: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r#"[ "'`|$*?><()\[\]{}&\\;#=]"#).unwrap());
static ARG_REGEX: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r#"[ "'`|$*?><()\[\]{}&\\;#]"#).unwrap());

pub(crate) fn conditional_escape(s: &str, options: ConditionalEscapeOptions) -> String {
    let regex = if options.is_main_command {
        &PROGRAM_NAME_REGEX
    } else {
        &ARG_REGEX
    };

    if regex.is_match(s) {
        unconditional_escape(s)
    } else {
        s.to_owned()
    }
}

pub(crate) fn unconditional_escape(s: &str) -> String {
    format!("'{}'", s.replace("\\", "\\\\").replace("'", "\\'"))
}
