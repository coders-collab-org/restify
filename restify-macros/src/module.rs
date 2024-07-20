use proc_macro::TokenStream;
use proc_macro2::Span;
use quote::quote;
use syn::{parenthesized, parse::Parse, parse_str, DeriveInput, Expr, Token, Type};

use crate::config::CONFIG;

pub fn expand(item: DeriveInput) -> Result<TokenStream, syn::Error> {
  let ident = &item.ident;

  let mut imports: Vec<Expr> = vec![];
  let mut controllers: Vec<Expr> = vec![];
  let mut state = None::<Type>;
  let mut context = None::<Type>;

  for attr in item.attrs.iter() {
    if !attr.path().is_ident("module") {
      continue;
    }

    attr.parse_nested_meta(|meta| {
      // #[module(imports(Module, Module))]
      if meta.path.is_ident("imports") {
        let content;
        parenthesized!(content in meta.input);

        let i = content.parse_terminated(Expr::parse, Token![,])?;
        imports.extend(i);
      }

      if meta.path.is_ident("controllers") {
        let content;
        parenthesized!(content in meta.input);

        let i = content.parse_terminated(Expr::parse, Token![,])?;
        controllers.extend(i);
      }

      if meta.path.is_ident("state") {
        let content;
        parenthesized!(content in meta.input);
        state = Some(content.parse()?);
      }

      if meta.path.is_ident("context") {
        let content;
        parenthesized!(content in meta.input);
        state = Some(content.parse()?);
      }

      Ok(())
    })?;
  }

  if let Some(path) = &CONFIG.state {
    if state.is_none() {
      state = Some(parse_str(path)?)
    }
  }

  if let Some(path) = &CONFIG.module_context {
    if context.is_none() {
      context = Some(parse_str(path)?)
    }
  }

  let controller_context = if cfg!(feature = "axum") {
    quote!(())
  } else {
    unreachable!()
  };

  let module_context = context.map_or_else(|| quote!(()), |c| quote!(#c));

  let return_content = if cfg!(feature = "axum") {
    quote!(::axum::routing::Router<#state>)
  } else {
    return Err(syn::Error::new(
      Span::call_site(),
      "Please unable adapter feature",
    ));
  };

  let module = quote! {
    impl Module for #ident {
      type Context = #module_context;
      type ControllerContext = #controller_context;
      type ControllerReturn = #return_content;


      fn details(&self, _ctx: &mut Self::Context) -> ::restify::ModuleDetails<Self::Context, Self::ControllerContext, Self::ControllerReturn> {
        ::restify::ModuleDetails {
          imports: vec![#(Box::new(#imports)),*],
          controllers: vec![#(Box::new(<#controllers as ::restify::Controller>::configure)),*]
        }
      }
    }
  };

  Ok(module.into())
}
