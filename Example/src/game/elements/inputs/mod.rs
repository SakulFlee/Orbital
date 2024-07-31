use orbital::{
    app::InputEvent,
    game::{Element, WorldChange},
    log::debug,
};

pub struct Input;

impl Element for Input {
    fn on_input_event(
        &mut self,
        _delta_time: f64,
        input_event: &InputEvent,
    ) -> Option<Vec<WorldChange>> {
        debug!("InputEvent: {:?}", input_event);

        None
    }
}
