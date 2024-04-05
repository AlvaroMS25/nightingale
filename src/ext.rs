use std::future::Future;
use async_trait::async_trait;
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

#[async_trait]
pub trait AsyncOptionExt<T> {

    async fn async_map<Fun, Fut, Ret>(self, predicate: Fun) -> Option<Ret>
    where
        Fun: FnOnce(T) -> Fut + Send,
        Fut: Future<Output = Ret> + Send;
}

#[async_trait]
impl<T: Send> AsyncOptionExt<T> for Option<T> {
    async fn async_map<Fun, Fut, Ret>(self, predicate: Fun) -> Option<Ret>
    where
        Fun: FnOnce(T) -> Fut + Send,
        Fut: Future<Output=Ret> + Send
    {
        match self {
            Some(inner) => Some(predicate(inner).await),
            None => None
        }
    }
}

#[async_trait]
pub trait AsyncIteratorExt: Iterator + Sized {
    async fn async_map<Fun, Fut, Ret, Container>(mut self, mut fun: Fun) -> Container
    where
        Self: Send,
        Self::Item: Send,
        Fun: FnMut(Self::Item) -> Fut + Send,
        Fut: Future<Output = Ret> + Send,
        Ret: Send,
        Container: GrowableContainer<Ret> + Send
    {
        let mut out = Container::new();

        for item in self {
            out.push(fun(item).await);
        }

        out
    }
}

pub trait GrowableContainer<T>: Sized {
    fn new() -> Self;
    fn push(&mut self, item: T);
}

impl<T> GrowableContainer<T> for Vec<T> {
    fn new() -> Self {
        Self::new()
    }

    fn push(&mut self, item: T) {
        self.push(item);
    }
}

impl<I> AsyncIteratorExt for I where I: Iterator {}

pub trait VecExt<T> {
    fn remove_optional(&mut self, _idx: usize) -> Option<T>;
}

impl<T> VecExt<T> for Vec<T> {
    fn remove_optional(&mut self, idx: usize) -> Option<T> {
        if self.len() > idx {
            Some(self.remove(idx))
        } else {
            None
        }
    }
}
