use proc_macro::TokenStream;
use quote::quote;
use syn::{parse_macro_input, DeriveInput};

#[proc_macro_derive(CustomDebug, attributes(debug))]
pub fn derive(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);

    let name = input.ident;

    let fields = match input.data {
        syn::Data::Struct(ref data) => match data.fields {
            syn::Fields::Named(ref fields) => &fields.named,
            _ => panic!("Only named fields are supported"),
        },
        _ => panic!("Only structs are supported"),
    };

    let generics = add_trait_bounds(input.generics, fields);
    let (impl_generics, ty_generics, where_clause) = generics.split_for_impl();

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

fn add_trait_bounds(
    mut generics: syn::Generics,
    fields: &syn::punctuated::Punctuated<syn::Field, syn::token::Comma>,
) -> syn::Generics {
    use syn::{GenericParam, PathArguments, Type};

    let mut phantom_types = std::collections::HashSet::new();

    for field in fields {
        if let Type::Path(type_path) = &field.ty {
            if let Some(seg) = type_path.path.segments.first() {
                if seg.ident == "PhantomData" {
                    if let PathArguments::AngleBracketed(args) = &seg.arguments {
                        for arg in &args.args {
                            if let syn::GenericArgument::Type(Type::Path(type_path)) = arg {
                                if let Some(ident) = type_path.path.get_ident() {
                                    phantom_types.insert(ident.to_string());
                                }
                            }
                        }
                    }
                }
            }
        }
    }

    for param in &mut generics.params {
        if let GenericParam::Type(type_param) = param {
            if !phantom_types.contains(&type_param.ident.to_string()) {
                type_param.bounds.push(syn::parse_quote!(std::fmt::Debug));
            }
        }
    }

    generics
}
