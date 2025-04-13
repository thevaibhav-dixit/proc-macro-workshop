use proc_macro::TokenStream;
use syn::{parse_macro_input, DeriveInput};

#[proc_macro_derive(Builder)]
pub fn derive(input: TokenStream) -> TokenStream {
    // parse the input token into a syntax tree
    let _input = parse_macro_input!(input as DeriveInput);

    TokenStream::new()
}
