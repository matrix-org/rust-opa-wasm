// Copyright 2022 The Matrix.org Foundation C.I.C.
//
// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License at
//
//     http://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and
// limitations under the License.

use std::{future::Future, marker::PhantomData, pin::Pin};

use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use wasmtime::Trap;

pub trait Builtin: Send + Sync {
    fn call<'a>(
        &'a self,
        args: &'a [&'a [u8]],
    ) -> Pin<Box<dyn Future<Output = Result<Vec<u8>, Trap>> + Send + 'a>>;
}

#[derive(Clone)]
struct WrappedBuiltin<F, const ASYNC: bool, const RESULT: bool, T> {
    func: F,
    _marker: PhantomData<fn() -> T>,
}

impl<F, const ASYNC: bool, const RESULT: bool, T: 'static> Builtin
    for WrappedBuiltin<F, ASYNC, RESULT, T>
where
    F: BuiltinFunc<ASYNC, RESULT, T>,
{
    fn call<'a>(
        &'a self,
        args: &'a [&'a [u8]],
    ) -> Pin<Box<dyn Future<Output = Result<Vec<u8>, Trap>> + Send + 'a>> {
        self.func.call(args)
    }
}

pub trait BuiltinFunc<const ASYNC: bool, const RESULT: bool, T: 'static>:
    Sized + Send + Sync + 'static
{
    fn call<'a>(
        &'a self,
        args: &'a [&'a [u8]],
    ) -> Pin<Box<dyn Future<Output = Result<Vec<u8>, Trap>> + Send + 'a>>;

    fn wrap(self) -> Box<dyn Builtin> {
        Box::new(WrappedBuiltin {
            func: self,
            _marker: PhantomData,
        })
    }
}

impl<F, R, Fut> BuiltinFunc<true, false, ()> for F
where
    F: Fn() -> Fut + Send + Sync + 'static,
    R: Serialize + 'static,
    Fut: Future<Output = R> + Send,
{
    fn call<'a>(
        &'a self,
        args: &'a [&'a [u8]],
    ) -> Pin<Box<dyn Future<Output = Result<Vec<u8>, Trap>> + Send + 'a>> {
        Box::pin(async move {
            let []: [&'a [u8]; 0] = args.try_into().ok().context("invalid arguments")?;
            let res = self().await;
            let res = serde_json::to_vec(&res).context("could not serialize result")?;
            Ok(res)
        })
    }
}

impl<F, R, E, Fut> BuiltinFunc<true, true, ()> for F
where
    F: Fn() -> Fut + Send + Sync + 'static,
    R: Serialize + 'static,
    E: 'static,
    Trap: From<E>,
    Fut: Future<Output = Result<R, E>> + Send,
{
    fn call<'a>(
        &'a self,
        args: &'a [&'a [u8]],
    ) -> Pin<Box<dyn Future<Output = Result<Vec<u8>, Trap>> + Send + 'a>> {
        Box::pin(async move {
            let []: [&'a [u8]; 0] = args.try_into().ok().context("invalid arguments")?;
            let res = self().await?;
            let res = serde_json::to_vec(&res).context("could not serialize result")?;
            Ok(res)
        })
    }
}

impl<F, R> BuiltinFunc<false, false, ()> for F
where
    F: Fn() -> R + Send + Sync + 'static,
    R: Serialize + Send + 'static,
{
    fn call<'a>(
        &'a self,
        args: &'a [&'a [u8]],
    ) -> Pin<Box<dyn Future<Output = Result<Vec<u8>, Trap>> + Send + 'a>> {
        Box::pin(std::future::ready((|| {
            let []: [&'a [u8]; 0] = args.try_into().ok().context("invalid arguments")?;
            let res = self();
            let res = serde_json::to_vec(&res).context("could not serialize result")?;
            Ok(res)
        })()))
    }
}

