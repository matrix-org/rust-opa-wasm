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

use anyhow::{Context, Result};
use wasmtime::{AsContextMut, Caller, Instance, Memory, TypedFunc};

use crate::types::{Addr, Ctx, EntrypointId, Heap, NulStr, OpaError, Value};

fn from_caller<Params, Results, T>(
    name: &'static str,
    caller: &mut Caller<'_, T>,
) -> Result<TypedFunc<Params, Results>>
where
    Params: wasmtime::WasmParams,
    Results: wasmtime::WasmResults,
{
    caller
        .get_export(name)
        .with_context(|| format!("could not find export {name:?}"))?
        .into_func()
        .with_context(|| format!("export {name:?} is not a function"))?
        .typed(caller)
        .with_context(|| format!("exported function {name:?} does not have the right signature"))
}

fn from_instance<Params, Results, T>(
    name: &'static str,
    mut store: impl AsContextMut<Data = T>,
    instance: &Instance,
) -> Result<TypedFunc<Params, Results>>
where
    Params: wasmtime::WasmParams,
    Results: wasmtime::WasmResults,
{
    instance
        .get_export(&mut store, name)
        .with_context(|| format!("could not find export {name:?}"))?
        .into_func()
        .with_context(|| format!("export {name:?} is not a function"))?
        .typed(&mut store)
        .with_context(|| format!("exported function {name:?} does not have the right signature"))
}

pub trait Func: Sized {
    const EXPORT: &'static str;
    type Params: wasmtime::WasmParams;
    type Results: wasmtime::WasmResults;

    fn from_func(func: TypedFunc<Self::Params, Self::Results>) -> Self;

    fn from_caller<T>(caller: &mut Caller<'_, T>) -> Result<Self> {
        Ok(Self::from_func(from_caller(Self::EXPORT, caller)?))
    }

    fn from_instance<T>(store: impl AsContextMut<Data = T>, instance: &Instance) -> Result<Self> {
        Ok(Self::from_func(from_instance(
            Self::EXPORT,
            store,
            instance,
        )?))
    }
}

/// `i32 eval(ctx_addr)`
pub struct Eval(TypedFunc<i32, i32>);

impl Func for Eval {
    const EXPORT: &'static str = "eval";
    type Params = i32;
    type Results = i32;

    fn from_func(func: TypedFunc<Self::Params, Self::Results>) -> Self {
        Self(func)
    }
}

impl Eval {
    #[tracing::instrument(name = "eval", skip_all, err)]
    pub async fn call<T: Send>(
        &self,
        store: impl AsContextMut<Data = T>,
        ctx: &Ctx,
    ) -> Result<i32> {
        let res = self.0.call_async(store, ctx.0).await?;
        Ok(res)
    }
}

/// `value_addr builtins()`
pub struct Builtins(TypedFunc<(), i32>);

impl Func for Builtins {
    const EXPORT: &'static str = "builtins";
    type Params = ();
    type Results = i32;

    fn from_func(func: TypedFunc<Self::Params, Self::Results>) -> Self {
        Self(func)
    }
}

impl Builtins {
    #[tracing::instrument(name = "builtins", skip_all, err)]
    pub async fn call<T: Send>(&self, store: impl AsContextMut<Data = T>) -> Result<Value> {
        let res = self.0.call_async(store, ()).await?;
        Ok(Value(res))
    }
}

/// `value_addr entrypoints()`
pub struct Entrypoints(TypedFunc<(), i32>);

impl Func for Entrypoints {
    const EXPORT: &'static str = "entrypoints";
    type Params = ();
    type Results = i32;

    fn from_func(func: TypedFunc<Self::Params, Self::Results>) -> Self {
        Self(func)
    }
}

impl Entrypoints {
    #[tracing::instrument(name = "entrypoints", skip_all, err)]
    pub async fn call<T: Send>(&self, store: impl AsContextMut<Data = T>) -> Result<Value> {
        let res = self.0.call_async(store, ()).await?;
        Ok(Value(res))
    }
}

/// `ctx_addr opa_eval_ctx_new(void)`
pub struct OpaEvalCtxNew(TypedFunc<(), i32>);

impl Func for OpaEvalCtxNew {
    const EXPORT: &'static str = "opa_eval_ctx_new";
    type Params = ();
    type Results = i32;

    fn from_func(func: TypedFunc<Self::Params, Self::Results>) -> Self {
        Self(func)
    }
}

