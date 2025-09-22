use orbital::{
    async_trait::async_trait,
    element::{Element, ElementRegistration},
};

#[derive(Debug)]
pub struct TestLights;

#[async_trait]
impl Element for TestLights {
    fn on_registration(&self) -> ElementRegistration {
        ElementRegistration::new("test_lights")
            // For now, we'll just register the element without lights
            // The lights will be added through the WorldEvent::Light directly
    }
}
