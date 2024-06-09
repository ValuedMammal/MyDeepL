//
use assert_cmd::Command;
use std::env;
use std::str;

const KEY: &'static str = env!("DEEPL_API_KEY");
const PROG: &'static str = "deepl";

#[test]
fn languages() {
    let _ = Command::cargo_bin(PROG)
        .unwrap()
        .env("DEEPL_API_KEY", KEY)
        .arg("languages")
        .assert()
        .success();
}

#[test]
fn translate_error() {
    let res = Command::cargo_bin(PROG)
        .unwrap()
        .env("DEEPL_API_KEY", KEY)
        .write_stdin("good morning")
        .arg("text")
        .args(["-s", "EN-GB", "-t", "PT"])
        .output()
        .unwrap();

    assert!(res.stdout.is_empty());

    let stderr = res.stderr;
    //let err = str::from_utf8(&stderr).unwrap();
    //dbg!(err);
    assert!(!stderr.is_empty());
}

#[rustfmt::skip]
#[test]
fn glossary_create() {
    // pass entries in cli options
    // test with trailing comma
    let entries = "hello=ciao, goodbye=ciao,";
    let res = Command::cargo_bin(PROG)
        .unwrap()
        .env("DEEPL_API_KEY", KEY)
        .arg("glossary")
        .arg("create")
        .args(["--name", "en-it", "-s", "EN", "-t", "IT", "--entries", entries])
        .output()
        .unwrap();

    assert!(!res.stdout.is_empty());
}
