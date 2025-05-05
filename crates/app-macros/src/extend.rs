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
        Attribute, Data, DeriveInput, Field, Fields, GenericParam, Generics, Ident,
        ImplGenerics, Lifetime, LifetimeParam, Path, TypePath, WhereClause,
        parse_macro_input, parse_str, punctuated::Punctuated, token::Comma
    }
};
#[cfg(not(feature = "std"))]
#[allow(unused_imports)]
use libc_print::std_name::*;

#[derive(Default)]
pub(crate) struct ExtendMacros {
    pub from_iter: bool
}

impl ExtendMacros {
    pub fn derive(mut self, input: TokenStream) -> TokenStream {
        let input = parse_macro_input!(input as DeriveInput);
        let expanded = self.parse(input);
        TokenStream::from(expanded)
    }

    fn parse(&self, input: DeriveInput) -> TokenStream2 {
        let struct_name = input.ident;
        let generics = input.generics;
        let mut generics_clone = generics.clone();
        let mut lifetime = LifetimeParam::new(Lifetime::new("'iter", Span::call_site()));
        generics
            .lifetimes()
            .for_each(|l| lifetime.bounds.push(l.lifetime.clone()));
        generics_clone.params.push(GenericParam::Lifetime(lifetime));
        let (.., ty_generics, where_clause) = generics.split_for_impl();
        let (impl_generics, ..) = generics_clone.split_for_impl();
        let fields = match input.data {
            Data::Struct(data_struct) => {
                match data_struct.fields {
                    Fields::Named(fields_named) => fields_named.named,
                    _ => panic!("Only structs with named fields are supported")
                }
            },
            _ => panic!("Only structs are supported")
        };
        let fields_set = self.parse_fields(&fields, &generics_clone);

        #[cfg(feature = "std")]
        let map_type: TypePath =
            syn::parse_str("::std::collections::HashMap<&'iter str, Option<&'iter str>>")
                .unwrap();

        #[cfg(not(feature = "std"))]
        let map_type: TypePath = syn::parse_str(
            "::indexmap::IndexMap<&'iter str, Option<&'iter str>, ::core::hash::BuildHasherDefault<ahash::AHasher>>",
        ).unwrap();

        let mut extend = quote! {
            impl #impl_generics Extend<(&'iter str, Option<&'iter str>)> for #struct_name #ty_generics #where_clause {
                fn extend<I: IntoIterator<Item = (&'iter str, Option<&'iter str>)>>(&mut self, iter: I) {
                    type MapType<'iter> = #map_type;
                    let mut map = MapType::from_iter(iter.into_iter());

                    #(#fields_set)*
                }
            }
        };

