use sycamore_router::Route;

#[derive(Route, Clone, Copy, Debug)]
pub enum AppRoutes {
    #[to("/")]
    Index,
    #[to("/Engine")]
    Engine,
    #[not_found]
    NotFound,
}
