use {
    alloc::string::ToString,
    proc_macro::TokenStream,
    proc_macro2::Span,
    quote::{quote, ToTokens},
    syn::{
        parse_macro_input, Data, DeriveInput, Fields, Ident, Lifetime, LifetimeParam,
        Path, TypePath
    }
};
#[cfg(not(feature = "std"))]
#[allow(unused_imports)]
use libc_print::std_name::*;

pub(crate) fn set_from_iter_derive(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);

    // If "std"
    #[cfg(feature = "std")]
    let rust_lib: Path = Ident::new("std", Span::call_site()).into();
    #[cfg(feature = "std")]
    let items_map_type: TypePath =
        syn::parse_str("::std::collections::HashMap<&'a str, Option<&'a str>>").unwrap();
    //
    // If no "std"
    #[cfg(not(feature = "std"))]
    let rust_lib: Path = Ident::new("alloc", Span::call_site()).into();
    #[cfg(not(feature = "std"))]
    let items_map_type: TypePath = syn::parse_str(
        "::indexmap::IndexMap<&'a str, Option<&'a str>, ::core::hash::BuildHasherDefault<ahash::AHasher>>",
    ).unwrap();

    let struct_name = input.ident;
    let struct_generics = input.generics;
    let (impl_generics, ty_generics, where_clause) = struct_generics.split_for_impl();
    let fields = match input.data {
        Data::Struct(data_struct) => {
            match data_struct.fields {
                Fields::Named(fields_named) => fields_named.named,
                _ => panic!("Only structs with named fields are supported")
            }
        },
        _ => panic!("Only structs are supported")
    };

    let fields_iter = fields.iter().map(|field| {
        let field_type = field.ty.to_token_stream().to_string();
        let field_name = field
            .ident
            .as_ref()
            .expect("Couldn't get ident of field")
            .to_string();

        quote! {
            (#field_name, #field_type)
        }
    });

    let fields_set = fields.iter().map(|field| {
        let field_ident = &field.ident;
        let field_name = field.ident.as_ref().expect("Couldn't get ident of field").to_string();

        if field_name.starts_with("_") {
            return quote! {};
        }

        let mut field_type = field.ty.to_token_stream().to_string();
        while let Some(p) = field_type.find("< '") {
            field_type.replace_range(p ..= p + field_type[p..].find('>').unwrap(), "");
        }
        let mut field_type_inner = field_type.get(
            field_type.rfind('<').map(|i| i+1).unwrap_or(0)
            ..field_type.find('>').unwrap_or(field_type.len())
        ).unwrap().trim();
        field_type_inner = field_type_inner.get(
            field_type_inner.rfind(' ').map(|i| i+1).unwrap_or(0)..
        ).unwrap().trim_matches(['[',']',' ']);

        let field_type_str = if field_type.contains("Vec <") || field_type.contains("[") {
            "Vec"
        } else {
            field_type_inner
        };

        let mut is_field_type_simple = true;
        let field_attr = field.attrs.first();

        let mut field_value = match field_type_str {
            ty @ ("bool" | "i8" | "i16" | "i32" | "i64" | "i128" | "u8" | "u16" | "u32"
            | "u64" | "u128" | "f32" | "f64" | "f128" | "isize" | "usize" | "c_char" | "c_short" | "c_ushort"
            | "c_int" | "c_uint" | "c_long" | "c_ulong" | "c_longlong" | "c_ulonglong" | "c_double" | "c_float" ) => {
                let ident = Ident::new(ty, Span::call_site());
                quote! {
                    v.parse::<#ident>().map_err(|_| concat!("Failed parse '{v}' to type ", #field_type).replace("{v}", v))?
                }
            },
            "char" => quote! {v.chars().next().unwrap_or_default()},
            "str" => quote! {v},
            "String" => quote! {v.to_string()},
            "Vec" => {
                let ident = Ident::new(field_type_inner, Span::call_site());
                match field_type_inner{
                    "String" | "str" => quote! {
                        v.split_terminator(',')
                            .map(|s| s.trim().into())
                            .collect::<::#rust_lib::vec::Vec<_>>()
                    },
                    _ => quote! {{
                        let mut arr = ::#rust_lib::vec::Vec::new();
                        for s in v.split_terminator(',') {
                            arr.push(
                                s.trim()
                                    .parse::<#ident>()
                                    .map_err(|_| concat!("Failed parse '{s}' to type ", #field_type).replace("{s}", s))?
                                    .into()
                            );
                        }
                        arr
                    }},
                }
            },
            ty => {
                if field_attr.as_ref().map(|a| a.path().is_ident("parse")) == Some(true) {
                    let ident = Ident::new(ty, Span::call_site());
                    quote! {
                        v.parse::<#ident>().map_err(|_| concat!("Failed parse '{v}' to type ", #field_type).replace("{v}", v))?
                    }
                } else {
                    is_field_type_simple = false;
                    quote! {{
                        map.iter_mut()
                            .filter_map(|(name, value)| {
                                name.starts_with(concat!(#field_name, "."))
                                    .then(|| (name.trim_start_matches(concat!(#field_name, ".")), value.take()))
                            })
                    }}
                }
            },
        };

        for mut ty in field_type.as_str()[..field_type.rfind('<').unwrap_or(0)].rsplit("<") {
            ty = ty.trim();
            ty = ty.get(ty.rfind(' ').map(|i| i+1).unwrap_or(0)..).unwrap();
            if ty.is_empty() == false {
                let type_ident = Ident::new(ty, Span::call_site());
                field_value = match ty {
                    "Option" | "Box" | "NonNull" => quote! {#type_ident::from(#field_value)},
                    "Vec" => field_value,
                    _ => quote! {#type_ident::new(#field_value)}
                }
            }
        }

        if is_field_type_simple {
            quote! {
                if let Some(Some(mut v)) = map.get_mut(#field_name).take() {
                    if v.is_empty() == false {
                        self.#field_ident = #field_value;
                    }
                }
            }
        } else {
            quote! {
                self.#field_ident.set_from_iter(#field_value)?;
            }
        }
    });

    let mut lifetime = LifetimeParam::new(Lifetime::new("'iter", Span::call_site()));
    struct_generics
        .lifetimes()
        .for_each(|l| lifetime.bounds.push(l.lifetime.clone()));

    let expanded = quote! {
        impl #impl_generics #struct_name #ty_generics #where_clause {
            pub fn struct_fields() -> &'static [(&'static str, &'static str)] {
                &[#(#fields_iter),*]
            }

            pub fn set_from_iter<#lifetime, I>(&mut self, iter: I) -> Result<(), ::#rust_lib::boxed::Box<dyn ::core::error::Error>>
            where
                I: ::core::iter::IntoIterator<Item = (&'iter str, Option<&'iter str>)>
            {
                type ItemsMap<'a> = #items_map_type;
                let mut map = ItemsMap::from_iter(iter.into_iter());

                #(#fields_set)*

                Ok(())
            }
        }
    };

    //eprintln!("{}", expanded.to_string());

    TokenStream::from(expanded)
}