impl OpaEvalCtxNew {
    #[tracing::instrument(name = "opa_eval_ctx_new", skip_all, err)]
    pub async fn call<T: Send>(&self, store: impl AsContextMut<Data = T>) -> Result<Ctx> {
        let res = self.0.call_async(store, ()).await?;
        Ok(Ctx(res))
    }
}

/// `void opa_eval_ctx_set_input(ctx_addr, value_addr)`
pub struct OpaEvalCtxSetInput(TypedFunc<(i32, i32), ()>);

impl Func for OpaEvalCtxSetInput {
    const EXPORT: &'static str = "opa_eval_ctx_set_input";
    type Params = (i32, i32);
    type Results = ();

    fn from_func(func: TypedFunc<Self::Params, Self::Results>) -> Self {
        Self(func)
    }
}

impl OpaEvalCtxSetInput {
    #[tracing::instrument(name = "opa_eval_ctx_set_input", skip_all, err)]
    pub async fn call<T: Send>(
        &self,
        store: impl AsContextMut<Data = T>,
        ctx: &Ctx,
        input: &Value,
    ) -> Result<()> {
        self.0.call_async(store, (ctx.0, input.0)).await?;
        Ok(())
    }
}

/// `void opa_eval_ctx_set_data(ctx_addr, value_addr)`
pub struct OpaEvalCtxSetData(TypedFunc<(i32, i32), ()>);

impl Func for OpaEvalCtxSetData {
    const EXPORT: &'static str = "opa_eval_ctx_set_data";
    type Params = (i32, i32);
    type Results = ();

    fn from_func(func: TypedFunc<Self::Params, Self::Results>) -> Self {
        Self(func)
    }
}

impl OpaEvalCtxSetData {
    #[tracing::instrument(name = "opa_eval_ctx_set_data", skip_all, err)]
    pub async fn call<T: Send>(
        &self,
        store: impl AsContextMut<Data = T>,
        ctx: &Ctx,
        data: &Value,
    ) -> Result<()> {
        self.0.call_async(store, (ctx.0, data.0)).await?;
        Ok(())
    }
}

/// `void opa_eval_ctx_set_entrypoint(ctx_addr, entrypoint_id)`
pub struct OpaEvalCtxSetEntrypoint(TypedFunc<(i32, i32), ()>);

impl Func for OpaEvalCtxSetEntrypoint {
    const EXPORT: &'static str = "opa_eval_ctx_set_entrypoint";
    type Params = (i32, i32);
    type Results = ();

    fn from_func(func: TypedFunc<Self::Params, Self::Results>) -> Self {
        Self(func)
    }
}

impl OpaEvalCtxSetEntrypoint {
    #[tracing::instrument(name = "opa_eval_ctx_set_entrypoint", skip_all, err)]
    pub async fn call<T: Send>(
        &self,
        store: impl AsContextMut<Data = T>,
        ctx: &Ctx,
        entrypoint: &EntrypointId,
    ) -> Result<()> {
        self.0.call_async(store, (ctx.0, entrypoint.0)).await?;
        Ok(())
    }
}

/// `value_addr opa_eval_ctx_get_result(ctx_addr)`
pub struct OpaEvalCtxGetResult(TypedFunc<i32, i32>);

impl Func for OpaEvalCtxGetResult {
    const EXPORT: &'static str = "opa_eval_ctx_get_result";
    type Params = i32;
    type Results = i32;

    fn from_func(func: TypedFunc<Self::Params, Self::Results>) -> Self {
        Self(func)
    }
}

impl OpaEvalCtxGetResult {
    #[tracing::instrument(name = "opa_eval_ctx_get_result", skip_all, err)]
    pub async fn call<T: Send>(
        &self,
        store: impl AsContextMut<Data = T>,
        ctx: &Ctx,
    ) -> Result<Value> {
        let res = self.0.call_async(store, ctx.0).await?;
        Ok(Value(res))
    }
}

/// `addr opa_malloc(int32 size)`
pub struct OpaMalloc(TypedFunc<i32, i32>);

impl Func for OpaMalloc {
    const EXPORT: &'static str = "opa_malloc";
    type Params = i32;
    type Results = i32;

    fn from_func(func: TypedFunc<Self::Params, Self::Results>) -> Self {
        Self(func)
    }
}

impl OpaMalloc {
    #[tracing::instrument(name = "opa_malloc", skip_all, err)]
    pub async fn call<T: Send>(
        &self,
        store: impl AsContextMut<Data = T>,
        len: usize,
    ) -> Result<Heap> {
        let len = len.try_into().context("invalid parameter")?;
        let ptr = self.0.call_async(store, len).await?;
        Ok(Heap {
            ptr,
            len,
            freed: false,
        })
    }
}

