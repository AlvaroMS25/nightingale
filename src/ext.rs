use std::io::{Read, Write};
use serde::de::DeserializeOwned;
use serde::Serialize;

pub trait ReadExt: Read {
    fn deserialize<M>(self) -> serde_json::Result<M>
    where
        Self: Sized,
        M: DeserializeOwned
    {
        serde_json::from_reader(self)
    }
}

pub trait WriteExt: Write {
    fn serialize<M>(self, model: &M) -> serde_json::Result<()>
    where
        Self: Sized,
        M: Serialize + ?Sized
    {
        serde_json::to_writer(self, model)
    }
}

impl<R: Read> ReadExt for R {}
impl<W: Write> WriteExt for W {}
