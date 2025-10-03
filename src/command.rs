use std::{ffi::OsStr, process::Command, str::Utf8Error};

use crate::{
    print_builder::PrintBuilder, shell_printable::ShellPrintableWithOptions, FormattingOptions,
    ShellPrintable,
};

pub(crate) fn add_arg_from_command_lossy(print_builder: &mut PrintBuilder, arg: &OsStr) {
    print_builder.add_single_arg(&arg.to_string_lossy());
}

pub(crate) fn add_arg_from_command(
    print_builder: &mut PrintBuilder,
    arg: &OsStr,
) -> Result<(), Utf8Error> {
    print_builder.add_single_arg(TryInto::<&str>::try_into(arg)?);
    Ok(())
}

impl ShellPrintableWithOptions for Command {
    fn printable_invocation_string_lossy_with_options(
        &self,
        formatting_options: FormattingOptions,
    ) -> String {
        let mut print_builder = PrintBuilder::new(formatting_options);
        print_builder.add_program_name(&self.get_program().to_string_lossy());
        for arg in self.get_args() {
            add_arg_from_command_lossy(&mut print_builder, arg);
        }
        print_builder.get()
    }

    fn printable_invocation_string_with_options(
        &self,
        formatting_options: FormattingOptions,
    ) -> Result<String, Utf8Error> {
        let mut print_builder = PrintBuilder::new(formatting_options);
        print_builder.add_program_name(TryInto::<&str>::try_into(self.get_program())?);
        for arg in self.get_args() {
            add_arg_from_command(&mut print_builder, arg)?;
        }
        Ok(print_builder.get())
    }
}

impl ShellPrintable for Command {
    fn printable_invocation_string(&self) -> Result<String, Utf8Error> {
        self.printable_invocation_string_with_options(Default::default())
    }

    fn printable_invocation_string_lossy(&self) -> String {
        self.printable_invocation_string_lossy_with_options(Default::default())
    }
}

#[cfg(test)]
mod tests {
    use std::process::Command;

    use crate::ShellPrintable;

    #[test]
    fn echo() -> Result<(), String> {
        let mut command = Command::new("echo");
        command.args(["#hi"]);
        // Not printed by tests, but we can at least check this doesn't panic.
        let _ = command.print_invocation();

        assert_eq!(
            command.printable_invocation_string().unwrap(),
            "echo \\
  '#hi'"
        );
        Ok(())
    }
}
