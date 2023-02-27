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

use std::{
    collections::{HashMap, HashSet},
    ffi::CString,
    fmt::Debug,
    ops::Deref,
    sync::Arc,
};

use anyhow::{Context, Result};
use tokio::sync::{Mutex, OnceCell};
use tracing::Instrument;
use wasmtime::{AsContextMut, Caller, Linker, Memory, MemoryType, Module};

use crate::{
    builtins::traits::Builtin,
    funcs::{self, Func},
    types::{AbiVersion, Addr, BuiltinId, EntrypointId, Heap, NulStr, Value},
    DefaultContext, EvaluationContext,
};

async fn alloc_str<V: Into<Vec<u8>>, T: Send>(
    opa_malloc: &funcs::OpaMalloc,
    mut store: impl AsContextMut<Data = T>,
    memory: &Memory,
    value: V,
) -> Result<Heap> {
    let value = CString::new(value)?;
    let value = value.as_bytes_with_nul();
    let heap = opa_malloc.call(&mut store, value.len()).await?;

    memory.write(
        &mut store,
        heap.ptr
            .try_into()
            .context("opa_malloc returned an invalid pointer value")?,
        value,
    )?;

    Ok(heap)
}

async fn load_json<V: serde::Serialize, T: Send>(
    opa_malloc: &funcs::OpaMalloc,
    opa_free: &funcs::OpaFree,
    opa_json_parse: &funcs::OpaJsonParse,
    mut store: impl AsContextMut<Data = T>,
    memory: &Memory,
    data: &V,
) -> Result<Value> {
    let json = serde_json::to_vec(data)?;
    let json = alloc_str(opa_malloc, &mut store, memory, json).await?;
    let data = opa_json_parse.call(&mut store, &json).await?;
    opa_free.call(&mut store, json).await?;
    Ok(data)
}

struct LoadedBuiltins<C> {
    builtins: HashMap<i32, (String, Box<dyn Builtin<C>>)>,
    context: Mutex<C>,
}

impl<C> std::fmt::Debug for LoadedBuiltins<C> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("LoadedBuiltins")
            .field("builtins", &())
            .finish()
    }
}

impl<C> LoadedBuiltins<C>
where
    C: EvaluationContext,
{
    fn from_map(map: HashMap<String, BuiltinId>, context: C) -> Result<Self> {
        let res: Result<_> = map
            .into_iter()
            .map(|(k, v)| {
                let builtin = crate::builtins::resolve(&k)?;
                Ok((v.0, (k, builtin)))
            })
            .collect();
        Ok(Self {
            builtins: res?,
            context: Mutex::new(context),
        })
    }

    async fn builtin<T: Send, const N: usize>(
        &self,
        mut caller: Caller<'_, T>,
        memory: &Memory,
        builtin_id: i32,
        args: [i32; N],
    ) -> Result<i32, anyhow::Error> {
        let (name, builtin) = self
            .builtins
            .get(&builtin_id)
            .with_context(|| format!("unknown builtin id {builtin_id}"))?;

        let span = tracing::info_span!("builtin", %name);
        let _enter = span.enter();

        let opa_json_dump = funcs::OpaJsonDump::from_caller(&mut caller)?;
        let opa_json_parse = funcs::OpaJsonParse::from_caller(&mut caller)?;
        let opa_malloc = funcs::OpaMalloc::from_caller(&mut caller)?;
        let opa_free = funcs::OpaFree::from_caller(&mut caller)?;

        // Call opa_json_dump on each argument
        let mut args_json = Vec::with_capacity(N);
        for arg in args {
            args_json.push(opa_json_dump.call(&mut caller, &Value(arg)).await?);
        }

        // Extract the JSON value of each argument
        let mut mapped_args = Vec::with_capacity(N);
        for arg_json in args_json {
            let arg = arg_json.read(&caller, memory)?;
            mapped_args.push(arg.to_bytes());
        }

        let mut ctx = self.context.lock().await;

        // Actually call the function
        let ret = (async move { builtin.call(&mut ctx, &mapped_args).await })
            .instrument(tracing::info_span!("builtin.call"))
            .await?;

        let json = alloc_str(&opa_malloc, &mut caller, memory, ret).await?;
        let data = opa_json_parse.call(&mut caller, &json).await?;
        opa_free.call(&mut caller, json).await?;

        Ok(data.0)
    }

    async fn evaluation_start(&self) {
        self.context.lock().await.evaluation_start();
    }
}

