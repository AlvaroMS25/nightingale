use std::future::Future;

pub trait AsyncOptionExt<T> {

    async fn async_map<Fun, Fut, Ret>(self, predicate: Fun) -> Option<Ret>
    where
        Fun: FnOnce(T) -> Fut + Send,
        Fut: Future<Output = Ret> + Send;
}

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
