use sycamore::prelude::*;

#[component]
#[allow(non_camel_case_types)]
pub fn PageEngine<G: Html>() -> View<G> {
    view! {
        p {
            "Engine here"
        }
    }
}
