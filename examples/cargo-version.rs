use printable_shell_command::{PrintableShellCommand, ShellPrintable};

fn main() {
    assert!(PrintableShellCommand::new("cargo")
        .arg("--version")
        .print_invocation()
        .unwrap()
        .status()
        .unwrap()
        .success());
}
