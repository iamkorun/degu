use assert_cmd::Command;
use predicates::prelude::*;
use std::io::Write;

fn degu() -> Command {
    Command::cargo_bin("degu").unwrap()
}

#[test]
fn json_schema_from_stdin() {
    degu()
        .write_stdin(r#"{"a":1,"b":"x"}"#)
        .assert()
        .success()
        .stdout(predicate::str::contains("\"$schema\""))
        .stdout(predicate::str::contains("\"a\""))
        .stdout(predicate::str::contains("\"integer\""))
        .stdout(predicate::str::contains("\"required\": [\"a\", \"b\"]"));
}

#[test]
fn typescript_output() {
    degu()
        .args(["--format", "typescript"])
        .write_stdin(r#"[{"a":1,"b":"x"},{"a":2}]"#)
        .assert()
        .success()
        .stdout(predicate::str::contains("export interface Root"))
        .stdout(predicate::str::contains("a: number;"))
        .stdout(predicate::str::contains("b?: string;"));
}

#[test]
fn typescript_strict_all_required() {
    degu()
        .args(["--format", "ts", "--strict"])
        .write_stdin(r#"[{"a":1,"b":"x"},{"a":2}]"#)
        .assert()
        .success()
        .stdout(predicate::str::contains("b: string;"))
        .stdout(predicate::str::contains("a: number;"));
}

#[test]
fn zod_output() {
    degu()
        .args(["--format", "zod", "--name", "User"])
        .write_stdin(r#"{"id":1,"name":"a","tags":["x","y"]}"#)
        .assert()
        .success()
        .stdout(predicate::str::contains("import { z }"))
        .stdout(predicate::str::contains("export const User ="))
        .stdout(predicate::str::contains("z.array(z.string())"))
        .stdout(predicate::str::contains("z.number().int()"));
}

#[test]
fn nullable_union() {
    degu()
        .args(["--format", "ts"])
        .write_stdin(r#"[{"a":1},{"a":null},{"a":"x"}]"#)
        .assert()
        .success()
        .stdout(predicate::str::contains("number | string | null"));
}

#[test]
fn file_input_merging() {
    let dir = tempfile::tempdir().unwrap();
    let a = dir.path().join("a.json");
    let b = dir.path().join("b.json");
    std::fs::write(&a, r#"{"id":1,"name":"x"}"#).unwrap();
    std::fs::write(&b, r#"{"id":2}"#).unwrap();

    degu()
        .args(["--format", "ts"])
        .arg(&a)
        .arg(&b)
        .assert()
        .success()
        .stdout(predicate::str::contains("id: number;"))
        .stdout(predicate::str::contains("name?: string;"));
}

#[test]
fn malformed_json_errors_cleanly() {
    degu()
        .write_stdin("{not json")
        .assert()
        .failure()
        .stderr(predicate::str::contains("failed to parse JSON"));
}

#[test]
fn top_level_array_of_scalars() {
    degu()
        .args(["--format", "ts"])
        .write_stdin("[1, 2, 3]")
        .assert()
        .success()
        .stdout(predicate::str::contains("number[]"));
}

#[test]
fn json_schema_strict_has_additional_properties_false() {
    degu()
        .args(["--strict"])
        .write_stdin(r#"{"a":1}"#)
        .assert()
        .success()
        .stdout(predicate::str::contains("\"additionalProperties\": false"));
}

#[test]
fn version_flag() {
    degu()
        .arg("--version")
        .assert()
        .success()
        .stdout(predicate::str::contains("degu"));
}

#[test]
fn help_flag() {
    degu()
        .arg("--help")
        .assert()
        .success()
        .stdout(predicate::str::contains("format"))
        .stdout(predicate::str::contains("strict"));
}

#[test]
fn no_input_errors() {
    // Provide empty stdin (not a tty in assert_cmd).
    let mut cmd = degu();
    cmd.write_stdin("");
    cmd.assert()
        .failure()
        .stderr(predicate::str::contains("no input").or(predicate::str::contains("parse")));
    let _ = writeln!(std::io::stderr());
}
