use std::error::Error;
use std::fmt::{Debug, Display, Formatter};
use songbird::input::AuxMetadata;

pub struct StringError(String);

impl<T: Error> From<T> for StringError {
    fn from(value: T) -> Self {
        Self(value.to_string())
    }
}

impl Debug for StringError {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        <String as Debug>::fmt(&self.0, f)
    }
}

impl Display for StringError {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        <String as Display>::fmt(&self.0, f)
    }
}
