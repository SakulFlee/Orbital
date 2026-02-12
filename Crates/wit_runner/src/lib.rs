use std::error::Error;

use log::warn;
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

        Ok(Self { instance, store })
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

#[cfg(test)]
mod tests {
    use crate::Module;

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
