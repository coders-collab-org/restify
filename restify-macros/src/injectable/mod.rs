use proc_macro2::TokenStream;
use syn::{spanned::Spanned, Ident};

pub mod attr;
mod axum;

pub(crate) fn expand(item: syn::Item) -> syn::Result<TokenStream> {
  if cfg!(feature = "axum") {
    axum::expand(item)
  } else {
    Err(syn::Error::new(
      item.span(),
      "You must active at least one adapter",
    ))
  }
}

pub fn parse_single_generic_type_on_struct(
  generics: syn::Generics,
  fields: &syn::Fields,
) -> syn::Result<Option<Ident>> {
  if let Some(where_clause) = generics.where_clause {
    return Err(syn::Error::new_spanned(
      where_clause,
      format_args!("#[derive(Injectable)] doesn't support structs with `where` clauses"),
    ));
  }

  match generics.params.len() {
    0 => Ok(None),
    1 => {
      let param = generics.params.first().unwrap();
      let ty_ident = match param {
        syn::GenericParam::Type(ty) => &ty.ident,
        syn::GenericParam::Lifetime(lifetime) => {
          return Err(syn::Error::new_spanned(
            lifetime,
            format_args!(
              "#[derive(Injectable)] doesn't support structs \
                             that are generic over lifetimes"
            ),
          ));
        }
        syn::GenericParam::Const(konst) => {
          return Err(syn::Error::new_spanned(
            konst,
            format_args!(
              "#[derive(Injectable)] doesn't support structs \
                             that have const generics"
            ),
          ));
        }
      };

      match fields {
        syn::Fields::Named(fields_named) => {
          return Err(syn::Error::new_spanned(
            fields_named,
            format_args!(
              "#[derive(Injectable)] doesn't support named fields \
                             for generic structs. Use a tuple struct instead"
            ),
          ));
        }
        syn::Fields::Unnamed(fields_unnamed) => {
          if fields_unnamed.unnamed.len() != 1 {
            return Err(syn::Error::new_spanned(
              fields_unnamed,
              format_args!(
                "#[derive(Injectable)] only supports generics on \
                                 tuple structs that have exactly one field"
              ),
            ));
          }

          let field = fields_unnamed.unnamed.first().unwrap();

          if let syn::Type::Path(type_path) = &field.ty {
            if type_path
              .path
              .get_ident()
              .map_or(true, |field_type_ident| field_type_ident != ty_ident)
            {
              return Err(syn::Error::new_spanned(
                type_path,
                format_args!(
                  "#[derive(Injectable)] only supports generics on \
                                     tuple structs that have exactly one field of the generic type"
                ),
              ));
            }
          } else {
            return Err(syn::Error::new_spanned(&field.ty, "Expected type path"));
          }
        }
        syn::Fields::Unit => return Ok(None),
      }

      Ok(Some(ty_ident.clone()))
    }
    _ => Err(syn::Error::new_spanned(
      generics,
      format_args!("#[derive(Injectable)] only supports 0 or 1 generic type parameters"),
    )),
  }
}

pub fn error_on_generic_ident(generic_ident: Option<Ident>) -> syn::Result<()> {
  if let Some(generic_ident) = generic_ident {
    Err(syn::Error::new_spanned(
      generic_ident,
      format_args!(
        "#[derive(Injectable)] only supports generics when used with #[injectable(via)]"
      ),
    ))
  } else {
    Ok(())
  }
}
