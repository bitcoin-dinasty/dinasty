use std::{
    io::Write,
    process::{Output, Stdio},
};

mod full;
mod help;

fn dinasty(args: Vec<String>, stdin_string: Option<String>) -> Output {
    let stdin = stdin_string
        .as_ref()
        .map(|_| Stdio::piped())
        .unwrap_or(Stdio::null());
    let exe = "./target/debug/dinasty";
    let mut child = std::process::Command::new(exe)
        .args(args)
        .stdin(stdin)
        .stderr(Stdio::piped())
        .stdout(Stdio::piped())
        .spawn()
        .unwrap();
    if let Some(stdin_string) = stdin_string {
        child
            .stdin
            .take()
            .unwrap()
            .write_all(stdin_string.as_bytes())
            .unwrap();
    }
    child.wait_with_output().unwrap()
}
