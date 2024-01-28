use std::fmt;
use std::num::NonZeroU64;
use serde::{Deserialize, Deserializer};
use serde::de::{Error, Unexpected, Visitor};

pub enum VoiceEvent {
    UpdateVoiceServer(UpdateVoiceServer),
    UpdateVoiceState(UpdateVoiceState)
}

#[derive(Deserialize, Debug)]
pub struct UpdateVoiceServer {
    pub endpoint: Option<String>,
    pub guild_id: NzU64,
    pub token: String,
}

#[derive(Deserialize, Debug)]
pub struct UpdateVoiceState {
    pub guild_id: Option<NzU64>,
    pub user_id: NzU64,
    pub session_id: String,
    pub channel_id: Option<NzU64>
}

pub struct NzU64(pub NonZeroU64);

impl fmt::Debug for NzU64 {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        <NonZeroU64 as fmt::Debug>::fmt(&self.0, f)
    }
}

impl<'de> Deserialize<'de> for NzU64 {
    fn deserialize<D: Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        Ok(Self(deserializer.deserialize_any(NzU64Visitor)?))
    }
}

struct NzU64Visitor;

impl<'de> Visitor<'de> for NzU64Visitor {
    type Value = NonZeroU64;

    fn expecting(&self, formatter: &mut fmt::Formatter) -> std::fmt::Result {
        formatter.write_str("Non zero discord snowflake")
    }

    fn visit_i64<E: Error>(self, v: i64) -> Result<Self::Value, E> {
        let u = u64::try_from(v)
            .map_err(|_| Error::invalid_value(Unexpected::Signed(v), &"Non zero integer"))?;

        self.visit_u64(u)
    }

    fn visit_u64<E: Error>(self, v: u64) -> Result<Self::Value, E> {
        NonZeroU64::new(v)
            .ok_or_else(|| Error::invalid_value(Unexpected::Unsigned(v), &"Non zero integer"))
    }

    fn visit_str<E: Error>(self, v: &str) -> Result<Self::Value, E> {
        let parsed = v.parse::<u64>()
            .map_err(|_| Error::invalid_value(Unexpected::Str(v), &"Non zero integer string"))?;

        self.visit_u64(parsed)
    }

    fn visit_newtype_struct<D: Deserializer<'de>>(self, deserializer: D) -> Result<Self::Value, D::Error> {
        deserializer.deserialize_any(NzU64Visitor)
    }
}
