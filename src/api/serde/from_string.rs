use std::fmt::Display;
use std::str::FromStr;
use serde::{Deserialize, Deserializer};
use serde::de::Error;

pub struct FromString<T>(pub T);

impl<'de, T> Deserialize<'de> for FromString<T>
where
    T: FromStr,
    T::Err: Display
{
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        String::deserialize(deserializer)?
            .parse::<T>()
            .map_err(|e| Error::custom(e.to_string()))
            .map(FromString)
    }
}
