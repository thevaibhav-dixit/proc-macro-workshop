use proc_macro::TokenStream;
use quote::quote;
use syn::{parse_macro_input, DeriveInput, PathArguments, Type};

#[proc_macro_derive(Builder, attributes(builder))]
pub fn derive(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);
    let name = input.ident;
    let builder_ident = syn::Ident::new(&format!("{}Builder", name), name.span());

    let fields = match input.data {
        syn::Data::Struct(ref data) => match data.fields {
            syn::Fields::Named(ref fields) => &fields.named,
            _ => panic!("Only named fields are supported"),
        },
        _ => panic!("Only structs are supported"),
    };

    struct FieldInfo<'a> {
        name: syn::Ident,
        ty: &'a Type,
        each_attr: Option<syn::Ident>,
    }

    fn get_each_attr(field: &syn::Field) -> Result<Option<syn::Ident>, proc_macro2::TokenStream> {
        for attr in &field.attrs {
            if attr.path().is_ident("builder") {
                let mut each_value = None;

                let result = attr.parse_nested_meta(|meta| {
                    if meta.path.is_ident("each") {
                        let value = meta.value()?;
                        let string_value = value.parse::<syn::LitStr>()?;
                        each_value =
                            Some(syn::Ident::new(&string_value.value(), string_value.span()));
                        Ok(())
                    } else {
                        Err(syn::Error::new_spanned(
                            attr.meta.clone(),
                            "expected `builder(each = \"...\")`",
                        ))
                    }
                });

                return match result {
                    Ok(_) => Ok(each_value),
                    Err(err) => Err(err.to_compile_error()),
                };
            }
        }
        Ok(None)
    }

    fn is_option(ty: &Type) -> bool {
        if let Type::Path(type_path) = ty {
            if let Some(segment) = type_path.path.segments.first() {
                return segment.ident == "Option"
                    && matches!(segment.arguments, PathArguments::AngleBracketed(_));
            }
        }
        false
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

    let mut field_infos = Vec::new();

    for f in fields {
        let name = f.ident.clone().expect("Expected named field");
        let ty = &f.ty;

        let each_attr = match get_each_attr(f) {
            Ok(attr) => attr,
            Err(err) => return TokenStream::from(err),
        };

        field_infos.push(FieldInfo {
            name,
            ty,
            each_attr,
        });
    }

    let builder_fields = field_infos.iter().map(|info| {
        let name = &info.name;
        if info.each_attr.is_some() {
            let inner_ty = inner_type_of_vec(info.ty);
            quote! { #name: Vec<#inner_ty>, }
        } else if is_option(info.ty) {
            let ty = info.ty;
            quote! { #name: #ty, }
        } else {
            let ty = info.ty;
            quote! { #name: Option<#ty>, }
        }
    });

    let builder_init = field_infos.iter().map(|info| {
        let name = &info.name;
        if info.each_attr.is_some() {
            quote! { #name: Vec::new(), }
        } else {
            quote! { #name: None, }
        }
    });

    let setters = field_infos.iter().map(|info| {
        let name = &info.name;
        let ty = info.ty;

        if let Some(each_name) = &info.each_attr {
            let inner_ty = inner_type_of_vec(ty);
            quote! {
                pub fn #each_name(&mut self, #each_name: #inner_ty) -> &mut Self {
                    self.#name.push(#each_name);
                    self }
            }
        } else if is_option(ty) {
            let inner_ty = inner_type_of_option(ty);
            quote! {
                pub fn #name(&mut self, #name: #inner_ty) -> &mut Self {
                    self.#name = Some(#name);
                    self
                }
            }
        } else {
            quote! {
                pub fn #name(&mut self, #name: #ty) -> &mut Self {
                    self.#name = Some(#name);
                    self
                }
            }
        }
    });

    let build_fields = field_infos.iter().map(|info| {
        let name = &info.name;
        let ty = info.ty;

        if info.each_attr.is_some() {
            quote! { #name: self.#name.clone(), }
        } else if is_option(ty) {
            quote! { #name: self.#name.take(), }
        } else {
            quote! {
                #name: self.#name.take().ok_or_else(|| format!("Field {} is not set", stringify!(#name)))?,
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

            pub fn build(&mut self) -> Result<#name, Box<dyn std::error::Error>> {
                Ok(#name {
                    #(#build_fields)*
                })
            }
        }
    };

    TokenStream::from(expanded)
}
