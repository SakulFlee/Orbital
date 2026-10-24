//! Procedural macros for the Orbital engine.
//!
//! The only export today is [`macro@main`], which replaces Orbital's
//! `make_desktop_main!` / `make_android_main!` / `make_ios_main!` macro trio
//! with a single attribute on the app's entrypoint.

use proc_macro::TokenStream;
use proc_macro_crate::{FoundCrate, crate_name};
use proc_macro2::Span;
use quote::quote;
use syn::{Ident, ItemFn, parse_macro_input};

/// Generates every platform entrypoint from a single app entrypoint.
///
/// Applied to a function taking
/// `Result<orbital::winit::event_loop::EventLoop<()>, orbital::winit::error::EventLoopError>`,
/// this emits a desktop `run()` plus the Android and iOS entrypoints:
///
/// ```ignore
/// #[orbital::main]
/// pub fn entrypoint(
///     event_loop: Result<
///         orbital::winit::event_loop::EventLoop<()>,
///         orbital::winit::error::EventLoopError,
///     >,
/// ) {
///     let event_loop = event_loop.expect("Event Loop failure");
///     // ...
/// }
/// ```
///
/// `src/main.rs` then only needs `fn main() { <crate>::run(); }`.
///
/// The entrypoint receives the result of `EventLoop::builder().build()`
/// unwrapped, so it can report a failed event loop however it likes instead of
/// panicking inside a macro-generated function.
#[proc_macro_attribute]
pub fn main(_attr: TokenStream, item: TokenStream) -> TokenStream {
    let input = parse_macro_input!(item as ItemFn);
    let ident = &input.sig.ident;
    let vis = &input.vis;
    let sig = &input.sig;
    let block = &input.block;
    let attrs = &input.attrs;

    quote! {
        #(#attrs)*
        #vis #sig #block

        /// Builds the event loop and hands it to the entrypoint.
        ///
        /// Call this from `fn main()`; Android and iOS use their own
        /// entrypoints instead.
        #[allow(dead_code)]
        pub fn run() {
            let event_loop = ::orbital::winit::event_loop::EventLoop::builder().build();

            #ident(event_loop);
        }

        #[cfg(target_os = "android")]
        #[allow(dead_code)]
        #[unsafe(no_mangle)]
        fn android_main(app: ::orbital::winit::platform::android::activity::AndroidApp) {
            ::orbital::logging::init();

            let _ = ::orbital::file_manager::FileManager::init_android_global(
                app.asset_manager(),
                app.internal_data_path(),
            );

            ::orbital::app::android_performance::init(&app);

            use ::orbital::winit::platform::android::EventLoopBuilderExtAndroid;

            let event_loop = match ::orbital::winit::event_loop::EventLoop::builder()
                .with_android_app(app)
                .build()
            {
                Ok(event_loop) => event_loop,
                Err(e) => {
                    ::orbital::logging::error!("Event loop build failed: {:?}", e);

                    // winit allows only one EventLoop per process. This
                    // android_main was invoked for a recreated activity in an
                    // already-running process, so there is nothing to do here.
                    // Return (and let this thread end) like Bevy does, leaving
                    // the original event loop alive so the app keeps running
                    // and resumes on the next open.
                    return;
                }
            };

            #ident(Ok(event_loop));

            // The event loop only returns when the app exits. Just return from
            // android_main; do not kill the process.
        }

        #[cfg(target_os = "ios")]
        #[allow(dead_code)]
        #[unsafe(no_mangle)]
        extern "C" fn ios_main() {
            ::orbital::logging::init();

            let _ = ::orbital::file_manager::FileManager::init_ios_global();

            let event_loop = ::orbital::winit::event_loop::EventLoop::builder().build();

            #ident(event_loop);
        }
    }
    .into()
}

/// Derives [`orbital_ecs_bridge::EditableFields`] for a struct with named fields.
///
/// Every included field's type must implement
/// `orbital_ecs_bridge::EditableValue`. Mark fields to leave out with
/// `#[editable(skip)]`.
#[proc_macro_derive(EditableFields, attributes(editable))]
pub fn editable_fields(item: TokenStream) -> TokenStream {
    let input = parse_macro_input!(item as syn::DeriveInput);
    let name = &input.ident;
    let (impl_generics, ty_generics, where_clause) = input.generics.split_for_impl();
    let support = support_path();

    let syn::Data::Struct(data) = &input.data else {
        return syn::Error::new_spanned(name, "EditableFields only supports structs")
            .to_compile_error()
            .into();
    };

    let syn::Fields::Named(fields) = &data.fields else {
        return syn::Error::new_spanned(name, "EditableFields requires named fields")
            .to_compile_error()
            .into();
    };

    let included = fields
        .named
        .iter()
        .filter(|field| !is_skipped(&field.attrs))
        .filter_map(|field| field.ident.as_ref())
        .collect::<Vec<_>>();

    let pushes = included.iter().map(|ident| {
        quote! {
            fields.push(#support::EditableField {
                id: ::std::string::String::from(stringify!(#ident)),
                label: ::std::string::String::from(stringify!(#ident)),
                value: #support::EditableValue::to_edit_string(&self.#ident),
            });
        }
    });

    let setters = included.iter().map(|ident| {
        quote! {
            stringify!(#ident) => {
                #support::EditableValue::set_from_str(&mut self.#ident, input)
            }
        }
    });

    quote! {
        impl #impl_generics #support::EditableFields for #name #ty_generics #where_clause {
            fn editable_fields(&self) -> ::std::vec::Vec<#support::EditableField> {
                let mut fields = ::std::vec::Vec::new();
                #(#pushes)*
                fields
            }

            fn set_editable_field(
                &mut self,
                id: &str,
                input: &str,
            ) -> ::std::result::Result<(), ::std::string::String> {
                match id {
                    #(#setters,)*
                    _ => ::std::result::Result::Err(::std::format!("unknown editable field `{id}`")),
                }
            }
        }
    }
    .into()
}

/// Resolves the crate that owns the editing traits for the deriving crate.
///
/// Engine components live in `orbital_ecs_bridge`. Applications commonly depend
/// only on `orbital`, which re-exports the bridge, so fall back to
/// `orbital::ecs_bridge` when the bridge is not a direct dependency.
fn support_path() -> proc_macro2::TokenStream {
    if let Ok(found) = crate_name("orbital_ecs_bridge") {
        return match found {
            FoundCrate::Itself => quote!(crate),
            FoundCrate::Name(name) => {
                let ident = Ident::new(&name, Span::call_site());
                quote!(::#ident)
            }
        };
    }

    if let Ok(found) = crate_name("orbital") {
        return match found {
            FoundCrate::Itself => quote!(crate::ecs_bridge),
            FoundCrate::Name(name) => {
                let ident = Ident::new(&name, Span::call_site());
                quote!(::#ident::ecs_bridge)
            }
        };
    }

    quote!(::orbital_ecs_bridge)
}

fn is_skipped(attrs: &[syn::Attribute]) -> bool {
    attrs.iter().any(|attr| {
        if !attr.path().is_ident("editable") {
            return false;
        }

        let mut skip = false;
        let _ = attr.parse_nested_meta(|meta| {
            if meta.path.is_ident("skip") {
                skip = true;
            }
            Ok(())
        });

        skip
    })
}
