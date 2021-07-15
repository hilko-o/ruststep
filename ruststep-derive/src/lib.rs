//! Procedural macros for ruststep
//!
//! ```
//! use ruststep_derive::{as_holder, Holder};
//! use std::collections::HashMap;
//!
//! pub struct Table {
//!     a: HashMap<u64, as_holder!(A)>,
//!     b: HashMap<u64, as_holder!(B)>,
//! }
//!
//! #[derive(Debug, Clone, PartialEq, Holder)]
//! #[holder(table = Table, field = a)]
//! pub struct A {
//!     pub x: f64,
//!     pub y: f64,
//! }
//!
//! #[derive(Debug, Clone, PartialEq, Holder)]
//! #[holder(table = Table, field = b)]
//! pub struct B {
//!     pub z: f64,
//!     #[holder(use_place_holder)]
//!     pub a: A,
//! }
//! ```
//!
//! `#[derive(Holder)]` generates followings:
//!
//! - `AHolder` struct
//!   - naming rule is `{}Holder`
//!   - This name is obtained by `as_holder!(A)`
//! - `impl Holder for AHolder`
//!

#![allow(dead_code, unused_imports)] // FIXME

use proc_macro::TokenStream;
use proc_macro2::{Span, TokenStream as TokenStream2};
use proc_macro_crate::{crate_name, FoundCrate};
use quote::quote;

mod field_type;
mod for_struct;
mod holder_attr;

use field_type::*;
use holder_attr::*;

#[proc_macro_derive(Holder, attributes(holder))]
pub fn derive_holder_entry(input: TokenStream) -> TokenStream {
    derive_holder(&syn::parse(input).unwrap()).into()
}

fn derive_holder(ast: &syn::DeriveInput) -> TokenStream2 {
    let table_attr = parse_table_attr(ast);
    let ident = &ast.ident;
    match &ast.data {
        syn::Data::Struct(st) => {
            let def_holder_tt = for_struct::def_holder(ident, st);
            let impl_holder_tt = for_struct::impl_holder(ident, &table_attr, st);
            let impl_entity_table_tt = for_struct::impl_entity_table(ident, &table_attr);
            quote! {
                #def_holder_tt
                #impl_holder_tt
                #impl_entity_table_tt
            }
        }
        _ => unimplemented!("Only struct is supprted currently"),
    }
}

/// Resolve Holder struct from owned type, e.g. `A` to `AHolder`
#[proc_macro]
pub fn as_holder(input: TokenStream) -> TokenStream {
    let path = as_holder_path(&syn::parse(input).unwrap());
    let ts = quote! { #path };
    ts.into()
}

fn as_holder_ident(input: &syn::Ident) -> syn::Ident {
    quote::format_ident!("{}Holder", input)
}

fn as_holder_path(input: &syn::Path) -> syn::Path {
    let syn::Path {
        leading_colon,
        segments,
    } = input;
    let mut segments = segments.clone();
    let mut last_seg = segments.last_mut().unwrap();
    match &mut last_seg.arguments {
        syn::PathArguments::None => {
            last_seg.ident = as_holder_ident(&last_seg.ident);
        }
        // Option<A> -> Option<AHolder>
        //       ^^^
        //       args
        syn::PathArguments::AngleBracketed(syn::AngleBracketedGenericArguments {
            args, ..
        }) => {
            for arg in args {
                if let syn::GenericArgument::Type(syn::Type::Path(path)) = arg {
                    path.path = as_holder_path(&path.path);
                }
            }
        }
        _ => unimplemented!(),
    }
    syn::Path {
        leading_colon: leading_colon.clone(),
        segments,
    }
}

/// Returns `crate` or `::ruststep` as in ruststep crate or not
fn ruststep_crate() -> syn::Path {
    let path = crate_name("ruststep").unwrap();
    match path {
        FoundCrate::Itself => {
            let mut segments = syn::punctuated::Punctuated::new();
            segments.push(syn::PathSegment {
                ident: syn::Ident::new("crate", Span::call_site()),
                arguments: syn::PathArguments::None,
            });
            syn::Path {
                leading_colon: None,
                segments,
            }
        }
        FoundCrate::Name(name) => {
            let mut segments = syn::punctuated::Punctuated::new();
            segments.push(syn::PathSegment {
                ident: syn::Ident::new(&name, Span::call_site()),
                arguments: syn::PathArguments::None,
            });
            syn::Path {
                leading_colon: Some(syn::token::Colon2::default()),
                segments,
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn holder_path() {
        let path: syn::Path = syn::parse_str("::some::Struct").unwrap();
        let holder = as_holder_path(&path);
        let ans: syn::Path = syn::parse_str("::some::StructHolder").unwrap();
        assert_eq!(holder, ans);
    }

    #[test]
    fn optional_holder_path() {
        let path: syn::Path = syn::parse_str("Option<::some::Struct>").unwrap();
        let holder = as_holder_path(&path);
        let ans: syn::Path = syn::parse_str("Option<::some::StructHolder>").unwrap();
        assert_eq!(holder, ans);
    }
}
