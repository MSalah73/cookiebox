extern crate proc_macro;

use proc_macro::TokenStream;
use quote::quote;
use syn::{parse_macro_input, Expr, ItemStruct, Lit, Meta};

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
