use std::collections::HashMap;
use wasmer::{Engine, Instance, Module, Store, Value};

// this contains only a single `test` function taking an i32, always returns 0i32 (see `test-wasm` crate)
static WASM: &[u8] = include_bytes!("../../test_wasm.wasm");

fn instantiate_and_call_wasm() {
    // for some reason the HashMap makes a difference. If you use the engine and module directly,
    // the failure does not occur (or maybe it's just less likely? maybe a timing issue?)
    let mut cache = HashMap::new();

    // compile wasm
    let (_engine, module) = compile(WASM);
    // store in cache
    cache.insert("asdf", (Engine::headless(), module)); // also happens when inserting `_engine` instead of `Engine::headless()`

    // removing from the map seems to be fine, so maybe the cloning is the problem?
    // let (cached_engine, cached_module) = cache.remove("asdf").unwrap();
    // also: if I reuse the `Singlepass` engine from the `compile` function, the failure occurs too
    let (cached_engine, cached_module) = cache.get("asdf").unwrap();
    let mut store = Store::new(cached_engine.clone());

    let instance = Instance::new(&mut store, cached_module, &wasmer::Imports::new()).unwrap();

    // call function
    let func = instance.exports.get_function("test").unwrap();
    let result = func.call(&mut store, &[Value::I32(0)]).unwrap();

    assert_eq!(result[0].unwrap_i32(), 0);
}

fn compile(code: &[u8]) -> (Engine, Module) {
    let engine = wasmer::Singlepass::default().into();
    let module = Module::new(&engine, code).unwrap();
    (engine, module)
}

pub fn main() {
    // run and wait until assertion fails
    loop {
        run_threads(instantiate_and_call_wasm);
    }
}

fn run_threads(function: fn() -> ()) {
    // spawn many threads, even if there are few functions
    const TARGET_THREAD_COUNT: usize = 20;
    let mut threads = Vec::with_capacity(TARGET_THREAD_COUNT);

    for _ in 0..TARGET_THREAD_COUNT {
        let thread = std::thread::spawn(function);
        threads.push(thread);
    }

    for thread in threads {
        thread.join().unwrap();
    }
}
