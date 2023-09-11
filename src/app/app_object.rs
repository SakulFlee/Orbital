use crate::{app::app_input_handler::AppInputHandler, engine::vertex::Vertex};

pub trait AppObject {
    fn on_dynamic_update(&mut self, delta_time: f64) {}
    fn do_dynamic_update(&self) -> bool {
        false
    }

    fn on_second_update(&mut self, delta_time: f64) {}
    fn do_second_update(&self) -> bool {
        false
    }

    fn on_input(&mut self, delta_time: f64, input_handler: &AppInputHandler) {}
    fn do_input(&self) -> bool {
        false
    }

    fn vertices(&self) -> &[Vertex] {
        &[]
    }
    fn indices(&self) -> &[u16] {
        &[]
    }
    fn do_render(&self) -> bool {
        true
    }
}
