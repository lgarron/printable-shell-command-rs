# `printable_shell_command`

Rust port of: https://github.com/lgarron/printable-shell-command

A helper library to print shell commands.

The goal is to make it easy to print commands that are being run by a program, in a way that makes it easy and safe for a user to copy-and-paste.

## Examples

### Using `Command` directly

```rust
use std::process::Command;

use printable_shell_command::ShellPrintable;

fn main() {
    let _ = Command::new("echo").args(["#hi"]).print_invocation();
}
```

Prints:

```text
echo \
  '#hi'
```

### Using `PrintableShellCommand` to group args

```rust
use printable_shell_command::{PrintableShellCommand, ShellPrintable};

fn main() {
    let _ = PrintableShellCommand::new("ffmpeg")
        .args(["-i", "./test/My video.mp4"])
        .args(["-filter:v", "setpts=2.0*PTS"])
        .args(["-filter:a", "atempo=0.5"])
        .arg("./test/My video (slow-mo).mov")
        .print_invocation()
        .unwrap();
}
```

Prints:

```text
ffmpeg \
  -i './test/My video.mp4' \
  -filter:v 'setpts=2.0*PTS' \
  -filter:a atempo=0.5 \
  './test/My video (slow-mo).mov'
```
