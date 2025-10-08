#![allow(dead_code)]

use bson::oid::ObjectId;
use serde::{Deserialize, Deserializer, Serializer};

/// Deserialize either String or ObjectId as String
pub fn deserialize<'de, D>(deserializer: D) -> Result<String, D::Error>
where
    D: Deserializer<'de>,
{
    #[derive(Deserialize)]
    #[serde(untagged)]
    enum StringOrObjectId {
        String(String),
        ObjectId(ObjectId),
    }

    match StringOrObjectId::deserialize(deserializer)? {
        StringOrObjectId::String(s) => Ok(s),
        StringOrObjectId::ObjectId(oid) => Ok(oid.to_hex()),
    }
}

/// Serialize String as-is
pub fn serialize<S>(value: &String, serializer: S) -> Result<S::Ok, S::Error>
where
    S: Serializer,
{
    serializer.serialize_str(value)
}

/// Deserialize Option<String> or Option<ObjectId> as Option<String>
pub fn deserialize_option<'de, D>(deserializer: D) -> Result<Option<String>, D::Error>
where
    D: Deserializer<'de>,
{
    #[derive(Deserialize)]
    #[serde(untagged)]
    enum StringOrObjectId {
        String(String),
        ObjectId(ObjectId),
    }

    let opt = Option::<StringOrObjectId>::deserialize(deserializer)?;
    match opt {
        Some(StringOrObjectId::String(s)) => Ok(Some(s)),
        Some(StringOrObjectId::ObjectId(oid)) => Ok(Some(oid.to_hex())),
        None => Ok(None),
    }
}

/// Serialize Option<String> as-is
pub fn serialize_option<S>(value: &Option<String>, serializer: S) -> Result<S::Ok, S::Error>
where
    S: Serializer,
{
    match value {
        Some(s) => serializer.serialize_some(s),
        None => serializer.serialize_none(),
    }
}
