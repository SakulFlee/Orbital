use sycamore::prelude::*;

#[component]
#[allow(non_camel_case_types)]
pub fn PageNotFound<G: Html>() -> View<G> {
    view! {
        h1 {
            "#404: Not found! This page doesn't exist. You may have broken this app. 🥳👏"
        }
        p {
            "Try reloading this page (CTRL+R or F5 on most devices) or going back and trying again."
        }
        hr()
        p {
            "If this issue persists, please open a "
            a(href="https://github.com/SakulFlee/Akimo-Project/issues/new/choose") {"issue on GitHub" }
            "!"
        }
        p {
            "Thank you :)"
        }
    }
}
