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

        pub struct #builder_ident {
            executable: Option<String>,
            args: Option<Vec<String>>,
            env: Option<Vec<String>>,
            current_dir: Option<String>,
        }

        impl #name {
            pub fn builder() -> #builder_ident {
                #builder_ident {
                    executable: None,
                    args: None,
                    env: None,
                    current_dir: None,
                }
            }
        }


    };

    TokenStream::from(expanded)
}
