use clap::Parser;
use serde::Serialize;
use serde_json::Value;
use std::ffi::OsStr;
use std::fs::File;
use std::io::BufReader;
use std::process;
use valico::json_schema;

// Converts provided JSON file to Azure EnvVar notation.
// EXAMPLE
// [
//   {
//     "name": "Serilog:WriteTo:1:Args:apiKey",
//     "value": "be9b742e38a63b65aafe85aace7f2db6",
//     "slotSetting": true
//   },
//   --snip
// ]

#[derive(Parser)]
#[command(version, about, long_about = None)]
struct Cli {
    #[arg(short, long)]
    path: std::path::PathBuf,
}

#[derive(Serialize)]
struct AzureOutputEntry {
    name: String,
    value: String,
    slot_setting: bool,
}

fn traverse_json(
    json_value: &Value,
    parent_key: &str,
    connection_strings: &mut Vec<AzureOutputEntry>,
    env_vars: &mut Vec<AzureOutputEntry>,
) {
    match json_value {
        Value::Object(obj) => {
            for (key, value) in obj {
                let current_key = if parent_key.is_empty() {
                    key.clone()
                } else {
                    format!("{}:{}", parent_key, key)
                };
                traverse_json(value, &current_key, connection_strings, env_vars);
            }
        }
        Value::Array(array) => {
            for (idx, val) in array.iter().enumerate() {
                let current_key = format!("{}:{}", parent_key, idx);
                traverse_json(val, &current_key, connection_strings, env_vars);
            }
        }
        Value::String(str) => {
            let mut entry = AzureOutputEntry {
                name: parent_key.to_string(),
                value: str.to_string(),
                slot_setting: true,
            };

            if parent_key.starts_with("ConnectionStrings") {
                let name = parent_key.replace("ConnectionStrings:", "");
                entry.name = name;
                connection_strings.push(entry);
            } else {
                env_vars.push(entry);
            }
        }
        Value::Number(num) => {
            let entry = AzureOutputEntry {
                name: parent_key.to_string(),
                value: num.to_string(),
                slot_setting: true,
            };
            env_vars.push(entry);
        }
        Value::Bool(b) => {
            let entry = AzureOutputEntry {
                name: parent_key.to_string(),
                value: b.to_string(),
                slot_setting: true,
            };
            env_vars.push(entry);
        }
        Value::Null => {
            let entry = AzureOutputEntry {
                name: parent_key.to_string(),
                value: "null".to_string(),
                slot_setting: true,
            };
            env_vars.push(entry);
        }
    }
}

fn main() {
    let args = Cli::parse();

    // File is .json
    let file_extension = args
        .path
        .extension()
        .and_then(OsStr::to_str)
        .unwrap_or_else(|| {
            eprintln!("Error: Unable to read file extension.");
            process::exit(1);
        });

    if file_extension != "json" {
        eprintln!("Error: Expected path to JSON file.");
        process::exit(1);
    }

    let file = File::open(&args.path).unwrap();
    let reader = BufReader::new(file);

    // VALIDATES JSON FILE
    let untyped_json: Value = serde_json::from_reader(reader).unwrap_or_else(|err| {
        eprintln!("Error: {}", err);
        process::exit(1);
    });

    let mut scope = json_schema::Scope::new();
    let schema = scope
        .compile_and_return(untyped_json.clone(), false)
        .unwrap();

    if !schema.validate(&untyped_json).is_valid() {
        eprintln!("Error: File contains invalid json.");
        process::exit(1);
    }

    // Start conversion
    let mut connection_strings: Vec<AzureOutputEntry> = Vec::new();
    let mut env_vars: Vec<AzureOutputEntry> = Vec::new();
    traverse_json(&untyped_json, "", &mut connection_strings, &mut env_vars);

    println!();
    println!("CONNECTION STRINGS");
    println!();
    let connection_string_output = serde_json::to_string_pretty(&connection_strings)
        .expect("Failed to serialize connection strings.");
    println!("{}", connection_string_output);

    println!();
    println!();
    println!("ENV VARS");
    println!();
    let env_var_output =
        serde_json::to_string_pretty(&env_vars).expect("Failed to serialize Env Vars.");
    println!("{}", env_var_output);
}
