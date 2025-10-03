use std::{
    ffi::{OsStr, OsString},
    ops::{Deref, DerefMut},
    process::Command,
    str::Utf8Error,
};

use itertools::Itertools;

use crate::{
    command::{add_arg_from_command, add_arg_from_command_lossy},
    print_builder::PrintBuilder,
    shell_printable::{ShellPrintable, ShellPrintableWithOptions},
    FormattingOptions,
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

    fn add_unadopted_args_lossy(&self, print_builder: &mut PrintBuilder) {
        for arg in self.args_to_adopt() {
            add_arg_from_command_lossy(print_builder, arg.as_os_str());
        }
    }

    fn add_unadopted_args(&self, print_builder: &mut PrintBuilder) -> Result<(), Utf8Error> {
        for arg in self.args_to_adopt() {
            add_arg_from_command(print_builder, arg.as_os_str())?;
        }
        Ok(())
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

impl ShellPrintableWithOptions for PrintableShellCommand {
    fn printable_invocation_string_lossy_with_options(
        &self,
        formatting_options: FormattingOptions,
    ) -> String {
        let mut print_builder = PrintBuilder::new(formatting_options);
        print_builder.add_program_name(&self.get_program().to_string_lossy());
        for arg_group in &self.arg_groups {
            let mut strings: Vec<String> = vec![];
            for arg in arg_group {
                strings.push(arg.to_string_lossy().to_string())
            }
            print_builder.add_arg_group(strings.iter());
        }
        self.add_unadopted_args_lossy(&mut print_builder);
        print_builder.get()
    }

    fn printable_invocation_string_with_options(
        &self,
        formatting_options: FormattingOptions,
    ) -> Result<String, Utf8Error> {
        let mut print_builder = PrintBuilder::new(formatting_options);
        print_builder.add_program_name(TryInto::<&str>::try_into(self.get_program())?);
        for arg_group in &self.arg_groups {
            let mut strings: Vec<&str> = vec![];
            for arg in arg_group {
                let s = TryInto::<&str>::try_into(arg.as_os_str())?;
                strings.push(s)
            }
            print_builder.add_arg_group(strings.into_iter());
        }
        self.add_unadopted_args(&mut print_builder)?;
        Ok(print_builder.get())
    }
}

impl ShellPrintable for PrintableShellCommand {
    fn printable_invocation_string(&self) -> Result<String, Utf8Error> {
        self.printable_invocation_string_with_options(Default::default())
    }

    fn printable_invocation_string_lossy(&self) -> String {
        self.printable_invocation_string_lossy_with_options(Default::default())
    }
}

#[cfg(test)]
mod tests {
    use std::{ops::DerefMut, process::Command, str::Utf8Error};

    use crate::{
        FormattingOptions, PrintableShellCommand, Quoting, ShellPrintable,
        ShellPrintableWithOptions,
    };

    #[test]
    fn echo() -> Result<(), Utf8Error> {
        let mut printable_shell_command = PrintableShellCommand::new("echo");
        printable_shell_command.args(["#hi"]);
        // Not printed by successful tests, but we can at least check this doesn't panic.
        let _ = printable_shell_command.print_invocation();

        assert_eq!(
            printable_shell_command.printable_invocation_string()?,
            "echo \\
  '#hi'"
        );
        assert_eq!(
            printable_shell_command.printable_invocation_string()?,
            printable_shell_command.printable_invocation_string_lossy(),
        );
        Ok(())
    }

    #[test]
    fn ffmpeg() -> Result<(), Utf8Error> {
        let mut printable_shell_command = PrintableShellCommand::new("ffmpeg");
        printable_shell_command
            .args(["-i", "./test/My video.mp4"])
            .args(["-filter:v", "setpts=2.0*PTS"])
            .args(["-filter:a", "atempo=0.5"])
            .arg("./test/My video (slow-mo).mov");
        // Not printed by successful tests, but we can at least check this doesn't panic.
        let _ = printable_shell_command.print_invocation();

        assert_eq!(
            printable_shell_command.printable_invocation_string()?,
            "ffmpeg \\
  -i './test/My video.mp4' \\
  -filter:v 'setpts=2.0*PTS' \\
  -filter:a atempo=0.5 \\
  './test/My video (slow-mo).mov'"
        );
        assert_eq!(
            printable_shell_command.printable_invocation_string()?,
            printable_shell_command.printable_invocation_string_lossy(),
        );
        Ok(())
    }

    #[test]
    fn from_command() -> Result<(), Utf8Error> {
        let mut command = Command::new("echo");
        command.args(["hello", "#world"]);
        // Not printed by tests, but we can at least check this doesn't panic.
        let mut printable_shell_command = PrintableShellCommand::from(command);
        let _ = printable_shell_command.print_invocation();

        assert_eq!(
            printable_shell_command.printable_invocation_string()?,
            "echo \\
  hello \\
  '#world'"
        );
        Ok(())
    }

    #[test]
    fn adoption() -> Result<(), Utf8Error> {
        let mut printable_shell_command = PrintableShellCommand::new("echo");

        {
            let command: &mut Command = printable_shell_command.deref_mut();
            command.arg("hello");
            command.arg("#world");
        }

        printable_shell_command.printable_invocation_string()?;
        assert_eq!(
            printable_shell_command.printable_invocation_string()?,
            "echo \\
  hello \\
  '#world'"
        );

        printable_shell_command.args(["wide", "web"]);

        printable_shell_command.printable_invocation_string()?;
        assert_eq!(
            printable_shell_command.printable_invocation_string()?,
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

    fn rsync_command_for_testing() -> PrintableShellCommand {
        let mut printable_shell_command = PrintableShellCommand::new("rsync");
        printable_shell_command
            .arg("-avz")
            .args(["--exclude", ".DS_Store"])
            .args(["--exclude", ".git"])
            .arg("./dist/web/experiments.cubing.net/test/deploy/")
            .arg("experiments.cubing.net:~/experiments.cubing.net/test/deploy/");
        printable_shell_command
    }

    #[test]
    fn extra_safe_quoting() -> Result<(), Utf8Error> {
        let printable_shell_command = rsync_command_for_testing();
        assert_eq!(
            printable_shell_command.printable_invocation_string_with_options(
                FormattingOptions {
                    quoting: Some(Quoting::ExtraSafe),
                    ..Default::default()
                }
            )?,
            "'rsync' \\
  '-avz' \\
  '--exclude' '.DS_Store' \\
  '--exclude' '.git' \\
  './dist/web/experiments.cubing.net/test/deploy/' \\
  'experiments.cubing.net:~/experiments.cubing.net/test/deploy/'"
        );
        Ok(())
    }

    #[test]
    fn indentation() -> Result<(), Utf8Error> {
        let printable_shell_command = rsync_command_for_testing();
        assert_eq!(
            printable_shell_command.printable_invocation_string_with_options(
                FormattingOptions {
                    arg_indentation: Some("\t   \t".to_owned()),
                    ..Default::default()
                }
            )?,
            "rsync \\
	   	-avz \\
	   	--exclude .DS_Store \\
	   	--exclude .git \\
	   	./dist/web/experiments.cubing.net/test/deploy/ \\
	   	experiments.cubing.net:~/experiments.cubing.net/test/deploy/"
        );
        assert_eq!(
            printable_shell_command.printable_invocation_string_with_options(
                FormattingOptions {
                    arg_indentation: Some("↪ ".to_owned()),
                    ..Default::default()
                }
            )?,
            "rsync \\
↪ -avz \\
↪ --exclude .DS_Store \\
↪ --exclude .git \\
↪ ./dist/web/experiments.cubing.net/test/deploy/ \\
↪ experiments.cubing.net:~/experiments.cubing.net/test/deploy/"
        );
        assert_eq!(
            printable_shell_command.printable_invocation_string_with_options(
                FormattingOptions {
                    main_indentation: Some("  ".to_owned()),
                    ..Default::default()
                }
            )?,
            "  rsync \\
    -avz \\
    --exclude .DS_Store \\
    --exclude .git \\
    ./dist/web/experiments.cubing.net/test/deploy/ \\
    experiments.cubing.net:~/experiments.cubing.net/test/deploy/"
        );
        assert_eq!(
            printable_shell_command.printable_invocation_string_with_options(
                FormattingOptions {
                    main_indentation: Some("🙈".to_owned()),
                    arg_indentation: Some("🙉".to_owned()),
                    ..Default::default()
                }
            )?,
            "🙈rsync \\
🙈🙉-avz \\
🙈🙉--exclude .DS_Store \\
🙈🙉--exclude .git \\
🙈🙉./dist/web/experiments.cubing.net/test/deploy/ \\
🙈🙉experiments.cubing.net:~/experiments.cubing.net/test/deploy/"
        );
        Ok(())
    }

    #[test]
    fn line_wrapping() -> Result<(), Utf8Error> {
        let printable_shell_command = rsync_command_for_testing();
        assert_eq!(
            printable_shell_command.printable_invocation_string_with_options(
                FormattingOptions {
                    argument_line_wrapping: Some(crate::ArgumentLineWrapping::ByEntry),
                    ..Default::default()
                }
            )?,
            printable_shell_command.printable_invocation_string()?
        );
        assert_eq!(
            printable_shell_command.printable_invocation_string_with_options(
                FormattingOptions {
                    argument_line_wrapping: Some(crate::ArgumentLineWrapping::NestedByEntry),
                    ..Default::default()
                }
            )?,
            "rsync \\
  -avz \\
  --exclude \\
    .DS_Store \\
  --exclude \\
    .git \\
  ./dist/web/experiments.cubing.net/test/deploy/ \\
  experiments.cubing.net:~/experiments.cubing.net/test/deploy/"
        );
        assert_eq!(
            printable_shell_command.printable_invocation_string_with_options(
                FormattingOptions {
                    argument_line_wrapping: Some(crate::ArgumentLineWrapping::ByArgument),
                    ..Default::default()
                }
            )?,
            "rsync \\
  -avz \\
  --exclude \\
  .DS_Store \\
  --exclude \\
  .git \\
  ./dist/web/experiments.cubing.net/test/deploy/ \\
  experiments.cubing.net:~/experiments.cubing.net/test/deploy/"
        );
        Ok(())
    }

    #[test]
    fn command_with_space_is_escaped_by_default() -> Result<(), Utf8Error> {
        let printable_shell_command =
            PrintableShellCommand::new("/Applications/My App.app/Contents/Resources/my-app");
        assert_eq!(
            printable_shell_command.printable_invocation_string_with_options(
                FormattingOptions {
                    argument_line_wrapping: Some(crate::ArgumentLineWrapping::ByArgument),
                    ..Default::default()
                }
            )?,
            "'/Applications/My App.app/Contents/Resources/my-app'"
        );
        Ok(())
    }

    #[test]
    fn command_with_equal_sign_is_escaped_by_default() -> Result<(), Utf8Error> {
        let printable_shell_command = PrintableShellCommand::new("THIS_LOOKS_LIKE_AN=env-var");
        assert_eq!(
            printable_shell_command.printable_invocation_string_with_options(
                FormattingOptions {
                    argument_line_wrapping: Some(crate::ArgumentLineWrapping::ByArgument),
                    ..Default::default()
                }
            )?,
            "'THIS_LOOKS_LIKE_AN=env-var'"
        );
        Ok(())
    }
}
