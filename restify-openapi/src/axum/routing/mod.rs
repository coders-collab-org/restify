mod method_filter;
mod method_routing;

use std::convert::Infallible;

pub use method_filter::*;
pub use method_routing::*;

use indexmap::IndexMap;
use once_cell::sync::Lazy;
use regex::Regex;

use crate::{
  paths::{OperationType, Paths},
  Components, DefinitionHolder,
};

pub struct Router<S = ()> {
  components: Components,
  paths: Paths,
  inner: axum::Router<S>,
}

impl<S> Router<S>
where
  S: Clone + Send + Sync + 'static,
{
  pub fn new() -> Self {
    Self {
      components: Components::default(),
      paths: Default::default(),
      inner: axum::Router::new(),
    }
  }

  pub fn route(self, path: &str, method_router: MethodRouter<S>) -> Self {
    self.inner = self.inner.route(path, method_router.inner);
  }

  fn update_from_def_holder(&mut self, definition_holder: &mut impl DefinitionHolder) {
    let components = definition_holder
      .components()
      .into_iter()
      .reduce(|mut acc, component| {
        acc.merge(component);
        acc
      });

    definition_holder.update_path_items(&mut self.paths.paths);

    let mut paths = IndexMap::new();

    for (path, mut item) in std::mem::take(&mut self.paths.paths) {
      let path = if path.starts_with('/') {
        path
      } else {
        "/".to_owned() + &path
      };

      item.operations.iter_mut().for_each(|(op_type, op)| {
        let operation_id = build_operation_id(&path, op_type);
        op.operation_id = op.operation_id.clone().or(Some(operation_id));
      });

      paths.insert(path, item);
    }

    // if !self.default_parameters.is_empty() {
    //   let mut parameter_components: BTreeMap<String, ReferenceOr<Parameter>> = self
    //     .default_parameters
    //     .iter()
    //     .flat_map(|p| &p.parameters)
    //     .map(|p| (p.name.clone(), ReferenceOr::Object(p.clone())))
    //     .collect();

    //   let mut schema_components: BTreeMap<String, ReferenceOr<Schema>> = self
    //     .default_parameters
    //     .iter()
    //     .flat_map(|p| p.components.clone())
    //     .collect();

    //   let mut parameter_refs = self
    //     .default_parameters
    //     .iter()
    //     .flat_map(|p| &p.parameters)
    //     .map(|p| ReferenceOr::Reference {
    //       _ref: format!("#/components/parameters/{}", p.name),
    //     })
    //     .collect();

    //   paths
    //     .values_mut()
    //     .flat_map(|pi| pi.operations.values_mut())
    //     .for_each(|op| op.parameters.append(&mut parameter_refs));

    //   if let Some(c) = components.as_mut() {
    //     c.parameters.append(&mut parameter_components);
    //     c.schemas.append(&mut schema_components);
    //   }
    // }

    // if !self.default_tags.is_empty() {
    //   paths
    //     .values_mut()
    //     .flat_map(|pi| pi.operations.values_mut())
    //     .for_each(|op| op.tags.append(&mut self.default_tags.clone()))
    // }

    self.paths.paths = paths;

    if let Some(components) = components {
      self.components.merge(components);
    }
  }
}

static PATH_RESOURCE_REGEX: Lazy<Regex> =
  Lazy::new(|| Regex::new(r"/(.*?)/\{(.*?)\}").expect("path template regex"));

fn build_operation_id(path: &str, operation_type: &OperationType) -> String {
  let resource = PATH_RESOURCE_REGEX
    .captures(path)
    .and_then(|c| c.get(1))
    .map(|_match| _match.as_str())
    .unwrap_or(path)
    .trim_matches('/');
  format!(
    "{:?}_{}-{:x}",
    operation_type,
    resource.replace('/', "-"),
    md5::compute(path)
  )
  .to_lowercase()
}