/// An instance of a policy with builtins and entrypoints resolved, but with no
/// data provided yet
pub struct Runtime<C> {
    version: AbiVersion,
    memory: Memory,
    entrypoints: HashMap<String, EntrypointId>,
    loaded_builtins: Arc<OnceCell<LoadedBuiltins<C>>>,

    eval_func: funcs::Eval,
    opa_eval_ctx_new_func: funcs::OpaEvalCtxNew,
    opa_eval_ctx_set_input_func: funcs::OpaEvalCtxSetInput,
    opa_eval_ctx_set_data_func: funcs::OpaEvalCtxSetData,
    opa_eval_ctx_set_entrypoint_func: funcs::OpaEvalCtxSetEntrypoint,
    opa_eval_ctx_get_result_func: funcs::OpaEvalCtxGetResult,
    opa_malloc_func: funcs::OpaMalloc,
    opa_free_func: funcs::OpaFree,
    opa_json_parse_func: funcs::OpaJsonParse,
    opa_json_dump_func: funcs::OpaJsonDump,
    opa_heap_ptr_set_func: funcs::OpaHeapPtrSet,
    opa_heap_ptr_get_func: funcs::OpaHeapPtrGet,
    opa_eval_func: Option<funcs::OpaEval>,
}

impl<C> Debug for Runtime<C> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Runtime")
            .field("version", &self.version)
            .field("memory", &self.memory)
            .field("entrypoints", &self.entrypoints)
            .finish_non_exhaustive()
    }
}

impl Runtime<DefaultContext> {
    /// Load a new WASM policy module into the given store, with the default
    /// evaluation context.
    ///
    /// # Errors
    ///
    /// It will raise an error if one of the following condition is met:
    ///
    ///  - the provided [`wasmtime::Store`] isn't an async one
    ///  - the [`wasmtime::Module`] was created with a different
    ///    [`wasmtime::Engine`] than the [`wasmtime::Store`]
    ///  - the WASM module is not a valid OPA WASM compiled policy, and lacks
    ///    some of the exported functions
    ///  - it failed to load the entrypoints or the builtins list
    #[allow(clippy::too_many_lines)]
    pub async fn new<T: Send>(store: impl AsContextMut<Data = T>, module: &Module) -> Result<Self> {
        let context = DefaultContext::default();
        Self::new_with_evaluation_context(store, module, context).await
    }
}

impl<C> Runtime<C> {
    /// Load a new WASM policy module into the given store, with a given
    /// evaluation context.
    ///
    /// # Errors
    ///
    /// It will raise an error if one of the following condition is met:
    ///
    ///  - the provided [`wasmtime::Store`] isn't an async one
    ///  - the [`wasmtime::Module`] was created with a different
    ///    [`wasmtime::Engine`] than the [`wasmtime::Store`]
    ///  - the WASM module is not a valid OPA WASM compiled policy, and lacks
    ///    some of the exported functions
    ///  - it failed to load the entrypoints or the builtins list
    #[allow(clippy::too_many_lines)]
    pub async fn new_with_evaluation_context<T: Send>(
        mut store: impl AsContextMut<Data = T>,
        module: &Module,
        context: C,
    ) -> Result<Self>
    where
        C: EvaluationContext,
    {
        let ty = MemoryType::new(2, None);
        let memory = Memory::new_async(&mut store, ty).await?;

        // TODO: make the context configurable and reset it on evaluation
        let eventually_builtins = Arc::new(OnceCell::<LoadedBuiltins<C>>::new());

        let mut linker = Linker::new(store.as_context_mut().engine());
        linker.define(&store, "env", "memory", memory)?;

        linker.func_wrap(
            "env",
            "opa_abort",
            move |caller: Caller<'_, _>, addr: i32| -> Result<(), anyhow::Error> {
                let addr = NulStr(addr);
                let msg = addr.read(&caller, &memory)?;
                let msg = msg.to_string_lossy().into_owned();
                tracing::error!("opa_abort: {}", msg);
                anyhow::bail!(msg)
            },
        )?;

