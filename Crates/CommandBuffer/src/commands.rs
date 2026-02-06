/// All available commands to be used by guests.
///
/// ## About 'String' types
/// To properly encode strings, each string is prefaced with a number specifing how many characters
/// there are in the string.
/// Thus, each "String" mentioned below is actually two types:
///
/// | Type | Amount | Description |
/// | - | - | - |
/// |  u32  | One | Character count of the string (i.e. how many  u8 / bytes  are following?) |
/// |  u8 | Many (Array) | Actual  u8  /  bytes  of the string |
#[repr(u8)]
#[derive(Debug)]
pub enum Commands {
    // --- Entity ---
    /// Spawns an [Entity].  
    /// Each [Entity] needs an ID which is assigned by the engine.
    /// To retrieve this ID later on, a query has to be submitted.
    ///
    /// Data layout:
    /// | Type | Amount | Description |
    /// | - | - | - |
    /// | `u32` | One | Component count; Can be zero if an "empty" Entity is desired! |
    /// | `ComponentType` | Many | The component and its values.
    ///
    ///  `ComponentType` :
    /// | Type | Amount | Description |
    /// | - | - | - |
    /// | `u32` | One | Component ID |
    /// | `u8` | Many (POD) | The **whole** components value as POD (= Plain-Old-Data) |
    ///
    /// > Note: **All** values of a component have to be set to be a valid command!
    SpawnEntiy = 0,
    /// Despawns an [Entity].
    /// > The [Entity] identifier needs to be queried first!
    ///
    /// Data layout:
    /// | Type | Amount | Description |
    /// | - | - | - |
    /// | `u32` | One | Entity ID |
    DespawnEntity = 1,
    // --- Component ---
    /// Registers a component.  
    /// This can **ONLY** be used during the startup sequence!
    ///
    /// The engine will assign a component ID to this component.
    /// After startup, the ID mapping will be shared, so that every guest can locally cache the
    /// correct IDs.
    ///
    /// Data layout:
    /// | Type | Amount | Description |
    /// | - | - | - |
    /// | `String` | One | Component name |
    /// | `u32` | One | Field count |
    /// | `u8` | Many | Type of the field (see [FieldType]), converted as u8 |
    RegisterComponent = 2,
    /// Updates a components values.
    ///     
    /// Data layout:
    /// | Type | Amount | Description |
    /// | - | - | - |
    /// | `u32` | One | Entity ID |
    /// | `u32` | One | Component ID |
    /// | `u8` | Many (POD) | The **whole** components value as POD (= Plain-Old-Data) |
    ///
    /// > Note: **All** values of a component have to be set to be a valid command!
    UpdateComponentValues = 3,
    /// Attaches a Component to an Entity.
    /// > The Entity identifier needs to be queried first!
    ///
    /// Data layout:
    /// | Type | Amount | Description |
    /// | - | - | - |    
    /// | `u32` | One | Entity ID |
    /// | `u32` | One | Component ID |
    /// | `u8` | Many (POD) | The **whole** components value as POD (= Plain-Old-Data) |
    ///
    /// > Note: **All** values of a component have to be set to be a valid command!
    AttachComponent = 4,
    // --- System ---
    /// Registers a system to be called by the engine with a query response.
    ///
    /// _Data layout_:    
    /// | Type | Amount | Description |
    /// | - | - | - |    
    /// | `String` | One | Function name (must be exported from WASM!) |
    /// | `Query` | One | Query this function expects. |
    ///
    /// ## Query
    /// A Query consits out of three parts:
    ///  - Components: Components the System actually requires in data (POD) form.
    ///  - Include: Components an Entity has to have to be included in the selection. Mostly used for including tags/markers.
    ///  - Exclude: Comonents an identifiery cannot have to be included in the selection. Mostly used for excluding tags / markers.
    ///     
    /// Each of these parts are a list of Strings.
    /// For the protocol, encoder and decoder to know how many strings there are and which belongs
    /// to which part, we preface with three counts, one for each.
    ///
    /// _Query Layout_:
    /// | Type | Amount | Description |
    /// | - | - | - |    
    /// | `u32` | One | Component count |
    /// | `u32` | One | Include count |
    /// | `u32` | One | Exclude count |
    /// | `String` | Many | Component names |
    /// | `String` | Many | Included component names |
    /// | `String` | Many | Excluded component names |
    RegisterSystem = 5,
}