impl<F, R, E> BuiltinFunc<false, true, ()> for F
where
    F: Fn() -> Result<R, E> + Send + Sync + 'static,
    R: Serialize + Send + 'static,
    E: 'static,
    Trap: From<E>,
{
    fn call<'a>(
        &'a self,
        args: &'a [&'a [u8]],
    ) -> Pin<Box<dyn Future<Output = Result<Vec<u8>, Trap>> + Send + 'a>> {
        Box::pin(std::future::ready((|| {
            let []: [&'a [u8]; 0] = args.try_into().ok().context("invalid arguments")?;
            let res = self()?;
            let res = serde_json::to_vec(&res).context("could not serialize result")?;
            Ok(res)
        })()))
    }
}

impl<F, T, R, Fut> BuiltinFunc<true, false, (T,)> for F
where
    F: Fn(T) -> Fut + Send + Sync + 'static,
    T: for<'de> Deserialize<'de> + Send + 'static,
    R: Serialize + 'static,
    Fut: Future<Output = R> + Send,
{
    fn call<'a>(
        &'a self,
        args: &'a [&'a [u8]],
    ) -> Pin<Box<dyn Future<Output = Result<Vec<u8>, Trap>> + Send + 'a>> {
        Box::pin(async move {
            let [p1]: [&'a [u8]; 1] = args.try_into().ok().context("invalid arguments")?;
            let p1 = serde_json::from_slice(p1).context("failed to convert first argument")?;
            let res = self(p1).await;
            let res = serde_json::to_vec(&res).context("could not serialize result")?;
            Ok(res)
        })
    }
}

impl<F, T, R, E, Fut> BuiltinFunc<true, true, (T,)> for F
where
    F: Fn(T) -> Fut + Send + Sync + 'static,
    T: for<'de> Deserialize<'de> + Send + 'static,
    R: Serialize + 'static,
    E: 'static,
    Trap: From<E>,
    Fut: Future<Output = Result<R, E>> + Send,
{
    fn call<'a>(
        &'a self,
        args: &'a [&'a [u8]],
    ) -> Pin<Box<dyn Future<Output = Result<Vec<u8>, Trap>> + Send + 'a>> {
        Box::pin(async move {
            let [p1]: [&'a [u8]; 1] = args.try_into().ok().context("invalid arguments")?;
            let p1 = serde_json::from_slice(p1).context("failed to convert first argument")?;
            let res = self(p1).await?;
            let res = serde_json::to_vec(&res).context("could not serialize result")?;
            Ok(res)
        })
    }
}

impl<F, T, R> BuiltinFunc<false, false, (T,)> for F
where
    F: Fn(T) -> R + Send + Sync + 'static,
    T: for<'de> Deserialize<'de> + Send + 'static,
    R: Serialize + 'static,
{
    fn call<'a>(
        &'a self,
        args: &'a [&'a [u8]],
    ) -> Pin<Box<dyn Future<Output = Result<Vec<u8>, Trap>> + Send + 'a>> {
        Box::pin(std::future::ready((|| {
            let [p1]: [&'a [u8]; 1] = args.try_into().ok().context("invalid arguments")?;
            let p1 = serde_json::from_slice(p1).context("failed to convert first argument")?;
            let res = self(p1);
            let res = serde_json::to_vec(&res).context("could not serialize result")?;
            Ok(res)
        })()))
    }
}

impl<F, T, R, E> BuiltinFunc<false, true, (T,)> for F
where
    F: Fn(T) -> Result<R, E> + Send + Sync + 'static,
    T: for<'de> Deserialize<'de> + Send + 'static,
    R: Serialize + 'static,
    E: 'static,
    Trap: From<E>,
{
    fn call<'a>(
        &'a self,
        args: &'a [&'a [u8]],
    ) -> Pin<Box<dyn Future<Output = Result<Vec<u8>, Trap>> + Send + 'a>> {
        Box::pin(std::future::ready((|| {
            let [p1]: [&'a [u8]; 1] = args.try_into().ok().context("invalid arguments")?;
            let p1 = serde_json::from_slice(p1).context("failed to convert first argument")?;
            let res = self(p1)?;
            let res = serde_json::to_vec(&res).context("could not serialize result")?;
            Ok(res)
        })()))
    }
}

