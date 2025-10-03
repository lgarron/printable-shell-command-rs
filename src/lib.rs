use std::{process::Command, str::Utf8Error, sync::LazyLock};

use regex::Regex;

pub trait ShellPrintable {
    fn printable_invocation_string(&mut self) -> Result<String, Utf8Error>;
    // Calls `.to_string_lossy()` on the program name and args.
    fn printable_invocation_string_lossy(&mut self) -> String;

    // Print the invocation to `stdout`.`
    fn print_invocation(&mut self) -> Result<&mut Self, Utf8Error> {
        println!("{}", self.printable_invocation_string_lossy());
        Ok(self)
    }
    // Print the invocation to `stdout`.`
    fn print_invocation_lossy(&mut self) -> &mut Self {
        println!("{}", self.printable_invocation_string_lossy());
        self
    }
}
struct SimpleEscapeOptions {
    is_command: bool,
}

static PROGRAM_NAME_REGEX: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r#"[ "'`|$*?><()\[\]{}&\\;#=]"#).unwrap());
static ARG_REGEX: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r#"[ "'`|$*?><()\[\]{}&\\;#]"#).unwrap());

fn simple_escape(s: &str, options: SimpleEscapeOptions) -> String {
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

impl ShellPrintable for Command {
    fn printable_invocation_string_lossy(&mut self) -> String {
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
    fn printable_invocation_string(&mut self) -> Result<String, Utf8Error> {
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

#[cfg(test)]
mod tests {
    use crate::ShellPrintable;
    use std::process::Command;

    #[test]
    fn my_test() -> Result<(), String> {
        let _ = Command::new("echo").args(["#hi"]).print_invocation();
        Ok(())
    }
}
