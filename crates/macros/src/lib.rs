//! Compile-time literal parsers for passacaglia.
//!
//! The macros here are specific to the standard heptatonic system; a different
//! pitch system would ship its own. Consumers typically alias them to short
//! names at the point of use:
//!
//! ```ignore
//! use passacaglia_macros::{std_hept_pitch as p, std_hept_interval as i};
//! let c = p!("c4");
//! let m3 = i!("m3");
//! ```

use proc_macro::TokenStream;

mod interval;
mod pitch;
mod scale;

#[proc_macro]
pub fn std_hept_pitch(input: TokenStream) -> TokenStream {
    pitch::expand(input.into()).into()
}

#[proc_macro]
pub fn std_hept_interval(input: TokenStream) -> TokenStream {
    interval::expand(input.into()).into()
}

#[proc_macro]
pub fn std_hept_scale(input: TokenStream) -> TokenStream {
    scale::expand(input.into()).into()
}
