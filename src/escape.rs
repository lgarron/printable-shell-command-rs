use std::sync::LazyLock;

use regex::Regex;

pub(crate) struct SimpleEscapeOptions {
    pub(crate) is_command: bool,
}

static PROGRAM_NAME_REGEX: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r#"[ "'`|$*?><()\[\]{}&\\;#=]"#).unwrap());
static ARG_REGEX: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r#"[ "'`|$*?><()\[\]{}&\\;#]"#).unwrap());

pub(crate) fn simple_escape(s: &str, options: SimpleEscapeOptions) -> String {
    let regex = if options.is_command {
        &PROGRAM_NAME_REGEX
    } else {
        &ARG_REGEX
    };

    if regex.is_match(s) {
        format!("'{}'", s.replace("\\", "\\\\").replace("'", "\\'"))
    } else {
        s.to_owned()
    }
}
