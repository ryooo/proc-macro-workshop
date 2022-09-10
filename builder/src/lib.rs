use proc_macro::TokenStream;

#[proc_macro_derive(Builder)]
pub fn derive_builder(_input: TokenStream) -> TokenStream {
    TokenStream::new()
}
