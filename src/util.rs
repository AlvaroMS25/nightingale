use std::io::Read;
use serde::de::DeserializeOwned;
use crate::error::Error;

#[allow(unused)]
pub fn deserialize_json<R, M>(source: R) -> Result<M, Error>
where
    R: Read,
    M: DeserializeOwned
{
    serde_json::from_reader(source)
        .map_err(From::from)
}

pub fn deserialize_yaml<R, M>(source: R) -> Result<M, Error>
where
    R: Read,
    M: DeserializeOwned
{
    serde_yaml::from_reader(source)
        .map_err(From::from)
}
