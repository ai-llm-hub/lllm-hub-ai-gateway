#![allow(dead_code)]

use bson::oid::ObjectId;
use serde::{Deserialize, Deserializer, Serializer};

/// Serialize ObjectId as String
pub fn serialize<S>(oid: &ObjectId, serializer: S) -> Result<S::Ok, S::Error>
where
    S: Serializer,
{
    serializer.serialize_str(&oid.to_hex())
}

/// Deserialize ObjectId from either ObjectId or String
pub fn deserialize<'de, D>(deserializer: D) -> Result<ObjectId, D::Error>
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
        StringOrObjectId::String(s) => {
            ObjectId::parse_str(&s).map_err(serde::de::Error::custom)
        }
        StringOrObjectId::ObjectId(oid) => Ok(oid),
    }
}

/// Serialize Option<ObjectId> as Option<String>
pub fn serialize_option<S>(oid: &Option<ObjectId>, serializer: S) -> Result<S::Ok, S::Error>
where
    S: Serializer,
{
    match oid {
        Some(oid) => serializer.serialize_some(&oid.to_hex()),
        None => serializer.serialize_none(),
    }
}

/// Deserialize Option<ObjectId> from either ObjectId or String
pub fn deserialize_option<'de, D>(deserializer: D) -> Result<Option<ObjectId>, D::Error>
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
        Some(StringOrObjectId::String(s)) => {
            ObjectId::parse_str(&s).map(Some).map_err(serde::de::Error::custom)
        }
        Some(StringOrObjectId::ObjectId(oid)) => Ok(Some(oid)),
        None => Ok(None),
    }
}
