use proc_macro::TokenStream;
use quote::quote;
use syn::{parse_macro_input, DeriveInput, PathArguments, Type};

#[proc_macro_derive(Builder, attributes(builder))]
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

    fn get_each_attr(field: &syn::Field) -> Option<syn::Ident> {
        for attr in &field.attrs {
            if attr.path().is_ident("builder") {
                let mut each_value = None;

                let result = attr.parse_nested_meta(|meta| {
                    if meta.path.is_ident("each") {
                        let value = meta.value()?;
                        let string_value = value.parse::<syn::LitStr>()?;
                        each_value = Some(syn::Ident::new(
                            string_value.value().as_str(),
                            string_value.span(),
                        ));
                        return Ok(());
                    }
                    Ok(())
                });

                if result.is_ok() && each_value.is_some() {
                    return each_value;
                }
            }
        }
        None
    }

    fn inner_type_of_vec(ty: &Type) -> &Type {
        if let Type::Path(type_path) = ty {
            if let Some(seg) = type_path.path.segments.first() {
                if seg.ident == "Vec" {
                    if let PathArguments::AngleBracketed(ref args) = seg.arguments {
                        if let Some(syn::GenericArgument::Type(inner_ty)) = args.args.first() {
                            return inner_ty;
                        }
                    }
                }
            }
        }
        ty
    }

    fn inner_type_of_option(ty: &Type) -> &Type {
        if let Type::Path(type_path) = ty {
            if let Some(seg) = type_path.path.segments.first() {
                if seg.ident == "Option" {
                    if let PathArguments::AngleBracketed(ref args) = seg.arguments {
                        if let Some(syn::GenericArgument::Type(inner_ty)) = args.args.first() {
                            return inner_ty;
                        }
                    }
                }
            }
        }
        ty
    }

    let builder_fields = fields.iter().map(|f| {
        let name = &f.ident;
        let ty = &f.ty;

        if get_each_attr(f).is_some() {
            let inner_ty = inner_type_of_vec(ty);
            return quote! {
                #name: Vec<#inner_ty>,
            };
        }
        if is_option(ty) {
            return quote! {
                #name: #ty,
            };
        }
        quote! {
            #name: Option<#ty>,
        }
    });

    let builder_init = fields.iter().map(|f| {
        let name = &f.ident;
        if get_each_attr(f).is_some() {
            return quote! {
                #name: Vec::new(),
            };
        }

        quote! {
            #name: None,
        }
    });

    let setters = fields.iter().map(|f| {
        let name = &f.ident;
        let ty = &f.ty;

        if get_each_attr(f).is_some() {
            let inner_ty = inner_type_of_vec(ty);
            let each_name = get_each_attr(f).expect("name should be present");
            return quote! {
                pub fn #each_name(&mut self, #each_name: #inner_ty) -> &mut Self {
                    self.#name.push(#each_name);
                    self
                }
            };
        }

        if is_option(ty) {
            let inner_ty = inner_type_of_option(ty);
            return quote! {
                pub fn #name(&mut self, #name: #inner_ty) -> &mut Self {
                    self.#name = Some(#name);
                    self
                }
            };
        }
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

        if get_each_attr(f).is_some() {
            return quote! {
                #name: self.#name.clone(),
            };
        }

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
