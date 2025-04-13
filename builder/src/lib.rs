use proc_macro::TokenStream;
use quote::quote;
use syn::{parse_macro_input, DeriveInput};

#[proc_macro_derive(Builder)]
pub fn derive(input: TokenStream) -> TokenStream {
    // parse the input token into a syntax tree
    let input = parse_macro_input!(input as DeriveInput);
    let name = input.ident;

    let builder_ident = syn::Ident::new(&format!("{}Builder", name), name.span());

    let expanded = quote! {
        impl #name {
            pub fn builder() -> () {}
        }

    };

    TokenStream::from(expanded)
}
