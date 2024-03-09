use sycamore::prelude::*;
use sycamore_router::{HistoryIntegration, Router};

use crate::app_routes::AppRoutes;
use crate::components::{PageEngine, PageIndex, PageNotFound};

#[allow(non_camel_case_types)]
pub fn render() {
    sycamore::render(|| {
        view! {
            Router(
                integration=HistoryIntegration::new(),
                view=| route: ReadSignal<AppRoutes>| {
                    view !{
                         div(class="app") {
                            (match route.get() {
                                AppRoutes::Index => view! {PageIndex {}},
                                AppRoutes::Engine => view! {PageEngine {}},
                                AppRoutes::NotFound => view! {PageNotFound {}},
                            })
                        }
                    }
                }
            )
        }
    });
}