        if self.from_iter {
            extend = quote! {
                #extend

                impl #impl_generics FromIterator<(&'iter str, Option<&'iter str>)> for #struct_name #ty_generics #where_clause {
                    fn from_iter<I: IntoIterator<Item = (&'iter str, Option<&'iter str>)>>(iter: I) -> Self {
                        let mut this = Self::default();
                        this.extend(iter);
                        this
                    }
                }
            }
        };

        extend
    }

    fn parse_fields<'a>(
        &'a self,
        fields: &'a Punctuated<Field, Comma>,
        generics: &'a Generics
    ) -> impl Iterator<Item = TokenStream2> + use<'a> {
        let generic_types = generics
            .params
            .iter()
            .filter_map(|p| {
                match p {
                    GenericParam::Type(t) => t.to_token_stream().to_string().into(),
                    GenericParam::Lifetime(l) => l.lifetime.to_string().into(),
                    _ => None
                }
            })
            .collect::<Vec<_>>();

        fields.iter().map(move |field| {
            let field_name = field
                .ident
                .as_ref()
                .expect("Couldn't get ident of field")
                .to_string();

            if field_name.starts_with("_") {
                return quote! {};
            }

            let mut field_type = field
                .ty
                .to_token_stream()
                .to_string()
                .replace('\n', "")
                .replace(" :: ", "::")
                .replace("'static", "");

            generic_types.iter().for_each(|ty| {
                field_type = field_type
                    .replace(&[" ", ty.as_str(), " "].concat(), "")
                    .replace(&[" ", ty.as_str(), ","].concat(), "");
            });

            let mut types: Vec<String> = Vec::new();
            let mut ty_full = String::new();
            for ty in field_type.split_terminator('<') {
                let ty = ty.trim_matches(['>', ' ', ',', '&']);
                ty_full.push_str(ty);

                if ty_full.chars().filter(|ch| *ch == '(').count()
                    != ty_full.chars().filter(|ch| *ch == ')').count()
                {
                    ty_full.push('<');
                    continue;
                }

                if ty_full.is_empty() == false {
                    types.push(ty_full.clone());
                    ty_full.clear();
                }
            }

            let mut ty_iter = types.into_iter().enumerate().peekable();
            let mut iterable = false;
            let field_token = self
                .get_value_token(&field_name, &field.attrs, ty_iter, None, &mut iterable);

            if iterable {
                quote! { #field_token }
            } else {
                quote! {
                    if let Some(&Some(v)) = map.get(#field_name).take() {
                        #field_token
                    }
                }
            }
        })
    }

    #[allow(clippy::only_used_in_recursion)]
    fn get_value_token<I: Iterator<Item = (usize, String)>>(
        &self,
        name: &str,
        attrs: &[Attribute],
        mut types: Peekable<I>,
        prev_ty: Option<&str>,
        iterable: &mut bool
    ) -> TokenStream2 {
        let Some((n, mut ty)) = types.next() else {
            return quote! { v };
        };
        if attrs.iter().any(|a| a.path().is_ident("skip")) {
            return quote! {};
        }
        if ty.contains('(') || ty.contains(',') || ty.contains(' ') {
            panic!("This type not supported yet: {ty}");
        }
        if let Some(pos) = ty.rfind("::") {
            ty = ty.get(pos + 2..).unwrap().to_string();
        }
        let ty_ident: TypePath = parse_str(&ty)
            .map_err(|e| format!("{name}: {ty} - {e}"))
            .unwrap();
        let name_ident: Ident = parse_str(name)
            .map_err(|e| format!("{name}: {ty} - {e}"))
            .unwrap();
        let next_ty = types.peek().map(|(.., ty)| Cow::from(ty));
        let is_parse = attrs.iter().any(|a| a.path().is_ident("parse"));

        *iterable = false;
        match ty.as_str() {
            "bool" => {
                let token = quote! {
                    ["0","off","false","","\0"].contains(&v.to_lowercase().as_str()) == false
                };
                if n == 0 {
                    quote! { self.#name_ident = #token; }
                } else {
                    token
                }
            },
            ty @ ("i8" | "i16" | "i32" | "i64" | "i128" | "u8" | "u16" | "u32"
            | "u64" | "u128" | "f32" | "f64" | "f128" | "isize" | "usize"
            | "c_char" | "c_short" | "c_ushort" | "c_int" | "c_uint" | "c_long"
            | "c_ulong" | "c_longlong" | "c_ulonglong" | "c_double" | "c_float") => {
                let token = quote! {
                    v.parse::<#ty_ident>().map_err(|_| format!("Failed parse '{v}' to type {}", #ty)).unwrap()
                };
                if n == 0 {
                    quote! { self.#name_ident = #token; }
                } else {
                    token
                }
            },
            "c_void" => {
                if n == 0 {
                    quote! { self.#name_ident = ::alloc::ffi::CString::new(v).unwrap().into_raw().cast(); }
                } else {
                    quote! { ::alloc::ffi::CString::new(v).unwrap().into_raw().cast() }
                }
            },
            "CString" => {
                if n == 0 {
                    quote! { self.#name_ident = ::alloc::ffi::CString::new(v).unwrap(); }
                } else {
                    quote! { ::alloc::ffi::CString::new(v).unwrap() }
                }
            },
            "char" => {
                if n == 0 {
                    quote! { self.#name_ident = v.chars().next().unwrap_or_default(); }
                } else {
                    quote! { v.chars().next().unwrap_or_default() }
                }
            },
            "str" => {
                if n == 0 {
                    quote! { self.#name_ident = v; }
                } else {
                    quote! {v}
                }
            },
            "String" => {
                if n == 0 {
                    quote! { self.#name_ident = v.to_string(); }
                } else {
                    quote! { v.to_string() }
                }
            },
            "Box" | "Arc" | "Rc" | "NonNull" if next_ty == Some("CStr".into()) => {
                if n == 0 {
                    quote! { self.#name_ident =  #ty_ident::from(Box::leak(::alloc::ffi::CString::new(v).unwrap().into_boxed_c_str())); }
                } else {
                    quote! { #ty_ident::from(Box::leak(::alloc::ffi::CString::new(v).unwrap().into_boxed_c_str())) }
                }
            },
            "Box" | "Arc" | "Rc" | "NonNull" if next_ty == Some("str".into()) => {
                if n == 0 {
                    quote! { self.#name_ident = #ty_ident::from(v); }
                } else {
                    quote! { #ty_ident::from(v) }
                }
            },
            "Option" => {
                let token = self.get_value_token(name, attrs, types, Some(&ty), iterable);
                if n == 0 {
                    quote! { self.#name_ident = #ty_ident::from(#token); }
                } else {
                    quote! { #ty_ident::from(#token) }
                }
            },
            "Box" => {
                let token = self.get_value_token(name, attrs, types, Some(&ty), iterable);
                if n == 0 {
                    quote! { self.#name_ident.extend(#token); }
                } else {
                    quote! { #ty_ident::new(#token) }
                }
            },
            "Arc" | "Rc" => {
                let token = self.get_value_token(name, attrs, types, Some(&ty), iterable);
                if n == 0 {
                    if *iterable {
                        quote! {
                            #ty_ident::get_mut(&mut self.#name_ident)
                                .ok_or_else(||
                                    format!(
                                        "Could not get mutable reference of {}.{}",
                                        ::core::any::type_name::<#ty_ident<Self>>(),
                                        #name
                                    )
                                )
                                .unwrap()
                                .extend(#token);
                        }
                    } else {
                        quote! { self.#name_ident = #ty_ident::new(#token); }
                    }
                } else {
                    quote! { #ty_ident::new(#token) }
                }
            },
            "RefCell" => {
                let token = self.get_value_token(name, attrs, types, Some(&ty), iterable);
                if n == 0 {
                    if *iterable {
                        quote! { self.#name_ident.borrow_mut().extend(#token); }
                    } else {
                        quote! { self.#name_ident = #ty_ident::new(#token); }
                    }
                } else {
                    quote! { #ty_ident::new(#token) }
                }
            },
            "Cell" | "NonZero" | "NonNull" => {
                let token = self.get_value_token(name, attrs, types, Some(&ty), iterable);
                if n == 0 {
                    quote! { self.#name_ident = #ty_ident::new(#token); }
                } else {
                    quote! { #ty_ident::new(#token) }
                }
            },
            "Vec" | "HashSet" | "IndexSet" => {
                let token = self.get_value_token(name, attrs, types, Some(&ty), iterable);
                let token = quote! {
                    v.split_terminator(',').map(|s| { let v = s.trim(); #token })
                };
                if n == 0 {
                    quote! { self.#name_ident.extend(#token); }
                } else {
                    quote! { #ty_ident::from_iter(#token) }
                }
            },
            ty if ty.starts_with("(") && ty.ends_with(")") => {
                let token = quote! {
                    v.parse::<#ty_ident>().map_err(|_| format!("Failed parse '{v}' to type {}", #ty)).unwrap()
                };
                if n == 0 {
                    quote! { self.#name_ident = #token; }
                } else {
                    token
                }
            },
            _ if is_parse => {
                if n == 0 {
                    quote! { self.#name_ident = v.parse::<#ty_ident>().unwrap(); }
                } else {
                    quote! { v.parse::<#ty_ident>().unwrap() }
                }
            },
            _ => {
                *iterable = true;
                let token = quote! {
                     map.iter()
                        .filter_map(|(&name, &value)| {
                            (value.is_some() && (name.starts_with(concat!(#name, ".")) || name == #name))
                                .then(|| {
                                    let mut name = name.trim_start_matches(#name).trim_start_matches(".");
                                    let mut value = value;

                                    if name.is_empty() && let Some((n, v)) = value.unwrap().split_once("=") {
                                        name = String::leak([name,".",n.trim()].concat()).trim_start_matches(".");
                                        value = Some(v.trim());
                                    }

                                    (name.into(), value.map(|v| v.into()))
                                })
                        })
                };
                if n == 0 {
                    quote! { self.#name_ident.extend(#token); }
                } else if n == 1
                    && prev_ty.map(|p| ["Box", "Arc", "Rc", "RefCell"].contains(&p))
                        == Some(true)
                {
                    quote! { #token }
                } else {
                    quote! { #ty_ident::from_iter(#token) }
                }
            }
        }
    }
}
