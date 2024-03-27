use crate::event::Event;

pub struct EventTest;

impl Event for EventTest {
    fn identifier(&self) -> String {
        String::from("test")
    }
}
