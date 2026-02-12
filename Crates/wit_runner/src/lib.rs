use std::error::Error;

use log::{debug, warn};
use wasmtime::{
    Config, Engine, Store,
    component::{Component, Instance, Linker},
};
use wasmtime_wasi::{
    ResourceTable, WasiCtx, WasiCtxBuilder, WasiCtxView, WasiView, p2::add_to_linker_sync,
};

struct ModuleCtx {
    wasi_ctx: WasiCtx,
    resource_table: ResourceTable,
}

impl WasiView for ModuleCtx {
    fn ctx(&mut self) -> wasmtime_wasi::WasiCtxView<'_> {
        WasiCtxView {
            ctx: &mut self.wasi_ctx,
            table: &mut self.resource_table,
        }
    }
}

pub struct Module {
    instance: Instance,
    store: Store<ModuleCtx>,
    component: Component,
}

impl Module {
    pub fn new(bytes: &[u8]) -> Result<Self, Box<dyn Error>> {
        let config = Self::make_config();
        let engine = Engine::new(&config)?;
        let component = Component::new(&engine, bytes)?;
        let component_type = component.component_type();

        let exports = component_type.exports(&engine);
        if exports.len() == 0 {
            warn!(
                "Attempting to load module with zero exports. This module won't be able to function!"
            );
        }

        #[cfg(debug_assertions)]
        {
            use log::info;

            info!("Exports defined by the module:");
            for (name, item) in exports {
                info!("- {}: {:?}", name, item);
            }
        }

        let wasi_ctx = Self::make_wasi_ctx();
        let mut store = Self::make_store(&engine, wasi_ctx);
        let linker = Self::make_linker(&engine)?;

        let instance = linker.instantiate(&mut store, &component)?;

        Ok(Self {
            instance,
            store,
            component,
        })
    }

    fn make_config() -> Config {
        let mut config = Config::new();
        config.wasm_component_model(true);

        config
    }

    fn make_wasi_ctx() -> WasiCtx {
        let mut builder = WasiCtxBuilder::new();
        builder.inherit_stdout();
        builder.inherit_stderr();
        builder.build()
    }

    fn make_store(engine: &Engine, wasi_ctx: WasiCtx) -> Store<ModuleCtx> {
        Store::new(
            engine,
            ModuleCtx {
                wasi_ctx,
                resource_table: ResourceTable::new(),
            },
        )
    }

    fn make_linker(engine: &Engine) -> Result<Linker<ModuleCtx>, Box<dyn Error>> {
        let mut linker = Linker::new(engine);
        add_to_linker_sync(&mut linker)?;

        Ok(linker)
    }

    pub fn call_interface_function(
        &mut self,
        interface: &str,
        function: &str,
    ) -> Result<(), Box<dyn Error>> {
        let (interface, interface_index) = self
            .instance
            .get_export(&mut self.store, None, interface)
            .unwrap();
        println!("Interface: {:?}", interface);

        let function_export_index = self
            .instance
            .get_export_index(&mut self.store, Some(&interface_index), function)
            .expect("Function not found");
        println!("FEI: {:?}", function_export_index);

        let function = self
            .instance
            .get_func(&mut self.store, function_export_index)
            .expect("Failed to resolve index to a callable function");
        println!("Function: {:?}", function);

        println!("Calling NOW!");
        // let mut results = vec![Val::Bool(false); 1];

        println!(">>> START: WASI <<<");
        function
            .call(&mut self.store, &[], &mut [])
            .expect("Runtime execution error");
        // println!("Results: {:?}", results);
        println!(">>> END: WASI <<<");

        println!("Cleanup");
        function
            .post_return(&mut self.store)
            .expect("Cleanup failure");

        Ok(())
    }

    pub fn call_startup_function(&mut self) -> Result<(), Box<dyn Error>> {
        self.call_interface_function("orbital:module/module-impl@0.1.0", "startup")
    }
}

fn run() {
    let mut config = Config::new();
    config.wasm_component_model(true);
    let engine = Engine::new(&config).expect("WasmTime engine startup failure");

    let component_bytes = include_bytes!("../../../target/wasm32-wasip2/debug/wit_test.wasm");
    let component = Component::new(&engine, component_bytes).expect("Component import failed");

    let component_type = component.component_type();
    println!("Checking for exports ...");
    for (export, item) in component_type.exports(&engine) {
        println!("Export: {export}");
    }

    let mut builder = WasiCtxBuilder::new();
    // Allow guest to print to terminal:
    builder.inherit_stdio();
    let wasi = builder.build();

    let mut store = Store::new(
        &engine,
        ModuleCtx {
            wasi_ctx: wasi,
            resource_table: ResourceTable::new(),
        },
    );

    let mut linker = Linker::new(&engine);
    // Add all standard WASI implementations to the linker:
    wasmtime_wasi::p2::add_to_linker_sync(&mut linker).expect("WASI std linking failure!");

    let instance = linker
        .instantiate(&mut store, &component)
        .expect("Failed to instanciate");

    let (interface, interface_index) = instance
        .get_export(&mut store, None, "orbital:wit-test/test-interface@0.1.0")
        .unwrap();
    println!("Interface: {:?}", interface);

    let function_export_index = instance
        .get_export_index(&mut store, Some(&interface_index), "test-function")
        .expect("Function not found");
    println!("FEI: {:?}", function_export_index);

    let function = instance
        .get_func(&mut store, function_export_index)
        .expect("Failed to resolve index to a callable function");
    println!("Function: {:?}", function);

    println!("Calling NOW!");
    // let mut results = vec![Val::Bool(false); 1];

    println!(">>> START: WASI <<<");
    function
        .call(&mut store, &[], &mut [])
        .expect("Runtime execution error");
    // println!("Results: {:?}", results);
    println!(">>> END: WASI <<<");

    println!("Cleanup");
    function.post_return(&mut store).expect("Cleanup failure");

    // let ComponentItem::ComponentInstance(exported_instance) = export_item else {
    //     panic!("Unexpected result: {:?} @ {:?}", export_item, export_index);
    // };
    // println!("C-Instance: {:?}", exported_instance);
    //
    // for (label, component) in exported_instance.exports(&engine) {
    //     println!(">>> {label}");
    // }
    //
    // let component = exported_instance
    //     .get_export(&engine, "test-function")
    //     .expect("Function missing!");
    // let ComponentItem::ComponentFunc(cfe) = component else {
    //     panic!("Unexpected type: {:?}", component);
    // };
    // println!("C-Fe: {:?}", cfe);
    //
    // // ---
    // let f = exported_instance
    //     .get_export(&mut store, "test-function")
    //     .expect("failed finding function");
    // f.func
}

#[cfg(test)]
mod tests {
    use crate::{Module, run};

    #[test]
    fn test() {
        run();
    }

    #[test]
    fn test_module() {
        simple_log::quick!("info");

        let mut module = Module::new(include_bytes!(
            "../../../target/wasm32-wasip2/debug/wit_test.wasm"
        ))
        .expect("Failed to initialize module");
        module
            .call_startup_function()
            .expect("Failed to call startup function!");
    }
}