        linker.func_wrap(
            "env",
            "opa_println",
            move |caller: Caller<'_, _>, addr: i32| {
                let addr = NulStr(addr);
                let msg = addr.read(&caller, &memory)?;
                tracing::info!("opa_print: {}", msg.to_string_lossy());
                Ok(())
            },
        )?;

        {
            let eventually_builtins = eventually_builtins.clone();
            linker.func_wrap2_async(
                "env",
                "opa_builtin0",
                move |caller: Caller<'_, _>, builtin_id: i32, _ctx: i32| {
                    let eventually_builtins = eventually_builtins.clone();
                    Box::new(async move {
                        eventually_builtins
                            .get()
                            .expect("builtins where never initialized")
                            .builtin(caller, &memory, builtin_id, [])
                            .await
                    })
                },
            )?;
        }

        {
            let eventually_builtins = eventually_builtins.clone();
            linker.func_wrap3_async(
                "env",
                "opa_builtin1",
                move |caller: Caller<'_, _>, builtin_id: i32, _ctx: i32, param1: i32| {
                    let eventually_builtins = eventually_builtins.clone();
                    Box::new(async move {
                        eventually_builtins
                            .get()
                            .expect("builtins where never initialized")
                            .builtin(caller, &memory, builtin_id, [param1])
                            .await
                    })
                },
            )?;
        }

        {
            let eventually_builtins = eventually_builtins.clone();
            linker.func_wrap4_async(
                "env",
                "opa_builtin2",
                move |caller: Caller<'_, _>,
                      builtin_id: i32,
                      _ctx: i32,
                      param1: i32,
                      param2: i32| {
                    let eventually_builtins = eventually_builtins.clone();
                    Box::new(async move {
                        eventually_builtins
                            .get()
                            .expect("builtins where never initialized")
                            .builtin(caller, &memory, builtin_id, [param1, param2])
                            .await
                    })
                },
            )?;
        }

        {
            let eventually_builtins = eventually_builtins.clone();
            linker.func_wrap5_async(
                "env",
                "opa_builtin3",
                move |caller: Caller<'_, _>,
                      builtin_id: i32,
                      _ctx: i32,
                      param1: i32,
                      param2: i32,
                      param3: i32| {
                    let eventually_builtins = eventually_builtins.clone();
                    Box::new(async move {
                        eventually_builtins
                            .get()
                            .expect("builtins where never initialized")
                            .builtin(caller, &memory, builtin_id, [param1, param2, param3])
                            .await
                    })
                },
            )?;
        }

        {
            let eventually_builtins = eventually_builtins.clone();
            linker.func_wrap6_async(
                "env",
                "opa_builtin4",
                move |caller: Caller<'_, _>,
                      builtin_id: i32,
                      _ctx: i32,
                      param1: i32,
                      param2: i32,
                      param3: i32,
                      param4: i32| {
                    let eventually_builtins = eventually_builtins.clone();
                    Box::new(async move {
                        eventually_builtins
                            .get()
                            .expect("builtins where never initialized")
                            .builtin(
                                caller,
                                &memory,
                                builtin_id,
                                [param1, param2, param3, param4],
                            )
                            .await
                    })
                },
            )?;
        }

        let instance = linker.instantiate_async(&mut store, module).await?;

        let version = AbiVersion::from_instance(&mut store, &instance)?;
        tracing::debug!(%version, "Module ABI version");

        let opa_json_dump_func = funcs::OpaJsonDump::from_instance(&mut store, &instance)?;

        // Load the builtins map
        let builtins = funcs::Builtins::from_instance(&mut store, &instance)?
            .call(&mut store)
            .await?;
        let builtins = opa_json_dump_func
            .decode(&mut store, &memory, &builtins)
            .await?;
        let builtins = LoadedBuiltins::from_map(builtins, context)?;
        eventually_builtins.set(builtins)?;

        // Load the entrypoints map
        let entrypoints = funcs::Entrypoints::from_instance(&mut store, &instance)?
            .call(&mut store)
            .await?;
        let entrypoints = opa_json_dump_func
            .decode(&mut store, &memory, &entrypoints)
            .await?;

        let opa_eval_func = version
            .has_eval_fastpath()
            .then(|| funcs::OpaEval::from_instance(&mut store, &instance))
            .transpose()?;

        Ok(Self {
            version,
            memory,
            entrypoints,
            loaded_builtins: eventually_builtins,

            eval_func: funcs::Eval::from_instance(&mut store, &instance)?,
            opa_eval_ctx_new_func: funcs::OpaEvalCtxNew::from_instance(&mut store, &instance)?,
            opa_eval_ctx_set_input_func: funcs::OpaEvalCtxSetInput::from_instance(
                &mut store, &instance,
            )?,
            opa_eval_ctx_set_data_func: funcs::OpaEvalCtxSetData::from_instance(
                &mut store, &instance,
            )?,
            opa_eval_ctx_set_entrypoint_func: funcs::OpaEvalCtxSetEntrypoint::from_instance(
                &mut store, &instance,
            )?,
            opa_eval_ctx_get_result_func: funcs::OpaEvalCtxGetResult::from_instance(
                &mut store, &instance,
            )?,
            opa_malloc_func: funcs::OpaMalloc::from_instance(&mut store, &instance)?,
            opa_free_func: funcs::OpaFree::from_instance(&mut store, &instance)?,
            opa_json_parse_func: funcs::OpaJsonParse::from_instance(&mut store, &instance)?,
            opa_json_dump_func,
            opa_heap_ptr_set_func: funcs::OpaHeapPtrSet::from_instance(&mut store, &instance)?,
            opa_heap_ptr_get_func: funcs::OpaHeapPtrGet::from_instance(&mut store, &instance)?,
            opa_eval_func,
        })
    }

    async fn load_json<V: serde::Serialize, T: Send>(
        &self,
        store: impl AsContextMut<Data = T>,
        data: &V,
    ) -> Result<Value> {
        load_json(
            &self.opa_malloc_func,
            &self.opa_free_func,
            &self.opa_json_parse_func,
            store,
            &self.memory,
            data,
        )
        .await
    }

    /// Instanciate the policy with an empty `data` object
    ///
    /// # Errors
    ///
    /// If it failed to load the empty data object in memory
    pub async fn without_data<T: Send>(
        self,
        store: impl AsContextMut<Data = T>,
    ) -> Result<Policy<C>> {
        let data = serde_json::Value::Object(serde_json::Map::default());
        self.with_data(store, &data).await
    }

    /// Instanciate the policy with the given `data` object
    ///
    /// # Errors
    ///
    /// If it failed to serialize and load the `data` object
    pub async fn with_data<V: serde::Serialize, T: Send>(
        self,
        mut store: impl AsContextMut<Data = T>,
        data: &V,
    ) -> Result<Policy<C>> {
        let data = self.load_json(&mut store, data).await?;
        let heap_ptr = self.opa_heap_ptr_get_func.call(&mut store).await?;
        Ok(Policy {
            runtime: self,
            data,
            heap_ptr,
        })
    }

    /// Get the default entrypoint of this module. May return [`None`] if no
    /// entrypoint with ID 0 was found
    #[must_use]
    pub fn default_entrypoint(&self) -> Option<&str> {
        self.entrypoints
            .iter()
            .find_map(|(k, v)| (v.0 == 0).then_some(k.as_str()))
    }

    /// Get the list of entrypoints found in this module.
    #[must_use]
    pub fn entrypoints(&self) -> HashSet<&str> {
        self.entrypoints.keys().map(String::as_str).collect()
    }

    /// Get the ABI version detected for this module
    #[must_use]
    pub fn abi_version(&self) -> AbiVersion {
        self.version
    }
}

