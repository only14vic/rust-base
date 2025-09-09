#![allow(unused)]
#![cfg_attr(not(feature = "std"), no_std)]

#[macro_use]
extern crate alloc;
extern crate core;
extern crate proc_macro2;

mod extend;
mod struct_fields;

use {
    crate::{extend::ExtendMacros, struct_fields::StructFields},
    proc_macro::TokenStream
};

#[proc_macro_derive(Extend, attributes(parse, skip))]
pub fn extend(input: TokenStream) -> TokenStream {
    ExtendMacros::default().derive(input)
}

#[proc_macro_derive(ExtendFromIter, attributes(parse, skip))]
pub fn extend_from_iter(input: TokenStream) -> TokenStream {
    ExtendMacros { from_iter: true }.derive(input)
}

#[proc_macro_derive(StructFields)]
pub fn struct_fields(input: TokenStream) -> TokenStream {
    StructFields.derive(input)
}
