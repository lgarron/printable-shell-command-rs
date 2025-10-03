# `printable_shell_command`

Rust port of: https://github.com/lgarron/printable-shell-command

A helper library to print shell commands.

The goal is to make it easy to print commands that are being run by a program, in a way that makes it easy and safe for a user to copy-and-paste.

## Example

```rust
use std::process::Command;

use printable_shell_command_rs::ShellPrintable;

fn main() {
    let _ = Command::new("echo").args(["#hi"]).print_invocation();
}
```