impl<F, T1, T2, R, Fut> BuiltinFunc<true, false, (T1, T2)> for F
where
    F: Fn(T1, T2) -> Fut + Send + Sync + 'static,
    T1: for<'de> Deserialize<'de> + Send + 'static,
    T2: for<'de> Deserialize<'de> + Send + 'static,
    R: Serialize + 'static,
    Fut: Future<Output = R> + Send,
{
    fn call<'a>(
        &'a self,
        args: &'a [&'a [u8]],
    ) -> Pin<Box<dyn Future<Output = Result<Vec<u8>, Trap>> + Send + 'a>> {
        Box::pin(async move {
            let [p1, p2]: [&'a [u8]; 2] = args.try_into().ok().context("invalid arguments")?;
            let p1 = serde_json::from_slice(p1).context("failed to convert first argument")?;
            let p2 = serde_json::from_slice(p2).context("failed to convert second argument")?;
            let res = self(p1, p2).await;
            let res = serde_json::to_vec(&res).context("coult not serialize result")?;
            Ok(res)
        })
    }
}

impl<F, T1, T2, R, E, Fut> BuiltinFunc<true, true, (T1, T2)> for F
where
    F: Fn(T1, T2) -> Fut + Send + Sync + 'static,
    T1: for<'de> Deserialize<'de> + Send + 'static,
    T2: for<'de> Deserialize<'de> + Send + 'static,
    R: Serialize + 'static,
    E: 'static,
    Trap: From<E>,
    Fut: Future<Output = Result<R, E>> + Send,
{
    fn call<'a>(
        &'a self,
        args: &'a [&'a [u8]],
    ) -> Pin<Box<dyn Future<Output = Result<Vec<u8>, Trap>> + Send + 'a>> {
        Box::pin(async move {
            let [p1, p2]: [&'a [u8]; 2] = args.try_into().ok().context("invalid arguments")?;
            let p1 = serde_json::from_slice(p1).context("failed to convert first argument")?;
            let p2 = serde_json::from_slice(p2).context("failed to convert second argument")?;
            let res = self(p1, p2).await?;
            let res = serde_json::to_vec(&res).context("coult not serialize result")?;
            Ok(res)
        })
    }
}

impl<F, T1, T2, R> BuiltinFunc<false, false, (T1, T2)> for F
where
    F: Fn(T1, T2) -> R + Send + Sync + 'static,
    T1: for<'de> Deserialize<'de> + Send + 'static,
    T2: for<'de> Deserialize<'de> + Send + 'static,
    R: Serialize + 'static,
{
    fn call<'a>(
        &'a self,
        args: &'a [&'a [u8]],
    ) -> Pin<Box<dyn Future<Output = Result<Vec<u8>, Trap>> + Send + 'a>> {
        Box::pin(std::future::ready((|| {
            let [p1, p2]: [&'a [u8]; 2] = args.try_into().ok().context("invalid arguments")?;
            let p1 = serde_json::from_slice(p1).context("failed to convert first argument")?;
            let p2 = serde_json::from_slice(p2).context("failed to convert second argument")?;
            let res = self(p1, p2);
            let res = serde_json::to_vec(&res).context("could not serialize result")?;
            Ok(res)
        })()))
    }
}

