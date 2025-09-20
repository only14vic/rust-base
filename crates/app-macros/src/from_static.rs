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
pub(crate) struct FromStatic;

impl FromStatic {
    pub fn derive(mut self, input: TokenStream) -> TokenStream {
        let input = parse_macro_input!(input as DeriveInput);
        let expanded = self.parse(input);
        TokenStream::from(expanded)
    }

    fn parse(&self, input: DeriveInput) -> TokenStream2 {
        let struct_name = &input.ident;
        let (impl_generics, ty_generics, where_clause) = input.generics.split_for_impl();

        let Data::Struct(DataStruct { .. }) = &input.data else {
            panic!("Only structs are supported to implement macros FromStatic");
        };

        quote! {
            impl #impl_generics FromStatic for #struct_name #ty_generics #where_clause {
                fn from_static() -> &'static Self
                {
                    unsafe { Self::from_static_mut() }
                }

                unsafe fn from_static_mut() -> &'static mut Self
                {
                    use {
                        ::core::sync::atomic::{AtomicBool, AtomicPtr, Ordering},
                        ::core::ptr::null_mut,
                        ::alloc::boxed::Box
                    };

                    static STORAGE: AtomicPtr<#struct_name> = AtomicPtr::new(null_mut());
                    static LOCK: AtomicBool = AtomicBool::new(false);

                    let mut obj = STORAGE.load(Ordering::Acquire);

                    if obj.is_null() {
                        if LOCK.swap(true, Ordering::SeqCst) == false {
                            obj = Box::leak(Box::new(Self::default()));
                            STORAGE.store(obj, Ordering::Release);
                        } else {
                            loop {
                                obj = STORAGE.load(Ordering::Acquire);
                                if obj.is_null() == false {
                                    break;
                                }
                            }
                        }
                    }

                    unsafe { &mut *obj }
                }
            }
        }
    }
}
