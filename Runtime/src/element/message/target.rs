/// Target of a message.
#[derive(Debug, PartialEq, Eq, Hash, Clone)]
pub enum Target {
    /// Used if a message targets all elements.
    /// Use only if necessary.
    Broadcast,
    /// Used if a message targets a specific element.
    Element {
        /// One or more label(s) of the target element(s).
        /// List multiple elements here if you want to send the same message to multiple elements.
        ///
        /// ⚠️ If you have an element with multiple labels defined and do list all labels here,
        ///    the on_message will be called multiple times on the same element!
        labels: Vec<String>,
    },
}
