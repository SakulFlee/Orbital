pub trait Event {
    fn identifier(&self) -> String;
}
