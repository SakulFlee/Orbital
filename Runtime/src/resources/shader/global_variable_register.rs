use hashbrown::HashMap;

use crate::resources::{Variable, VariableType};

pub type GlobalVariableRegister = GlobalSharedVariableRegister;
pub type GlobalRegister = GlobalVariableRegister;

#[derive(Debug)]
pub struct GlobalSharedVariableRegister {
    register: HashMap<String, Variable>,
}

impl GlobalSharedVariableRegister {
    pub fn new() -> Self {
        Self {
            register: HashMap::new(),
        }
    }

    pub fn register(&mut self, variable_type: VariableType) {
        Variable::
    }
}
