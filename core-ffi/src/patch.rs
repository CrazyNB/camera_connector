use serde::{de::Error as DeError, Deserialize, Deserializer};
use serde_json::Value;

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub(crate) enum JsonPatchField<T> {
    #[default]
    Missing,
    Null,
    Value(T),
}

pub(crate) fn deserialize_patch_field<'de, D, T>(
    deserializer: D,
) -> Result<JsonPatchField<T>, D::Error>
where
    D: Deserializer<'de>,
    T: Deserialize<'de>,
{
    let value = Option::<Value>::deserialize(deserializer)?;
    match value {
        None => Ok(JsonPatchField::Null),
        Some(value) => T::deserialize(value)
            .map(JsonPatchField::Value)
            .map_err(D::Error::custom),
    }
}