/// `void opa_free(addr)`
pub struct OpaFree(TypedFunc<i32, ()>);

impl Func for OpaFree {
    const EXPORT: &'static str = "opa_free";
    type Params = i32;
    type Results = ();

    fn from_func(func: TypedFunc<Self::Params, Self::Results>) -> Self {
        Self(func)
    }
}

impl OpaFree {
    #[tracing::instrument(name = "opa_free", skip_all, err)]
    pub async fn call<T: Send>(
        &self,
        store: impl AsContextMut<Data = T>,
        mut heap: Heap,
    ) -> Result<()> {
        self.0.call_async(store, heap.ptr).await?;
        heap.freed = true;
        drop(heap);
        Ok(())
    }
}

/// `value_addr opa_json_parse(str_addr, size)`
pub struct OpaJsonParse(TypedFunc<(i32, i32), i32>);

impl Func for OpaJsonParse {
    const EXPORT: &'static str = "opa_json_parse";
    type Params = (i32, i32);
    type Results = i32;

    fn from_func(func: TypedFunc<Self::Params, Self::Results>) -> Self {
        Self(func)
    }
}

impl OpaJsonParse {
    #[tracing::instrument(name = "opa_json_parse", skip_all, err)]
    pub async fn call<T: Send>(
        &self,
        store: impl AsContextMut<Data = T>,
        heap: &Heap,
    ) -> Result<Value> {
        let res = self.0.call_async(store, (heap.ptr, heap.len)).await?;
        Ok(Value(res))
    }
}

/// `value_addr opa_value_parse(str_addr, size)`
pub struct OpaValueParse(TypedFunc<(i32, i32), i32>);

impl Func for OpaValueParse {
    const EXPORT: &'static str = "opa_value_parse";
    type Params = (i32, i32);
    type Results = i32;

    fn from_func(func: TypedFunc<Self::Params, Self::Results>) -> Self {
        Self(func)
    }
}

impl OpaValueParse {
    #[allow(dead_code)]
    #[tracing::instrument(name = "opa_value_parse", skip_all, err)]
    pub async fn call<T: Send>(
        &self,
        store: impl AsContextMut<Data = T>,
        heap: &Heap,
    ) -> Result<Value> {
        let res = self.0.call_async(store, (heap.ptr, heap.len)).await?;
        Ok(Value(res))
    }
}

/// `str_addr opa_json_dump(value_addr)`
pub struct OpaJsonDump(TypedFunc<i32, i32>);

impl Func for OpaJsonDump {
    const EXPORT: &'static str = "opa_json_dump";
    type Params = i32;
    type Results = i32;

    fn from_func(func: TypedFunc<Self::Params, Self::Results>) -> Self {
        Self(func)
    }
}

impl OpaJsonDump {
    #[tracing::instrument(name = "opa_json_dump", skip_all, err)]
    pub async fn call<T: Send>(
        &self,
        store: impl AsContextMut<Data = T>,
        value: &Value,
    ) -> Result<NulStr> {
        let res = self.0.call_async(store, value.0).await?;
        Ok(NulStr(res))
    }

    pub async fn decode<V: for<'de> serde::Deserialize<'de>, T: Send>(
        &self,
        mut store: impl AsContextMut<Data = T>,
        memory: &Memory,
        value: &Value,
    ) -> Result<V> {
        let json = self.call(&mut store, value).await?;
        let json = json.read(&store, memory)?;
        let json = serde_json::from_slice(json.to_bytes())?;
        Ok(json)
    }
}

/// `void opa_heap_ptr_set(addr)`
pub struct OpaHeapPtrSet(TypedFunc<i32, ()>);

impl Func for OpaHeapPtrSet {
    const EXPORT: &'static str = "opa_heap_ptr_set";
    type Params = i32;
    type Results = ();

    fn from_func(func: TypedFunc<Self::Params, Self::Results>) -> Self {
        Self(func)
    }
}

impl OpaHeapPtrSet {
    #[tracing::instrument(name = "opa_heap_ptr_set", skip_all, err)]
    pub async fn call<T: Send>(
        &self,
        store: impl AsContextMut<Data = T>,
        addr: &Addr,
    ) -> Result<()> {
        self.0.call_async(store, addr.0).await?;
        Ok(())
    }
}

