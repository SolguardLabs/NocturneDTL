use crate::errors::Result;
use serde::de::DeserializeOwned;
use serde::Serialize;

pub fn from_json<T: DeserializeOwned>(input: &str) -> Result<T> {
    Ok(serde_json::from_str(input)?)
}

pub fn to_json<T: Serialize>(value: &T) -> Result<String> {
    Ok(serde_json::to_string_pretty(value)?)
}

pub fn to_json_line<T: Serialize>(value: &T) -> Result<String> {
    Ok(serde_json::to_string(value)?)
}
