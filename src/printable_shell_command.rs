use std::{
    ffi::{OsStr, OsString},
    ops::{Deref, DerefMut},
    process::Command,
    str::Utf8Error,
};

use itertools::Itertools;

use crate::{
    escape::{SimpleEscapeOptions, simple_escape},
    shell_printable::ShellPrintable,
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

    pub fn arg<S: AsRef<OsStr>>(&mut self, arg: S) -> &mut Self {
        self.adopt_args();
        self.arg_groups.push(vec![(&arg).into()]);
        self.command.arg(arg);
        self
    }

    pub fn args<I, S>(&mut self, args: I) -> &mut Self
    where
        I: IntoIterator<Item = S>,
        S: AsRef<OsStr>,
    {
        self.adopt_args();
        let args: Vec<OsString> = args
            .into_iter()
            .map(|arg| std::convert::Into::<OsString>::into(&arg))
            .collect();
        self.arg_groups.push(args.clone());
        self.command.args(args);
        self
    }

    fn args_to_adopt(&self) -> Vec<OsString> {
        let mut to_adopt: Vec<OsString> = vec![];
        for either_or_both in self
            .arg_groups
            .iter()
            .flatten()
            .zip_longest(self.command.get_args())
        {
            match either_or_both {
                itertools::EitherOrBoth::Both(a, b) => {
                    if a != b {
                        panic!("Command args do not match. This should not be possible.")
                    }
                }
                itertools::EitherOrBoth::Left(_) => {
                    panic!("Command is missing a previously seen arg. This should not be possible.")
                }
                itertools::EitherOrBoth::Right(arg) => {
                    to_adopt.push(arg.to_owned());
                }
            }
        }

        to_adopt
    }

    /// Adopt any args that were added to the underlying `Command` (from a
    /// `Deref`). Calling this function caches args instead of requiring
    /// throwaway work when subsequently generating printable strings (which
    /// would be inefficient when done multiple times).
    pub fn adopt_args(&mut self) -> &mut Self {
        for arg in self.args_to_adopt() {
            self.arg(arg);
        }
        self
    }
}

impl Deref for PrintableShellCommand {
    type Target = Command;

    fn deref(&self) -> &Command {
        &self.command
    }
}

impl DerefMut for PrintableShellCommand {
    /// If args are added to the underlying command, they will be added as individual arg groups by `PrintableShellCommand`.
    fn deref_mut(&mut self) -> &mut Command {
        &mut self.command
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

        // TODO: make this more efficient. (Take `&mut self` in the trait? Interior mutability?)
        let a = self.arg_groups.iter();
        let b: Vec<Vec<OsString>> = self
            .args_to_adopt()
            .into_iter()
            .map(|arg| vec![arg])
            .collect();

        for arg_group in a.chain(&b) {
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

        // TODO: make this more efficient. (Take `&mut self` in the trait? Interior mutability?)
        let a = self.arg_groups.iter();
        let b: Vec<Vec<OsString>> = self
            .args_to_adopt()
            .into_iter()
            .map(|arg| vec![arg])
            .collect();

        for arg_group in a.chain(&b) {
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
