use std::error::Error;

use log::{info, warn};
use wasmtime::{
    Config, Engine, Store,
    component::{Component, Linker},
};
use wasmtime_wasi::{
    ResourceTable, WasiCtx, WasiCtxBuilder, WasiCtxView, WasiView, p2::add_to_linker_sync,
};

use crate::bindings::Orbital;

mod bindings;

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
    orbital_guest_instance: Orbital,
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

        let orbital_guest_instance = Orbital::instantiate(&mut store, &component, &linker)?;

        Ok(Self {
            orbital_guest_instance,
            store,
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

    pub fn call_startup_function(&mut self) -> Result<(), Box<dyn Error>> {
        let guest = self.orbital_guest_instance.orbital_core_module();
        let result = guest.call_startup(&mut self.store)?;
        info!("Resulting CommandBuffer: {:?}", result);

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use crate::Module;

    #[test]
    fn test_module() {
        println!(
            "NOTE: If the following test FAILS, make sure you have (re-)build the 'wit_test_module' crate for WASM-WASI-P2:"
        );
        println!("cargo build --target wasm32-wasip2 --package wit_test_module");

        let mut module = Module::new(include_bytes!(
            "../../../target/wasm32-wasip2/debug/wit_test_module.wasm"
        ))
        .expect("Failed to initialize module");
        module
            .call_startup_function()
            .expect("Failed to call startup function!");
    }
}
