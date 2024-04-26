use std::{
    fs::File,
    path::{Path, PathBuf},
};

use anyhow::{anyhow, bail, Context};
use indexmap::IndexSet;
use serde::de::DeserializeOwned;
use serde_yaml::{value::TaggedValue, Mapping, Value};

pub fn handle_mapping_yaml<P: AsRef<Path>>(path: P) -> anyhow::Result<Mapping> {
    let path = path.as_ref();
    match handle_yaml(path)? {
        Value::Mapping(map) => Ok(map),
        _ => bail!("Expected a mapping"),
    }
}

pub fn handle_yaml<P: AsRef<Path>>(path: P) -> anyhow::Result<Value> {
    let path = path.as_ref().canonicalize()?;
    let v = load_internal(&path)?;

    handle_include(
        match v {
            Value::Mapping(map) => Value::Mapping(handle_extends(map, &path)?),
            _ => v,
        },
        path.as_ref(),
    )
}

fn handle_extends<P: AsRef<Path>>(mut map: Mapping, path: P) -> anyhow::Result<Mapping> {
    let extend_key = Value::String(String::from("extend"));
    let extend = map.get(&extend_key).cloned();
    let dir = path.as_ref().parent().unwrap();
    if let Some(extend) = extend {
        match extend {
            Value::Sequence(s) => {
                let mut ext_map = Mapping::new();
                for extend_path in s {
                    let extend_path: PathBuf = serde_yaml::from_value(extend_path)?;
                    let extend_path = from_relative_path(extend_path, dir)?;
                    let extended_values = handle_mapping_yaml(extend_path)?;
                    ext_map = merge_mapping(&extended_values, &ext_map);
                }
                map = merge_mapping(&map, &ext_map);
            }
            Value::String(extend_path) => {
                map = extend_by(map, dir, extend_path)?;
            }
            _ => {
                bail!(
                    "extend must be a sequence or a string, in {}",
                    path.as_ref().display()
                );
            }
        }
    }

    map.remove(&extend_key);

    return Ok(map);
}

fn extend_by<P: AsRef<Path>>(values: Mapping, root_dir: &Path, path: P) -> anyhow::Result<Mapping> {
    let path = from_relative_path(path, root_dir)?;
    let extended_values = handle_mapping_yaml(path)?;
    Ok(merge_mapping(&values, &extended_values))
}

fn merge_value(default: &Value, value: &Value) -> Value {
    match (default, value) {
        (Value::Mapping(default), Value::Mapping(value)) => {
            Value::from(merge_mapping(value, default))
        }
        (x, Value::Null) => x.clone(),
        (_, x) => x.clone(),
    }
}

fn handle_include(value: Value, path: &Path) -> anyhow::Result<Value> {
    match value {
        Value::Mapping(map) => {
            let mut result = Mapping::new();
            for (key, value) in map {
                result.insert(key, handle_include(value, path)?);
            }
            Ok(Value::Mapping(result))
        }
        Value::Sequence(seq) => {
            let mut result = Vec::new();
            for value in seq {
                result.push(handle_include(value, &path)?);
            }
            Ok(Value::Sequence(result))
        }
        Value::Tagged(value) => load_include(value, path)
            .context(format!("Failed to load import at '{}'", path.display())),
        _ => Ok(value),
    }
}

fn load_include(value: Box<TaggedValue>, path: &Path) -> anyhow::Result<Value> {
    let dir = path
        .parent()
        .ok_or(anyhow!("Failed to get parent directory"))?;

    if value.tag == "!include" {
        match &value.value {
            Value::String(import_path) => {
                let import_path = from_relative_path(import_path, dir)?;
                return handle_yaml(&import_path).context(format!(
                    "Failed load import '{}' in {}",
                    import_path.display(),
                    path.display()
                ));
            }
            _ => bail!("!import should be a string"),
        }
    }

    Ok(serde_yaml::to_value(value)?)
}

fn from_relative_path<P: AsRef<Path>>(path: P, dir: &Path) -> anyhow::Result<PathBuf> {
    let path = path.as_ref();

    Ok(if path.is_absolute() {
        path.to_path_buf()
    } else {
        dir.join(path)
            .canonicalize()
            .context(format!("File '{}' not found", path.display()))?
    })
}

fn merge_mapping(value_map: &Mapping, default_map: &Mapping) -> Mapping {
    let mut result: Mapping = Mapping::new();
    let mut keys = value_map.keys().map(|k| k.clone()).collect::<IndexSet<_>>();

    keys.extend(default_map.keys().map(|k| k.clone()));

    for key in keys {
        if let Some("extend") = key.as_str() {
            continue;
        }
        let new_value = merge_value(
            default_map.get(&key).unwrap_or(&Value::Null),
            value_map.get(&key).unwrap_or(&Value::Null),
        );
        result.insert(key, new_value);
    }

    return result;
}

fn load_internal<T>(path: &Path) -> anyhow::Result<T>
where
    T: DeserializeOwned,
{
    let path_string = path.display();
    let reader = File::open(&path).context(format!("Failed to open file {}", path_string))?;
    Ok(serde_yaml::from_reader(reader)
        .context(format!("Failed to parse yaml in file {}", path_string))?)
}