impl<F, T1, T2, R, E> BuiltinFunc<false, true, (T1, T2)> for F
where
    F: Fn(T1, T2) -> Result<R, E> + Send + Sync + 'static,
    T1: for<'de> Deserialize<'de> + Send + 'static,
    T2: for<'de> Deserialize<'de> + Send + 'static,
    R: Serialize + 'static,
    E: 'static,
    Trap: From<E>,
{
    fn call<'a>(
        &'a self,
        args: &'a [&'a [u8]],
    ) -> Pin<Box<dyn Future<Output = Result<Vec<u8>, Trap>> + Send + 'a>> {
        Box::pin(std::future::ready((|| {
            let [p1, p2]: [&'a [u8]; 2] = args.try_into().ok().context("invalid arguments")?;
            let p1 = serde_json::from_slice(p1).context("failed to convert first argument")?;
            let p2 = serde_json::from_slice(p2).context("failed to convert second argument")?;
            let res = self(p1, p2)?;
            let res = serde_json::to_vec(&res).context("could not serialize result")?;

            Ok(res)
        })()))
    }
}

impl<F, T1, T2, T3, R, Fut> BuiltinFunc<true, false, (T1, T2, T3)> for F
where
    F: Fn(T1, T2, T3) -> Fut + Send + Sync + 'static,
    T1: for<'de> Deserialize<'de> + Send + 'static,
    T2: for<'de> Deserialize<'de> + Send + 'static,
    T3: for<'de> Deserialize<'de> + Send + 'static,
    R: Serialize + 'static,
    Fut: Future<Output = R> + Send,
{
    fn call<'a>(
        &'a self,
        args: &'a [&'a [u8]],
    ) -> Pin<Box<dyn Future<Output = Result<Vec<u8>, Trap>> + Send + 'a>> {
        Box::pin(async move {
            let [p1, p2, p3]: [&'a [u8]; 3] = args.try_into().ok().context("invalid arguments")?;
            let p1 = serde_json::from_slice(p1).context("failed to convert first argument")?;
            let p2 = serde_json::from_slice(p2).context("failed to convert second argument")?;
            let p3 = serde_json::from_slice(p3).context("failed to convert third argument")?;
            let res = self(p1, p2, p3).await;
            let res = serde_json::to_vec(&res).context("coult not serialize result")?;
            Ok(res)
        })
    }
}

impl<F, T1, T2, T3, R, E, Fut> BuiltinFunc<true, true, (T1, T2, T3)> for F
where
    F: Fn(T1, T2, T3) -> Fut + Send + Sync + 'static,
    T1: for<'de> Deserialize<'de> + Send + 'static,
    T2: for<'de> Deserialize<'de> + Send + 'static,
    T3: for<'de> Deserialize<'de> + Send + 'static,
    R: Serialize + 'static,
    E: 'static,
    Fut: Future<Output = Result<R, E>> + Send,
    Trap: From<E>,
{
    fn call<'a>(
        &'a self,
        args: &'a [&'a [u8]],
    ) -> Pin<Box<dyn Future<Output = Result<Vec<u8>, Trap>> + Send + 'a>> {
        Box::pin(async move {
            let [p1, p2, p3]: [&'a [u8]; 3] = args.try_into().ok().context("invalid arguments")?;
            let p1 = serde_json::from_slice(p1).context("failed to convert first argument")?;
            let p2 = serde_json::from_slice(p2).context("failed to convert second argument")?;
            let p3 = serde_json::from_slice(p3).context("failed to convert third argument")?;
            let res = self(p1, p2, p3).await?;
            let res = serde_json::to_vec(&res).context("coult not serialize result")?;
            Ok(res)
        })
    }
}

impl<F, T1, T2, T3, R> BuiltinFunc<false, false, (T1, T2, T3)> for F
where
    F: Fn(T1, T2, T3) -> R + Send + Sync + 'static,
    T1: for<'de> Deserialize<'de> + Send + 'static,
    T2: for<'de> Deserialize<'de> + Send + 'static,
    T3: for<'de> Deserialize<'de> + Send + 'static,
    R: Serialize + 'static,
{
    fn call<'a>(
        &'a self,
        args: &'a [&'a [u8]],
    ) -> Pin<Box<dyn Future<Output = Result<Vec<u8>, Trap>> + Send + 'a>> {
        Box::pin(std::future::ready((|| {
            let [p1, p2, p3]: [&'a [u8]; 3] = args.try_into().ok().context("invalid arguments")?;
            let p1 = serde_json::from_slice(p1).context("failed to convert first argument")?;
            let p2 = serde_json::from_slice(p2).context("failed to convert second argument")?;
            let p3 = serde_json::from_slice(p3).context("failed to convert third argument")?;
            let res = self(p1, p2, p3);
            let res = serde_json::to_vec(&res).context("could not serialize result")?;

            Ok(res)
        })()))
    }
}

