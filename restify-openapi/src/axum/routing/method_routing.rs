use std::convert::Infallible;

use axum::{
  extract::Request,
  handler::Handler,
  response::IntoResponse,
  routing::{self, Route},
};
use tower_layer::Layer;
use tower_service::Service;

use crate::{
  paths::{Operation, OperationType, PathItem},
  Components, DefinitionHolder, PathItemDefinition,
};

use super::MethodFilter;

pub const AXUM_METHODS: &'static [(MethodFilter, OperationType)] = &[
  (MethodFilter::GET, OperationType::Get),
  (MethodFilter::DELETE, OperationType::Delete),
  (MethodFilter::HEAD, OperationType::Head),
  (MethodFilter::OPTIONS, OperationType::Options),
  (MethodFilter::PATCH, OperationType::Patch),
  (MethodFilter::POST, OperationType::Post),
  (MethodFilter::PUT, OperationType::Put),
  (MethodFilter::TRACE, OperationType::Trace),
];

pub fn on<H, T, S>(filter: MethodFilter, handler: H) -> MethodRouter<S, Infallible>
where
  H: Handler<T, S>,
  H::Future: PathItemDefinition,
  T: 'static,
  S: Clone + Send + Sync + 'static,
{
  MethodRouter::new().on(filter, handler)
}

pub struct MethodRouter<S = (), E = Infallible> {
  pub(crate) inner: routing::MethodRouter<S, E>,
  pub(crate) def: PathItem,
  pub(crate) components: Vec<Components>,
}

pub struct RouteWrapper<S, E = Infallible> {
  path: String,
  method: MethodRouter<S, E>
}

impl<S> MethodRouter<S, Infallible>
where
  S: Clone,
{
  #[track_caller]
  pub fn on<H, T>(mut self, filter: MethodFilter, handler: H) -> Self
  where
    H: Handler<T, S>,
    H::Future: PathItemDefinition,
    T: 'static,
    S: Send + Sync + 'static,
  {
    if H::Future::is_visible() {}

    self.inner = self.inner.on(filter.clone().into(), handler);

    self.components.extend(H::Future::components());

    let operation: Operation = H::Future::operation();

    for (method, op) in AXUM_METHODS {
      if filter.contains(*method) {
        self.def.operations.insert(op.clone(), operation.clone());
      }
    }

    self
  }
}

impl<S, E> MethodRouter<S, E>
where
  S: Clone,
{
  pub fn new() -> Self {
    Self {
      inner: routing::MethodRouter::new(),
      def: Default::default(),
      components: Default::default(),
    }
  }

  pub fn with_state<S2>(self, state: S) -> MethodRouter<S2, E> {
    MethodRouter {
      inner: self.inner.with_state(state),
      components: self.components,
      def: self.def,
    }
  }

  #[track_caller]
  pub fn on_service<T>(mut self, filter: MethodFilter, svc: T) -> Self
  where
    T: Service<Request, Error = E> + Clone + Send + 'static,
    T::Response: IntoResponse + 'static,
    T::Future: Send + 'static,
  {
    self.inner = self.inner.on_service(filter.into(), svc);

    self
  }

  pub fn layer<L, NewError>(self, layer: L) -> MethodRouter<S, NewError>
  where
    L: Layer<Route<E>> + Clone + Send + 'static,
    L::Service: Service<Request> + Clone + Send + 'static,
    <L::Service as Service<Request>>::Response: IntoResponse + 'static,
    <L::Service as Service<Request>>::Error: Into<NewError> + 'static,
    <L::Service as Service<Request>>::Future: Send + 'static,
    E: 'static,
    S: 'static,
    NewError: 'static,
  {
    MethodRouter {
      inner: self.inner.layer(layer),
      components: self.components,
      def: self.def,
    }
  }

  #[track_caller]
  pub fn route_layer<L>(self, layer: L) -> MethodRouter<S, E>
  where
    L: Layer<Route<E>> + Clone + Send + 'static,
    L::Service: Service<Request, Error = E> + Clone + Send + 'static,
    <L::Service as Service<Request>>::Response: IntoResponse + 'static,
    <L::Service as Service<Request>>::Future: Send + 'static,
    E: 'static,
    S: 'static,
  {
    MethodRouter {
      inner: self.inner.route_layer(layer),
      components: self.components,
      def: self.def,
    }
  }

  #[track_caller]
  pub fn merge(mut self, other: MethodRouter<S, E>) -> Self {
    self.inner = self.inner.merge(other.inner);

    self.components.extend(other.components);

    self.def.parameters.extend(other.def.parameters);
    self.def.operations.extend(other.def.operations);

    self
  }
}


impl<S, E> DefinitionHolder for RouteWrapper<S, E>
where
  S: Clone,
{
    fn path(&self) -> &str {
        &self.path
    }

    fn operations(&mut self) -> indexmap::IndexMap<OperationType, Operation> {
        todo!()
    }

    fn components(&mut self) -> Vec<Components> {
        todo!()
    }
}