/// `addr opa_heap_ptr_get()`
pub struct OpaHeapPtrGet(TypedFunc<(), i32>);

impl Func for OpaHeapPtrGet {
    const EXPORT: &'static str = "opa_heap_ptr_get";
    type Params = ();
    type Results = i32;

    fn from_func(func: TypedFunc<Self::Params, Self::Results>) -> Self {
        Self(func)
    }
}

impl OpaHeapPtrGet {
    #[tracing::instrument(name = "opa_heap_ptr_get", skip_all, err)]
    pub async fn call<T: Send>(&self, store: impl AsContextMut<Data = T>) -> Result<Addr> {
        let res = self.0.call_async(store, ()).await?;
        Ok(Addr(res))
    }
}

/// `int32 opa_value_add_path(base_value_addr, path_value_addr, value_addr)`
pub struct OpaValueAddPath(TypedFunc<(i32, i32, i32), i32>);

impl Func for OpaValueAddPath {
    const EXPORT: &'static str = "opa_value_add_path";
    type Params = (i32, i32, i32);
    type Results = i32;

    fn from_func(func: TypedFunc<Self::Params, Self::Results>) -> Self {
        Self(func)
    }
}

impl OpaValueAddPath {
    #[allow(dead_code)]
    #[tracing::instrument(name = "opa_value_add_path", skip_all, err)]
    pub async fn call<T: Send>(
        &self,
        store: impl AsContextMut<Data = T>,
        base: &Value,
        path: &Value,
        value: &Value,
    ) -> Result<()> {
        let res = self.0.call_async(store, (base.0, path.0, value.0)).await?;
        Ok(OpaError::from_code(res)?)
    }
}

/// `int32 opa_value_remove_path(base_value_addr, path_value_addr)`
pub struct OpaValueRemovePath(TypedFunc<(i32, i32), i32>);

impl Func for OpaValueRemovePath {
    const EXPORT: &'static str = "opa_value_remove_path";
    type Params = (i32, i32);
    type Results = i32;

    fn from_func(func: TypedFunc<Self::Params, Self::Results>) -> Self {
        Self(func)
    }
}

impl OpaValueRemovePath {
    #[allow(dead_code)]
    #[tracing::instrument(name = "opa_value_remove_path", skip_all, err)]
    pub async fn call<T: Send>(
        &self,
        store: impl AsContextMut<Data = T>,
        base: &Value,
        path: &Value,
    ) -> Result<()> {
        let res = self.0.call_async(store, (base.0, path.0)).await?;
        Ok(OpaError::from_code(res)?)
    }
}

/// `str_addr opa_value_dump(value_addr)`
pub struct OpaValueDump(TypedFunc<i32, i32>);

impl Func for OpaValueDump {
    const EXPORT: &'static str = "opa_value_dump";
    type Params = i32;
    type Results = i32;

    fn from_func(func: TypedFunc<Self::Params, Self::Results>) -> Self {
        Self(func)
    }
}

impl OpaValueDump {
    #[allow(dead_code)]
    #[tracing::instrument(name = "opa_value_dump", skip_all, err)]
    pub async fn call<T: Send>(
        &self,
        store: impl AsContextMut<Data = T>,
        value: &Value,
    ) -> Result<Value> {
        let res = self.0.call_async(store, value.0).await?;
        Ok(Value(res))
    }
}

/// `str_addr opa_eval(_ addr, entrypoint_id int32, data value_addr, input
/// str_addr, input_len int32, heap_ptr addr, format int32)`
#[allow(clippy::type_complexity)]
pub struct OpaEval(TypedFunc<(i32, i32, i32, i32, i32, i32, i32), i32>);

impl Func for OpaEval {
    const EXPORT: &'static str = "opa_eval";
    type Params = (i32, i32, i32, i32, i32, i32, i32);
    type Results = i32;

    fn from_func(func: TypedFunc<Self::Params, Self::Results>) -> Self {
        Self(func)
    }
}

impl OpaEval {
    #[tracing::instrument(name = "opa_eval", skip_all, err)]
    pub async fn call<T: Send>(
        &self,
        store: impl AsContextMut<Data = T>,
        entrypoint: &EntrypointId,
        data: &Value,
        input: &Heap,
        heap_ptr: &Addr,
    ) -> Result<NulStr> {
        let res = self
            .0
            .call_async(
                store,
                (0, entrypoint.0, data.0, input.ptr, input.len, heap_ptr.0, 0),
            )
            .await?;
        Ok(NulStr(res))
    }
}
