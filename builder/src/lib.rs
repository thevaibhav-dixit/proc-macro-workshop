use proc_macro::TokenStream;
use quote::quote;
use syn::{parse_macro_input, DeriveInput};

#[proc_macro_derive(Builder)]
pub fn derive(input: TokenStream) -> TokenStream {
    // parse the input token into a syntax tree
    let input = parse_macro_input!(input as DeriveInput);
    let name = input.ident;

    let builder_ident = syn::Ident::new(&format!("{}Builder", name), name.span());

    // trying to get the fields
    let fields = match input.data {
        syn::Data::Struct(ref data) => match data.fields {
            syn::Fields::Named(ref fields) => &fields.named,
            _ => panic!("Only named fields are supported"),
        },
        _ => panic!("Only structs are supported"),
    };

    let builder_fields = fields.iter().map(|f| {
        let name = &f.ident;
        let ty = &f.ty;
        quote! {
            #name: Option<#ty>,
        }
    });

    let builder_init = fields.iter().map(|f| {
        let name = &f.ident;
        quote! {
            #name: None,
        }
    });

    let setters = fields.iter().map(|f| {
        let name = &f.ident;
        let ty = &f.ty;
        quote! {
            pub fn #name(&mut self, #name: #ty) -> &mut Self {
                self.#name = Some(#name);
                self
            }
        }
    });

    let expanded = quote! {

        pub struct #builder_ident {
            #(#builder_fields)*
        }

        impl #name {
            pub fn builder() -> #builder_ident {
                #builder_ident {
                    #(#builder_init)*
                }
            }
        }

        impl #builder_ident {
            #(#setters)*
        }

    };

    TokenStream::from(expanded)
}