impl<F, T1, T2, T3, R, E> BuiltinFunc<false, true, (T1, T2, T3)> for F
where
    F: Fn(T1, T2, T3) -> Result<R, E> + Send + Sync + 'static,
    T1: for<'de> Deserialize<'de> + Send + 'static,
    T2: for<'de> Deserialize<'de> + Send + 'static,
    T3: for<'de> Deserialize<'de> + Send + 'static,
    R: Serialize + 'static,
    E: 'static,
    Trap: From<E>,
{
    fn call<'a>(
        &'a self,
        args: &'a [&'a [u8]],
    ) -> Pin<Box<dyn Future<Output = Result<Vec<u8>, Trap>> + Send + 'a>> {
        Box::pin(std::future::ready((|| {
            let [p1, p2, p3]: [&'a [u8]; 3] = args.try_into().ok().context("invalid arguments")?;
            let p1 = serde_json::from_slice(p1).context("failed to convert first argument")?;
            let p2 = serde_json::from_slice(p2).context("failed to convert second argument")?;
            let p3 = serde_json::from_slice(p3).context("failed to convert third argument")?;
            let res = self(p1, p2, p3)?;
            let res = serde_json::to_vec(&res).context("could not serialize result")?;

            Ok(res)
        })()))
    }
}

impl<F, T1, T2, T3, T4, R, Fut> BuiltinFunc<true, false, (T1, T2, T3, T4)> for F
where
    F: Fn(T1, T2, T3, T4) -> Fut + Send + Sync + 'static,
    T1: for<'de> Deserialize<'de> + Send + 'static,
    T2: for<'de> Deserialize<'de> + Send + 'static,
    T3: for<'de> Deserialize<'de> + Send + 'static,
    T4: for<'de> Deserialize<'de> + Send + 'static,
    R: Serialize + 'static,
    Fut: Future<Output = R> + Send,
{
    fn call<'a>(
        &'a self,
        args: &'a [&'a [u8]],
    ) -> Pin<Box<dyn Future<Output = Result<Vec<u8>, Trap>> + Send + 'a>> {
        Box::pin(async move {
            let [p1, p2, p3, p4]: [&'a [u8]; 4] =
                args.try_into().ok().context("invalid arguments")?;
            let p1 = serde_json::from_slice(p1).context("failed to convert first argument")?;
            let p2 = serde_json::from_slice(p2).context("failed to convert second argument")?;
            let p3 = serde_json::from_slice(p3).context("failed to convert third argument")?;
            let p4 = serde_json::from_slice(p4).context("failed to convert fourth argument")?;
            let res = self(p1, p2, p3, p4).await;
            let res = serde_json::to_vec(&res).context("coult not serialize result")?;
            Ok(res)
        })
    }
}

impl<F, T1, T2, T3, T4, R, E, Fut> BuiltinFunc<true, true, (T1, T2, T3, T4)> for F
where
    F: Fn(T1, T2, T3, T4) -> Fut + Send + Sync + 'static,
    T1: for<'de> Deserialize<'de> + Send + 'static,
    T2: for<'de> Deserialize<'de> + Send + 'static,
    T3: for<'de> Deserialize<'de> + Send + 'static,
    T4: for<'de> Deserialize<'de> + Send + 'static,
    R: Serialize + 'static,
    E: 'static,
    Trap: From<E>,
    Fut: Future<Output = Result<R, E>> + Send,
{
    fn call<'a>(
        &'a self,
        args: &'a [&'a [u8]],
    ) -> Pin<Box<dyn Future<Output = Result<Vec<u8>, Trap>> + Send + 'a>> {
        Box::pin(async move {
            let [p1, p2, p3, p4]: [&'a [u8]; 4] =
                args.try_into().ok().context("invalid arguments")?;
            let p1 = serde_json::from_slice(p1).context("failed to convert first argument")?;
            let p2 = serde_json::from_slice(p2).context("failed to convert second argument")?;
            let p3 = serde_json::from_slice(p3).context("failed to convert third argument")?;
            let p4 = serde_json::from_slice(p4).context("failed to convert fourth argument")?;
            let res = self(p1, p2, p3, p4).await?;
            let res = serde_json::to_vec(&res).context("coult not serialize result")?;
            Ok(res)
        })
    }
}

