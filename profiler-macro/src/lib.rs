extern crate proc_macro;

use proc_macro::TokenStream;
use syn::{parse_quote, parse_macro_input};
use quote::quote;

#[proc_macro_attribute]
pub fn profile(_attr: TokenStream, item: TokenStream) -> TokenStream {
    let mut func: syn::ItemFn = parse_macro_input!(item as syn::ItemFn);
    let ident = &func.ident;
    let block = &func.block;
    func.block = Box::new(parse_quote! {
        {
            profiler::open_block(file!(), line!(), stringify!(#ident));
            let result = #block;
            profiler::close_block();
            result
        }
    });
    let result = quote!(#func);
    result.into()
}