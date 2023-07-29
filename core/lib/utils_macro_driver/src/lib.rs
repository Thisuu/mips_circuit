extern crate proc_macro;

use proc_macro::TokenStream;
use quote::quote;
use syn;


#[proc_macro_derive(UtilsMacro)]
pub fn utils_macro_derive(input: TokenStream) -> TokenStream {
    let ast = syn::parse(input).unwrap();
    impl_utils_macro(&ast)
}

fn impl_utils_macro(ast: &syn::DeriveInput) -> TokenStream {
    let name = &ast.ident;
    let gen = quote! {
        impl UtilsMacro for #name {
            fn get_type_name<'a>() -> &'a str {
                stringify!(#name)
            }

            fn from_json_str(input: &str) -> Self {
                      serde_json::from_str(input).unwrap()}
            }
    };

    gen.into()
}