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

//! Traits definitions to help managing builtin functions

use std::{future::Future, marker::PhantomData, pin::Pin};

use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};

use crate::EvaluationContext;

/// A OPA builtin function
pub trait Builtin<C>: Send + Sync {
    /// Call the function, with a list of arguments, each argument being a JSON
    /// reprensentation of the parameter value.
    fn call<'a>(
        &'a self,
        context: &'a mut C,
        args: &'a [&'a [u8]],
    ) -> Pin<Box<dyn Future<Output = Result<Vec<u8>, anyhow::Error>> + Send + 'a>>;
}

#[derive(Clone)]
struct WrappedBuiltin<F, C, const ASYNC: bool, const RESULT: bool, const CONTEXT: bool, P> {
    func: F,
    _marker: PhantomData<fn() -> (C, P)>,
}

impl<F, C: 'static, const ASYNC: bool, const RESULT: bool, const CONTEXT: bool, P: 'static>
    Builtin<C> for WrappedBuiltin<F, C, ASYNC, RESULT, CONTEXT, P>
where
    F: BuiltinFunc<C, ASYNC, RESULT, CONTEXT, P>,
{
    fn call<'a>(
        &'a self,
        context: &'a mut C,
        args: &'a [&'a [u8]],
    ) -> Pin<Box<dyn Future<Output = Result<Vec<u8>, anyhow::Error>> + Send + 'a>> {
        self.func.call(context, args)
    }
}

/// A utility trait used to help constructing [`Builtin`]s out of a regular
/// function, abstracting away the parameters deserialization, the return value
/// serialization, for async/non-async variants, and Result/non-Result variants
pub(crate) trait BuiltinFunc<
    C: 'static,
    const ASYNC: bool,
    const RESULT: bool,
    const CONTEXT: bool,
    P: 'static,
>: Sized + Send + Sync + 'static
{
    fn call<'a>(
        &'a self,
        context: &'a mut C,
        args: &'a [&'a [u8]],
    ) -> Pin<Box<dyn Future<Output = Result<Vec<u8>, anyhow::Error>> + Send + 'a>>;

    fn wrap(self) -> Box<dyn Builtin<C>> {
        Box::new(WrappedBuiltin {
            func: self,
            _marker: PhantomData,
        })
    }
}

macro_rules! count {
    () => (0usize);
    ( $x:tt $($xs:tt)* ) => (1usize + count!($($xs)*));
}

/// A macro to process a builtin return type, based on whether it's an async
/// function and if it returns a [`Result`] or not.
macro_rules! unwrap {
    ($tok:expr, result = true, async = true) => {
        $tok.await?
    };
    ($tok:expr, result = true, async = false) => {
        $tok?
    };
    ($tok:expr, result = false, async = true) => {
        $tok.await
    };
    ($tok:expr, result = false, async = false) => {
        $tok
    };
}

macro_rules! call {
    ($self:ident, $ctx:expr, ($($pname:ident),*), context = true) => {
        $self($ctx, $($pname),*)
    };
    ($self:ident, $ctx:expr, ($($pname:ident),*), context = false) => {
        {
            let _ctx = $ctx;
            $self($($pname),*)
        }
    };
}

macro_rules! trait_body {
    (($($pname:ident: $ptype:ident),*), async = $async:tt, result = $result:tt, context = $context:tt) => {
        fn call<'a>(
            &'a self,
            context: &'a mut C,
            args: &'a [&'a [u8]],
        ) -> Pin<Box<dyn Future<Output = Result<Vec<u8>, anyhow::Error>> + Send + 'a>> {
            Box::pin(async move {
                let [$($pname),*]: [&'a [u8]; count!($($pname)*)] =
                    args.try_into().ok().context("invalid arguments")?;
                $(
                    let $pname: $ptype = serde_json::from_slice($pname)
                        .context(concat!("failed to convert ", stringify!($pname), " argument"))?;
                )*
                let res = call!(self, context, ($($pname),*), context = $context);
                let res = unwrap!(res, result = $result, async = $async);
                let res = serde_json::to_vec(&res).context("could not serialize result")?;
                Ok(res)
            })
        }
    };
}

