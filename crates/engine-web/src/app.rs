use sycamore::prelude::*;
use sycamore_router::{HistoryIntegration, Router};

use crate::app_routes::AppRoutes;

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
                                AppRoutes::Index => view! {
                                    p {
                                        a(href="/Engine") { "Click here" }
                                        " to go to Engine"
                                    }
                                },
                                AppRoutes::Engine => view! {
                                    "Engine"
                                },
                                AppRoutes::NotFound => view! {
                                    "#404 - This page doesn't exist!"
                                },
                            })
                        }
                    }
                }
            )
        }
    });
}
