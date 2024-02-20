use proc_macro::TokenStream;

mod core;

#[proc_macro_derive(Component)]
pub fn derive_component(input: TokenStream) -> proc_macro::TokenStream {
    core::derive_component(input)
}