/// An instance of a policy, ready to be executed
#[derive(Debug)]
pub struct Policy<C> {
    runtime: Runtime<C>,
    data: Value,
    heap_ptr: Addr,
}

impl<C> Policy<C> {
    /// Evaluate a policy with the given entrypoint and input.
    ///
    /// # Errors
    ///
    /// Returns an error if the policy evaluation failed, or if this policy did
    /// not belong to the given store.
    pub async fn evaluate<V: serde::Serialize, R: for<'de> serde::Deserialize<'de>, T: Send>(
        &self,
        mut store: impl AsContextMut<Data = T>,
        entrypoint: &str,
        input: &V,
    ) -> Result<R>
    where
        C: EvaluationContext,
    {
        // Lookup the entrypoint
        let entrypoint = self
            .runtime
            .entrypoints
            .get(entrypoint)
            .with_context(|| format!("could not find entrypoint {entrypoint}"))?;

        self.loaded_builtins
            .get()
            .expect("builtins where never initialized")
            .evaluation_start()
            .await;

        // Take the fast path if it is awailable
        if let Some(opa_eval) = &self.runtime.opa_eval_func {
            // Write the input
            let input = serde_json::to_vec(&input)?;
            let input_heap = Heap {
                ptr: self.heap_ptr.0,
                len: input.len().try_into().context("input too long")?,
                // Not managed by a malloc
                freed: true,
            };

            // Check if we need to grow the memory first
            let current_pages = self.runtime.memory.size(&store);
            let needed_pages = input_heap.pages();
            if current_pages < needed_pages {
                self.runtime
                    .memory
                    .grow_async(&mut store, needed_pages - current_pages)
                    .await?;
            }

            // Write the JSON input to memory
            self.runtime.memory.write(
                &mut store,
                input_heap.ptr.try_into().context("invalid heap pointer")?,
                &input[..],
            )?;

            let heap_ptr = Addr(input_heap.end());

            // Call the eval fast-path
            let result = opa_eval
                .call(&mut store, entrypoint, &self.data, &input_heap, &heap_ptr)
                .await?;

            // Read back the JSON-formatted result
            let result = result.read(&store, &self.runtime.memory)?;
            let result = serde_json::from_slice(result.to_bytes())?;
            Ok(result)
        } else {
            // Reset the heap pointer
            self.runtime
                .opa_heap_ptr_set_func
                .call(&mut store, &self.heap_ptr)
                .await?;

            // Load the input
            let input = self.runtime.load_json(&mut store, input).await?;

            // Create a new evaluation context
            let ctx = self.runtime.opa_eval_ctx_new_func.call(&mut store).await?;

            // Set the data location
            self.runtime
                .opa_eval_ctx_set_data_func
                .call(&mut store, &ctx, &self.data)
                .await?;
            // Set the input location
            self.runtime
                .opa_eval_ctx_set_input_func
                .call(&mut store, &ctx, &input)
                .await?;

            // Set the entrypoint
            self.runtime
                .opa_eval_ctx_set_entrypoint_func
                .call(&mut store, &ctx, entrypoint)
                .await?;

            // Evaluate the policy
            self.runtime.eval_func.call(&mut store, &ctx).await?;

            // Get the results back
            let result = self
                .runtime
                .opa_eval_ctx_get_result_func
                .call(&mut store, &ctx)
                .await?;

            let result = self
                .runtime
                .opa_json_dump_func
                .decode(&mut store, &self.runtime.memory, &result)
                .await?;

            Ok(result)
        }
    }
}

impl<C> Deref for Policy<C> {
    type Target = Runtime<C>;
    fn deref(&self) -> &Self::Target {
        &self.runtime
    }
}
