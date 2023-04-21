use std::future::Future;

pub trait AsyncResultExt<T, E> {
    fn async_map_err<Fun, Fut, Ret>(
        self,
        map_fn: Fun,
    ) -> Pin<Box<dyn Future<Output = Result<T, Ret>> + Send + 'static>>
    where
        Fun: Fn(E) -> Fut,
        Fut: Future<Output = Ret>;

    fn async_map_ok<Fun, Fut, Ret>(
        self,
        map_fn: Fun,
    ) -> Pin<Box<dyn Future<Output = Result<Ret, E>> + Send + 'static>>
    where
        Fun: Fn(T) -> Fut,
        Fut: Future<Output = Ret>;
}

impl<T, E> AsyncResultExt<T, E> for Result<T, E> {
    fn async_map_err<Fun, Fut, Ret>(
        self,
        map_fn: Fun,
    ) -> Pin<Box<dyn Future<Output = Result<T, Ret>> + Send + 'static>>
    where
        Fun: Fn(E) -> Fut,
        Fut: Future<Output = Ret>,
    {
        Box::pin(async move {
            match self {
                Ok(t) => Ok(t),
                Err(e) => Err(map_fn(e).await),
            }
        })
    }

    fn async_map_ok<Fun, Fut, Ret>(
        self,
        map_fn: Fun,
    ) -> Pin<Box<dyn Future<Output = Result<Ret, E>> + Send + 'static>>
    where
        Fun: Fn(T) -> Fut,
        Fut: Future<Output = Ret>,
    {
        Box::pin(async move {
            match self {
                Ok(t) => Ok(map_fn(t).await),
                Err(e) => Err(e),
            }
        })
    }
}

pub trait AsyncMap {
    fn async_map<Fun, Fut, Ret>(self, map_fn: Fun) -> Fut
    where
        Fun: Fn(Self) -> Fut,
        Fut: Future,
    {
        map_fn(self)
    }
}

impl<T> AsyncMap for T {}
