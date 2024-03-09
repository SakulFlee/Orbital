use sycamore::prelude::*;

#[component]
#[allow(non_camel_case_types)]
pub fn PageIndex<G: Html>() -> View<G> {
    view! {
        p {
            a(href="/Engine") { "Click here" }
            " to go to Engine"
        }
    }
}
