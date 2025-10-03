use std::process::Command;

use printable_shell_command::ShellPrintableRef;

fn main() {
    let _ = Command::new("echo").args(["#hi"]).print_invocation();
}
