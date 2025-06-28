use std::{
    collections::HashMap,
    hash::Hash,
    ops::{Deref, DerefMut},
};

use crate::resources::Variable;

/// Variables bundled into a struct to support `Eq` and `Hash`.
/// **This struct implements `Deref`!**  
/// Most, if not all, operations have to be done after dereferencing the value like so:
/// ```rust
/// # use shader::Variable;
/// # use shader::Variables;
///
/// let variables = Variables::new();
///
/// let first_variable = (*variables).get(&0);
/// let second_variable = (*variables).get(&1);
/// // etc. ...
///
/// // Or using a e.g. for-loop:
/// for (key, variable) in &*variables {
/// }
/// ```
#[derive(Debug, PartialEq, Eq)]
pub struct Variables {
    variables: HashMap<u32, Variable>,
}

impl Default for Variables {
    fn default() -> Self {
        Self::new()
    }
}

impl Variables {
    pub fn new() -> Self {
        Self {
            variables: HashMap::new(),
        }
    }
}

impl Hash for Variables {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        for (key, value) in &self.variables {
            key.hash(state);
            value.hash(state);
        }
    }
}

impl Deref for Variables {
    type Target = HashMap<u32, Variable>;

    fn deref(&self) -> &Self::Target {
        &self.variables
    }
}

impl DerefMut for Variables {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.variables
    }
}
