use proc_macro::TokenStream;
use quote::quote;
use syn::{parse_macro_input, ItemFn};

fn entrypoint_impl(item: TokenStream) -> TokenStream {
    let input = parse_macro_input!(item as ItemFn);
    let fn_name = &input.sig.ident;
    let vis = &input.vis;
    let attrs = &input.attrs;
    let block = &input.block;
    let sig = &input.sig;

    let output = quote! {
        #(#attrs)*
        #vis #sig #block

        #[cfg(not(target_os = "android"))]
        #[allow(dead_code)]
        fn main() {
            let event_loop = ::winit::event_loop::EventLoop::builder()
                .build()
                .expect("Failed to create EventLoop");
            #fn_name(event_loop);
        }

        #[cfg(target_os = "android")]
        #[allow(non_snake_case)]
        #[no_mangle]
        fn android_main(app: ::winit::platform::android::activity::AndroidApp) {
            let event_loop = ::winit::event_loop::EventLoop::builder()
                .with_android_app(app)
                .build()
                .expect("Failed to create EventLoop");
            #fn_name(event_loop);
        }
    };

    output.into()
}

#[proc_macro_attribute]
pub fn entrypoint(_attr: TokenStream, item: TokenStream) -> TokenStream {
    entrypoint_impl(item)
}

#[proc_macro_attribute]
pub fn main(_attr: TokenStream, item: TokenStream) -> TokenStream {
    entrypoint_impl(item)
}
