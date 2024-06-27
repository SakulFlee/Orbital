use akimo_runtime::{
    game::{Element, ElementRegistration, World},
    hashbrown::HashMap,
    log::{debug, info},
    ulid::Ulid,
};

#[derive(Debug, Default)]
pub struct TestElement {}

impl Element for TestElement {
    fn on_registration(&mut self, ulid: &Ulid) -> ElementRegistration {
        ElementRegistration::default()
    }

    fn on_update(&mut self, delta_time: f64, world: &mut World) {
        debug!("Delta: {}ms", delta_time);
    }

    fn on_message(&mut self, message: HashMap<String, akimo_runtime::variant::Variant>) {
        info!("On Message: {:#?}", message);
    }
}
