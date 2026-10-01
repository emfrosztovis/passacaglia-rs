use std::str::FromStr;

use proc_macro2::TokenStream;
use quote::quote;

use passacaglia_core::std_hept::Interval;

pub fn expand(input: TokenStream) -> TokenStream {
    let lit: syn::LitStr = match syn::parse2(input) {
        Ok(lit) => lit,
        Err(e) => return e.to_compile_error(),
    };
    let s = lit.value();
    match Interval::from_str(&s) {
        Ok(i) => {
            let steps = i.steps;
            let num = *i.distance.numer();
            let den = *i.distance.denom();
            let sign = i.sign;
            quote! {
                ::passacaglia_core::std_hept::Interval::new(
                    #steps,
                    ::passacaglia_core::Rational::new_raw(#num, #den),
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
