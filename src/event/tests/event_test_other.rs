use crate::event::Event;

pub struct OtherEventTest;

impl Event for OtherEventTest {
    fn identifier(&self) -> String {
        String::from("other.test")
    }
}
