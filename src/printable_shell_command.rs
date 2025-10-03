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
        let arg = self.arg_without_adoption(arg);
        self.command.arg(arg);
        self
    }

    fn arg_without_adoption<S: AsRef<OsStr>>(&mut self, arg: S) -> S {
        self.arg_groups.push(vec![(&arg).into()]);
        arg
    }

    pub fn args<I, S>(&mut self, args: I) -> &mut Self
    where
        I: IntoIterator<Item = S>,
        S: AsRef<OsStr>,
    {
        self.adopt_args();
        let args = self.args_without_adoption(args);
        self.command.args(args);
        self
    }

    fn args_without_adoption<I, S>(&mut self, args: I) -> Vec<OsString>
    where
        I: IntoIterator<Item = S>,
        S: AsRef<OsStr>,
    {
        let args: Vec<OsString> = args
            .into_iter()
            .map(|arg| std::convert::Into::<OsString>::into(&arg))
            .collect();
        self.arg_groups.push(args.clone());
        args
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
        dbg!(&to_adopt);

        to_adopt
    }

    /// Adopt any args that were added to the underlying `Command` (from a
    /// `Deref`). Calling this function caches args instead of requiring
    /// throwaway work when subsequently generating printable strings (which
    /// would be inefficient when done multiple times).
    pub fn adopt_args(&mut self) -> &mut Self {
        for arg in self.args_to_adopt() {
            self.arg_without_adoption(arg);
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
        let mut printable_shell_command = Self {
            arg_groups: vec![],
            command,
        };
        printable_shell_command.adopt_args();
        printable_shell_command
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

#[cfg(test)]
mod tests {
    use std::{ops::DerefMut, process::Command};

    use crate::{PrintableShellCommand, ShellPrintable};

    #[test]
    fn echo() -> Result<(), String> {
        let mut printable_shell_command = PrintableShellCommand::new("echo");
        printable_shell_command.args(["#hi"]);
        // Not printed by successful tests, but we can at least check this doesn't panic.
        let _ = printable_shell_command.print_invocation();

        assert_eq!(
            printable_shell_command
                .printable_invocation_string()
                .unwrap(),
            "echo \\
  '#hi'"
        );
        assert_eq!(
            printable_shell_command
                .printable_invocation_string()
                .unwrap(),
            printable_shell_command.printable_invocation_string_lossy(),
        );
        Ok(())
    }

    #[test]
    fn ffmpeg() -> Result<(), String> {
        let mut printable_shell_command = PrintableShellCommand::new("ffmpeg");
        printable_shell_command
            .args(["-i", "./test/My video.mp4"])
            .args(["-filter:v", "setpts=2.0*PTS"])
            .args(["-filter:a", "atempo=0.5"])
            .arg("./test/My video (slow-mo).mov");
        // Not printed by successful tests, but we can at least check this doesn't panic.
        let _ = printable_shell_command.print_invocation();

        assert_eq!(
            printable_shell_command
                .printable_invocation_string()
                .unwrap(),
            "ffmpeg \\
  -i './test/My video.mp4' \\
  -filter:v 'setpts=2.0*PTS' \\
  -filter:a atempo=0.5 \\
  './test/My video (slow-mo).mov'"
        );
        assert_eq!(
            printable_shell_command
                .printable_invocation_string()
                .unwrap(),
            printable_shell_command.printable_invocation_string_lossy(),
        );
        Ok(())
    }

    #[test]
    fn from_command() -> Result<(), String> {
        let mut command = Command::new("echo");
        command.args(["hello", "#world"]);
        // Not printed by tests, but we can at least check this doesn't panic.
        let mut printable_shell_command = PrintableShellCommand::from(command);
        let _ = printable_shell_command.print_invocation();

        assert_eq!(
            printable_shell_command
                .printable_invocation_string()
                .unwrap(),
            "echo \\
  hello \\
  '#world'"
        );
        Ok(())
    }

    #[test]
    fn adoption() -> Result<(), String> {
        let mut printable_shell_command = PrintableShellCommand::new("echo");

        {
            let command: &mut Command = printable_shell_command.deref_mut();
            command.arg("hello");
            command.arg("#world");
        }

        printable_shell_command
            .printable_invocation_string()
            .unwrap();
        assert_eq!(
            printable_shell_command
                .printable_invocation_string()
                .unwrap(),
            "echo \\
  hello \\
  '#world'"
        );

        printable_shell_command.args(["wide", "web"]);

        printable_shell_command
            .printable_invocation_string()
            .unwrap();
        assert_eq!(
            printable_shell_command
                .printable_invocation_string()
                .unwrap(),
            "echo \\
  hello \\
  '#world' \\
  wide web"
        );

        // Second adoption
        {
            let command: &mut Command = printable_shell_command.deref_mut();
            command.arg("to").arg("the").arg("internet");
        }
        // Test adoption idempotency.
        printable_shell_command.adopt_args();
        printable_shell_command.adopt_args();
        printable_shell_command.adopt_args();
        assert_eq!(
            printable_shell_command
                .printable_invocation_string()
                .unwrap(),
            "echo \\
  hello \\
  '#world' \\
  wide web \\
  to \\
  the \\
  internet"
        );

        Ok(())
    }

    // TODO: test invalid UTF-8
}
