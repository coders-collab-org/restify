use proc_macro::TokenStream;
use proc_macro2::Span;
use quote::quote;
use syn::{parenthesized, parse::Parse, DeriveInput, Expr, Token, Type};

pub fn expand(item: DeriveInput) -> Result<TokenStream, syn::Error> {
  let ident = &item.ident;

  let mut imports: Vec<Expr> = vec![];
  let mut controllers: Vec<Expr> = vec![];
  let mut state = None::<Type>;

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

      Ok(())
    })?;
  }

  let controller_context = if cfg!(feature = "axum") {
    quote!(())
  } else {
    unreachable!()
  };

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
      type Context = ();
      type ControllerContext = #controller_context;
      type ControllerReturn = #return_content;


      fn imports(&self, _ctx: &mut Self::Context) -> Vec<::restify::BoxedModule<Self::Context, Self::ControllerContext, Self::ControllerReturn>> {
        vec![#(Box::new(#imports)),*]
      }

      fn controllers(
        &self,
        _ctx: &mut Self::Context,
      ) -> Vec<::restify::BoxedControllerFn<Self::ControllerContext, Self::ControllerReturn>> {
        vec![#(Box::new(<#controllers as ::restify::Controller>::configure)),*]
      }
    }
  };

  Ok(module.into())
}
