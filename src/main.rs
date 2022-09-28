use anyhow::anyhow;
use either::Either;
use json::JsonValue;
use std::{
    env,
    error::Error,
    ffi::{OsStr, OsString},
    fs::read_to_string,
    path::Path,
    process::Command,
};
use tempfile::TempPath;
use toml::Value as TomlValue;

fn main() -> Result<(), Box<dyn Error>> {
    let transformed = env::args_os()
        .skip(1)
        .map(transform_argument)
        .collect::<Result<Vec<_>, anyhow::Error>>()?;
    Command::new("jq")
        .args(transformed.iter().map(|e| match e {
            Either::Left(os) => os.as_os_str(),
            Either::Right(tp) => tp.as_os_str(),
        }))
        .status()
        .map_err(|err| anyhow!("jq failed to start: {}", err))?;
    Ok(())
}

fn transform_argument(arg: OsString) -> Result<Either<OsString, TempPath>, anyhow::Error> {
    if Path::new(&arg).extension() != Some(OsStr::new("toml")) {
        return Ok(Either::Left(arg));
    }

    let toml_value: TomlValue = toml::from_str(
        &read_to_string(&arg)
            .map_err(|err| anyhow!("error reading {}: {}", arg.to_string_lossy(), err))?,
    )
    .map_err(|err| anyhow!("error parsing {}: {}", arg.to_string_lossy(), err))?;
    let named_tmp_file = tempfile::NamedTempFile::new()
        .map_err(|err| anyhow!("error creating tempfile: {}", err))?;
    to_json(toml_value)
        .write(&mut named_tmp_file.as_file())
        .map_err(|err| anyhow!("error writing to temp file: {}", err))?;
    Ok(Either::Right(named_tmp_file.into_temp_path()))
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
