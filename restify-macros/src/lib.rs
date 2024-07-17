mod attr_parsing;
mod config;
mod controller;
mod injectable;
mod module;
mod route;

use proc_macro::TokenStream;
use syn::{parse_macro_input, DeriveInput, Type};

#[proc_macro_attribute]
pub fn controller(args: TokenStream, input: TokenStream) -> TokenStream {
  controller::expand(input.into(), args.into())
    .unwrap_or_else(|e| e)
    .into()
}

#[proc_macro_derive(Injectable, attributes(injectable))]
pub fn injectable(item: TokenStream) -> TokenStream {
  let item = parse_macro_input!(item as syn::Item);

  injectable::expand(item)
    .unwrap_or_else(|e| e.into_compile_error().into())
    .into()
}

#[proc_macro_derive(Module, attributes(module))]
pub fn module(item: TokenStream) -> TokenStream {
  let item = parse_macro_input!(item as DeriveInput);

  module::expand(item)
    .unwrap_or_else(|e| e.into_compile_error().into())
    .into()
}

fn infer_state_types<'a, I>(types: I) -> impl Iterator<Item = Type> + 'a
where
  I: Iterator<Item = &'a Type> + 'a,
{
  types
    .filter_map(|ty| {
      if let Type::Path(path) = ty {
        Some(&path.path)
      } else {
        None
      }
    })
    .filter_map(|path| {
      if let Some(last_segment) = path.segments.last() {
        if last_segment.ident != "State" {
          return None;
        }

        match &last_segment.arguments {
          syn::PathArguments::AngleBracketed(args) if args.args.len() == 1 => {
            Some(args.args.first().unwrap())
          }
          _ => None,
        }
      } else {
        None
      }
    })
    .filter_map(|generic_arg| {
      if let syn::GenericArgument::Type(ty) = generic_arg {
        Some(ty)
      } else {
        None
      }
    })
    .cloned()
}
