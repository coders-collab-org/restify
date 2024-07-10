mod controller;
mod module;
mod route;

use proc_macro::TokenStream;
use syn::{parse_macro_input, DeriveInput};

#[proc_macro_attribute]
pub fn controller(args: TokenStream, input: TokenStream) -> TokenStream {
  controller::expand(input.into(), args.into())
    .unwrap_or_else(|e| e)
    .into()
}

#[proc_macro_derive(Module, attributes(module))]
pub fn derive_answer_fn(item: TokenStream) -> TokenStream {
  let item = parse_macro_input!(item as DeriveInput);

  module::expand(item)
    .unwrap_or_else(|e| e.into_compile_error().into())
    .into()
}
