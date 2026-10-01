use std::str::FromStr;

use proc_macro2::TokenStream;
use quote::quote;
use syn::parse::Parser;
use syn::punctuated::Punctuated;
use syn::Token;

use passacaglia_core::std_hept::Interval;

pub fn expand(input: TokenStream) -> TokenStream {
    let parser = Punctuated::<syn::LitStr, Token![,]>::parse_terminated;
    let lits = match parser.parse2(input) {
        Ok(lits) => lits,
        Err(e) => return e.to_compile_error(),
    };

    if lits.is_empty() {
        let msg = "scale! requires at least one interval literal";
        return quote! { compile_error!(#msg) };
    }

    let mut intervals = Vec::with_capacity(lits.len());
    for lit in &lits {
        let s = lit.value();
        match Interval::from_str(&s) {
            Ok(i) => {
                let steps = i.steps;
                let num = *i.distance.numer();
                let den = *i.distance.denom();
                let sign = i.sign;
                intervals.push(quote! {
                    ::passacaglia_core::std_hept::Interval::new(
                        #steps,
                        ::passacaglia_core::Rational::new_raw(#num, #den),
                        #sign,
                    )
                });
            }
            Err(e) => {
                let msg = format!("invalid interval literal `{s}` in scale!: {e}");
                return quote! { compile_error!(#msg) };
            }
        }
    }

    quote! {
        ::passacaglia_core::std_hept::Scale::from_intervals(
            ::passacaglia_core::std_hept::Pitch::new(
                0usize,
                ::passacaglia_core::Rational::new_raw(0i64, 1i64),
                0i32,
            ),
            &[#(#intervals),*],
        )
    }
}
