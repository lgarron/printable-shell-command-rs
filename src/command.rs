use std::{process::Command, str::Utf8Error};

use crate::{
    ShellPrintable,
    escape::{SimpleEscapeOptions, simple_escape},
    shell_printable::ShellPrintableRef,
};

impl ShellPrintable for Command {
    fn printable_invocation_string_lossy(&self) -> String {
        let mut lines: Vec<String> = vec![simple_escape(
            &self.get_program().to_string_lossy(),
            SimpleEscapeOptions { is_command: true },
        )];
        for arg in self.get_args() {
            lines.push(simple_escape(
                &arg.to_string_lossy(),
                SimpleEscapeOptions { is_command: false },
            ))
        }
        lines.join(
            " \\
  ",
        )
    }
    fn printable_invocation_string(&self) -> Result<String, Utf8Error> {
        let mut lines: Vec<String> = vec![simple_escape(
            TryInto::<&str>::try_into(self.get_program())?,
            SimpleEscapeOptions { is_command: true },
        )];
        for arg in self.get_args() {
            lines.push(simple_escape(
                TryInto::<&str>::try_into(arg)?,
                SimpleEscapeOptions { is_command: false },
            ))
        }
        Ok(lines.join(
            " \\
  ",
        ))
    }
}

impl ShellPrintableRef for Command {}

#[cfg(test)]
mod tests {
    use std::process::Command;

    use crate::shell_printable::ShellPrintableRef;

    #[test]
    fn my_test() -> Result<(), String> {
        let _ = Command::new("echo").args(["#hi"]).print_invocation();
        Ok(())
    }
}
