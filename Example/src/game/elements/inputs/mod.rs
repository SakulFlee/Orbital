use orbital::{
    game::{Element, WorldChange},
    input::InputFrame,
    log::debug,
};

pub struct Input;

impl Element for Input {
    fn on_update(
        &mut self,
        delta_time: f64,
        input_frame: &Option<&InputFrame>,
    ) -> Option<Vec<WorldChange>> {
        debug!("Input Frame: {:?}@{}", input_frame, delta_time);

        None
    }
}
