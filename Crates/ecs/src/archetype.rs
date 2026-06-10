pub struct Archetype {
    /// Bitmask of ComponentIDs
    pub mask: u64,
    /// Each inner Vec stores a contiguous buffer of bytes for one component type each.
    /// E.g.:
    ///  - Vec[0] -> All Positions
    ///  - Vec[0][0..(size of Position struct)] -> First Position
    ///  - Vec[0][(size of Position struct)..(offset + size of Position struct)] -> Second Position
    ///  - Vec[1] -> All Velocities
    ///  - etc.
    pub columns: Vec<Vec<u8>>,
    /// "Stride"; How many bytes to skip to get the the next component.
    /// Or, how big a single component inside this Archetype is.
    pub column_sizes: Vec<usize>,
    /// Indices of entities in this archetype
    pub entities: Vec<usize>,
}

impl Archetype {
    /// `mask`: the ComponentID
    /// `component_sizes`: an order set of the exact length in bytes per component **within** this
    /// archetype.
    pub fn new(mask: u64, component_sizes: Vec<usize>) -> Self {
        let mut columns = Vec::new();
        for _ in 0..component_sizes.len() {
            columns.push(Vec::new());
        }

        Self {
            mask,
            columns,
            column_sizes: component_sizes,
            entities: Vec::new(),
        }
    }

    pub fn push_entity(&mut self, entity_index: usize, component_data: Vec<&[u8]>) {
        for (i, data_bytes) in component_data.into_iter().enumerate() {
            self.columns[i].extend_from_slice(data_bytes);
        }
        self.entities.push(entity_index);
    }
}
