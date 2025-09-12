use {
    alloc::{
        borrow::Cow,
        string::{String, ToString},
        vec::Vec
    },
    core::{
        iter::{Enumerate, Peekable},
        marker::PhantomData
    },
    proc_macro::TokenStream,
    proc_macro2::{Span, TokenStream as TokenStream2, TokenTree},
    quote::{ToTokens, quote},
    syn::{
        Attribute, Data, DataStruct, DeriveInput, Field, Fields, FieldsNamed,
        GenericParam, Generics, Ident, ImplGenerics, Lifetime, LifetimeParam, Path,
        TypePath, WhereClause, parse_macro_input, parse_str, punctuated::Punctuated,
        token::Comma
    }
};
#[cfg(not(feature = "std"))]
#[allow(unused_imports)]
use libc_print::std_name::*;

#[derive(Default)]
pub(crate) struct StructFields;

impl StructFields {
    pub fn derive(mut self, input: TokenStream) -> TokenStream {
        let input = parse_macro_input!(input as DeriveInput);
        let expanded = self.parse(input);
        TokenStream::from(expanded)
    }

    fn parse(&self, input: DeriveInput) -> TokenStream2 {
        let struct_name = &input.ident;
        let (impl_generics, ty_generics, where_clause) = input.generics.split_for_impl();

        let Data::Struct(DataStruct {
            fields: Fields::Named(FieldsNamed { named, .. }),
            ..
        }) = &input.data
        else {
            panic!("Only structs with named fields are supported");
        };

        let field_names = named
            .iter()
            .filter_map(|f| f.ident.as_ref().map(|i| i.to_string()))
            .collect::<Vec<_>>();

        let field_types = named
            .iter()
            .filter_map(|f| {
                f.ident.as_ref().map(|i| {
                    let id = i.to_string();
                    let ty = f.ty.to_token_stream().to_string();
                    quote! {(#id,#ty)}
                })
            })
            .collect::<Vec<_>>();

        /*
        let mut fields_into_bounds = parse_str::<Generics>("<'a, T>").unwrap();
        let fields_into = named
            .iter()
            .filter_map(|f| {
                if f.attrs.iter().any(|a| a.path().is_ident("skip_as")) {
                    return None;
                }
                f.ident.as_ref().map(|id| {
                    let ty = f.ty.to_token_stream().to_string();
                    fields_into_bounds
                        .type_params_mut()
                        .last()
                        .unwrap()
                        .bounds
                        .push(parse_str(&format!("TryFrom<&'a {ty}>")).unwrap());
                    quote! {(&self.#id).try_into().ok()}
                })
            })
            .collect::<Vec<_>>();

            quote! {
                impl #impl_generics #struct_name #ty_generics #where_clause {
                    pub fn fields_as #fields_into_bounds (&'a self) -> Vec<T> {
                        [
                            #(#fields_into),*
                        ]
                        .into_iter()
                        .filter_map(|r| r)
                        .collect()
                    }
                }
            }
        */

        quote! {
            impl #impl_generics #struct_name #ty_generics #where_clause {
                pub const FIELD_NAMES: &'static [&'static str] = &[#(#field_names),*];
                pub const FIELD_TYPES: &'static [(&'static str, &'static str)] = &[#(#field_types),*];

                #[inline]
                pub const fn field_names(&self) -> &'static [&'static str] {
                    Self::FIELD_NAMES
                }

                #[inline]
                pub const fn field_types(&self) -> &'static [(&'static str, &'static str)] {
                    Self::FIELD_TYPES
                }
            }
        }
    }
}
