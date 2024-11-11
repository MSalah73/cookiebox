extern crate proc_macro;
use proc_macro::TokenStream;
use syn::{parse_macro_input, Expr, ItemStruct, Lit,  Meta, Token};
use quote::{quote};
use syn::punctuated::Punctuated;

#[proc_macro_attribute]
pub fn cookie(attr: TokenStream, item: TokenStream) -> TokenStream {
    let input = parse_macro_input!(item as ItemStruct);

    let parsed_attrs = parse_macro_input!(attr with Punctuated<Meta, Token![,]>::parse_terminated);

    if parsed_attrs.len() > 2 {
        return syn::Error::new_spanned(parsed_attrs, 
        "Expected #[cookie(name = \"...\")] with optional signed or private attributes.")
        .into_compile_error()
        .into();
    }

    let mut attr_iter = parsed_attrs.into_iter();

    let mut cookie_name = String::new();

    if let Some(meta) = attr_iter.next() {
        if !meta.path().is_ident("name"){
            return syn::Error::new_spanned(meta.path().get_ident(), "Expected `name` parameter: #[cookie(name = \"...\")]").into_compile_error().into();
        }
        if let Meta::NameValue(meta_nv) = meta {
            if let Expr::Lit(expr) = &meta_nv.value {
                if let Lit::Str(lit_str) = &expr.lit {
                    cookie_name.push_str(&lit_str.value());
                }
            }
        }
    }


    if let Some(meta) = attr_iter.next() {
        match meta {
            Meta::Path(path) if path.is_ident("private") => eprint!("This is a private cookie"),
            Meta::Path(path) if path.is_ident("signed") => eprint!("This is a singed cookie"),
            value => {
                return syn::Error::new_spanned(value, "Expected private or signed attribute")
                    .to_compile_error()
                    .into();
            }
        }
    };

    let cookie_struct = &input.ident;

    let expanded = quote! {
        #input



        impl CookieName for #cookie_struct {
            const COOKIE_NAME: &'static str = #cookie_name;
        }
    };

    expanded.into()
}