//! Compile-time literal parsers for passacaglia.

use proc_macro::TokenStream;

mod pitch;
mod interval;
mod scale;

#[proc_macro]
pub fn pitch(input: TokenStream) -> TokenStream {
    pitch::expand(input.into()).into()
}

#[proc_macro]
pub fn interval(input: TokenStream) -> TokenStream {
    interval::expand(input.into()).into()
}

#[proc_macro]
pub fn scale(input: TokenStream) -> TokenStream {
    scale::expand(input.into()).into()
}
