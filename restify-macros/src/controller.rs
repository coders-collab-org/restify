use proc_macro2::{Span, TokenStream};
use quote::{quote, ToTokens};
use syn::{
  parse::{Parse, ParseStream},
  parse2, parse_str,
  punctuated::Punctuated,
  Attribute, Error, Expr, Generics, ImplItem, ItemImpl, LitStr, Meta, Token, Type,
};

use crate::{config::CONFIG, route::Route};

struct Controller {
  path: LitStr,
  state: Option<Type>,
  wrappers: Vec<Expr>,
  attrs: Vec<Attribute>,
  type_: Type,
  routes: Vec<Route>,
  items: Vec<ImplItem>,
  generics: Generics,
}

impl Controller {
  fn new(
    ItemImpl {
      attrs,
      mut items,
      self_ty,
      generics,
      ..
    }: ItemImpl,
    args: Args,
  ) -> Result<Self, Error> {
    let routes = items
      .iter_mut()
      .map(Route::new)
      .collect::<Result<Vec<_>, Error>>()?
      .into_iter()
      .flatten()
      .collect();

    let mut wrappers = vec![];
    let mut state = None::<Type>;

    for nv in args.options {
      if nv.path().is_ident("wrap") {
        wrappers.push(nv.require_name_value()?.value.clone());
      } else if nv.path().is_ident("state") {
        state = Some(parse2(nv.require_list()?.tokens.clone())?);
      } else {
        return Err(syn::Error::new_spanned(
          nv.path(),
          "Unknown attribute key is specified; allowed: wrap",
        ));
      }
    }

    if let Some(path) = &CONFIG.state {
      if state.is_none() {
        state = Some(parse_str(path)?)
      }
    }

    Ok(Self {
      attrs,
      routes,
      path: args.path,
      items,
      state,
      wrappers,
      generics,
      type_: *self_ty,
    })
  }
}

struct Args {
  pub path: LitStr,
  pub options: Punctuated<Meta, Token![,]>,
}

impl ToTokens for Controller {
  fn to_tokens(&self, tokens: &mut TokenStream) {
    let Controller {
      attrs: _attrs,
      type_,
      routes,
      items,
      wrappers,
      state,
      path,
      generics,
    } = self;

    let (impl_generics, ty_generics, where_clause) = generics.split_for_impl();

    let controller = if cfg!(feature = "axum") {
      quote! {
        impl #impl_generics ::restify::Controller for #type_ #ty_generics #where_clause {
          type Context = ();
          type Return = ::axum::Router<#state>;


          fn configure(_ctx: &mut Self::Context) -> ::restify::ControllerDetails<Self::Return> {
            use ::axum::{routing, Router};
            ::restify::ControllerDetails {
              path: #path.into(),
              return_: Router::new()
              #(.#routes)*
              #(.layer(#wrappers))*,
            }

          }
        }
      }
    } else {
      quote!(compile_error!("Please unable adapter feature"))
    };

    let stream = quote! {
      impl #type_ {
        #(#items)*
      }


      #controller
    };

    tokens.extend(stream);
  }
}

impl Parse for Args {
  fn parse(input: ParseStream) -> Result<Self, Error> {
    let path = input.parse::<LitStr>().map_err(|mut err| {
      err.combine(Error::new(
        err.span(),
        format!(r#" must be start with path, #[macro("<path>"),] found {input}"#),
      ));

      err
    })?;

    // if there's no comma, assume that no options are provided
    if !input.peek(Token![,]) {
      return Ok(Self {
        path,
        options: Punctuated::new(),
      });
    }

    // advance past comma separator
    input.parse::<Token![,]>()?;

    // if next char is a literal, assume that it is a string and show multi-path error
    if input.cursor().literal().is_some() {
      return Err(syn::Error::new(
        Span::call_site(),
        r#"Multiple paths specified! There should be only one."#,
      ));
    }

    // zero or more options: name = "foo"
    let options = input.parse_terminated(Meta::parse, Token![,])?;

    Ok(Self { path, options })
  }
}

pub fn expand(input: TokenStream, args: TokenStream) -> Result<TokenStream, TokenStream> {
  let item = parse2::<ItemImpl>(input.clone())
    .and_then(|m| {
      if m.trait_.is_some() {
        return Err(syn::Error::new(
          Span::call_site(),
          r#"Unsupported impl Trait"#,
        ));
      }

      Ok(m)
    })
    .map_err(|e| input_and_compile_error(input.clone(), e))?;
  let args: Args = parse2(args).map_err(|e| input_and_compile_error(input.clone(), e))?;

  Ok(
    Controller::new(item, args)
      .map_err(Error::into_compile_error)?
      .into_token_stream(),
  )
}

/// Converts the error to a token stream and appends it to the original input.
///
/// Returning the original input in addition to the error is good for IDEs which can gracefully
/// recover and show more precise errors within the macro body.
///
/// See <https://github.com/rust-analyzer/rust-analyzer/issues/10468> for more info.
fn input_and_compile_error(mut item: TokenStream, err: syn::Error) -> TokenStream {
  item.extend(err.to_compile_error());
  item
}
