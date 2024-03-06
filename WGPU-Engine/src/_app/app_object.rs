use crate::{app::app_input_handler::AppInputHandler, Model};

pub trait AppObject {
    fn on_dynamic_update(&mut self, _delta_time: f64) {}
    fn do_dynamic_update(&self) -> bool {
        false
    }

    fn on_second_update(&mut self, _delta_time: f64) {}
    fn do_second_update(&self) -> bool {
        false
    }

    fn on_input(&mut self, _delta_time: f64, _input_handler: &AppInputHandler) {}
    fn do_input(&self) -> bool {
        false
    }

    fn model(&self) -> Option<&Model> {
        None
    }
    fn do_render(&self) -> bool {
        false
    }
}
