use proc_macro2::TokenStream;
use quote::quote;
use syn::{parse_macro_input, ItemFn};

#[proc_macro]
pub fn mini_rt(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    let input_fn = parse_macro_input!(input as ItemFn);
    let fn_name = &input_fn.sig.ident;
    let fn_block = &input_fn.block;
    let fn_async = input_fn.sig.asyncness.is_some();
    
    let expanded = if fn_async {
        quote! {
            fn #fn_name() {
                let mut rt = crate::components::MiniRuntime::new();
                rt.block_on(async #fn_block);
            }
        }
    } else {
        quote! {
            fn #fn_name() {
                #fn_block
            }
        }
    };
    
    expanded.into()
}

#[proc_macro]
pub fn join_all(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    let handles = parse_macro_input!(input as syn::ExprArray);
    let expanded = quote! {
        {
            let mut handles = vec![#handles];
            async move {
                let mut results = Vec::with_capacity(handles.len());
                for handle in handles {
                    results.push(handle.await);
                }
                results
            }
        }
    };
    expanded.into()
} 