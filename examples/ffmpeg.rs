use printable_shell_command::{PrintableShellCommand, ShellPrintableSelf};

fn main() {
    let _ = PrintableShellCommand::new("ffmpeg")
        .args(["-i", "./test/My video.mp4"])
        .args(["-filter:v", "setpts=2.0*PTS"])
        .args(["-filter:a", "atempo=0.5"])
        .args(["-filter:a", "atempo=0.5"])
        .arg("./test/My video (slow-mo).mov")
        .print_invocation()
        .unwrap()
        .command();
}