macro_rules! trait_impl {
    ($($pname:ident: $ptype:ident),*) => {
        // Implementation for a non-async, non-result function, without context
        impl<F, C, $($ptype,)* R> BuiltinFunc<C, false, false, false, ($($ptype,)*)> for F
        where
            C: EvaluationContext,
            F: Fn($($ptype),*) -> R + Send + Sync + 'static,
            $(
                $ptype: for<'de> Deserialize<'de> + Send + 'static,
            )*
            R: Serialize + Send + 'static,
        {
            trait_body! {
                ($($pname: $ptype),*),
                async = false,
                result = false,
                context = false
            }
        }

        // Implementation for a non-async, result function, without context
        impl<F, C, $($ptype,)* R, E> BuiltinFunc<C, true, false, false, ($($ptype,)*)> for F
        where
            C: EvaluationContext,
            F: Fn($($ptype),*) -> Result<R, E> + Send + Sync + 'static,
            $(
                $ptype: for<'de> Deserialize<'de> + Send + 'static,
            )*
            R: Serialize + Send + 'static,
            E: 'static,
            anyhow::Error: From<E>,
        {
            trait_body! {
                ($($pname: $ptype),*),
                async = false,
                result = true,
                context = false
            }
        }

        // Implementation for an async, non-result function, without context
        impl<F, C, $($ptype,)* R, Fut> BuiltinFunc<C, false, true, false, ($($ptype,)*)> for F
        where
            C: EvaluationContext,
            F: Fn($($ptype),*) -> Fut + Send + Sync + 'static,
            $(
                $ptype: for<'de> Deserialize<'de> + Send + 'static,
            )*
            R: Serialize + 'static,
            Fut: Future<Output = R> + Send,
        {
            trait_body! {
                ($($pname: $ptype),*),
                async = true,
                result = false,
                context = false
            }
        }

        // Implementation for an async, result function, without context
        impl<F, C, $($ptype,)* R, E, Fut> BuiltinFunc<C, true, true, false, ($($ptype,)*)> for F
        where
            C: EvaluationContext,
            F: Fn($($ptype),*) -> Fut + Send + Sync + 'static,
            $(
                $ptype: for<'de> Deserialize<'de> + Send + 'static,
            )*
            R: Serialize + 'static,
            E: 'static,
            anyhow::Error: From<E>,
            Fut: Future<Output = Result<R, E>> + Send,
        {
            trait_body! {
                ($($pname: $ptype),*),
                async = true,
                result = true,
                context = false
            }
        }
        //
        // Implementation for a non-async, non-result function, with context
        impl<F, C, $($ptype,)* R> BuiltinFunc<C, false, false, true, ($($ptype,)*)> for F
        where
            C: EvaluationContext,
            F: Fn(&mut C, $($ptype),*) -> R + Send + Sync + 'static,
            $(
                $ptype: for<'de> Deserialize<'de> + Send + 'static,
            )*
            R: Serialize + Send + 'static,
        {
            trait_body! {
                ($($pname: $ptype),*),
                async = false,
                result = false,
                context = true
            }
        }

        // Implementation for a non-async, result function, with context
        impl<F, C, $($ptype,)* R, E> BuiltinFunc<C, true, false, true, ($($ptype,)*)> for F
        where
            C: EvaluationContext,
            F: Fn(&mut C, $($ptype),*) -> Result<R, E> + Send + Sync + 'static,
            $(
                $ptype: for<'de> Deserialize<'de> + Send + 'static,
            )*
            R: Serialize + Send + 'static,
            E: 'static,
            anyhow::Error: From<E>,
        {
            trait_body! {
                ($($pname: $ptype),*),
                async = false,
                result = true,
                context = true
            }
        }

        // Implementation for an async, non-result function, with context
        impl<F, C, $($ptype,)* R, Fut> BuiltinFunc<C, false, true, true, ($($ptype,)*)> for F
        where
            C: EvaluationContext,
            F: Fn(&mut C, $($ptype),*) -> Fut + Send + Sync + 'static,
            $(
                $ptype: for<'de> Deserialize<'de> + Send + 'static,
            )*
            R: Serialize + 'static,
            Fut: Future<Output = R> + Send,
        {
            trait_body! {
                ($($pname: $ptype),*),
                async = true,
                result = false,
                context = true
            }
        }

        // Implementation for an async, result function, with context
        impl<F, C, $($ptype,)* R, E, Fut> BuiltinFunc<C, true, true, true, ($($ptype,)*)> for F
        where
            C: EvaluationContext,
            F: Fn(&mut C, $($ptype),*) -> Fut + Send + Sync + 'static,
            $(
                $ptype: for<'de> Deserialize<'de> + Send + 'static,
            )*
            R: Serialize + 'static,
            E: 'static,
            anyhow::Error: From<E>,
            Fut: Future<Output = Result<R, E>> + Send,
        {
            trait_body! {
                ($($pname: $ptype),*),
                async = true,
                result = true,
                context = true
            }
        }
    }
}

trait_impl!();
trait_impl!(first: P1);
trait_impl!(first: P1, second: P2);
trait_impl!(first: P1, second: P2, third: P3);
trait_impl!(first: P1, second: P2, third: P3, fourth: P4);

#[cfg(test)]
mod tests {
    use super::*;
    use crate::DefaultContext;

    #[tokio::test]
    async fn builtins_call() {
        let mut ctx = DefaultContext::default();
        let uppercase = |foo: String| foo.to_uppercase();
        let uppercase: Box<dyn Builtin<DefaultContext>> = uppercase.wrap();
        let args = [b"\"hello\"" as &[u8]];
        let result = uppercase.call(&mut ctx, &args[..]).await.unwrap();
        assert_eq!(result, b"\"HELLO\"");
    }
}
