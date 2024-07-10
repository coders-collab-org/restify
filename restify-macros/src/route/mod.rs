use proc_macro2::Span;
use quote::{quote, ToTokens};
use syn::{
  parse::{Parse, ParseStream},
  parse2,
  punctuated::Punctuated,
  spanned::Spanned,
  Attribute, Error, FnArg, Ident, ImplItem, LitStr, Meta, MetaNameValue, Path, Signature, Token,
};

pub struct Route {
  pub name: Ident,
  pub descriptions: Vec<Attribute>,
  pub sig: Signature,
  pub method_args: MethodArgs,
  pub attrs: Vec<Attribute>,
}

impl Route {
  pub fn new(item: &mut ImplItem) -> Result<Option<Self>, Error> {
    let ImplItem::Fn(item) = item else {
      return Ok(None);
    };

    let name = item.sig.ident.clone();

    let mut method_args = None;
    let mut attrs = vec![];
    let mut descriptions: Vec<Attribute> = vec![];

    for i in 0..item.attrs.len() {
      let attr = item.attrs[i].clone();

      if let Ok(m) = MethodType::from_path(attr.path()) {
        if method_args.is_some() {
          return Err(syn::Error::new(
            attr.span(),
            r#"Unsupported to use more than one method"#,
          ));
        }

        if item
          .sig
          .inputs
          .iter()
          .any(|a| matches!(a, FnArg::Receiver(_)))
        {
          return Err(syn::Error::new(attr.span(), r#"Unsupported self method"#));
        }

        method_args = Some(MethodArgs::new(attr.meta, m)?);

        item.attrs.remove(i);

        continue;
      }

      if attr.path().is_ident("doc") {
        descriptions.push(attr);
        continue;
      }

      attrs.push(attr.clone());
    }

    let Some(method_args) = method_args else {
      return Ok(None);
    };

    if matches!(item.sig.output, syn::ReturnType::Default) {
      return Err(syn::Error::new_spanned(
        item,
        "Function has no return type. Cannot be used as handler (You can return no type if you add #[return_json] attribute",
      ));
    }

    if item.sig.asyncness.is_none() {
      return Err(syn::Error::new_spanned(item, "Function must be async"));
    }

    Ok(Some(Self {
      method_args,
      descriptions,
      sig: item.sig.clone(),
      attrs,
      name,
    }))
  }
}

impl ToTokens for Route {
  fn to_tokens(&self, tokens: &mut proc_macro2::TokenStream) {
    let Route {
      method_args:
        MethodArgs {
          path,
          guards: _,
          method,
          wrappers,
          ..
        },
      name,
      ..
    } = self;

    let method = Ident::new(method.as_lower_str(), Span::call_site());

    let stream = if cfg!(feature = "axum") {
      quote! {
        route(
          #path,
          routing::#method(Self::#name)
          #(.layer(#wrappers))*
        )
      }
    } else {
      quote!(compile_error!(
        "One of the features `actix` or `axum` must be enabled"
      ))
    };

    tokens.extend(stream);
  }
}
struct Args {
  pub path: Option<LitStr>,
  pub options: Punctuated<MetaNameValue, Token![,]>,
}

impl Parse for Args {
  fn parse(input: ParseStream) -> Result<Self, Error> {
    let path = input.parse::<LitStr>().ok();

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
    let options = input.parse_terminated(syn::MetaNameValue::parse, Token![,])?;

    Ok(Self { path, options })
  }
}

pub struct MethodArgs {
  pub path: syn::LitStr,
  pub resource_name: Option<syn::LitStr>,
  pub guards: Vec<Path>,
  pub wrappers: Vec<syn::Expr>,
  pub method: MethodType,
}

impl MethodArgs {
  fn new(meta: Meta, method: MethodType) -> syn::Result<Self> {
    let span = meta.span();
    let mut path = None;
    let mut resource_name = None;
    let mut guards = Vec::new();
    let mut wrappers = Vec::new();

    match meta {
      Meta::Path(_) => {}
      Meta::NameValue(v) => return Err(Error::new(v.span(), "Unsupported meta")),
      Meta::List(list) => {
        let args = parse2::<Args>(list.tokens)?;

        path = args.path;

        for nv in args.options {
          if nv.path.is_ident("name") {
            if let syn::Expr::Lit(syn::ExprLit {
              lit: syn::Lit::Str(lit),
              ..
            }) = nv.value
            {
              resource_name = Some(lit);
            } else {
              return Err(syn::Error::new_spanned(
                nv.value,
                "Attribute name expects literal string",
              ));
            }
          } else if nv.path.is_ident("guard") {
            if let syn::Expr::Path(syn::ExprPath { path, .. }) = nv.value {
              guards.push(path);
            } else {
              return Err(syn::Error::new_spanned(
                nv.value,
                "Attribute guard expects literal string",
              ));
            }
          } else if nv.path.is_ident("wrap") {
            wrappers.push(nv.value);
          } else {
            return Err(syn::Error::new_spanned(
              nv.path,
              "Unknown attribute key is specified; allowed: guard and wrap",
            ));
          }
        }
      }
    }

    Ok(Self {
      path: path
        .unwrap_or_else(|| LitStr::new(if cfg!(feature = "axum") { "/" } else { "" }, span)),
      resource_name,
      guards,
      wrappers,
      method,
    })
  }
}

// actix-web-codegen
macro_rules! standard_method_type {
  (
      $($variant:ident, $upper:ident, $lower:ident,)+
  ) => {
      #[derive(Debug, Clone, PartialEq, Eq, Hash)]
      pub enum MethodType {
          $(
              $variant,
          )+
      }

      impl MethodType {
          // fn as_str(&self) -> &'static str {
          //     match self {
          //         $(Self::$variant => stringify!($variant),)+
          //     }
          // }

          fn as_lower_str(&self) -> &'static str {
            match self {
                $(Self::$variant => stringify!($lower),)+
            }
          }

          // fn parse(method: &str) -> Result<Self, String> {
          //     match method {
          //         $(stringify!($upper) => Ok(Self::$variant),)+
          //         _ => Err(format!("HTTP method must be uppercase: `{}`", method)),
          //     }
          // }

          fn from_path(method: &Path) -> Result<Self, ()> {
              match () {
                  $(_ if method.is_ident(stringify!($lower)) => Ok(Self::$variant),)+
                  _ => Err(()),
              }
          }
      }
  };
}

standard_method_type! {
  Get,       GET,     get,
  Post,      POST,    post,
  Put,       PUT,     put,
  Delete,    DELETE,  delete,
  Head,      HEAD,    head,
  Connect,   CONNECT, connect,
  Options,   OPTIONS, options,
  Trace,     TRACE,   trace,
  Patch,     PATCH,   patch,
}
