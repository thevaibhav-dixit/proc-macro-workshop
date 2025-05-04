use proc_macro::TokenStream;
use quote::quote;
use syn::{parse_macro_input, DeriveInput};

#[proc_macro_derive(CustomDebug, attributes(debug))]
pub fn derive(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);

    let name = input.ident;
    let generics = add_trait_bounds(input.generics);
    let (impl_generics, ty_generics, where_clause) = generics.split_for_impl();

    let fields = match input.data {
        syn::Data::Struct(ref data) => match data.fields {
            syn::Fields::Named(ref fields) => &fields.named,
            _ => panic!("Only named fields are supported"),
        },
        _ => panic!("Only structs are supported"),
    };

    struct FieldInfo {
        field: syn::Ident,
        accessor: proc_macro2::TokenStream,
        debug_format: Option<String>,
    }

    fn get_debug_attribute_value(field: &syn::Field) -> Option<String> {
        for attr in &field.attrs {
            if attr.meta.path().is_ident("debug") {
                if let syn::Meta::NameValue(name_value) = &attr.meta {
                    if let syn::Expr::Lit(expr_lit) = &name_value.value {
                        if let syn::Lit::Str(lit_str) = &expr_lit.lit {
                            return Some(lit_str.value());
                        }
                    }
                }
            }
        }
        None
    }

    let field_infos: Vec<FieldInfo> = fields
        .iter()
        .map(|f| {
            let ident = f.ident.as_ref().expect("Expected named field");
            let debug_format = get_debug_attribute_value(f);
            FieldInfo {
                field: ident.clone(),
                accessor: quote!(&self.#ident),
                debug_format,
            }
        })
        .collect();

    let field_names = field_infos.iter().map(|f| {
        let name = &f.field;
        let accessor = &f.accessor;
        let debug_format = &f.debug_format;
        if let Some(format) = debug_format {
            quote! {
                s.field(stringify!(#name), &format_args!(#format, #accessor));
            }
        } else {
            quote! {
                s.field(stringify!(#name), #accessor);
            }
        }
    });

    let debug_impl = quote! {
        impl #impl_generics std::fmt::Debug for #name  #ty_generics #where_clause {
            fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                let mut s = f.debug_struct(stringify!(#name));
                #(#field_names)*
                s.finish()
            }
        }
    };

    TokenStream::from(debug_impl)
}

fn add_trait_bounds(mut generics: syn::Generics) -> syn::Generics {
    for param in &mut generics.params {
        if let syn::GenericParam::Type(ref mut type_param) = *param {
            type_param.bounds.push(syn::parse_quote!(std::fmt::Debug));
        }
    }
    generics
}
