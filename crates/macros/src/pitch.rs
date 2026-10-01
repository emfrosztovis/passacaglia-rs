use std::str::FromStr;

use proc_macro2::TokenStream;
use quote::quote;

use passacaglia_core::std_hept::Pitch;

pub fn expand(input: TokenStream) -> TokenStream {
    let lit: syn::LitStr = match syn::parse2(input) {
        Ok(lit) => lit,
        Err(e) => return e.to_compile_error(),
    };
    let s = lit.value();
    match Pitch::from_str(&s) {
        Ok(p) => {
            let index = p.index;
            let num = *p.acci.numer();
            let den = *p.acci.denom();
            let period = p.period;
            quote! {
                ::passacaglia_core::std_hept::Pitch::new(
                    #index,
                    ::passacaglia_core::Rational::new_raw(#num, #den),
                    #period,
                )
            }
        }
        Err(e) => {
            let msg = format!("invalid pitch literal `{s}`: {e}");
            quote! { compile_error!(#msg) }
        }
    }
}
