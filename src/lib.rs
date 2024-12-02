use std::collections::HashMap;

/// Representation of a Json value
pub enum Value {
    /// literal characters `null`
    Null,

    /// literal characters `true` or `false`
    Boolean(bool),

    /// characters within double quotes "..."
    String(String),

    /// numbers stored as a 64-bit floating point
    Number(f64),

    /// Zero to many JSON values
    Array(Vec<Value>),

    /// String keys with JSON values
    Object(HashMap<String, Value>),
}
