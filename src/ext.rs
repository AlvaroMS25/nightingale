use serde_json::{Map, Number, Value};

pub trait JsonValueExt {
    fn into_array(self) -> Option<Vec<Value>>;
    fn into_number(self) -> Option<Number>;
    fn into_object(self) -> Option<Map<String, Value>>;
    fn into_string(self) -> Option<String>;
    fn get_owned_object(&mut self, key: &str) -> Option<Map<String, Value>>;
    fn get_owned_array(&mut self, key: &str) -> Option<Vec<Value>>;
    fn get_owned_string(&mut self, key: &str) -> Option<String>;
}

impl JsonValueExt for Value {
    fn into_array(self) -> Option<Vec<Value>> {
        match self {
            Self::Array(a) => Some(a),
            _ => None
        }
    }

    fn into_number(self) -> Option<Number> {
        match self {
            Self::Number(n) => Some(n),
            _ => None
        }
    }

    fn into_object(self) -> Option<Map<String, Value>> {
        match self {
            Self::Object(o) => Some(o),
            _ => None
        }
    }

    fn into_string(self) -> Option<String> {
        match self {
            Self::String(s) => Some(s),
            _ => None
        }
    }

    fn get_owned_object(&mut self, key: &str) -> Option<Map<String, Value>> {
        self.as_object_mut()?.remove(key)?.into_object()
    }

    fn get_owned_array(&mut self, key: &str) -> Option<Vec<Value>> {
        self.as_object_mut()?.remove(key)?.into_array()
    }

    fn get_owned_string(&mut self, key: &str) -> Option<String> {
        self.as_object_mut()?.remove(key)?.into_string()
    }
}