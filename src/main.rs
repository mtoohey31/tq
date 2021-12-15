use json::JsonValue;
use lazy_static::lazy_static;
use regex::Regex;
use std::env;
use std::error::Error;
use std::ffi::OsString;
use std::fs::read_to_string;
use std::fs::File;
use std::path::Path;
use std::process::Command;
use tempdir::TempDir;
use toml::Value as TomlValue;

fn main() -> Result<(), Box<dyn Error>> {
    let tmp_dir = TempDir::new("tq")?;
    let tmp_dir_path = tmp_dir.path();
    Command::new("jq")
        .args(
            env::args_os()
                .skip(1)
                .map(|arg| transform_argument(arg, tmp_dir_path))
                .collect::<Vec<OsString>>(),
        )
        .status()
        .expect("jq failed to start");
    tmp_dir.close()?;
    Ok(())
}

fn transform_argument(arg: OsString, tmp_dir_path: &Path) -> OsString {
    lazy_static! {
        static ref RE: Regex = Regex::new(r"^.*\.toml$").unwrap();
    }
    let lossy_arg = arg.clone();
    let lossy_arg = lossy_arg.to_string_lossy();
    if RE.is_match(&lossy_arg) {
        let toml_value: TomlValue = toml::from_str(
            &read_to_string(&arg)
                .unwrap_or_else(|err| panic!("error reading {}: {}", lossy_arg, err)),
        )
        .unwrap_or_else(|err| panic!("error parsing {}: {}", lossy_arg, err));
        let tmp_file_path = tmp_dir_path.join(arg);
        to_json(toml_value)
            .write(
                &mut File::create(&tmp_file_path)
                    .unwrap_or_else(|err| panic!("error opening temp file: {}", err)),
            )
            .unwrap_or_else(|err| panic!("error writing to temp file: {}", err));
        tmp_file_path
            .canonicalize()
            .unwrap()
            .as_os_str()
            .to_os_string()
    } else {
        arg
    }
}

fn to_json(value: TomlValue) -> JsonValue {
    match value {
        TomlValue::Boolean(b) => JsonValue::Boolean(b),
        TomlValue::Integer(i) => JsonValue::Number(i.into()),
        TomlValue::Float(f) => JsonValue::Number(f.into()),
        TomlValue::String(s) => JsonValue::String(s),
        TomlValue::Array(a) => {
            JsonValue::Array(a.into_iter().map(to_json).collect::<Vec<JsonValue>>())
        }
        TomlValue::Table(t) => {
            let mut obj = json::object::Object::new();
            for (k, v) in t {
                obj.insert(&k, to_json(v));
            }
            JsonValue::Object(obj)
        }
        TomlValue::Datetime(dt) => JsonValue::String(dt.to_string()),
    }
}
