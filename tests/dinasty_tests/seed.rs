use std::str::from_utf8;

use crate::dinasty_tests::dinasty;

#[test]
fn help() {
    let output = dinasty(vec!["seed".to_string()], None);
    assert!(output.status.success());
    let stdout = from_utf8(&output.stdout).unwrap();

    assert!(stdout.starts_with("Usage: dinasty [OPTIONS] <COMMAND>"));
}
