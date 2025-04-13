use proc_macro::TokenStream;
use quote::quote;
use syn::{parse_macro_input, DeriveInput, PathArguments, Type};

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

    fn is_option(ty: &Type) -> bool {
        if let Type::Path(type_path) = ty {
            if let Some(segment) = type_path.path.segments.first() {
                return segment.ident == "Option"
                    && matches!(segment.arguments, PathArguments::AngleBracketed(_));
            }
        }
        false
    }
    let build_fields = fields.iter().map(|f| {
        let name = &f.ident;
        let ty = &f.ty;
        if is_option(ty) {
            return quote! {
                #name: self.#name.take(),
            };
        }
        quote! {
            #name: self.#name.take().ok_or_else(|| format!("Field {} is not set", stringify!(#name)))?,
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

            pub fn build(&mut self) -> Result<#name, Box<dyn std::error::Error>> {
                Ok(#name {
                    #(#build_fields)*
                })
            }
        }

    };

    TokenStream::from(expanded)
}
