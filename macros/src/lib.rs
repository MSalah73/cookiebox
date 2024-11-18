extern crate proc_macro;

use proc_macro::TokenStream;
use quote::quote;
use syn::{
    parse_macro_input, DeriveInput, Expr, Fields, ItemStruct, Lit, Meta, PathArguments, Type,
};

/// Implements a CookieName trait using passed in name from the macro attribute
#[proc_macro_attribute]
pub fn cookie(attr: TokenStream, item: TokenStream) -> TokenStream {
    let input = parse_macro_input!(item as ItemStruct);

    let parsed_attr = parse_macro_input!(attr as Meta);

    let mut cookie_name = String::new();

    if !parsed_attr.path().is_ident("name") {
        return syn::Error::new_spanned(
            parsed_attr.path().get_ident(),
            "Expected `name` parameter: #[cookie(name = \"...\")]",
        )
        .into_compile_error()
        .into();
    }
    if let Meta::NameValue(nv) = parsed_attr {
        if let Expr::Lit(expr) = &nv.value {
            if let Lit::Str(lit_str) = &expr.lit {
                cookie_name.push_str(&lit_str.value());
            }
        }
    }

    let cookie_struct = &input.ident;

    let expanded = quote! {
        #input

        impl CookieName for #cookie_struct {
            const COOKIE_NAME: &'static str = #cookie_name;
        }
    };

    expanded.into()
}

/// Implements a FromRequest for a struct that holds cookie types
///
/// **Note**: only allows structs with either a single unnamed field or multiple unnamed fields
#[proc_macro_derive(FromRequest)]
pub fn cookie_collection(item: TokenStream) -> TokenStream {
    let input = parse_macro_input!(item as DeriveInput);
    let collection_struct = &input.ident;

    // Extract the field types based on whether it's a tuple or named struct.
    let (field_names, field_types) = match extract_fields_types(&input) {
        Ok(fields) => fields,
        Err(e) => return e.into_compile_error().into(),
    };

    // Extract the generic type argument from a Cookie<'c, SomeType> type.
    let inner_types = field_types
        .iter()
        .try_fold(
            Vec::new(),
            |mut types, field_type| match extract_cookie_inner_type(field_type) {
                Some(inner_type) => {
                    types.push(inner_type);
                    Ok(types)
                }
                None => Err(syn::Error::new_spanned(
                    field_type,
                    "Expected field type to be `Cookie<'c, SomeType>`",
                )),
            },
        );

    let inner_types = match inner_types {
        Ok(types) => types,
        Err(error) => return error.into_compile_error().into(),
    };

    let generated_types = if let Some(field_names) = field_names {
        quote! { #collection_struct { #( #field_names: Cookie::<#inner_types>::new(&storage),)* }}
    } else {
        quote! { #collection_struct ( #( Cookie::<#inner_types>::new(&storage),)* )}
    };

    // Generate the implementation for FromRequest
    let expanded = quote! {
        impl actix_web::FromRequest for #collection_struct<'static> {
            type Error = Box<dyn std::error::Error>;
            type Future = std::future::Ready<Result<Self, Self::Error>>;

            fn from_request(req: &actix_web::HttpRequest, _payload: &mut actix_web::dev::Payload) -> Self::Future {
                match req.extensions().get::<cookiebox::Storage>() {
                    Some(storage) => {
                        std::future::ready(Ok( #generated_types ))
                    }
                    None => std::future::ready(Err("Storage not found in request extension".into())),
                }
            }
        }
    };

    expanded.into()
}

fn extract_fields_types(
    input: &DeriveInput,
) -> Result<(Option<Vec<syn::Ident>>, Vec<&Type>), syn::Error> {
    match &input.data {
        syn::Data::Struct(data_struct) => match &data_struct.fields {
            Fields::Unnamed(fields) if fields.unnamed.len() == 1 => {
                Ok((None, vec![&fields.unnamed[0].ty]))
            }
            Fields::Named(fields) => {
                // Unwrap here is okay since Fields::Named require a field name which make a None ident value impossible to represent
                let field_names = fields
                    .named
                    .iter()
                    .map(|f| f.ident.clone().unwrap())
                    .collect();
                let field_types = fields.named.iter().map(|f| &f.ty).collect();
                Ok((Some(field_names), field_types))
            }
            // Units and unnamed with more than 1 fields
            token => Err(syn::Error::new_spanned(
                token,
                "Expected a single unnamed field or multiple named fields",
            )),
        },
        // Enum and union
        _ => Err(syn::Error::new_spanned(input, "Expected a struct")),
    }
}

/// Extracts the inner type (SomeType) from a `Cookie<'c, SomeType>` type.
fn extract_cookie_inner_type(field_type: &Type) -> Option<&Type> {
    if let Type::Path(type_path) = field_type {
        let segment = type_path.path.segments.first()?;
        if segment.ident == "Cookie" {
            if let PathArguments::AngleBracketed(generics) = &segment.arguments {
                if generics.args.len() == 2 {
                    if let syn::GenericArgument::Type(inner_type) = &generics.args[1] {
                        return Some(inner_type);
                    }
                }
            }
        }
    }
    None
}
