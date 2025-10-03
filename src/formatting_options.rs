#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum Quoting {
    /// Quote only arguments that need it for safety. This tries to be
    /// portable and safe across shells, but true safety and portability is hard
    /// to guarantee.
    Auto,

    /// Quote all arguments, even ones that don't need it. This is
    /// more likely to be safe under all circumstances.
    ExtraSafe,
}

impl Default for Quoting {
    fn default() -> Self {
        Self::Auto
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ArgumentLineWrapping {
    ByEntry,
    NestedByEntry,
    ByArgument,
    Inline,
}

impl Default for ArgumentLineWrapping {
    fn default() -> Self {
        Self::ByEntry
    }
}

#[derive(Default)]
pub struct FormattingOptions {
    pub main_indentation: Option<String>,
    pub arg_indentation: Option<String>,
    pub quoting: Option<Quoting>,
    // Line wrapping to use between arguments.
    pub argument_line_wrapping: Option<ArgumentLineWrapping>,
    // TODO: text styling
}
