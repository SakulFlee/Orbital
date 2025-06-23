/// Origin of a message.
/// This is not enforced, use carefully!
#[derive(Debug, PartialEq, Eq, Hash, Clone)]
pub enum Origin {
    /// Used if a message originates from the app.
    App,
    /// Used if a message originates from an element.
    Element { label: String },
}
