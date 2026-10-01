use proc_macro2::TokenStream;
use quote::quote;

pub fn expand(input: TokenStream) -> TokenStream {
    let lit: syn::LitStr = match syn::parse2(input) {
        Ok(lit) => lit,
        Err(e) => return e.to_compile_error(),
    };
    let s = lit.value();
    match passacaglia_parser::parse_pitch(&s) {
        Ok(p) => {
            let index = p.index;
            let num = p.acci_num;
            let den = p.acci_den;
            let period = p.period;
            quote! {
                ::passacaglia_core::std_hept::Pitch::new(
                    #index,
                    ::passacaglia_common::Rational::new_raw(#num, #den),
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
