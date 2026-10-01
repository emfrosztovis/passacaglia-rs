use proc_macro2::TokenStream;
use quote::quote;

pub fn expand(input: TokenStream) -> TokenStream {
    let lit: syn::LitStr = match syn::parse2(input) {
        Ok(lit) => lit,
        Err(e) => return e.to_compile_error(),
    };
    let s = lit.value();
    match passacaglia_parser::parse_interval(&s) {
        Ok(i) => {
            let steps = i.steps;
            let num = i.distance_num;
            let den = i.distance_den;
            let sign = i.sign;
            quote! {
                ::passacaglia_core::std_hept::Interval::new(
                    #steps,
                    ::passacaglia_common::Rational::new_raw(#num, #den),
                    #sign,
                )
            }
        }
        Err(e) => {
            let msg = format!("invalid interval literal `{s}`: {e}");
            quote! { compile_error!(#msg) }
        }
    }
}
