use std::{
    fs::File,
    path::{Path, PathBuf},
};

use anyhow::{bail, Context};
use indexmap::IndexSet;
use serde::de::DeserializeOwned;
use serde_yaml::{value::TaggedValue, Mapping, Value};

pub fn handle_mapping_yaml<P: AsRef<Path>>(path: P) -> anyhow::Result<Mapping> {
    match handle_yaml(path)? {
        Value::Mapping(map) => Ok(map),
        _ => bail!("Expected a mapping"),
    }
}

pub fn handle_yaml<P: AsRef<Path>>(path: P) -> anyhow::Result<Value> {
    let v = load_internal(&path)?;

    let path = path.as_ref().canonicalize()?;

    handle_import(
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
                for extend_path in s {
                    let extend_path: PathBuf = serde_yaml::from_value(extend_path)?;
                    map = extend_by(map, dir, extend_path)?;
                }
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
    let path = path.as_ref();
    let path = if path.is_absolute() {
        path.to_path_buf()
    } else {
        root_dir.join(path)
    };
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

fn handle_import(value: Value, path: &Path) -> anyhow::Result<Value> {
    match value {
        Value::Mapping(map) => {
            let mut result = Mapping::new();
            for (key, value) in map {
                result.insert(key, handle_import(value, path)?);
            }
            Ok(Value::Mapping(result))
        }
        Value::Sequence(seq) => {
            let mut result = Vec::new();
            for value in seq {
                result.push(handle_import(value, &path)?);
            }
            Ok(Value::Sequence(result))
        }
        Value::Tagged(value) => load_import(value, path),
        _ => Ok(value),
    }
}

fn load_import(value: Box<TaggedValue>, path: &Path) -> anyhow::Result<Value> {
    let dir = path.parent().unwrap();
    if value.tag == "!import" {
        match &value.value {
            Value::String(path) => {
                let path = PathBuf::from(path);
                let path = if path.is_absolute() {
                    path.to_path_buf()
                } else {
                    dir.join(path)
                };
                return handle_yaml(path);
            }
            _ => bail!("!import should be a string"),
        }
    }

    Ok(serde_yaml::to_value(value)?)
}

fn merge_mapping(value_map: &Mapping, patch_map: &Mapping) -> Mapping {
    let mut result: Mapping = Mapping::new();
    let mut keys = value_map.keys().map(|k| k.clone()).collect::<IndexSet<_>>();

    keys.extend(patch_map.keys().map(|k| k.clone()));

    for key in keys {
        if let Some("extend") = key.as_str() {
            continue;
        }
        let new_value = merge_value(
            patch_map.get(&key).unwrap_or(&Value::Null),
            value_map.get(&key).unwrap_or(&Value::Null),
        );
        result.insert(key, new_value);
    }

    return result;
}

fn load_internal<'de, T, P>(path: P) -> anyhow::Result<T>
where
    T: DeserializeOwned,
    P: AsRef<Path>,
{
    let path_string = path.as_ref().display();
    let reader = File::open(&path).context(format!("Failed to open file {}", path_string))?;
    Ok(serde_yaml::from_reader(reader)
        .context(format!("Failed to parse yaml in file {}", path_string))?)
}
