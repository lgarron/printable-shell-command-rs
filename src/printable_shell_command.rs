use std::{
    ffi::{OsStr, OsString},
    process::Command,
    str::Utf8Error,
};

use crate::{
    escape::{SimpleEscapeOptions, simple_escape},
    shell_printable::{ShellPrintable, ShellPrintableSelf},
};

pub struct PrintableShellCommand {
    arg_groups: Vec<Vec<OsString>>,
    command: Command,
}

// TODO: this depends on the interface to `Command` supporting the *appending*
// of algs but not the removal/reordering/editing of any args added so far. Is
// it even remotely possible to fail compilation if the args in `Command` become
// mutable like this?
impl PrintableShellCommand {
    pub fn new<S: AsRef<OsStr>>(program: S) -> Self {
        Self {
            arg_groups: vec![],
            command: Command::new(program),
        }
    }

    pub fn arg<S: AsRef<OsStr>>(mut self, arg: S) -> Self {
        self.arg_groups.push(vec![(&arg).into()]);
        self.command.arg(arg);
        self
    }

    pub fn args<I, S>(mut self, args: I) -> Self
    where
        I: IntoIterator<Item = S>,
        S: AsRef<OsStr>,
    {
        let args: Vec<OsString> = args
            .into_iter()
            .map(|arg| std::convert::Into::<OsString>::into(&arg))
            .collect();
        self.arg_groups.push(args.clone());
        self.command.args(args);
        self
    }

    // Implementing `Deref` would mutating the underlying `Command` to add args,
    // which is not clean to work around. So we only allow explicit consumption
    // via function call.
    pub fn command(self) -> Command {
        self.command
    }
}

impl From<Command> for PrintableShellCommand {
    /// Adopts a `Command`, treating each arg as its own group (i.e. each arg will be printed on a separate line).
    fn from(command: Command) -> Self {
        Self {
            arg_groups: command
                .get_args()
                .map(|arg| vec![arg.to_os_string()])
                .collect(),
            command,
        }
    }
}

impl ShellPrintable for PrintableShellCommand {
    fn printable_invocation_string_lossy(&self) -> String {
        let mut lines: Vec<String> = vec![simple_escape(
            &self.command.get_program().to_string_lossy(),
            SimpleEscapeOptions { is_command: true },
        )];
        for arg_group in &self.arg_groups {
            let mut line_parts = vec![];
            for arg in arg_group {
                line_parts.push(simple_escape(
                    &arg.to_string_lossy(),
                    SimpleEscapeOptions { is_command: false },
                ))
            }
            lines.push(line_parts.join(" "))
        }
        lines.join(
            " \\
  ",
        )
    }
    fn printable_invocation_string(&self) -> Result<String, Utf8Error> {
        let mut lines: Vec<String> = vec![simple_escape(
            TryInto::<&str>::try_into(self.command.get_program())?,
            SimpleEscapeOptions { is_command: true },
        )];
        for arg_group in &self.arg_groups {
            let mut line_parts = vec![];
            for arg in arg_group {
                let s = TryInto::<&str>::try_into(arg.as_os_str())?;
                line_parts.push(simple_escape(s, SimpleEscapeOptions { is_command: false }))
            }
            lines.push(line_parts.join(" "))
        }
        Ok(lines.join(
            " \\
  ",
        ))
    }
}

impl ShellPrintableSelf for PrintableShellCommand {}
