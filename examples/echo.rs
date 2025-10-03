use std::process::Command;

use printable_shell_command::ShellPrintable;

fn main() {
    let _ = Command::new("echo").args(["#hi"]).print_invocation();
}
