#![allow(unused)]
#![cfg_attr(not(feature = "std"), no_std)]

#[macro_use]
extern crate alloc;
extern crate core;
extern crate proc_macro2;

mod set_from_iter;

use proc_macro::TokenStream;

#[proc_macro_derive(SetFromIter, attributes(parse))]
pub fn set_from_iter(input: TokenStream) -> TokenStream {
    set_from_iter::set_from_iter_derive(input)
}
