use assert_cmd::prelude::*;
use predicates::prelude::*;
use std::process::Command;

mod helpers;

#[test]
fn invalid_path() -> Result<(), Box<dyn std::error::Error>> {
    let mut cmd = Command::cargo_bin("j2a")?;

    cmd.arg("--path").arg("test/file/doesnt/exist");
    cmd.assert()
        .failure()
        .stderr(predicate::str::contains("Unable to read file extension."));

    Ok(())
}

#[test]
fn file_without_json_extension() -> Result<(), Box<dyn std::error::Error>> {
    let mut cmd = Command::cargo_bin("j2a")?;

    cmd.arg("--path").arg("test/file/doesnt/exist.txt");
    cmd.assert()
        .failure()
        .stderr(predicate::str::contains("Expected path to JSON file."));

    Ok(())
}

#[test]
fn invalid_json_content() -> Result<(), Box<dyn std::error::Error>> {
    let mut cmd = Command::cargo_bin("j2a")?;

    cmd.arg("--path").arg("./tests/files/invalid-json.json");
    cmd.assert()
        .failure()
        .stderr(predicate::str::contains("File contains invalid JSON"));

    Ok(())
}

#[test]
fn converts_to_azure_syntax() -> Result<(), Box<dyn std::error::Error>> {
    let mut cmd = Command::cargo_bin("j2a")?;

    cmd.arg("--path").arg("./tests/files/valid.json");

    let expected_output = r#"
    CONNECTION STRINGS
    [
        {
            "name": "SomeConnectionString",
            "value": "my-stringy-value",
            "slot_setting": true
        }
    ]

    ENV VARS
    [
        {
            "name": "Serilog:MinimumLevel:Default",
            "value": "Information",
            "slot_setting": true
        },
        {
            "name": "Serilog:WriteTo:0:Name",
            "value": "Console",
            "slot_setting": true
        }
    ]
    "#;

    cmd.assert()
        .success()
        .stdout(predicates::function::function(|output: &str| {
            // TODO: include whitespace in tests.
            return helpers::remove_whitespace(output)
                == helpers::remove_whitespace(expected_output);
        }))
        .stderr(predicate::str::is_empty());

    Ok(())
}
