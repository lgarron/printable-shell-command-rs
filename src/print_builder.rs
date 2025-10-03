use itertools::Itertools;

use crate::{
    format::{conditional_escape, unconditional_escape, ConditionalEscapeOptions},
    ArgumentLineWrapping, FormattingOptions,
};

const DEFAULT_MAIN_INDENTATION: &str = "";
const DEFAULT_ARG_INDENTATION: &str = "  ";

const INLINE_SEPARATOR: &str = " ";
const LINE_WRAP_LINE_END: &str = " \\\n";

struct CachedFormattingInfo {
    formatting_options: FormattingOptions,

    // TODO: construct lazily for perf?
    main_indentation: String,
    // arg_indentation: String,
    // line_wrap_separator: String,
    arg_tuple_separator: String,
    entry_separator: String,
}

impl CachedFormattingInfo {
    pub fn new(formatting_options: FormattingOptions) -> Self {
        let main_indentation = formatting_options
            .main_indentation
            .clone()
            .unwrap_or(DEFAULT_MAIN_INDENTATION.to_owned());
        let arg_indentation = formatting_options
            .arg_indentation
            .clone()
            .unwrap_or(DEFAULT_ARG_INDENTATION.to_owned());
        let line_wrap_separator = format!("{}{}", LINE_WRAP_LINE_END, arg_indentation);
        let arg_tuple_separator = match formatting_options
            .argument_line_wrapping
            .unwrap_or_default()
        {
            ArgumentLineWrapping::ByEntry => INLINE_SEPARATOR.to_owned(),
            ArgumentLineWrapping::NestedByEntry => {
                format!("{}{}", line_wrap_separator, arg_indentation)
            }
            ArgumentLineWrapping::ByArgument => line_wrap_separator.clone(),
            ArgumentLineWrapping::Inline => INLINE_SEPARATOR.to_owned(),
        };
        let entry_separator = match formatting_options
            .argument_line_wrapping
            .unwrap_or_default()
        {
            ArgumentLineWrapping::ByEntry
            | ArgumentLineWrapping::NestedByEntry
            | ArgumentLineWrapping::ByArgument => {
                format!(
                    "{}{}{}",
                    LINE_WRAP_LINE_END, main_indentation, arg_indentation
                )
            }
            ArgumentLineWrapping::Inline => INLINE_SEPARATOR.to_owned(),
        };
        Self {
            formatting_options,
            main_indentation,
            // arg_indentation,
            // line_wrap_separator,
            arg_tuple_separator,
            entry_separator,
        }
    }

    fn format_program_name(&self, program_name: &str) -> String {
        self.escape_arglike(program_name, true)
    }

    fn escape_arglike(&self, arglike: &str, is_main_command: bool) -> String {
        match self.formatting_options.quoting.unwrap_or_default() {
            crate::Quoting::Auto => {
                conditional_escape(arglike, ConditionalEscapeOptions { is_main_command })
            }
            crate::Quoting::ExtraSafe => unconditional_escape(arglike),
        }
    }
}

pub(crate) struct PrintBuilder {
    serialized_entries: Vec<String>,
    cached_formatting_info: CachedFormattingInfo,
}

impl PrintBuilder {
    pub fn new(formatting_options: FormattingOptions) -> Self {
        let cached_formatting_info = CachedFormattingInfo::new(formatting_options);
        Self {
            serialized_entries: vec![],
            cached_formatting_info,
        }
    }

    /// It is on the caller to call this:
    ///
    /// - exactly once,
    /// - before any args.
    pub fn add_program_name(&mut self, program_name: &str) {
        self.serialized_entries.push(format!(
            "{}{}",
            self.cached_formatting_info.main_indentation,
            self.cached_formatting_info
                .format_program_name(program_name),
        ))
    }

    pub fn add_single_arg(&mut self, arg: &str) {
        self.serialized_entries
            .push(self.cached_formatting_info.escape_arglike(arg, false));
    }

    pub fn add_arg_group<T: AsRef<str>>(&mut self, args: impl Iterator<Item = T>) {
        self.serialized_entries.push(
            args.map(|arg| {
                self.cached_formatting_info
                    .escape_arglike(arg.as_ref(), false)
            })
            .join(&self.cached_formatting_info.arg_tuple_separator),
        );
    }

    pub fn get(&self) -> String {
        self.serialized_entries
            .join(&self.cached_formatting_info.entry_separator)
    }
}