impl<F, T1, T2, T3, T4, R> BuiltinFunc<false, false, (T1, T2, T3, T4)> for F
where
    F: Fn(T1, T2, T3, T4) -> R + Send + Sync + 'static,
    T1: for<'de> Deserialize<'de> + Send + 'static,
    T2: for<'de> Deserialize<'de> + Send + 'static,
    T3: for<'de> Deserialize<'de> + Send + 'static,
    T4: for<'de> Deserialize<'de> + Send + 'static,
    R: Serialize + 'static,
{
    fn call<'a>(
        &'a self,
        args: &'a [&'a [u8]],
    ) -> Pin<Box<dyn Future<Output = Result<Vec<u8>, Trap>> + Send + 'a>> {
        Box::pin(std::future::ready((|| {
            let [p1, p2, p3, p4]: [&'a [u8]; 4] =
                args.try_into().ok().context("invalid arguments")?;
            let p1 = serde_json::from_slice(p1).context("failed to convert first argument")?;
            let p2 = serde_json::from_slice(p2).context("failed to convert second argument")?;
            let p3 = serde_json::from_slice(p3).context("failed to convert third argument")?;
            let p4 = serde_json::from_slice(p4).context("failed to convert fourth argument")?;
            let res = self(p1, p2, p3, p4);
            let res = serde_json::to_vec(&res).context("could not serialize result")?;

            Ok(res)
        })()))
    }
}

impl<F, T1, T2, T3, T4, R, E> BuiltinFunc<false, true, (T1, T2, T3, T4)> for F
where
    F: Fn(T1, T2, T3, T4) -> Result<R, E> + Send + Sync + 'static,
    T1: for<'de> Deserialize<'de> + Send + 'static,
    T2: for<'de> Deserialize<'de> + Send + 'static,
    T3: for<'de> Deserialize<'de> + Send + 'static,
    T4: for<'de> Deserialize<'de> + Send + 'static,
    R: Serialize + 'static,
    E: 'static,
    Trap: From<E>,
{
    fn call<'a>(
        &'a self,
        args: &'a [&'a [u8]],
    ) -> Pin<Box<dyn Future<Output = Result<Vec<u8>, Trap>> + Send + 'a>> {
        Box::pin(std::future::ready((|| {
            let [p1, p2, p3, p4]: [&'a [u8]; 4] =
                args.try_into().ok().context("invalid arguments")?;
            let p1 = serde_json::from_slice(p1).context("failed to convert first argument")?;
            let p2 = serde_json::from_slice(p2).context("failed to convert second argument")?;
            let p3 = serde_json::from_slice(p3).context("failed to convert third argument")?;
            let p4 = serde_json::from_slice(p4).context("failed to convert fourth argument")?;
            let res = self(p1, p2, p3, p4)?;
            let res = serde_json::to_vec(&res).context("could not serialize result")?;

            Ok(res)
        })()))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn builtins_call() {
        let uppercase = |foo: String| foo.to_uppercase();
        let uppercase: Box<dyn Builtin> = uppercase.wrap();
        let args = [b"\"hello\"" as &[u8]];
        let result = uppercase.call(&args[..]).await.unwrap();
        assert_eq!(result, b"\"HELLO\"");
    }
}
