use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, PartialOrd, Serialize, Deserialize)]
pub enum Variant {
    // Normal types
    String(String),
    Boolean(bool),
    // Unsigned Integers
    U8(u8),
    U16(u16),
    U32(u32),
    U64(u64),
    U128(u128),
    // Signed Integers
    I8(i8),
    I16(i16),
    I32(i32),
    I64(i64),
    I128(i128),
    // Floating point numbers
    F32(f32),
    F64(f64),
}
